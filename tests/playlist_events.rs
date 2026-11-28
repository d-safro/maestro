//! `encore.events`' `playlist` event from the writers that aren't an app's own call: a song that arrived joining
//! the playlist it was asked for, "On trial", an import (and its undo), ListenBrainz's discovery playlists and
//! Spotify sync. Each is heard by the playlist's owner only. A server's state in-process (no HTTP: the event log
//! is read straight from `Live`), stand-ins for ListenBrainz and Spotify, and rows put straight into the database.

use std::{process::Command, sync::Arc};

use conductor::{
    api::AppState,
    auth::{self, Secrets},
    config::{Config, LibraryConfig},
    db, library,
    live::Since,
};
use serde_json::json;

const OWNER: &str = "owner";
const OTHER: &str = "other";

fn have_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok_and(|o| o.status.success())
}

fn tone(path: &std::path::Path, title: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ok = Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "sine=frequency=440:duration=2"])
        .args(["-metadata", &format!("title={title}"), "-metadata", "artist=Alpha", "-metadata", "album=First"])
        .arg(path)
        .status()
        .unwrap()
        .success();
    assert!(ok, "ffmpeg failed for {}", path.display());
}

struct State {
    st: Arc<AppState>,
    owner: String,
    other: String,
    _dir: tempfile::TempDir,
}

/// An admin (who owns the discovery and Spotify playlists) and another account; songs in the library only when
/// `songs` (they need ffmpeg). `tweak` sets what else the test needs.
async fn state(songs: &[&str], tweak: impl FnOnce(&mut Config)) -> State {
    let dir = tempfile::tempdir().unwrap();
    let music = dir.path().join("music");
    std::fs::create_dir_all(&music).unwrap();
    for (i, t) in songs.iter().enumerate() {
        tone(&music.join(format!("Alpha/First/0{} {t}.flac", i + 1)), t);
    }
    let mut cfg = Config {
        data_dir: dir.path().join("data"),
        libraries: vec![LibraryConfig { name: "Music".into(), path: music, sentinel: None, scan_threads: None }],
        ..Config::default()
    };
    // nothing online but the stand-ins
    cfg.catalog = false;
    cfg.deezer_url = "http://127.0.0.1:9".into();
    cfg.lyrics_online = false;
    cfg.covers_fetch = false;
    cfg.concerts = false;
    cfg.info_online = false;
    tweak(&mut cfg);
    std::fs::create_dir_all(&cfg.data_dir).unwrap();
    let pool = db::open(&cfg.data_dir.join("conductor.db")).await.unwrap();
    let secrets = Secrets::load_or_create(&cfg.data_dir.join("secret.key")).unwrap();
    auth::create_user(&pool, &secrets, OWNER, "owner pass", true).await.unwrap();
    auth::create_user(&pool, &secrets, OTHER, "other pass", false).await.unwrap();
    let libs = library::ensure_libraries(&pool, &cfg.libraries).await.unwrap();
    let st = Arc::new(AppState::new(pool, secrets, cfg, libs));
    library::scan(&st.db, &st.libs, &st.scan, false).await.unwrap();
    let uid = async |name: &str| -> String {
        sqlx::query_scalar("SELECT id FROM user WHERE name = ?").bind(name).fetch_one(&st.db).await.unwrap()
    };
    let (owner, other) = (uid(OWNER).await, uid(OTHER).await);
    State { st, owner, other, _dir: dir }
}

impl State {
    /// The `playlist` events this account's stream would hear: (id, deleted).
    fn heard(&self, user: &str) -> Vec<(String, bool)> {
        let Since::New { events, .. } = self.st.live.events_since(0, user, None) else { panic!("the log keeps them all") };
        events
            .iter()
            .filter(|e| e.kind == "playlist")
            .map(|e| (e.data["id"].as_str().unwrap().to_string(), e.data["deleted"].as_bool().unwrap()))
            .collect()
    }

    async fn playlist(&self, id: &str, owner: &str, name: &str) {
        sqlx::query("INSERT INTO playlist (id, owner_id, name, created_at, updated_at) VALUES (?, ?, ?, 1, 1)")
            .bind(id)
            .bind(owner)
            .bind(name)
            .execute(&self.st.db)
            .await
            .unwrap();
    }

    async fn songs_of(&self, pid: &str) -> Vec<String> {
        sqlx::query_scalar("SELECT track_id FROM playlist_entry WHERE playlist_id = ? ORDER BY position")
            .bind(pid)
            .fetch_all(&self.st.db)
            .await
            .unwrap()
    }

    async fn track(&self, title: &str) -> String {
        sqlx::query_scalar("SELECT id FROM track WHERE title = ?").bind(title).fetch_one(&self.st.db).await.unwrap()
    }
}

#[tokio::test]
async fn an_arrived_song_joining_a_playlist_tells_its_owner() {
    let s = state(&[], |_| {}).await;
    // the other account asked for a song to go into the owner's playlist (shared with them)
    s.playlist("pl-shared", &s.owner, "Shared").await;
    sqlx::query("INSERT INTO request (key, title, artists_json, status, asked_at) VALUES ('dz-t-1', 'One', '[\"Alpha\"]', 'done', 1)")
        .execute(&s.st.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO request_want (key, user_id, playlist_id) VALUES ('dz-t-1', ?, 'pl-shared')")
        .bind(&s.other)
        .execute(&s.st.db)
        .await
        .unwrap();
    conductor::catalog::acquire::apply_wants(&s.st, "dz-t-1", "local-1").await.unwrap();
    assert_eq!(s.songs_of("pl-shared").await, ["local-1"]);
    assert_eq!(s.heard(&s.owner), [("pl-shared".to_string(), false)]);
    assert!(s.heard(&s.other).is_empty(), "not the asker's playlist");
    // already there (asked for again): nothing changed, nothing said
    sqlx::query("UPDATE request_want SET done = 0").execute(&s.st.db).await.unwrap();
    conductor::catalog::acquire::apply_wants(&s.st, "dz-t-1", "local-1").await.unwrap();
    assert_eq!(s.heard(&s.owner).len(), 1);
}

#[tokio::test]
async fn on_trial_tells_whose_it_is() {
    let s = state(&[], |_| {}).await;
    let now = db::now();
    sqlx::query(
        "INSERT INTO request (key, title, artists_json, status, asked_at, trial, trial_at, folder, trial_user, local_id, rel, arrived_at)
         VALUES ('dz-t-1', 'Trying', '[\"Beta\"]', 'done', ?1, 1, ?1, 'Trial', ?2, 'local-1', 'Trial/x.flac', ?1)",
    )
    .bind(now)
    .bind(&s.other)
    .execute(&s.st.db)
    .await
    .unwrap();
    conductor::catalog::trial::tick(&s.st).await.unwrap();
    let pid = conductor::catalog::trial::playlist_id(&s.st, &s.other, false).await.unwrap().expect("made");
    assert_eq!(s.songs_of(&pid).await, ["local-1"]);
    assert_eq!(s.heard(&s.other), [(pid.clone(), false)]);
    assert!(s.heard(&s.owner).is_empty());
    // nothing changed: nothing said
    conductor::catalog::trial::tick(&s.st).await.unwrap();
    assert_eq!(s.heard(&s.other).len(), 1);
}

#[tokio::test]
async fn an_import_and_its_undo_tell_the_importer() {
    use conductor::import::apply::{self, Options, Plan, PlanList};
    let s = state(&[], |_| {}).await;
    // a playlist the import fills again, and one it makes
    s.playlist("pl-old", &s.other, "Road").await;
    sqlx::query("INSERT INTO playlist_entry (playlist_id, position, track_id) VALUES ('pl-old', 0, 'local-0')")
        .execute(&s.st.db)
        .await
        .unwrap();
    sqlx::query("UPDATE playlist SET comment = 'Imported from Spotify on 2026-01-01' WHERE id = 'pl-old'").execute(&s.st.db).await.unwrap();
    sqlx::query("INSERT INTO import_batch (id, user_id, status, created_at) VALUES ('b1', ?, 'running', 1)")
        .bind(&s.other)
        .execute(&s.st.db)
        .await
        .unwrap();
    let list = |name: &str, ids: &[&str]| PlanList {
        name: name.into(),
        description: String::new(),
        ids: ids.iter().map(|x| x.to_string()).collect(),
        missing: vec![],
    };
    let plan = Plan { source: "Spotify".into(), playlists: vec![list("Road", &["local-1", "local-2"]), list("New one", &["local-3"])], ..Plan::default() };
    apply::apply(&s.st, "b1", &s.other, &plan, &Options::default()).await.unwrap();
    let heard = s.heard(&s.other);
    assert_eq!(heard.len(), 2, "{heard:?}");
    assert!(heard.contains(&("pl-old".to_string(), false)), "{heard:?}");
    let new = heard.iter().find(|(id, _)| id != "pl-old").unwrap().0.clone();
    assert_eq!(s.songs_of(&new).await, ["local-3"]);
    assert!(s.heard(&s.owner).is_empty());

    // undone: the one it made is deleted, the one it filled gets its songs back
    apply::undo(&s.st, "b1").await.unwrap();
    let heard = s.heard(&s.other);
    assert_eq!(heard.len(), 4, "{heard:?}");
    assert!(heard[2..].contains(&(new.clone(), true)), "{heard:?}");
    assert!(heard[2..].contains(&("pl-old".to_string(), false)), "{heard:?}");
    assert_eq!(s.songs_of("pl-old").await, ["local-0"]);
    assert!(s.heard(&s.owner).is_empty());
}

/// ListenBrainz and Spotify, as Conductor uses them: one list each, with a song the library has.
async fn stand_in() -> String {
    use axum::{extract::Request, response::IntoResponse};
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let app = axum::Router::new().fallback(move |req: Request| async move {
        let t = |id: &str, name: &str| json!({ "id": id, "name": name, "type": "track", "artists": [{ "name": "Alpha" }], "duration_ms": 2000 });
        let body = match req.uri().path() {
            "/lb/1/user/lbuser/playlists/createdfor" => json!({ "playlists": [{ "playlist": {
                "title": "Weekly Exploration for lbuser, week 1", "identifier": "https://listenbrainz.org/playlist/pl-1",
                "date": "2026-11-10T00:00:00" } }] }),
            "/lb/1/playlist/pl-1" => json!({ "playlist": { "track": [{ "title": "One", "creator": "Alpha", "duration": 2000 }] } }),
            "/sp/api/token" => json!({ "access_token": "acc", "expires_in": 3600 }),
            "/sp/v1/me/tracks" => json!({ "items": [{ "track": t("sp-1", "One") }], "next": null }),
            "/sp/v1/me/playlists" => json!({ "items": [{ "id": "car", "name": "Car" }], "next": null }),
            "/sp/v1/playlists/car/items" => json!({ "items": [{ "item": t("sp-2", "Two") }], "next": null }),
            _ => return (axum::http::StatusCode::NOT_FOUND, "{}").into_response(),
        };
        axum::Json(body).into_response()
    });
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    base
}

#[tokio::test]
async fn discovery_and_spotify_playlists_tell_their_owner() {
    if !have_ffmpeg() {
        eprintln!("skipping: ffmpeg not found");
        return;
    }
    let base = stand_in().await;
    let s = state(&["One", "Two"], |c| {
        c.listenbrainz_url = format!("{base}/lb");
        c.listenbrainz_user = Some("lbuser".into());
        c.discovery = vec!["weekly-exploration".into()];
        c.spotify_api_url = format!("{base}/sp/v1");
        c.spotify_accounts_url = format!("{base}/sp");
    })
    .await;
    let (one, two) = (s.track("One").await, s.track("Two").await);

    conductor::listenbrainz::refresh(&s.st, "weekly-exploration").await.unwrap();
    let heard = s.heard(&s.owner);
    assert_eq!(heard.len(), 1, "{heard:?}");
    assert_eq!(s.songs_of(&heard[0].0).await, std::slice::from_ref(&one));
    // the same list again: nothing changed, nothing said
    conductor::listenbrainz::refresh(&s.st, "weekly-exploration").await.unwrap();
    assert_eq!(s.heard(&s.owner).len(), 1);

    for (k, v) in [("spotify_token", r#"{"client_id": "cid", "refresh_token": "r"}"#), ("spotify_settings", r#"{"liked": true, "playlists": ["Car"]}"#)] {
        sqlx::query("INSERT INTO setting (key, value) VALUES (?, ?)").bind(k).bind(v).execute(&s.st.db).await.unwrap();
    }
    conductor::spotify::sync_once(&s.st).await.unwrap();
    let heard = s.heard(&s.owner);
    assert_eq!(heard.len(), 3, "{heard:?}");
    let mut lists = vec![];
    for (id, deleted) in &heard[1..] {
        assert!(!deleted);
        lists.push(s.songs_of(id).await);
    }
    lists.sort();
    let mut want = vec![vec![one], vec![two]];
    want.sort();
    assert_eq!(lists, want);
    assert!(s.heard(&s.other).is_empty(), "nobody else hears");
}

/// Spotify for the liked-order test: three liked songs, newest first, each with when it was liked.
async fn liked_stand_in() -> String {
    use axum::{extract::Request, response::IntoResponse};
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let app = axum::Router::new().fallback(move |req: Request| async move {
        let t = |id: &str, name: &str| json!({ "id": id, "name": name, "type": "track", "artists": [{ "name": "Alpha" }], "duration_ms": 2000 });
        let body = match req.uri().path() {
            "/sp/api/token" => json!({ "access_token": "acc", "expires_in": 3600 }),
            "/sp/v1/me/tracks" => json!({ "items": [
                { "added_at": "2026-11-20T10:00:00Z", "track": t("sp-3", "Three") },
                { "added_at": "2025-01-01T00:00:00Z", "track": t("sp-1", "One") },
                { "added_at": "2024-06-01T12:00:00Z", "track": t("sp-2", "Two") },
            ], "next": null }),
            "/sp/v1/me/playlists" => json!({ "items": [], "next": null }),
            _ => return (axum::http::StatusCode::NOT_FOUND, "{}").into_response(),
        };
        axum::Json(body).into_response()
    });
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    base
}

#[tokio::test]
async fn spotify_liked_songs_are_hearted_in_spotifys_order() {
    if !have_ffmpeg() {
        eprintln!("skipping: ffmpeg not found");
        return;
    }
    let base = liked_stand_in().await;
    let s = state(&["One", "Two", "Three", "Four"], |c| {
        c.listenbrainz_user = Some("lbuser".into());
        c.spotify_api_url = format!("{base}/sp/v1");
        c.spotify_accounts_url = format!("{base}/sp");
    })
    .await;
    let heart = async |title: &str, at: i64| {
        sqlx::query("INSERT INTO annotation (user_id, item_kind, item_id, starred_at) VALUES (?, 'track', ?, ?)")
            .bind(&s.owner)
            .bind(s.track(title).await)
            .bind(at)
            .execute(&s.st.db)
            .await
            .unwrap();
    };
    // Two was hearted by an earlier sync (when it arrived, not when it was liked); Four only here, on the phone
    let now = db::now();
    heart("Two", now - 60).await;
    heart("Four", now).await;
    for (k, v) in [("spotify_token", r#"{"client_id": "cid", "refresh_token": "r"}"#), ("spotify_settings", r#"{"liked": true, "playlists": []}"#)] {
        sqlx::query("INSERT INTO setting (key, value) VALUES (?, ?)").bind(k).bind(v).execute(&s.st.db).await.unwrap();
    }
    conductor::spotify::sync_once(&s.st).await.unwrap();

    let at = async |title: &str| -> Option<i64> {
        sqlx::query_scalar("SELECT starred_at FROM annotation WHERE user_id = ? AND item_kind = 'track' AND item_id = ?")
            .bind(&s.owner)
            .bind(s.track(title).await)
            .fetch_one(&s.st.db)
            .await
            .unwrap()
    };
    // each Spotify-liked song carries the time it was liked there; the one hearted only here keeps its own
    assert_eq!(at("Three").await, db::parse_iso("2026-11-20T10:00:00Z"));
    assert_eq!(at("One").await, db::parse_iso("2025-01-01T00:00:00Z"));
    assert_eq!(at("Two").await, db::parse_iso("2024-06-01T12:00:00Z"), "an earlier sync's time is corrected");
    assert_eq!(at("Four").await, Some(now));
    // so the hearted list (newest heart first, as getStarred2 lists it) is Spotify's order
    let titles: Vec<String> = sqlx::query_scalar(
        "SELECT t.title FROM annotation an JOIN track t ON t.id = an.item_id
         WHERE an.user_id = ? AND an.item_kind = 'track' AND an.starred_at IS NOT NULL ORDER BY an.starred_at DESC",
    )
    .bind(&s.owner)
    .fetch_all(&s.st.db)
    .await
    .unwrap();
    assert_eq!(titles, ["Four", "Three", "One", "Two"]);
}
