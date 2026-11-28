# Changes

What changed in each release, newest first. The full notes for each are on the
[releases page](https://github.com/d-safro/conductor/releases); what stays the same from 1.0 on is in
[docs/STABILITY.md](docs/STABILITY.md).

## 1.7.2 (2026-11-28)

### Fixed
- **Liked songs in Spotify's order:** Spotify sync hearted each liked song with the time of the sync (or of its
  arrival), so the hearted list, newest heart first, was in the order songs reached the server. Each song's heart
  now has the time it was liked on Spotify, set again every sync, which also corrects earlier syncs and songs that
  arrived since. A heart made only here keeps its own time.

## 1.7.1 (2026-11-24)

### Fixed
- **An album named with a subtitle shows all its songs:** an album filed as "Акустика: История Аквариума - Том I"
  is just "Акустика" on Deezer, so its page showed only your songs. Conductor now also looks for its main title
  (before ": " or ". "), for each name of an artist written "Aquarium [Аквариум]", and for the title alone when
  Deezer has the artist under a name in another alphabet ("Aquarium" for "Аквариум"). A match by the main title
  counts only when at least half your songs are on it.

## 1.7.0 (2026-11-24)

### New
- **Remove a song for good** (`encore.removeSongs`, admins only; Encore 1.19's "Remove from library…"): a song and
  every other copy of it (the same title by the same artist, as long, in any library) is deleted from the server.
  The app asks what would go first and shows each file. Then the files go, the song leaves every playlist and play
  queue, a song Conductor downloaded is set aside so it isn't fetched again, and every app hears it and drops its
  download. Nothing is deleted without `confirm=permanently`, or when the drive isn't connected or the server is
  read-only.

## 1.6.0 (2026-11-23)

### New
- **An album's song list is kept once it's looked up:** opening an album that Conductor matched to Deezer reads the
  list it kept, not Deezer again, also after a restart and once you have every song. Each song on it points to your
  copy, or, for one you don't have, to its download, and to your copy once it arrives.

### Fixed
- **Songs fetched for an album join it:** a song asked for from an album of yours (⬇ on it) is tagged, when it's
  downloaded, with the album's name, artist, year and genre and its own disc and track number there, instead of
  keeping the tags of whoever shared it, which made it an album of its own (or several).
- **Songs with no disc number play in order with those on disc 1:** they were listed before disc 1, so an album with
  some songs tagged "disc 1" and others with no disc started in the middle.

## 1.5.3 (2026-11-23)

### Fixed
- **After upgrading, albums are completed the new way at once:** an album 1.5.1 or older found nothing for on Deezer
  was kept as "nothing" for a week, so 1.5.2's "Various Artists" search didn't reach it until then. Those are
  looked for again now.

## 1.5.2 (2026-11-23)

### Fixed
- **Albums Deezer lists as "Various Artists" are completed** (a record with guests on most songs, filed under its
  own artist in your library): when nothing turns up by the artist, Deezer's "Various Artists" album of that name is
  taken, but only when at least half your songs are on it, so someone's compilation that shares the name isn't.

## 1.5.1 (2026-11-23)

### Fixed
- **Albums by artists credited together are completed again** ("Андрей Макаревич & Папоротник", "A feat. B", "A и
  группа B"): when Deezer has nothing under the whole name, the album is looked for under the first-named artist,
  where Deezer usually has it, so its songs you don't have show (and can be fetched) as on any other album.

## 1.5.0 (2026-11-23)

### New
- **"Sounds like" from the audio itself**, without AudioMuse-AI (`audio_analysis`, on): every song is analysed once
  in the background, after sound levelling and as gently (a minute of it: tempo and beat, brightness, timbre, key,
  dynamics), and Instant Mix (`getSonicSimilarTracks`, `findSonicPath`), song radio (`getSimilarSongs`, blended with
  which artists go together) and the mixes go by how songs sound. AudioMuse-AI still comes first when it's set up.
  A rescan keeps the analysis, a changed file is analysed again, and Podium shows how far it is. The answers are the
  same shape as before; apps need no change.

### Changed
- **Every playlist change is heard live** (`encore.events`' `playlist` event): also when an import fills or makes a
  playlist (and when it's undone), when ListenBrainz's discovery playlists or Spotify sync fill theirs, when "On trial"
  changes, and when a song that was asked for arrives and joins its playlist. As before, only the playlist's owner
  hears.

### Fixed
- **Security:** the `cookie` crate (Podium's sign-in cookie) is updated to 0.18.3 for RUSTSEC-2026-0594 (private
  cookie nonces could repeat in a process that forks; Conductor doesn't, but the fix is in).
- **searchRequestFiles on a read-only server** (and Podium's search) is refused at once with a clear message instead
  of searching Soulseek for files that couldn't be picked there.

## 1.4.0 (2026-11-23)

### New
- **Live news of what you asked for** (`encore.events`' `requests` event): an app with the event stream open hears
  within a few seconds when what its account asked for (`encore.requests`) changes: a song found, its download moving
  on (by 10%), arrived, kept or set aside; an admin hears of any change, Lidarr's albums too. A hint, as the other
  events are: the app fetches the list with getRequests. Nothing is looked at while no stream is open.

## 1.3.0 (2026-11-23)

### New
- **What was asked for, from any app** (`encore.requests`): the songs asked for (hearted, added to a playlist, played
  on trial, in a mix, from ListenBrainz) and Lidarr's albums, as Podium lists them: who asked and why, how it's going
  (searching, queued, downloading with its progress, done, not found, failed, set aside), when, on trial or kept; a
  page at a time, by state. An admin sees all of them and can retry, cancel, set one aside or bring it back, search
  Soulseek in their own words and pick a file; anyone else sees only what they asked for. Tools that read Podium's
  private API for this can use these public calls instead (they won't change under them).

## 1.2.0 (2026-11-23)

### New
- **Live updates** (`encore.events`): an app keeps one event stream open and hears at once what changed for its
  account: a command from your other device, a device starting or stopping a song, a friend in your Jam or adding to
  your playlist, a setting changed on another device, what's playing, the library scan. Encore uses it instead of
  asking every few seconds. Only what the account may see; resumes after a dropped connection; works behind Caddy.
- **Levelling in conversions** (`level_conversions`: off, track or album; off by default), for apps that don't level
  songs themselves (some Subsonic apps, a car stereo): a song converted for one is turned up or down by its
  ReplayGain (the tags', else measured), never so far that its peak passes −1 dB, and sent without ReplayGain tags
  of its own; the answer says so (`X-Levelled: track -6.20 dB`). Levelled and unlevelled copies are cached apart. Apps
  that level themselves (Encore, Symfonium, Feishin, Recital) are left alone unless listed in
  `level_conversions_clients`; a stream of the file itself, and the song's `replayGain`, never change.

### Fixed
- **Converted songs start sooner**: ffmpeg kept a few hundred KB (about 15 s of the song) before it wrote anything,
  so an app waited for that much to be converted before it got its first byte. Now each piece is sent as it's made:
  on a Raspberry Pi 4 core a song starts in 0.9 s instead of 2 s, on a desktop in 0.16 s instead of 0.33.
- **Skipping through songs on a small server**: every song skipped went on being converted to its end, so the one
  you stayed on started late (a Raspberry Pi 4, eight songs skipped a second apart: 4.1 s). A conversion nobody has
  listened to for 3 seconds is now stopped (an app seeking or asking again within that time still follows it):
  0.5 s.

## 1.1.2 (2026-11-23)

### Fixed
- **Albums still completed from a deluxe edition** (for real this time; checked against Deezer itself): an edition
  with two takes of one of your songs counted it twice ("Sticky Fingers (Super Deluxe)" has "Wild Horses (2009 Mix)"
  and "(Acoustic Version)"), so it looked like the better match. Each of your songs now counts once, and both of
  Deezer's searches are always looked at. "Sticky Fingers" shows its 10 songs. Albums are matched again.

## 1.1.1 (2026-11-22)

### Fixed
- **Albums still completed from a bigger edition**: when Deezer's exact search finds only a deluxe edition ("Sticky
  Fingers (Super Deluxe)", 33 songs) and the album itself ("Sticky Fingers (Remastered)", 10) comes up only in its
  plain search, the plain search is now looked at too. Albums matched since 1.1.0 are matched again.

## 1.1.0 (2026-11-22)

### New

- **Sound levelling for every song.** Songs whose files have no ReplayGain tags (most downloads) are measured in the
  background, gently (ffmpeg at the lowest priority, one at a time, never during a scan), and the apps get the
  measured gain as their ReplayGain, so Encore, Symfonium and Feishin play every song at the same loudness. Tags in
  the file always win; a rescan keeps what was measured; a changed file is measured again. Podium shows how far it
  is ("measured 812 of 1676"). Settings: `loudness_analysis` (on), `loudness_threads` (1).
- **Recital and share links level songs too**, with the same gains, never past a song's peak (nothing clips). A
  button beside Recital's volume turns it off.
- **Picking a music folder in your app** (Symfonium, Tempus, Ultrasonic, DSub, Feishin…) now shows only that
  folder's music: `musicFolderId` is used by artists, album lists of every kind, random songs, songs by genre, genre
  counts, starred items and search. A folder that doesn't exist is "not found" (code 70), as in Navidrome.
- **Downloading a whole album** (or an artist) from an app: `download` with an album or artist id sends a zip of the
  original files ("01 Title.flac", each album in its own folder for an artist), made as it's sent. It said "Song
  not found" before.

### Fixed
- **Two apps asking for the same conversion at once** (two devices, an app that asks twice) each started their own
  ffmpeg. The second now follows the one being made, which goes on to the end even when the app that started it
  stops listening.
- **`transcode_cache_mb = 0`**: a very short song could be done converting before it was sent, was removed from the
  cache at once, and the app got "This song can't be transcoded". 0 now means nothing is kept once it's sent; every
  conversion still plays whole.
- **`getPlaylists?username=`** was ignored: an admin now gets that user's playlists, as the Subsonic spec says, and
  anyone else asking for another user's gets an error (code 50).
- **Raw `.aac` files** were labelled `audio/mp4` (that's `.m4a`), so apps and players that go by the type could
  refuse them. They're `audio/aac` now, songs scanned before included; `.m4a` and `.mp4` stay `audio/mp4`.
- **Podcast episodes were never converted**, whatever format or `maxBitRate` the app asked for. A saved episode now
  goes through the same conversion as a song (and the same cache); one that isn't saved still comes straight from
  the podcast's server.
- **An album completed from the wrong edition**: "Sticky Fingers" (10 songs, two of them yours) was filled out with
  the 33-song Super Deluxe edition, because Deezer had only that and a remaster, both counted as "editions" alike.
  The plain album now comes first, then a remaster of it (the same songs), and only then a deluxe edition (unless
  yours is one); albums matched before are matched again.
- **Recording the API contract** (`CONDUCTOR_BLESS=1`) with only some tests running wrote each answer's shape from
  just those tests, so `search3.json` lost fields. Recording now only adds fields; one a whole run never sees is
  listed in `target/tmp/contract-unseen/`, and CI fails on it.
- **Stopping with systemd or `docker stop`** (SIGTERM) killed Conductor at once, in the middle of whatever it was
  doing; only Ctrl-C stopped it cleanly. Both now stop it the same way, as does stopping the Windows service: it
  stops taking new connections, gives songs playing and other requests under way a few seconds to finish, stops its
  background work and closes the database.
- **The Windows service is started again when it fails.** It told Windows it had stopped cleanly (exit code 0) even
  when it failed, so Windows' restart-on-failure never happened. It now reports a failure as one, and the service is
  set to be started again 10 s after a failure, then 30 s, then every minute (`install.ps1` sets this on a service
  installed by 1.0.0 too).
- **`install.ps1 -OpenFirewall` run again** (an upgrade) added another firewall rule each time. It now replaces the
  one it made, so there's one rule, for the port given last.
- **Podium's Settings, saved by two admins at once**, could lose one of the changes, or fail with an error about
  secrets.toml (both saves wrote the same temporary file). Saves now take turns, and config.toml is written whole
  under another name and then put in place, as secrets.toml already was (which stays readable only by Conductor), so
  nothing ever reads half a file.
- **Docker's health check with the address only in config.toml** (Prelude's compose file) asked port 4533, so a
  server listening elsewhere was "unhealthy" for good. The image now checks with `conductor health`, which reads the
  settings as the server does and asks its own address (exit 0 or 1; it opens nothing else).
- **The one-time `/setup` link is no longer written to the logs** (the journal, `docker logs`, conductor.log): its
  token lets whoever has it make the first admin. Conductor still prints it when it runs in a terminal; the logs say
  where to find it, and `conductor setup link` shows it (in Docker: `docker exec conductor conductor setup link`).
- **Removing the Linux package** (.deb, .rpm) left systemd still listing a `conductor.service` whose file was gone,
  until the next reboot or `systemctl daemon-reload`; that's now run on removal. The guide says how to remove the
  `conductor` user and `/var/lib/conductor` by hand, which removing the package keeps on purpose.

## 1.0.0 (2026-11-22)

1.0: from here on, the promises in [docs/STABILITY.md](docs/STABILITY.md) hold. It is 1.0.0-rc.1 with the fixes
from a third full bug hunt (a review of the server's core, its installing and running, and a run of every feature
against a real server):

### Fixed
- **An album downloaded with someone else's tags** (a song from Soulseek whose album artist is made up: "Sticky
  Fingers" by "Champagne Parker") showed only the songs you have, without the ⬇ ones to play or fetch. When the album
  artist finds nothing, the album is now also looked up by the artist all its songs have.
- **Web pages built on Windows**: each page may run only its own script, named by its hash, but a Windows checkout
  gives the pages CRLF line ends and browsers hash scripts with LF, so nothing matched and Podium, setup, Recital and
  the Jam, Chorus and share pages did nothing. Scripts are now hashed as browsers read them, and the pages are checked
  out with LF everywhere.
- **Ogg and Opus conversions** failed ("This song can't be transcoded") for 96 kHz songs, for mono songs at 320k and
  for stereo ones at 32k (Vorbis), and for mono above 256k (Opus). They're now made in stereo (Vorbis at 44.1 kHz,
  48-480 kbps), which every song converts to.
- **A conversion that fails part way** (ffmpeg killed, the disk full) breaks off, instead of being sent as a whole
  song that ends early.
- **Made for you stays the same all day**, also once you play its songs (playing one reshuffled the whole list).
- **Podium's Settings**: a Jam address that isn't one (`jam_listen`) was saved as fine and the server then didn't
  start; it's refused now. A huge `transcode_slots` made the server panic at start (now taken as 1024).
  `trial_hours` and `lidarr_match_ratio` take decimals. Six online-info addresses showed other settings'
  descriptions.
- **Search ignored case only in A-Z**: "кино" didn't find "Кино" (SQLite's LIKE folds only ASCII). `search2`/`search3` and
  the Jam and Chorus guest pages now find Cyrillic, Greek and other names typed in any case.
- **Recital, and the Jam, Chorus and share pages**, showed a broken-image icon (in Chrome, Edge and other Chromium
  browsers) for every song or album without a cover: the server answers "not found" so the page can draw its own
  square, but the pages never did (Jam's fallback picture was itself a broken GIF). They now show a plain square.
- **`conductor restore` into another data folder** (a new computer, a moved folder) put the database there but brought
  back `config.toml` still naming the old folder, so the next start used the old one and the restored library seemed
  gone. The restored settings now keep the data folder the database went to.
- **`conductor --help`**, or a mistyped command, made a database in the default data folder before saying "unknown
  command; see the top of src/main.rs". Now nothing is opened, and the commands are listed.
- **Docs**: the settings reference (also Podium's Settings help) described `deezer_url`, `info_lang`, `wikidata_url`,
  `musicbrainz_url` and `coverart_url` with another setting's text; the `encore.*` specs left out the JSON parameters
  (`state`, `command`, `value`, `login`); FEATURES.md now says `musicFolderId` isn't used (every call covers all folders).
- **Setting up the Linux package's service** (`sudo -u conductor conductor --config /var/lib/conductor/config.toml
  setup`) put the database in the conductor user's `~/.local/share/conductor`, while the service reads
  `/var/lib/conductor`: the admin made in setup wasn't there and couldn't sign in. A new settings file given with
  `--config` now keeps its data beside it (an existing one keeps what it has). If you set it up this way with 1.0.0-rc.1,
  stop the service, move the files from `/var/lib/conductor/.local/share/conductor` into `/var/lib/conductor`, and
  set `data_dir = "/var/lib/conductor"` in its config.toml.
- **A backup from 1.0.0-rc.1** would have been refused by 1.0.0 as "newer than this one": the prerelease was read as 1.0.0.1.
  A prerelease now comes before its release when a backup's version is checked.
- **`conductor restore --force`** of a backup without settings in it (a server set up with environment settings only,
  as in Docker) moved the config.toml in use aside and put nothing back: the server came up without its music
  folders. The settings in use are now only replaced by the backup's own.
- **Commands run with sudo on the Linux package's data** (`sudo conductor --config /var/lib/conductor/config.toml
  restore …`, `setup`, `user add`, …) left the database and key owned by root, and the service, running as the
  conductor user, stopped with "attempt to write a readonly database". What a command run as root makes in a data
  folder that belongs to someone else is now given to that folder's owner.
- **A podcast feed or cover of any size** was read into memory whole, and feeds are read again every 2 hours: a broken
  or hostile feed could run the server out of memory. Feeds stop at 64 MB and covers at 16 MB.
  An episode saved on the server stops at 8 GB (a live stream as an episode filled the disk for an hour), and a
  download that fails no longer leaves its `.part` file behind.
- **Upgrading on Windows with install.ps1** gave the service two seconds to stop before replacing conductor.exe; one
  slower to stop kept the file locked, the copy failed, and the service was left stopped. It now waits for the
  service to stop (only if it runs), tries the copy again for a while, and starts the old version again if it
  still can't.
- **A Soulseek file name with `..` in it** (the name is the peer's) made Conductor look for the finished download
  beside the downloads folder instead of in it, and move a file of that name from there into the music folder.
- **`conductor setup` without a terminal** (`< /dev/null`, a script, a service) asked for the admin's password (or a
  music folder) forever, writing hundreds of MB a second. When its input ends it now stops and points to `--yes`.
- **The Docker image's health check** always asked port 4533: with `-e CONDUCTOR_LISTEN=0.0.0.0:8080` the container
  was "unhealthy" for good. It now asks the port in `CONDUCTOR_LISTEN`. The image's license label also says
  AGPL-3.0-or-later now, as the project does (it said AGPL-3.0-only).
- **Upgrading the Arch package** left the old version running: pacman runs the install script only on a first
  install. An upgrade now restarts a running service, as the .deb and .rpm do.
- **A restore where only the database's journal was left** (`conductor.db-wal` without `conductor.db`, deleted by
  hand) kept that journal, which SQLite would play into the restored database it doesn't belong to. It's moved aside
  with the rest now.
- **The Windows service with settings it can't read** (a typo in config.toml) stopped without writing why anywhere,
  while install.ps1 says to look in conductor.log. The error is now written to conductor.log beside config.toml.
- **A backup stopped halfway** (Conductor stopped or the computer went off during one) left a whole copy of the
  database in `<data>/.backup-…` and an unfinished `.zip.part`, for good. The daily backup now clears ones over an
  hour old.
- **The Arch package's version** left out the prerelease (1.0.0-rc.1 was 1.0.0-1), so pacman would not have replaced
  it with 1.0.0. A prerelease is now spelled the way pacman sorts it (1.0.0rc.1), and CI checks that every installed
  package's version sorts before its release on Debian, Fedora and Arch.
- **SHA256SUMS.txt** named the .deb and .rpm files with the `~` that GitHub turns into `.`; the files are now named as
  published, so `sha256sum -c` finds them.

### New
- `RUST_LOG=conductor::api=debug` logs every Subsonic error the server answers with, and the app that got it.

## 1.0.0-rc.1 (2026-11-22)

### Fixed
- **Podium on a phone**: the username box no longer capitalises the first letter (so signing in failed); a wrong
  password says so instead of "sign in"; the open tab is marked again after switching (it lost its highlight, and
  screen readers which tab was open); labels sit above their boxes.
- **Readable buttons in dark mode** in Podium and setup (white on light blue was 2.3:1, now dark ink at ~8:1); boxes
  named only by their placeholder have that name for screen readers too; the share page has a main landmark.

### Security
- **Every web page runs only its own script** (Content-Security-Policy with the script's hash, instead of allowing any
  inline script), so markup that got into a page some other way (a tag in a song's title, say) can't run.
- **Podium and setup** had no such policy at all; they now also refuse to be shown inside another site's frame (so
  nobody can lay a page over them and have an admin click through) and aren't cached.
- **Podium's sign-in cookie is Secure** when you sign in over HTTPS through a proxy on the server (Prelude's Caddy, a
  Cloudflare Tunnel, Tailscale), so a browser never sends it over plain HTTP.
- Every answer says `nosniff` and `same-origin` referrers; setup also counts requests that came through Cloudflare or
  Tailscale as from elsewhere by their own headers.

### New
- **Linux packages**: .deb (Debian, Ubuntu), .rpm (Fedora, openSUSE) and Arch, for x86-64 and ARM64, with a system
  service that runs as its own user (not started until it's set up). Each is installed and removed again in its
  system on every change.
- **"Made for you" without AudioMuse-AI** (`encore.forYou`, `getForYou`): a mix from the artists you play most and
  those that go with them, about 60% new to you, the same all day and new each day. Encore uses it when the server has
  no AudioMuse-AI.
- **Settings in Podium**: every setting in the settings reference can be changed from the admin page (keys and
  passwords go to secrets.toml and are never shown; a value Conductor can't use isn't saved; a restart uses them).
  Music folders and the data folder stay with `conductor setup`.
- **Conversions are kept** (`<data>/cache/transcode`, `transcode_cache_mb`, 1000 MB): a song converted for an app
  (another format, a lower bitrate) is written to the cache while the app is already sent it, so ffmpeg is done in a
  second or two however slowly the app listens; the same song again comes from the cache, with seeking.
- **ffmpeg can't swamp the server**: `transcode_slots` (half the cores) run at once as a rule; when they're busy a
  song waits half a second, then starts anyway, up to four times as many; ffmpeg runs at a lower priority than
  everything else. Measured: 64 apps converting at once used every core and 64 ffmpegs (~3 GB) before; now at most
  the limit, the other services and the API stay quick, and nobody waits seconds while it's light.

## 0.5.1 (2026-11-22)

### Fixed
- **Security:** the SQLite built into Conductor had a vulnerability (CVE-2026-119375, RUSTSEC-2026-0459). It's the
  fixed one now (sqlx 0.9, libsqlite3-sys 0.40). Every place that builds SQL as it runs was checked again.
- The API's shapes are recorded and checked (docs/api/), with a spec for each of Conductor's own extensions; the list
  of extensions no longer names calls that don't exist.

## 0.5.0 (2026-11-22)

### New
- **Sounds like, without AI**: song radio, similar songs and similar artists are worked out from your library and what
  you play (played together, genres and their words, the script of names and titles, era), so they stay close
  without AudioMuse-AI or any online service. On a real library, radio after a Russian rock song now stays with
  Russian rock instead of wandering off after two songs.

### Fixed
- Related artists from Deezer were only looked up with downloads on; they're used with online info too.
- Genres written with "|" between them ("Rock | Blues | Hard Rock") are split like the others.

## 0.4.0 (2026-11-22)

### New
- **Scrobbling for every user**: everyone links their own ListenBrainz and Last.fm in Encore; plays go to each, and
  ones a service missed are sent later, in order. (Before, only the owner's plays went to ListenBrainz.)
- **A picture for albums without a cover** for apps other than Encore, as Navidrome sends, instead of "not found".
- **Conductor on your domain** (`conductor setup domain`): HTTPS on your own domain with Caddy, a Cloudflare Tunnel,
  or Tailscale Funnel, each set up for you.
- **Prelude finds your music**: it looks at the computer's drives and the music on them, and offers the folder with
  the most songs, or a new one on the drive with the most room (with the drive guard on a USB drive).
- **Prelude sets up the whole music lab for you** in one Docker folder: the downloader (optionally inside a VPN with
  a kill switch), Lidarr, AudioMuse-AI, Pitch and Metronome, and only with AI on, Ollama and Interlude's voice. Each
  one's address and keys go into Conductor's settings by themselves. Works on SELinux systems (Fedora, RHEL).
- **Downloads are optional**: without a downloader, songs you don't have aren't shown.
- **Linux ARM64** downloads and a **Docker image** (`ghcr.io/d-safro/conductor`, x86-64 and ARM64).
- **Backups**: one a day by itself, `conductor backup` and `conductor restore`.
- **Shares**, **user management** and **avatars** over the Subsonic API; the **index-based play queue**.
- **Every setting from the environment** (`CONDUCTOR_<NAME>`); a settings reference and a user guide.
- **Works with web apps** on other addresses (CORS), such as Feishin's web version.
- OpenSubsonic's song and album fields are always there, as apps expect.

### Faster
- With 100,000 songs: fetching the whole library (Symfonium and others) ~10 s instead of 2½ minutes; stars, genres
  and searches 5-100× quicker; first scan ~10 s, ~75 MB of memory.

### Fixed
- Setting up a new server with an export to bring in (`--transpose`, or typed in Prelude): the songs were matched
  before the music was scanned, so none matched, and with `--yes` nothing came in at all. The music is scanned
  first now, and an export you name is brought in unless you say no.
- Setting up next to another Conductor: the Jam listener's port (4535) was taken, and Jam and share links quietly
  didn't work. Prelude picks a free port near it and says so.
- MP3s whose Cyrillic tags were written as Windows-1251 but marked Latin-1 showed garbled names ("Àêâàðèóì" for
  «Аквариум»), and the same for UTF-8 tags marked Latin-1 ("ÐÐ½Ð´ÑÐµÐ¹" for «Андрей»); they're read right now, songs
  already scanned too (read again once). Western accents stay.
- Plays an app hands in together (made offline) are sent to ListenBrainz and Last.fm in order, one after another.
- A ListenBrainz token that ListenBrainz stops accepting is shown as needing a new link, as Last.fm's are.
- Plays waiting to be sent again: one user's long wait (500 or more, their service turning them away for now) no
  longer keeps everyone else's waiting plays from being sent.
- Podcasts: changes are for admins, and feeds can't reach your own network (`podcast_local` allows it).
- A smart playlist could read someone else's private playlist.
- Phone sign-in codes and wrong passwords are limited per address.
- "What's playing?" failed for every recording from an Android phone (M4A through a pipe).
- AudioMuse was given songs you don't have to analyse.
- A service's account (AudioMuse's) stopped the first admin from being made.
- Interlude's voice failed while it was still downloading.
- An export with a certain kind of date crashed the import.
- `getUser` answered for any name.
- Prelude's domain step: with Conductor on one of the computer's addresses (not 0.0.0.0), "only Caddy reaches it"
  moved Conductor to 127.0.0.1 after the Caddyfile was written, so Caddy pointed at nothing. Run again, it no longer
  moves Conductor back to 127.0.0.1 unasked; an email that isn't one is no longer written into the Caddyfile.
- A better copy swapped in for a song with another file type (FLAC for an MP3) could leave the song twice in the
  library: the old file went only after scans were let run again.
- Prelude's look at the drives: music found on a slow drive just as its time ran out was left out as empty; on
  Windows a music folder on a USB drive wasn't seen as on that drive (no drive guard offered); `--music auto` in a
  terminal, then "Type a path", offered "auto" as the folder.
- Prelude's import step stopped with an error (pointing at a `--replace` it doesn't have) when given a Navidrome
  database on a server already in use; it now asks before replacing, and with `--yes` leaves things as they are.
- Choosing Tailscale Funnel in Prelude's domain step left Caddy or the Cloudflare Tunnel set up before in place,
  still opening Conductor on its old name; they're now offered to be taken out, as switching between those two does.
- A downloaded song filed over an hour before was given up on ("not found", the hearts and playlists it was for
  never applied) while a long scan of the library was running, though its file was there.

## 0.2.1 (2026-11-22)
- Moving from Navidrome keeps your password.

## 0.2.0 (2026-11-22)
The first release as Conductor (it was Maestro): Prelude, the setup wizard, with a VPN for downloads; Recital, the
web player; Transpose, imports from Spotify, Apple Music, YouTube Music, Last.fm, ListenBrainz, Deezer and playlist
files; better copies of downloads; picking files by hand; artist bios; Encore 1.9 and 1.10's features; Windows
service; 17 fixes from a full review.

## 0.1.0-rc1
The first test version, as Maestro: a Subsonic server for Encore, standing in for Navidrome and music-lookup.
