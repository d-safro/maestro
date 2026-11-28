//! Spotify sync (what spotify-sync + music-download did, in "songs" mode): every few hours your Spotify Liked Songs
//! and chosen playlists are read, and playlists of the same names here (marked "Synced from Spotify") are filled with
//! the songs you have, in Spotify's order; Liked Songs are hearted. Songs you don't have are asked for (matched on
//! Deezer by ISRC, then by name) and join their playlists when they arrive.
//!
//! The Spotify login (a public app's client id + refresh token; no secret) and the choices (Liked Songs on/off, which
//! playlists) come from spotify.py's files through `import-music-lookup`, and are kept in the `setting` table.

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::{
    api::AppState,
    catalog,
    db::{self, now},
};

pub const MARK: &str = "Synced from Spotify";
const LIKED: &str = "Liked Songs";

fn http() -> reqwest::Client {
    reqwest::Client::builder().timeout(Duration::from_secs(30)).build().expect("http client")
}

async fn setting(st: &AppState, key: &str) -> Option<Value> {
    let v: Option<String> =
        sqlx::query_scalar("SELECT value FROM setting WHERE key = ?").bind(key).fetch_optional(&st.db).await.ok().flatten();
    v.and_then(|s| serde_json::from_str(&s).ok())
}

async fn set_setting(st: &AppState, key: &str, v: &Value) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO setting (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(v.to_string())
        .execute(&st.db)
        .await?;
    Ok(())
}

/// spotify.py's files (token.json, settings.json, songs.json) → Conductor. Returns (login found, songs mapped).
pub async fn import_from(st_db: &db::Db, dir: &std::path::Path, lib_id: Option<i64>) -> Result<(bool, usize)> {
    let mut login = false;
    if let Ok(t) = std::fs::read_to_string(dir.join("token.json")) {
        let t: Value = serde_json::from_str(&t).context("token.json")?;
        if let (Some(c), Some(r)) = (t["client_id"].as_str(), t["refresh_token"].as_str()) {
            let v = json!({ "client_id": c, "refresh_token": r });
            sqlx::query("INSERT OR REPLACE INTO setting (key, value) VALUES ('spotify_token', ?)")
                .bind(v.to_string())
                .execute(st_db)
                .await?;
            login = true;
        }
    }
    if let Ok(s) = std::fs::read_to_string(dir.join("settings.json")) {
        let s: Value = serde_json::from_str(&s).context("settings.json")?;
        let v = json!({ "liked": s["liked"].as_bool().unwrap_or(true), "playlists": s["playlists"].clone(), "mode": s["mode"].clone() });
        sqlx::query("INSERT OR REPLACE INTO setting (key, value) VALUES ('spotify_settings', ?)")
            .bind(v.to_string())
            .execute(st_db)
            .await?;
    }
    let mut mapped = 0;
    if let (Ok(s), Some(lib)) = (std::fs::read_to_string(dir.join("songs.json")), lib_id) {
        let songs: Value = serde_json::from_str(&s).context("songs.json")?;
        for (sid, e) in songs.as_object().into_iter().flatten() {
            let Some(rel) = e["rel"].as_str() else { continue };
            let tid: Option<String> = sqlx::query_scalar("SELECT id FROM track WHERE library_id = ? AND rel_path = ?")
                .bind(lib)
                .bind(rel.replace('\\', "/"))
                .fetch_optional(st_db)
                .await?;
            if let Some(t) = tid {
                sqlx::query("INSERT OR REPLACE INTO spotify_map (spotify_id, track_id) VALUES (?, ?)")
                    .bind(sid)
                    .bind(t)
                    .execute(st_db)
                    .await?;
                mapped += 1;
            }
        }
    }
    Ok((login, mapped))
}

pub struct Client {
    api: String,
    accounts: String,
    client_id: String,
    refresh: String,
    access: String,
}

impl Client {
    async fn token(&mut self, st: &AppState) -> Result<()> {
        let form =
            [("grant_type", "refresh_token"), ("refresh_token", self.refresh.as_str()), ("client_id", self.client_id.as_str())];
        let r = http().post(format!("{}/api/token", self.accounts)).form(&form).send().await?;
        if !r.status().is_success() {
            bail!("Spotify sign-in answered {} (sign in again with spotify.py export, then import-music-lookup)", r.status());
        }
        let t: Value = r.json().await?;
        self.access = t["access_token"].as_str().unwrap_or("").to_string();
        if let Some(new) = t["refresh_token"].as_str() {
            // Spotify may hand out a new refresh token: keep it
            self.refresh = new.to_string();
            set_setting(st, "spotify_token", &json!({ "client_id": self.client_id, "refresh_token": new })).await?;
        }
        Ok(())
    }

    pub(crate) async fn get(&self, url: &str) -> Result<Value> {
        let url = if url.starts_with("http") { url.to_string() } else { format!("{}{url}", self.api) };
        for _ in 0..5 {
            let r = http().get(&url).bearer_auth(&self.access).send().await?;
            if r.status().as_u16() == 429 {
                let wait =
                    r.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok()).unwrap_or(5u64);
                tokio::time::sleep(Duration::from_secs(wait + 1)).await;
                continue;
            }
            if !r.status().is_success() {
                bail!("Spotify {url}: {}", r.status());
            }
            return Ok(r.json().await?);
        }
        bail!("Spotify keeps saying slow down for {url}")
    }

    pub(crate) async fn pages(&self, url: &str) -> Result<Vec<Value>> {
        let mut out = vec![];
        let mut next = Some(url.to_string());
        while let Some(u) = next {
            let page = self.get(&u).await?;
            out.extend(page["items"].as_array().cloned().unwrap_or_default());
            next = page["next"].as_str().map(String::from);
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
struct Song {
    id: String,
    name: String,
    artists: Vec<String>,
    isrc: Option<String>,
    ms: i64,
    /// When it was liked on Spotify (Liked Songs only): its heart here gets the same time, so the hearted list
    /// (newest heart first) is in Spotify's order
    liked_at: Option<i64>,
}

fn song(t: &Value) -> Option<Song> {
    if t.is_null() || t["is_local"] == true || t["type"].as_str().is_some_and(|x| x != "track") {
        return None;
    }
    Some(Song {
        id: t["id"].as_str()?.to_string(),
        name: t["name"].as_str().unwrap_or("").to_string(),
        artists: t["artists"].as_array().into_iter().flatten().filter_map(|a| a["name"].as_str().map(String::from)).collect(),
        isrc: t["external_ids"]["isrc"].as_str().map(String::from),
        ms: t["duration_ms"].as_i64().unwrap_or(0),
        liked_at: None,
    })
}

/// Your copy of a Spotify song: the file it became before, else title + artist (+ length).
async fn find_local(st: &AppState, s: &Song) -> Option<String> {
    let known: Option<String> = sqlx::query_scalar("SELECT m.track_id FROM spotify_map m JOIN track t ON t.id = m.track_id WHERE m.spotify_id = ? AND t.missing_since IS NULL")
        .bind(&s.id)
        .fetch_optional(&st.db)
        .await
        .ok()
        .flatten();
    if known.is_some() {
        return known;
    }
    let artist = s.artists.first().cloned().unwrap_or_default();
    let found = catalog::find(st, &s.name, &artist, Some(s.ms / 1000).filter(|x| *x > 0)).await?;
    let _ = sqlx::query("INSERT OR REPLACE INTO spotify_map (spotify_id, track_id) VALUES (?, ?)")
        .bind(&s.id)
        .bind(&found.id)
        .execute(&st.db)
        .await;
    Some(found.id)
}

/// The Deezer song for a Spotify song: by ISRC, else by name.
async fn deezer_key(st: &AppState, s: &Song) -> Option<String> {
    let artist = s.artists.first().cloned().unwrap_or_default();
    if let Some(isrc) = &s.isrc {
        let t = st.catalog.dz.get(&format!("track/isrc:{isrc}"), &[]).await;
        if t["id"].as_i64().is_some_and(|i| i > 0) {
            return Some(format!("dz-t-{}", t["id"]));
        }
    }
    let q = format!(r#"artist:"{artist}" track:"{}""#, catalog::text::clean_title(&s.name));
    let hit = st.catalog.dz.list("search/track", &[("q", &q), ("limit", "3")]).await;
    hit.into_iter()
        .find(|t| {
            catalog::text::same_artist(&artist, t["artist"]["name"].as_str().unwrap_or(""))
                && catalog::text::tk(t["title"].as_str().unwrap_or("")) == catalog::text::tk(&s.name)
        })
        .map(|t| format!("dz-t-{}", t["id"]))
}

async fn playlist(st: &AppState, owner: &str, name: &str) -> Result<String> {
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT id FROM playlist WHERE owner_id = ? AND comment = ? AND (name = ? OR name = ?) ORDER BY created_at LIMIT 1",
    )
    .bind(owner)
    .bind(MARK)
    .bind(name)
    .bind(format!("{name} (Spotify)"))
    .fetch_optional(&st.db)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let clash: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM playlist WHERE owner_id = ? AND name = ?")
        .bind(owner)
        .bind(name)
        .fetch_one(&st.db)
        .await?;
    let id = db::random_id();
    sqlx::query("INSERT INTO playlist (id, owner_id, name, comment, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(owner)
        .bind(if clash > 0 { format!("{name} (Spotify)") } else { name.to_string() })
        .bind(MARK)
        .bind(now())
        .bind(now())
        .execute(&st.db)
        .await?;
    Ok(id)
}

async fn signed_in(st: &AppState, tok: &Value) -> Result<Client> {
    let mut c = Client {
        api: st.cfg.spotify_api_url.trim_end_matches('/').to_string(),
        accounts: st.cfg.spotify_accounts_url.trim_end_matches('/').to_string(),
        client_id: tok["client_id"].as_str().unwrap_or("").to_string(),
        refresh: tok["refresh_token"].as_str().unwrap_or("").to_string(),
        access: String::new(),
    };
    c.token(st).await?;
    Ok(c)
}

/// The Spotify account Spotify sync is signed in to (for imports), if any.
pub async fn client(st: &AppState) -> Result<Option<Client>> {
    match setting(st, "spotify_token").await {
        Some(tok) => Ok(Some(signed_in(st, &tok).await?)),
        None => Ok(None),
    }
}

/// One sync. Returns (lists, songs you have, songs asked for).
pub async fn sync_once(st: &AppState) -> Result<(usize, usize, usize)> {
    let Some(tok) = setting(st, "spotify_token").await else { return Ok((0, 0, 0)) };
    let choices = setting(st, "spotify_settings").await.unwrap_or(json!({ "liked": true, "playlists": [] }));
    let Some(owner) = crate::listenbrainz::owner(st).await else { return Ok((0, 0, 0)) };
    let c = signed_in(st, &tok).await?;
    let wanted: Vec<String> =
        choices["playlists"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(String::from)).collect();
    let mut lists: Vec<(String, Vec<Song>)> = vec![];
    if choices["liked"].as_bool().unwrap_or(true) {
        // newest liked at the top, each with when it was liked
        let liked: Vec<Song> = c
            .pages("/me/tracks?limit=50")
            .await?
            .iter()
            .filter_map(|i| song(&i["track"]).map(|s| Song { liked_at: i["added_at"].as_str().and_then(db::parse_iso), ..s }))
            .collect();
        lists.push((LIKED.into(), liked));
    }
    for p in c.pages("/me/playlists?limit=50").await? {
        let name = p["name"].as_str().unwrap_or("").to_string();
        if !wanted.contains(&name) {
            continue;
        }
        let id = p["id"].as_str().unwrap_or("");
        let items = match c.pages(&format!("/playlists/{id}/items?limit=100")).await {
            Ok(i) => i,
            Err(_) => match c.pages(&format!("/playlists/{id}/tracks?limit=100")).await {
                Ok(i) => i,
                Err(e) => {
                    warn!(playlist = %name, "Spotify playlist not read, left as it is: {e:#}");
                    continue;
                }
            },
        };
        lists.push((
            name,
            items.iter().filter_map(|i| song(if i["item"].is_object() { &i["item"] } else { &i["track"] })).collect(),
        ));
    }
    let (mut have_n, mut asked_n) = (0, 0);
    for (name, songs) in &lists {
        let pid = playlist(st, &owner, name).await?;
        let mut ids: Vec<String> = vec![];
        let mut liked_at: Vec<Option<i64>> = vec![]; // beside ids: when Spotify says each was liked
        for s in songs {
            match find_local(st, s).await {
                Some(t) => {
                    if !ids.contains(&t) {
                        ids.push(t);
                        liked_at.push(s.liked_at);
                    }
                }
                None => {
                    if st.cfg.read_only {
                        continue;
                    }
                    if let Some(key) = deezer_key(st, s).await
                        && catalog::ask_synced(st, &key, &owner, name == LIKED, Some(&pid), None).await?
                    {
                        asked_n += 1;
                    }
                }
            }
        }
        have_n += ids.len();
        let mut tx = st.db.begin().await?;
        sqlx::query("DELETE FROM playlist_entry WHERE playlist_id = ?").bind(&pid).execute(&mut *tx).await?;
        for (i, id) in ids.iter().enumerate() {
            sqlx::query("INSERT INTO playlist_entry (playlist_id, position, track_id) VALUES (?, ?, ?)")
                .bind(&pid)
                .bind(i as i64)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query("UPDATE playlist SET updated_at = ? WHERE id = ?").bind(now()).bind(&pid).execute(&mut *tx).await?;
        if name == LIKED {
            for (id, at) in ids.iter().zip(&liked_at) {
                sqlx::query("INSERT OR IGNORE INTO annotation (user_id, item_kind, item_id) VALUES (?, 'track', ?)")
                    .bind(&owner)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                // Spotify's time when it says one (every sync: a song that arrived since, hearted then, moves to
                // its place), else the heart it has, else now
                sqlx::query("UPDATE annotation SET starred_at = COALESCE(?, starred_at, ?) WHERE user_id = ? AND item_kind = 'track' AND item_id = ?")
                    .bind(*at)
                    .bind(now())
                    .bind(&owner)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        st.live.playlist_changed(&owner, &pid, false);
        info!(playlist = %name, have = ids.len(), of = songs.len(), "Spotify playlist synced");
    }
    Ok((lists.len(), have_n, asked_n))
}

/// Every `spotify_interval_hours`.
pub async fn worker(st: Arc<AppState>) {
    if !st.cfg.spotify_sync {
        return;
    }
    tokio::time::sleep(Duration::from_secs(if st.cfg.slskd_fast_test { 0 } else { 300 })).await;
    loop {
        match sync_once(&st).await {
            Ok((0, _, _)) => {}
            Ok((lists, have, asked)) => info!(lists, have, asked, "Spotify sync done"),
            Err(e) => warn!("Spotify sync: {e:#}"),
        }
        tokio::time::sleep(Duration::from_secs(if st.cfg.slskd_fast_test {
            2
        } else {
            st.cfg.spotify_interval_hours.max(1) * 3600
        }))
        .await;
    }
}
