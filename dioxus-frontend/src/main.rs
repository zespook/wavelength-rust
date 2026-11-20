mod app;
mod browser;
pub(crate) mod components;

#[cfg(feature = "desktop")]
mod desktop;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod audio_id_tests {
    use super::Track;

    #[test]
    fn empty_shared_audio_ids_are_treated_as_missing() {
        let missing: Track = serde_json::from_str(r#"{"audioId":""}"#).unwrap();
        let shared: Track = serde_json::from_str(
            r#"{"audioId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.flac"}"#,
        )
        .unwrap();

        assert_eq!(missing.audio_id, None);
        assert_eq!(
            shared.audio_id.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.flac")
        );
    }
}

use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

const AUDIO_SERVICE: &str = "http://127.0.0.1:8787";

fn now_epoch_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn random_unit_f64() -> f64 {
    rand::random()
}

fn encode_uri_component(value: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::encode_uri_component(value)
            .as_string()
            .unwrap_or_else(|| value.to_string())
    }
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    {
        url::form_urlencoded::byte_serialize(value.as_bytes())
            .collect::<String>()
            .replace('+', "%20")
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
    {
        value.to_string()
    }
}

async fn sleep_ms(milliseconds: u64) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(milliseconds.min(u32::MAX as u64) as u32).await;
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    tokio::time::sleep(std::time::Duration::from_millis(milliseconds)).await;
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
    std::thread::sleep(std::time::Duration::from_millis(milliseconds));
}

fn timestamp_millis(value: &serde_json::Value) -> Option<f64> {
    value
        .as_i64()
        .map(|milliseconds| milliseconds as f64)
        .or_else(|| value.as_f64())
        .or_else(|| {
            value
                .as_str()
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                .map(|date| date.timestamp_millis() as f64)
        })
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct UpdateInfo {
    version: String,
    notes: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    NowPlaying,
    Search,
    Library,
    SpotifyLibrary,
    SpotifyPlaylists,
    SpotifyPlaylistDetail,
    Playlists,
    PlaylistDetail,
    AlbumDetail,
    History,
    Settings,
    Activity,
    Equalizer,
    Customization,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RightPanel {
    Queue,
    Room,
}

impl Page {
    fn class(self) -> &'static str {
        match self {
            Self::Home => "page-home",
            Self::NowPlaying => "page-now-playing",
            Self::Search => "page-search",
            Self::Library => "page-library",
            Self::SpotifyLibrary => "page-library",
            Self::SpotifyPlaylists => "page-playlists",
            Self::SpotifyPlaylistDetail => "page-playlists",
            Self::Playlists => "page-playlists",
            Self::PlaylistDetail => "page-playlists",
            Self::AlbumDetail => "page-library",
            Self::History => "page-history",
            Self::Settings => "page-settings",
            Self::Activity => "page-activity",
            Self::Equalizer => "page-equalizer",
            Self::Customization => "page-customization",
        }
    }
}

fn go_back(
    mut page: Signal<Page>,
    mut back_stack: Signal<Vec<Page>>,
    mut forward_stack: Signal<Vec<Page>>,
    mut navigating_history: Signal<bool>,
) {
    let mut back = back_stack();
    let Some(target) = back.pop() else {
        return;
    };
    let mut forward = forward_stack();
    forward.push(page());
    back_stack.set(back);
    forward_stack.set(forward);
    navigating_history.set(true);
    page.set(target);
}

fn go_forward(
    mut page: Signal<Page>,
    mut back_stack: Signal<Vec<Page>>,
    mut forward_stack: Signal<Vec<Page>>,
    mut navigating_history: Signal<bool>,
) {
    let mut forward = forward_stack();
    let Some(target) = forward.pop() else {
        return;
    };
    let mut back = back_stack();
    back.push(page());
    back_stack.set(back);
    forward_stack.set(forward);
    navigating_history.set(true);
    page.set(target);
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Track {
    #[serde(default)]
    id: String,
    #[serde(default, rename = "videoId")]
    video_id: Option<String>,
    #[serde(default)]
    title: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    length: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default, rename = "folderPath")]
    folder_path: Option<String>,
    #[serde(default, rename = "audioId")]
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    audio_id: Option<String>,
    #[serde(default, rename = "modifiedAt")]
    modified_at: Option<u64>,
}

fn deserialize_non_empty_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.filter(|value| !value.trim().is_empty()))
}

impl Track {
    fn key(&self) -> &str {
        if self.id.is_empty() {
            self.video_id.as_deref().unwrap_or_default()
        } else {
            &self.id
        }
    }

    fn audio_service_id(&self) -> String {
        let key = self.key();
        if key.is_empty() {
            return String::new();
        }
        if self
            .source
            .as_deref()
            .is_some_and(|source| source.eq_ignore_ascii_case("youtube"))
            && !key.starts_with("yt-")
        {
            format!("yt-{key}")
        } else {
            key.to_string()
        }
    }

    fn is_local(&self) -> bool {
        self.source.as_deref() == Some("local") || self.folder_path.is_some()
    }

    fn is_spotify(&self) -> bool {
        self.source
            .as_deref()
            .is_some_and(|source| source.eq_ignore_ascii_case("spotify"))
            || self.id.starts_with("spotify:track:")
    }

    fn duration_label(&self) -> String {
        if let Some(length) = self.length.as_ref().filter(|s| !s.is_empty()) {
            return length.clone();
        }
        self.duration
            .filter(|n| n.is_finite() && *n >= 0.0)
            .map(|n| format!("{}:{:02}", n as u64 / 60, n as u64 % 60))
            .unwrap_or_else(|| "—".to_string())
    }
}

fn latest_track_modified_at(tracks: &[Track]) -> Option<u64> {
    tracks
        .iter()
        .filter(|track| track.is_local())
        .filter_map(|track| track.modified_at)
        .max()
}

fn sort_track_collections<T, F>(collections: &mut [T], tracks_for: F)
where
    F: for<'a> Fn(&'a T) -> &'a [Track],
{
    collections
        .sort_by_cached_key(|item| std::cmp::Reverse(latest_track_modified_at(tracks_for(item))));
}

#[cfg(test)]
mod track_collection_sort_tests {
    use super::{Track, sort_track_collections};

    #[test]
    fn sorts_collections_by_newest_local_track_modification() {
        let track = |id: &str, source: &str, modified_at| Track {
            id: id.to_string(),
            source: Some(source.to_string()),
            modified_at,
            ..Track::default()
        };
        let mut collections = vec![
            ("Older".to_string(), vec![track("old", "local", Some(100))]),
            (
                "Undated".to_string(),
                vec![track("remote", "spotify", Some(300))],
            ),
            ("Newer".to_string(), vec![track("new", "local", Some(200))]),
        ];

        sort_track_collections(&mut collections, |(_, tracks)| tracks);

        assert_eq!(
            collections
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["Newer", "Older", "Undated"]
        );
    }
}

fn search_local_tracks(tracks: &[Track], query: &str) -> Vec<Track> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }

    tracks
        .iter()
        .filter(|track| {
            track.is_local()
                && format!(
                    "{} {} {}",
                    track.title,
                    track.artist,
                    track.album.as_deref().unwrap_or_default()
                )
                .to_lowercase()
                .contains(&query)
        })
        .cloned()
        .collect()
}

fn normalized_search_match(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn preferred_local_track(track: Track, local_tracks: &[Track]) -> Track {
    if track.is_local() {
        return track;
    }

    let online_title = normalized_search_match(&track.title);
    let online_artist = normalized_search_match(
        track
            .artist
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .trim_end_matches("- Topic")
            .trim_end_matches("VEVO")
            .trim(),
    );
    if online_title.is_empty() || online_artist.is_empty() || online_artist == "unknownartist" {
        return track;
    }

    local_tracks
        .iter()
        .filter(|candidate| candidate.is_local())
        .filter_map(|candidate| {
            let local_title = normalized_search_match(&candidate.title);
            let local_artist = normalized_search_match(
                candidate
                    .artist
                    .split(',')
                    .next()
                    .unwrap_or_default()
                    .trim(),
            );
            if local_title != online_title || local_artist != online_artist {
                return None;
            }

            let duration_difference = match (
                track
                    .duration
                    .filter(|duration| duration.is_finite() && *duration > 0.0),
                candidate
                    .duration
                    .filter(|duration| duration.is_finite() && *duration > 0.0),
            ) {
                (Some(online), Some(local)) => {
                    let difference = (online - local).abs();
                    if difference > 5.0_f64.max(online * 0.03) {
                        return None;
                    }
                    difference
                }
                _ => 0.0,
            };

            Some((candidate, duration_difference))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(candidate, _)| candidate.clone())
        .unwrap_or(track)
}

#[derive(Clone, Debug, PartialEq)]
struct TrackContextMenu {
    track: Track,
    x: f64,
    y: f64,
}

fn merge_local_tracks(existing: Vec<Track>, incoming: Vec<Track>) -> Vec<Track> {
    let replaced_roots = incoming
        .iter()
        .filter_map(|track| track.folder_path.as_deref()?.split('/').next())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut tracks = existing
        .into_iter()
        .filter(|track| {
            let Some(path) = track.folder_path.as_deref() else {
                return true;
            };
            let root = path.split('/').next().unwrap_or_default();
            !replaced_roots.iter().any(|folder| folder == root)
        })
        .collect::<Vec<_>>();
    tracks.extend(incoming);
    tracks
}

fn group_local_tracks(tracks: Vec<Track>, sort_by: &str) -> Vec<(String, Vec<Track>)> {
    let mut groups = Vec::<(String, Vec<Track>)>::new();
    for track in tracks {
        let name = match sort_by {
            "album" => track
                .album
                .as_deref()
                .filter(|value| !value.is_empty())
                .unwrap_or("Singles"),
            "artist" if track.artist.trim().is_empty() => "Unknown artist",
            "artist" => track.artist.as_str(),
            _ => track
                .folder_path
                .as_deref()
                .filter(|value| !value.is_empty())
                .unwrap_or("Imported"),
        }
        .to_string();
        if let Some((_, grouped_tracks)) = groups.iter_mut().find(|(group, _)| group == &name) {
            grouped_tracks.push(track);
        } else {
            groups.push((name, vec![track]));
        }
    }

    groups.sort_by(|left, right| left.0.to_lowercase().cmp(&right.0.to_lowercase()));
    for (_, grouped_tracks) in &mut groups {
        grouped_tracks.sort_by(|left, right| {
            if sort_by == "artist" {
                let albums = left
                    .album
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .cmp(&right.album.as_deref().unwrap_or_default().to_lowercase());
                if albums != std::cmp::Ordering::Equal {
                    return albums;
                }
            }
            left.title.to_lowercase().cmp(&right.title.to_lowercase())
        });
    }
    groups
}

#[derive(Clone, Debug, Default, PartialEq)]
struct LocalFolderNode {
    name: String,
    path: String,
    track_count: usize,
    tracks: Vec<Track>,
    children: Vec<LocalFolderNode>,
}

#[derive(Clone, Debug, PartialEq)]
struct LocalFolderRow {
    name: String,
    path: String,
    depth: usize,
    track_count: usize,
    expanded: bool,
    tracks: Vec<Track>,
}

fn local_folder_rows(tracks: &[Track], expanded_paths: &[String]) -> Vec<LocalFolderRow> {
    let mut roots = Vec::new();
    for track in tracks {
        let folder_path = track
            .folder_path
            .as_deref()
            .filter(|path| !path.is_empty())
            .or(track.album.as_deref())
            .unwrap_or("Imported");
        let segments = folder_path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();
        insert_local_folder_track(&mut roots, &segments, "", track);
    }

    let mut rows = Vec::new();
    flatten_local_folder_rows(&mut roots, 0, expanded_paths, &mut rows);
    rows
}

fn insert_local_folder_track(
    siblings: &mut Vec<LocalFolderNode>,
    segments: &[&str],
    parent_path: &str,
    track: &Track,
) {
    let Some((name, remaining)) = segments.split_first() else {
        return;
    };
    let path = if parent_path.is_empty() {
        (*name).to_string()
    } else {
        format!("{parent_path}/{name}")
    };
    let index = siblings
        .iter()
        .position(|node| node.name == *name)
        .unwrap_or_else(|| {
            siblings.push(LocalFolderNode {
                name: (*name).to_string(),
                path: path.clone(),
                ..LocalFolderNode::default()
            });
            siblings.len() - 1
        });
    let node = &mut siblings[index];
    node.track_count += 1;
    if remaining.is_empty() {
        node.tracks.push(track.clone());
    } else {
        insert_local_folder_track(&mut node.children, remaining, &path, track);
    }
}

fn flatten_local_folder_rows(
    nodes: &mut [LocalFolderNode],
    depth: usize,
    expanded_paths: &[String],
    rows: &mut Vec<LocalFolderRow>,
) {
    nodes.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    for node in nodes {
        node.tracks
            .sort_by(|left, right| left.title.to_lowercase().cmp(&right.title.to_lowercase()));
        let expanded = expanded_paths.iter().any(|path| path == &node.path);
        rows.push(LocalFolderRow {
            name: node.name.clone(),
            path: node.path.clone(),
            depth,
            track_count: node.track_count,
            expanded,
            tracks: node.tracks.clone(),
        });
        if expanded {
            flatten_local_folder_rows(&mut node.children, depth + 1, expanded_paths, rows);
        }
    }
}

fn reordered_queue<T>(
    mut items: Vec<T>,
    source_index: usize,
    target_index: usize,
) -> Option<Vec<T>> {
    if source_index >= items.len() || target_index >= items.len() || source_index == target_index {
        return None;
    }
    let track = items.remove(source_index);
    items.insert(target_index, track);
    Some(items)
}

fn queue_source_label(track: &Track) -> &'static str {
    let source = track.source.as_deref().unwrap_or_default();
    if source.eq_ignore_ascii_case("youtube") {
        return "YouTube";
    }

    let has_flac_extension = [
        track.id.as_str(),
        track.audio_id.as_deref().unwrap_or_default(),
    ]
    .iter()
    .any(|value| value.to_ascii_lowercase().ends_with(".flac"));
    if source.eq_ignore_ascii_case("flac") || has_flac_extension {
        return "FLAC";
    }
    if source.eq_ignore_ascii_case("local") || track.folder_path.is_some() {
        return "Local";
    }
    if source.eq_ignore_ascii_case("spotify") {
        return "Spotify";
    }
    if source.eq_ignore_ascii_case("qobuz") {
        return "Qobuz";
    }

    "Audio"
}

fn queue_source_class(track: &Track) -> &'static str {
    match queue_source_label(track) {
        "YouTube" => "youtube",
        "Spotify" => "spotify",
        "Qobuz" => "qobuz",
        "Local" => "local",
        "FLAC" => "flac",
        _ => "audio",
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Playlist {
    id: String,
    name: String,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    tracks: Vec<Track>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct DownloadStatus {
    status: String,
    progress: Option<f64>,
    error: Option<String>,
    downloaded: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DownloadFilter {
    #[default]
    All,
    Downloaded,
    NotDownloaded,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct DownloadStatusResponse {
    #[serde(default)]
    tracks: std::collections::HashMap<String, DownloadStatus>,
}

fn matches_download_filter(
    track: &Track,
    filter: DownloadFilter,
    statuses: &std::collections::HashMap<String, DownloadStatus>,
) -> bool {
    let downloaded = track.is_local()
        || statuses
            .get(&track.audio_service_id())
            .is_some_and(|status| status.downloaded);
    match filter {
        DownloadFilter::All => true,
        DownloadFilter::Downloaded => downloaded,
        DownloadFilter::NotDownloaded => !downloaded && !track.is_local(),
    }
}

#[cfg(test)]
mod download_filter_tests {
    use super::{
        DownloadFilter, DownloadStatus, Playlist, RoomSharedPlaylist, SpotifyPlaylist,
        SpotifyProfile, Track, library_playlist_count, matches_download_filter,
        spotify_playlist_is_owned,
    };

    #[test]
    fn spotify_playlist_editing_requires_matching_owner() {
        let profile = SpotifyProfile {
            id: "user-1".to_string(),
            ..SpotifyProfile::default()
        };
        let owned = SpotifyPlaylist {
            id: "owned".to_string(),
            owner_id: "user-1".to_string(),
            ..SpotifyPlaylist::default()
        };
        let shared = SpotifyPlaylist {
            id: "shared".to_string(),
            owner_id: "other-user".to_string(),
            ..SpotifyPlaylist::default()
        };
        let unknown_owner = SpotifyPlaylist {
            id: "unknown".to_string(),
            ..SpotifyPlaylist::default()
        };

        assert!(spotify_playlist_is_owned(Some(&profile), &owned));
        assert!(!spotify_playlist_is_owned(Some(&profile), &shared));
        assert!(!spotify_playlist_is_owned(Some(&profile), &unknown_owner));
        assert!(!spotify_playlist_is_owned(None, &owned));
    }

    #[test]
    fn downloaded_filter_treats_local_tracks_as_available() {
        let track = Track {
            id: "local-1".to_string(),
            source: Some("local".to_string()),
            ..Track::default()
        };
        let statuses = std::collections::HashMap::new();

        assert!(matches_download_filter(
            &track,
            DownloadFilter::Downloaded,
            &statuses
        ));
        assert!(!matches_download_filter(
            &track,
            DownloadFilter::NotDownloaded,
            &statuses
        ));
    }

    #[test]
    fn downloaded_filter_requires_ready_status_for_remote_tracks() {
        let ready = Track {
            id: "remote-ready".to_string(),
            ..Track::default()
        };
        let queued = Track {
            id: "remote-queued".to_string(),
            ..Track::default()
        };
        let statuses = std::collections::HashMap::from([
            (
                ready.audio_service_id(),
                DownloadStatus {
                    status: "ready".to_string(),
                    downloaded: true,
                    ..DownloadStatus::default()
                },
            ),
            (
                queued.audio_service_id(),
                DownloadStatus {
                    status: "queued".to_string(),
                    ..DownloadStatus::default()
                },
            ),
        ]);

        assert!(matches_download_filter(
            &ready,
            DownloadFilter::Downloaded,
            &statuses
        ));
        assert!(!matches_download_filter(
            &queued,
            DownloadFilter::Downloaded,
            &statuses
        ));
        assert!(matches_download_filter(
            &queued,
            DownloadFilter::NotDownloaded,
            &statuses
        ));
    }

    #[test]
    fn downloaded_filter_excludes_remote_tracks_only_cached_for_playback() {
        let cached = Track {
            id: "remote-cached".to_string(),
            ..Track::default()
        };
        let statuses = std::collections::HashMap::from([(
            cached.audio_service_id(),
            DownloadStatus {
                status: "ready".to_string(),
                downloaded: false,
                ..DownloadStatus::default()
            },
        )]);

        assert!(!matches_download_filter(
            &cached,
            DownloadFilter::Downloaded,
            &statuses
        ));
        assert!(matches_download_filter(
            &cached,
            DownloadFilter::NotDownloaded,
            &statuses
        ));
    }

    #[test]
    fn library_playlist_count_excludes_favorites_and_unconnected_spotify() {
        let local = vec![
            Playlist {
                id: "local-1".to_string(),
                name: "Local playlist".to_string(),
                ..Playlist::default()
            },
            Playlist {
                id: "favorites".to_string(),
                name: "Favorite Songs".to_string(),
                ..Playlist::default()
            },
        ];
        let shared = vec![RoomSharedPlaylist {
            id: "shared-1".to_string(),
            name: "Shared playlist".to_string(),
            ..RoomSharedPlaylist::default()
        }];
        let spotify = vec![
            SpotifyPlaylist {
                id: "spotify-1".to_string(),
                name: "Spotify playlist".to_string(),
                ..SpotifyPlaylist::default()
            },
            SpotifyPlaylist {
                id: "spotify-favorites".to_string(),
                name: "Favorite Songs".to_string(),
                ..SpotifyPlaylist::default()
            },
        ];

        assert_eq!(library_playlist_count(&local, &shared, &spotify, false), 2);
        assert_eq!(library_playlist_count(&local, &shared, &spotify, true), 3);
    }

    #[test]
    fn local_playlist_artwork_is_optional_and_persisted() {
        let mut playlist: Playlist =
            serde_json::from_str(r#"{"id":"local-1","name":"Local mix","tracks":[]}"#).unwrap();
        assert_eq!(playlist.thumbnail, None);

        playlist.thumbnail = Some("https://images.example/cover.jpg".to_string());
        let saved = serde_json::to_value(playlist).unwrap();
        assert_eq!(
            saved.get("thumbnail").and_then(serde_json::Value::as_str),
            Some("https://images.example/cover.jpg")
        );
    }
}

fn refresh_library_download_status(
    mut tracks: Vec<Track>,
    include_spotify_library: bool,
    mut spotify_tracks: Signal<Vec<Track>>,
    mut spotify_busy: Signal<bool>,
    mut download_status: Signal<std::collections::HashMap<String, DownloadStatus>>,
    mut download_status_busy: Signal<bool>,
    mut download_status_error: Signal<String>,
) {
    download_status_busy.set(true);
    download_status_error.set(String::new());
    spawn(async move {
        let mut errors = Vec::new();
        if include_spotify_library && spotify_tracks().is_empty() {
            spotify_busy.set(true);
            match browser::spotify_library().await {
                Ok(items) => {
                    tracks.extend(items.clone());
                    spotify_tracks.set(items);
                }
                Err(error) => errors.push(format!("Could not load Spotify saved tracks: {error}")),
            }
            spotify_busy.set(false);
        } else if include_spotify_library {
            tracks.extend(spotify_tracks());
        }

        let mut unique_tracks = Vec::new();
        let mut seen_tracks = std::collections::HashSet::new();
        for track in tracks {
            if !track.key().is_empty() && seen_tracks.insert(track.key().to_string()) {
                unique_tracks.push(track);
            }
        }

        let mut track_ids = unique_tracks
            .iter()
            .filter(|track| !track.is_local())
            .map(Track::audio_service_id)
            .filter(|id| !id.is_empty())
            .collect::<Vec<_>>();
        track_ids.sort();
        track_ids.dedup();

        let mut statuses = std::collections::HashMap::new();
        for chunk in track_ids.chunks(50) {
            match browser::download_status(chunk).await {
                Ok(items) => statuses.extend(items),
                Err(error) => {
                    errors.push(format!("Could not load download status: {error}"));
                    break;
                }
            }
        }
        download_status.set(statuses);
        download_status_error.set(errors.join("\n"));
        download_status_busy.set(false);
    });
}

fn add_track_to_playlist(
    track: Track,
    playlist_id: &str,
    mut playlists: Signal<Vec<Playlist>>,
    shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    room_role: Signal<String>,
    local_tracks: Signal<Vec<Track>>,
    favorites: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    queue: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    let mut all = playlists();
    let target = if let Some(target) = all.iter_mut().find(|item| item.id == playlist_id) {
        target
    } else {
        if room_role() != "host" {
            notice.set("Only the room host can edit a shared playlist.".to_string());
            return;
        }
        let Some(shared) = shared_playlists()
            .into_iter()
            .find(|playlist| playlist.id == playlist_id)
        else {
            notice.set("That playlist is no longer available.".to_string());
            return;
        };
        all.push(resolve_room_playlist(
            &shared,
            &local_tracks(),
            &favorites(),
            &history(),
            &queue(),
        ));
        all.last_mut().expect("the shared playlist was just added")
    };

    if target.tracks.iter().any(|item| item.key() == track.key()) {
        notice.set(format!("{} is already in {}.", track.title, target.name));
        return;
    }

    let playlist_name = target.name.clone();
    target.tracks.push(track.clone());
    playlists.set(all.clone());
    match browser::save("wavelength-playlists", &all) {
        Ok(()) => notice.set(format!("{} added to {}.", track.title, playlist_name)),
        Err(error) => notice.set(error),
    }
}

fn playlist_matches(query: &str, name: &str) -> bool {
    let query = query.trim();
    query.is_empty() || name.to_lowercase().contains(&query.to_lowercase())
}

fn album_matches(query: &str, name: &str, tracks: &[Track]) -> bool {
    playlist_matches(query, name)
        || tracks
            .iter()
            .any(|track| playlist_matches(query, &track.artist))
}

fn selected_favorite_tracks(favorites: &[Track], selected_keys: &[String]) -> Vec<Track> {
    favorites
        .iter()
        .filter(|track| selected_keys.iter().any(|key| key.as_str() == track.key()))
        .cloned()
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct SpotifyProfile {
    #[serde(default)]
    id: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    images: Vec<SpotifyImage>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct SpotifyImage {
    #[serde(default)]
    url: String,
}

#[derive(Deserialize)]
struct SpotifyRestore {
    profile: SpotifyProfile,
    connected_now: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct SpotifyPlaylist {
    id: String,
    name: String,
    #[serde(default)]
    tracks_total: usize,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    owner_id: String,
    #[serde(default)]
    owner_name: String,
}

fn spotify_playlist_is_owned(profile: Option<&SpotifyProfile>, playlist: &SpotifyPlaylist) -> bool {
    profile.is_some_and(|profile| {
        !profile.id.is_empty()
            && !playlist.owner_id.is_empty()
            && playlist.owner_id == profile.id
    })
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
struct SpotifyAlbum {
    id: String,
    name: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    tracks_total: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum PlaylistMenuTarget {
    Favorites,
    Album { name: String, tracks: Vec<Track> },
    SpotifyAlbum(SpotifyAlbum),
    Local(Playlist),
    Shared(RoomSharedPlaylist),
    Spotify(SpotifyPlaylist),
}

#[derive(Clone, Debug, PartialEq)]
struct PlaylistContextMenu {
    target: PlaylistMenuTarget,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct EqualizerPreset {
    id: String,
    name: String,
    #[serde(default)]
    gains: Vec<f64>,
    #[serde(default)]
    frequencies: Vec<f64>,
    #[serde(default)]
    qs: Vec<f64>,
    #[serde(default)]
    types: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct EqualizerSettings {
    gains: Vec<f64>,
    frequencies: Vec<f64>,
    qs: Vec<f64>,
    types: Vec<String>,
    enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RoomPermissions {
    can_add: bool,
    can_manage_queue: bool,
    can_control_playback: bool,
}

#[derive(Clone, Copy)]
enum RoomPermission {
    AddTracks,
    ManageQueue,
    ControlPlayback,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RoomParty {
    code: String,
    members: Vec<String>,
    host: String,
    queue: Vec<Track>,
    playlists: Vec<RoomSharedPlaylist>,
    current_track_id: String,
    is_playing: bool,
    start_epoch: Option<i64>,
    paused_at: f64,
    repeat_mode: String,
    shuffle: bool,
    permissions: RoomPermissions,
    sync_revision: i64,
    host_authorized: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RoomSession {
    code: String,
    role: String,
    name: String,
    host_token: String,
    room_url: String,
    access_token: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct RoomSharedPlaylist {
    id: String,
    name: String,
    tracks: Vec<RoomPlaylistTrackReference>,
}

fn library_playlist_count(
    local_playlists: &[Playlist],
    shared_playlists: &[RoomSharedPlaylist],
    spotify_playlists: &[SpotifyPlaylist],
    spotify_connected: bool,
) -> usize {
    let is_library_playlist = |name: &str| !name.eq_ignore_ascii_case("Favorite Songs");
    let local_count = local_playlists
        .iter()
        .filter(|playlist| is_library_playlist(&playlist.name))
        .count();
    let shared_count = shared_playlists
        .iter()
        .filter(|playlist| is_library_playlist(&playlist.name))
        .count();
    let spotify_count = if spotify_connected {
        spotify_playlists
            .iter()
            .filter(|playlist| is_library_playlist(&playlist.name))
            .count()
    } else {
        0
    };

    local_count + shared_count + spotify_count
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct RoomPlaylistTrackReference {
    id: String,
    platform: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct RoomSocketEvent {
    event: String,
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivityLog {
    #[serde(default)]
    timestamp: serde_json::Value,
    #[serde(default)]
    level: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    message: String,
    #[serde(default)]
    track_id: Option<String>,
    #[serde(default)]
    progress: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct ActivityLogsResponse {
    #[serde(default)]
    logs: Vec<ActivityLog>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct NotificationLog {
    #[serde(default)]
    timestamp: serde_json::Value,
    #[serde(default)]
    message: String,
}

fn record_notification(mut logs: Signal<Vec<NotificationLog>>, message: &str) {
    let saved_logs = logs.with_mut(|entries| {
        entries.insert(
            0,
            NotificationLog {
                timestamp: serde_json::json!(now_epoch_millis()),
                message: message.to_string(),
            },
        );
        entries.truncate(200);
        entries.clone()
    });
    if let Err(error) = browser::save("wavelength-notification-logs", &saved_logs) {
        eprintln!("Could not save notification activity: {error}");
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayEvent {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    played_at: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct QobuzSettings {
    #[serde(default)]
    configured: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyricsResponse {
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct LyricLine {
    time: Option<f64>,
    text: String,
}

#[cfg(feature = "desktop")]
fn main() {
    desktop::run(app::App);
}

#[cfg(all(not(feature = "desktop"), feature = "web"))]
fn main() {
    dioxus::launch(app::App);
}

fn load_equalizer_values(
    key: &str,
    defaults: Vec<f64>,
    min: f64,
    max: f64,
) -> Result<Vec<f64>, String> {
    match browser::load::<Vec<f64>>(key)? {
        Some(values) if values.len() == 10 && values.iter().all(|value| value.is_finite()) => {
            Ok(values
                .into_iter()
                .map(|value| value.clamp(min, max))
                .collect())
        }
        Some(_) => Err(format!("Saved {key} values are invalid.")),
        None => Ok(defaults),
    }
}

fn normalize_eq_values(values: &[f64], defaults: &[f64], min: f64, max: f64) -> Vec<f64> {
    if values.len() == 10 && values.iter().all(|value| value.is_finite()) {
        values.iter().map(|value| value.clamp(min, max)).collect()
    } else {
        defaults.to_vec()
    }
}

fn normalize_eq_types(values: &[String]) -> Vec<String> {
    if values.len() == 10 && values.iter().all(|value| equalizer_type_valid(value)) {
        values.to_vec()
    } else {
        vec!["peaking".to_string(); 10]
    }
}

fn equalizer_type_valid(value: &str) -> bool {
    matches!(value, "peaking" | "lowshelf" | "highshelf")
}

fn equalizer_builtin_preset(name: &str) -> Option<EqualizerPreset> {
    let gains = equalizer_preset(name)?.to_vec();
    let names = [
        ("flat", "Flat"),
        ("bass", "Bass boost"),
        ("vocal", "Vocal"),
        ("treble", "Treble boost"),
        ("rock", "Rock"),
    ];
    let preset_name = names.iter().find(|(id, _)| *id == name)?.1;
    Some(EqualizerPreset {
        id: name.to_string(),
        name: preset_name.to_string(),
        gains,
        frequencies: vec![
            60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0, 14000.0, 16000.0,
        ],
        qs: vec![1.1; 10],
        types: vec!["peaking".to_string(); 10],
    })
}

fn format_frequency(frequency: f64) -> String {
    if frequency >= 1000.0 {
        let value = frequency / 1000.0;
        if value.fract() == 0.0 {
            format!("{} kHz", value as u32)
        } else {
            format!("{value:.1} kHz")
        }
    } else {
        format!("{} Hz", frequency.round() as u32)
    }
}

fn equalizer_frequency_from_input(value: &str) -> Option<f64> {
    let value = value.trim().parse::<f64>().ok()?;
    value
        .is_finite()
        .then_some(value.clamp(20.0, 20_000.0).round())
}

fn equalizer_gain_from_input(value: &str) -> Option<f64> {
    let value = value.trim().parse::<f64>().ok()?;
    value
        .is_finite()
        .then_some((value.clamp(-18.0, 18.0) * 10.0).round() / 10.0)
}

fn equalizer_frequency_slider(frequency: f64) -> u32 {
    ((frequency.clamp(20.0, 20000.0) / 20.0).ln() / 1000.0f64.ln() * 1000.0).round() as u32
}

fn equalizer_frequency_from_slider(position: f64) -> f64 {
    (20.0 * 1000.0f64.powf(position.clamp(0.0, 1000.0) / 1000.0)).round()
}

const EQ_PLOT_LEFT: f64 = 44.0;
const EQ_PLOT_RIGHT: f64 = 14.0;
const EQ_PLOT_TOP: f64 = 16.0;
const EQ_PLOT_BOTTOM: f64 = 30.0;

fn equalizer_plot_size(width: f64, height: f64) -> (f64, f64) {
    (
        (width - EQ_PLOT_LEFT - EQ_PLOT_RIGHT).max(1.0),
        (height - EQ_PLOT_TOP - EQ_PLOT_BOTTOM).max(1.0),
    )
}

fn equalizer_frequency_to_x(frequency: f64, width: f64) -> f64 {
    let (plot_width, _) = equalizer_plot_size(width, 0.0);
    EQ_PLOT_LEFT + (frequency.clamp(20.0, 20000.0) / 20.0).ln() / 1000.0f64.ln() * plot_width
}

fn equalizer_gain_to_y(gain: f64, height: f64) -> f64 {
    let (_, plot_height) = equalizer_plot_size(0.0, height);
    EQ_PLOT_TOP + (18.0 - gain.clamp(-18.0, 18.0)) / 36.0 * plot_height
}

fn equalizer_graph_values(x: f64, y: f64, width: f64, height: f64) -> (f64, f64) {
    let (plot_width, plot_height) = equalizer_plot_size(width, height);
    let x_ratio = ((x - EQ_PLOT_LEFT) / plot_width).clamp(0.0, 1.0);
    let y_ratio = ((y - EQ_PLOT_TOP) / plot_height).clamp(0.0, 1.0);
    let frequency = (20.0 * 1000.0f64.powf(x_ratio)).round();
    let gain = ((18.0 - y_ratio * 36.0) * 10.0).round() / 10.0;
    (frequency, gain.clamp(-18.0, 18.0))
}

fn nearest_equalizer_band(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    frequencies: &[f64],
    gains: &[f64],
) -> usize {
    (0..10)
        .min_by(|left, right| {
            let distance = |index: usize| {
                let frequency = frequencies.get(index).copied().unwrap_or(20.0);
                let gain = gains.get(index).copied().unwrap_or(0.0);
                let dx = equalizer_frequency_to_x(frequency, width) - x;
                let dy = equalizer_gain_to_y(gain, height) - y;
                dx * dx + dy * dy
            };
            distance(*left).total_cmp(&distance(*right))
        })
        .unwrap_or(0)
}

const EQ_NODE_HIT_RADIUS: f64 = 16.0;

fn equalizer_band_hit_test(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    frequencies: &[f64],
    gains: &[f64],
) -> Option<usize> {
    let index = nearest_equalizer_band(x, y, width, height, frequencies, gains);
    let node_x = equalizer_frequency_to_x(frequencies.get(index).copied().unwrap_or(20.0), width);
    let node_y = equalizer_gain_to_y(gains.get(index).copied().unwrap_or(0.0), height);
    let dx = node_x - x;
    let dy = node_y - y;
    (dx * dx + dy * dy <= EQ_NODE_HIT_RADIUS * EQ_NODE_HIT_RADIUS).then_some(index)
}

fn set_equalizer_graph_point(
    index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    mut gains: Signal<Vec<f64>>,
    mut frequencies: Signal<Vec<f64>>,
    qs: Signal<Vec<f64>>,
    types: Signal<Vec<String>>,
    mut preset_id: Signal<String>,
    mut unavailable: Signal<bool>,
    enabled: bool,
    notice: Signal<String>,
) {
    if index >= 10 {
        return;
    }
    let (frequency, gain) = equalizer_graph_values(x, y, width, height);
    let mut next_frequencies = frequencies();
    let mut next_gains = gains();
    next_frequencies[index] = frequency;
    next_gains[index] = gain;
    frequencies.set(next_frequencies);
    gains.set(next_gains);
    preset_id.set("custom".to_string());
    unavailable.set(
        set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), enabled, notice).is_err(),
    );
}

fn set_equalizer_state(
    gains: &[f64],
    frequencies: &[f64],
    qs: &[f64],
    types: &[String],
    enabled: bool,
    mut notice: Signal<String>,
) -> Result<(), ()> {
    let settings = EqualizerSettings {
        gains: gains.to_vec(),
        frequencies: frequencies.to_vec(),
        qs: qs.to_vec(),
        types: types.to_vec(),
        enabled,
    };
    match browser::set_equalizer(&settings) {
        Ok(()) => Ok(()),
        Err(error) => {
            notice.set(error);
            Err(())
        }
    }
}

fn initialize_saved_equalizer(mut notice: Signal<String>) {
    let enabled = match browser::load::<bool>("wavelength-equalizer-enabled") {
        Ok(Some(enabled)) => enabled,
        Ok(None) => false,
        Err(error) => {
            notice.set(error);
            return;
        }
    };
    if !enabled {
        return;
    }

    let mut errors = Vec::new();
    let gains = load_equalizer_values("wavelength-equalizer-gains", vec![0.0; 10], -18.0, 18.0)
        .unwrap_or_else(|error| {
            errors.push(error);
            vec![0.0; 10]
        });
    let frequencies = load_equalizer_values(
        "wavelength-equalizer-frequencies",
        vec![
            60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0, 14000.0, 16000.0,
        ],
        20.0,
        20000.0,
    )
    .unwrap_or_else(|error| {
        errors.push(error);
        vec![
            60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0, 14000.0, 16000.0,
        ]
    });
    let qs = load_equalizer_values("wavelength-equalizer-qs", vec![1.1; 10], 0.3, 10.0)
        .unwrap_or_else(|error| {
            errors.push(error);
            vec![1.1; 10]
        });
    let types = match browser::load::<Vec<String>>("wavelength-equalizer-types") {
        Ok(Some(values))
            if values.len() == 10 && values.iter().all(|value| equalizer_type_valid(value)) =>
        {
            values
        }
        Ok(Some(_)) => {
            errors.push("Saved equalizer filter types are invalid.".to_string());
            vec!["peaking".to_string(); 10]
        }
        Ok(None) => vec!["peaking".to_string(); 10],
        Err(error) => {
            errors.push(error);
            vec!["peaking".to_string(); 10]
        }
    };

    let settings = EqualizerSettings {
        gains,
        frequencies,
        qs,
        types,
        enabled: true,
    };
    if let Err(error) = browser::set_equalizer(&settings) {
        errors.push(error);
    }
    if !errors.is_empty() {
        notice.set(errors.join(" "));
    }
}

fn persist_equalizer(
    gains: &[f64],
    frequencies: &[f64],
    qs: &[f64],
    types: &[String],
    preset_id: &str,
    mut notice: Signal<String>,
) {
    let results = [
        browser::save("wavelength-equalizer-gains", &gains),
        browser::save("wavelength-equalizer-frequencies", &frequencies),
        browser::save("wavelength-equalizer-qs", &qs),
        browser::save("wavelength-equalizer-types", &types),
        browser::save("wavelength-equalizer-preset", &preset_id),
    ];
    if let Some(Err(error)) = results.into_iter().find(Result::is_err) {
        notice.set(error);
    }
}

const ACCENT_PRESETS: [(&str, &str); 20] = [
    ("Orange", "#fb6b1d"),
    ("Red", "#e83b3b"),
    ("Plum", "#831c5d"),
    ("Magenta", "#c32454"),
    ("Pink", "#f04f78"),
    ("Coral pink", "#f68181"),
    ("Sand", "#ab947a"),
    ("Teal", "#0b5e65"),
    ("Cyan", "#0b8a8f"),
    ("Green", "#1ebc73"),
    ("Lime", "#91db69"),
    ("Gold", "#fbb954"),
    ("Copper", "#cd683d"),
    ("Rust", "#9e4539"),
    ("Wine", "#7a3045"),
    ("Violet", "#6b3e75"),
    ("Purple", "#905ea9"),
    ("Lavender", "#a884f3"),
    ("Blue", "#4d9be6"),
    ("Indigo", "#4d65b4"),
];

fn parse_hex_color(value: &str) -> Option<[f64; 3]> {
    let value = value.strip_prefix('#')?;
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }

    Some([
        u8::from_str_radix(&value[0..2], 16).ok()? as f64 / 255.0,
        u8::from_str_radix(&value[2..4], 16).ok()? as f64 / 255.0,
        u8::from_str_radix(&value[4..6], 16).ok()? as f64 / 255.0,
    ])
}

fn normalize_hex_color(value: &str) -> Option<String> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }

    Some(format!("#{}", hex.to_ascii_lowercase()))
}

fn color_luminance([red, green, blue]: [f64; 3]) -> f64 {
    let linearize = |channel: f64| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * linearize(red) + 0.7152 * linearize(green) + 0.0722 * linearize(blue)
}

fn color_contrast_ratio(first: [f64; 3], second: [f64; 3]) -> f64 {
    let first = color_luminance(first);
    let second = color_luminance(second);
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

fn mix_color(color: [f64; 3], tint: [f64; 3], tint_weight: f64) -> [f64; 3] {
    std::array::from_fn(|index| color[index] * (1.0 - tint_weight) + tint[index] * tint_weight)
}

const DEFAULT_DARK_BACKGROUND_COLOR: &str = "#0a0a0a";
const DEFAULT_LIGHT_BACKGROUND_COLOR: &str = "#fff5d2";
const DEFAULT_LIGHT_PANEL_COLOR: &str = "#fff6d6";
const DEFAULT_LIGHT_RAISED_COLOR: &str = "#fff7d9";

fn default_background_color(theme: &str) -> &'static str {
    if theme == "light" {
        DEFAULT_LIGHT_BACKGROUND_COLOR
    } else {
        DEFAULT_DARK_BACKGROUND_COLOR
    }
}

fn appearance_surfaces(theme: &str, background: &str) -> Option<Vec<[f64; 3]>> {
    if !background.is_empty() {
        let background = parse_hex_color(background)?;
        let tint = if color_luminance(background) > 0.179 {
            [1.0, 1.0, 1.0]
        } else {
            [0.0, 0.0, 0.0]
        };
        return Some(vec![
            background,
            mix_color(background, tint, 0.08),
            mix_color(background, tint, 0.16),
        ]);
    }

    let colors = match theme {
        "light" => [
            DEFAULT_LIGHT_BACKGROUND_COLOR,
            DEFAULT_LIGHT_PANEL_COLOR,
            DEFAULT_LIGHT_RAISED_COLOR,
        ],
        "dark" => [DEFAULT_DARK_BACKGROUND_COLOR, "#151714", "#1c1e19"],
        _ => return None,
    };
    colors
        .into_iter()
        .map(parse_hex_color)
        .collect::<Option<Vec<_>>>()
}

fn accent_color_has_contrast(accent: &str, theme: &str, background: &str) -> bool {
    let Some(accent) = parse_hex_color(accent) else {
        return false;
    };
    let Some(surfaces) = appearance_surfaces(theme, background) else {
        return false;
    };

    surfaces
        .into_iter()
        .all(|surface| color_contrast_ratio(accent, surface) >= 4.5)
}

fn contrast_adjusted_accent(accent: &str, theme: &str, background: &str) -> String {
    let Some(base) = parse_hex_color(accent) else {
        return accessible_accent_color(theme, background);
    };
    let Some(surfaces) = appearance_surfaces(theme, background) else {
        return accessible_accent_color(theme, background);
    };
    let passes_contrast = |color| {
        surfaces
            .iter()
            .all(|surface| color_contrast_ratio(color, *surface) >= 4.5)
    };
    if passes_contrast(base) {
        return accent.to_string();
    }

    let find_variant = |target: [f64; 3]| {
        if !passes_contrast(target) {
            return None;
        }

        let mut previous = 0.0;
        for step in 1..=1000 {
            let weight = step as f64 / 1000.0;
            if !passes_contrast(mix_color(base, target, weight)) {
                previous = weight;
                continue;
            }

            let (mut low, mut high) = (previous, weight);
            for _ in 0..24 {
                let middle = (low + high) / 2.0;
                if passes_contrast(mix_color(base, target, middle)) {
                    high = middle;
                } else {
                    low = middle;
                }
            }

            let mut weight = high;
            for _ in 0..8 {
                let channels = mix_color(base, target, weight)
                    .map(|channel| (channel * 255.0).round().clamp(0.0, 255.0) as u8);
                let color = format!("#{:02x}{:02x}{:02x}", channels[0], channels[1], channels[2]);
                if accent_color_has_contrast(&color, theme, background) {
                    return Some((weight, color));
                }
                weight = (weight + 0.001).min(1.0);
            }

            let channels = target.map(|channel| (channel * 255.0).round() as u8);
            return Some((
                1.0,
                format!("#{:02x}{:02x}{:02x}", channels[0], channels[1], channels[2]),
            ));
        }
        None
    };

    [([0.0, 0.0, 0.0]), ([1.0, 1.0, 1.0])]
        .into_iter()
        .filter_map(find_variant)
        .min_by(|first, second| first.0.total_cmp(&second.0))
        .map(|(_, color)| color)
        .unwrap_or_else(|| accessible_accent_color(theme, background))
}

fn accessible_accent_color(theme: &str, background: &str) -> String {
    let surfaces = appearance_surfaces(theme, background).unwrap_or_default();
    let is_light = surfaces
        .first()
        .is_some_and(|surface| color_luminance(*surface) > 0.179);
    let candidates = if is_light {
        ["#4f621e", "#42551d", "#35451b"]
    } else {
        ["#d7c7a4", "#d7e98b", "#f0e4c4"]
    };

    candidates
        .into_iter()
        .find(|color| accent_color_has_contrast(color, theme, background))
        .or_else(|| {
            ["#000000", "#ffffff"].into_iter().max_by(|first, second| {
                let minimum_ratio = |color: &str| {
                    let Some(color) = parse_hex_color(color) else {
                        return 0.0;
                    };
                    surfaces
                        .iter()
                        .map(|surface| color_contrast_ratio(color, *surface))
                        .fold(f64::INFINITY, f64::min)
                };
                minimum_ratio(first).total_cmp(&minimum_ratio(second))
            })
        })
        .unwrap_or("#d7c7a4")
        .to_string()
}

fn equalizer_preset(name: &str) -> Option<[f64; 10]> {
    match name {
        "flat" => Some([0.0; 10]),
        "bass" => Some([6.0, 5.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        "vocal" => Some([-2.0, -1.0, 0.0, 2.0, 4.0, 4.0, 2.0, 0.0, -1.0, -2.0]),
        "treble" => Some([0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 3.0, 5.0, 6.0, 6.0]),
        "rock" => Some([4.0, 3.0, 1.0, -1.0, -2.0, 1.0, 3.0, 4.0, 4.0, 4.0]),
        _ => None,
    }
}

fn load_saved<T: for<'de> Deserialize<'de> + 'static>(
    key: &str,
    mut signal: Signal<T>,
    mut notice: Signal<String>,
) {
    match browser::load::<T>(key) {
        Ok(Some(value)) => signal.set(value),
        Ok(None) => {}
        Err(error) => notice.set(error),
    }
}

fn load_room_server_passcode(mut passcode: Signal<String>, mut notice: Signal<String>) {
    match browser::room_server_passcode() {
        Ok(Some(saved)) => passcode.set(saved),
        Ok(None) => {}
        Err(error) => notice.set(error),
    }
}

fn load_accessibility_font_scale(mut scale: Signal<f64>, mut notice: Signal<String>) {
    match browser::load::<f64>("wavelength-font-scale") {
        Ok(Some(value)) if value.is_finite() && (0.8..=1.3).contains(&value) => scale.set(value),
        Ok(Some(_)) => {
            scale.set(1.0);
            match browser::save("wavelength-font-scale", &1.0) {
                Ok(()) => {
                    notice.set("Saved text size was invalid; restored the default.".to_string())
                }
                Err(error) => notice.set(error),
            }
        }
        Ok(None) => {}
        Err(error) => notice.set(error),
    }
}

fn save_saved<T: Serialize>(key: &str, value: &T, mut notice: Signal<String>) {
    if let Err(error) = browser::save(key, value) {
        notice.set(error);
    }
}

fn save_string(key: &str, value: &str, mut notice: Signal<String>) {
    if let Err(error) = browser::save(key, value) {
        notice.set(error);
    }
}

fn set_accessibility_font_scale(mut scale: Signal<f64>, value: f64, mut notice: Signal<String>) {
    if !value.is_finite() || !(0.8..=1.3).contains(&value) {
        notice.set("Text size is outside the supported range.".to_string());
        return;
    }
    let value = (value * 10.0).round() / 10.0;
    if let Err(error) = browser::set_font_scale(value) {
        notice.set(error);
        return;
    }
    scale.set(value);
    if let Err(error) = browser::save("wavelength-font-scale", &value) {
        notice.set(error);
    }
}

fn search_tracks(
    query: String,
    source: String,
    mut results: Signal<Vec<Track>>,
    mut busy: Signal<bool>,
    mut notice: Signal<String>,
) {
    let query = query.trim().to_string();
    if query.is_empty() {
        notice.set("Enter a search term.".to_string());
        return;
    }
    busy.set(true);
    spawn(async move {
        match browser::search(&query, &source).await {
            Ok(tracks) => {
                results.set(tracks);
                notice.set(String::new());
            }
            Err(error) => notice.set(error),
        }
        busy.set(false);
    });
}

fn load_spotify_library(
    mut tracks: Signal<Vec<Track>>,
    mut busy: Signal<bool>,
    mut notice: Signal<String>,
) {
    busy.set(true);
    spawn(async move {
        match browser::spotify_library().await {
            Ok(saved_tracks) => {
                tracks.set(saved_tracks);
                notice.set(String::new());
            }
            Err(error) => notice.set(error),
        }
        busy.set(false);
    });
}

fn load_spotify_playlists(
    mut playlists: Signal<Vec<SpotifyPlaylist>>,
    mut busy: Signal<bool>,
    mut notice: Signal<String>,
) {
    busy.set(true);
    spawn(async move {
        match browser::spotify_playlists().await {
            Ok(items) => {
                playlists.set(items);
                notice.set(String::new());
            }
            Err(error) => notice.set(error),
        }
        busy.set(false);
    });
}

fn load_spotify_collections(
    mut playlists: Signal<Vec<SpotifyPlaylist>>,
    mut playlists_busy: Signal<bool>,
    mut albums: Signal<Vec<SpotifyAlbum>>,
    mut albums_busy: Signal<bool>,
    mut notice: Signal<String>,
) {
    playlists_busy.set(true);
    albums_busy.set(true);
    spawn(async move {
        let playlist_result = browser::spotify_playlists().await;
        let album_result = browser::spotify_albums().await;
        let mut errors = Vec::new();
        match playlist_result {
            Ok(items) => playlists.set(items),
            Err(error) => errors.push(format!("Could not load Spotify playlists: {error}")),
        }
        match album_result {
            Ok(items) => albums.set(items),
            Err(error) => errors.push(format!("Could not load Spotify albums: {error}")),
        }
        notice.set(errors.join("\n"));
        playlists_busy.set(false);
        albums_busy.set(false);
    });
}

fn load_spotify_playlist(
    playlist: SpotifyPlaylist,
    mut tracks: Signal<Vec<Track>>,
    mut selected: Signal<Option<SpotifyPlaylist>>,
    mut selected_name: Signal<String>,
    mut busy: Signal<bool>,
    mut load_error: Signal<Option<String>>,
    mut load_id: Signal<u64>,
    mut notice: Signal<String>,
    mut page: Signal<Page>,
) {
    let request_id = (load_id)().wrapping_add(1);
    load_id.set(request_id);
    selected.set(Some(playlist.clone()));
    selected_name.set(playlist.name.clone());
    tracks.set(Vec::new());
    load_error.set(None);
    notice.set(String::new());
    busy.set(true);
    page.set(Page::SpotifyPlaylistDetail);
    spawn_forever(async move {
        if let Err(error) = browser::scroll_content_to_top().await {
            notice.set(error);
        }
    });
    spawn_forever(async move {
        let result = browser::spotify_playlist_tracks(&playlist.id).await;
        if load_id() != request_id
            || !selected()
                .as_ref()
                .is_some_and(|item| item.id == playlist.id)
        {
            return;
        }
        match result {
            Ok(items) => {
                tracks.set(items);
                load_error.set(None);
                notice.set(String::new());
            }
            Err(error) => {
                load_error.set(Some(error));
            }
        }
        busy.set(false);
    });
}

fn load_spotify_album(
    album: SpotifyAlbum,
    mut selected: Signal<Option<SpotifyAlbum>>,
    mut tracks: Signal<Vec<Track>>,
    mut busy: Signal<bool>,
    mut load_error: Signal<Option<String>>,
    mut load_id: Signal<u64>,
    mut album_name: Signal<Option<String>>,
    mut page: Signal<Page>,
    mut notice: Signal<String>,
) {
    let request_id = (load_id)().wrapping_add(1);
    load_id.set(request_id);
    selected.set(Some(album.clone()));
    tracks.set(Vec::new());
    load_error.set(None);
    notice.set(String::new());
    album_name.set(Some(album.name.clone()));
    page.set(Page::AlbumDetail);
    busy.set(true);
    spawn_forever(async move {
        let result = browser::spotify_album_tracks(&album).await;
        if load_id() != request_id || !selected().as_ref().is_some_and(|item| item.id == album.id) {
            return;
        }
        match result {
            Ok(items) => {
                tracks.set(items);
                load_error.set(None);
                notice.set(String::new());
            }
            Err(error) => {
                load_error.set(Some(error));
            }
        }
        busy.set(false);
    });
}

fn focus_search_bar(mut notice: Signal<String>) {
    spawn(async move {
        let script = "const input = document.getElementById('topbar-search-input'); if (!(input instanceof HTMLInputElement)) throw new Error('Search field not found'); input.focus();";
        if let Err(error) = dioxus::document::eval(script).await {
            notice.set(format!("Could not focus the search field: {error}"));
        }
    });
}

fn play_spotify_album(
    album: SpotifyAlbum,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    mut notice: Signal<String>,
    queue_panel_open: Signal<bool>,
    room_panel_open: Signal<bool>,
) {
    spawn(async move {
        match browser::spotify_album_tracks(&album).await {
            Ok(tracks) => play_playlist_tracks(
                tracks,
                current,
                audio_src,
                playing,
                queue,
                history,
                notice,
                queue_panel_open,
                room_panel_open,
            ),
            Err(error) => notice.set(error),
        }
    });
}

fn queue_spotify_album(album: SpotifyAlbum, queue: Signal<Vec<Track>>, mut notice: Signal<String>) {
    spawn(async move {
        match browser::spotify_album_tracks(&album).await {
            Ok(tracks) => add_tracks_to_queue(tracks, queue, notice),
            Err(error) => notice.set(error),
        }
    });
}

fn download_spotify_album(album: SpotifyAlbum, format: String, mut notice: Signal<String>) {
    spawn(async move {
        match browser::spotify_album_tracks(&album).await {
            Ok(tracks) => download_tracks(tracks, format, notice),
            Err(error) => notice.set(error),
        }
    });
}

fn queue_spotify_playlist(
    playlist: SpotifyPlaylist,
    queue: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    spawn(async move {
        match browser::spotify_playlist_tracks(&playlist.id).await {
            Ok(tracks) => add_tracks_to_queue(tracks, queue, notice),
            Err(error) => notice.set(error),
        }
    });
}

fn download_spotify_playlist(
    playlist: SpotifyPlaylist,
    format: String,
    mut notice: Signal<String>,
) {
    spawn(async move {
        match browser::spotify_playlist_tracks(&playlist.id).await {
            Ok(tracks) => download_tracks(tracks, format, notice),
            Err(error) => notice.set(error),
        }
    });
}

fn add_tracks_to_queue(
    tracks: Vec<Track>,
    mut queue: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    if tracks.is_empty() {
        notice.set("This collection has no tracks to add.".to_string());
        return;
    }

    let total = tracks.len();
    spawn(async move {
        let mut items = queue();
        let mut local_added = 0;
        let mut room_sent = 0;
        for track in tracks {
            match browser::emit_room_command(
                "add-track",
                &serde_json::json!({"track":room_track_json(&track)}),
            )
            .await
            {
                Ok(status) if status == "sent" => {
                    room_sent += 1;
                    continue;
                }
                Ok(status) if status == "unavailable" => {
                    notice.set("The room connection is not ready. Try again shortly.".to_string());
                    return;
                }
                Ok(status) if status == "permission-denied" => {
                    notice.set(room_permission_notice("add-track").to_string());
                    return;
                }
                Ok(_) => {}
                Err(error) => {
                    notice.set(error);
                    return;
                }
            }
            items.push(track);
            local_added += 1;
        }

        if local_added > 0 {
            queue.set(items.clone());
            if let Err(error) = browser::save("wavelength-queue", &items) {
                notice.set(error);
                return;
            }
        }
        if room_sent == total {
            notice.set(format!("Sent {room_sent} tracks to the room queue."));
        } else {
            notice.set(format!(
                "Added {} tracks to the queue.",
                local_added + room_sent
            ));
        }
    });
}

fn play_playlist_tracks(
    tracks: Vec<Track>,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    mut queue: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    mut notice: Signal<String>,
    mut queue_panel_open: Signal<bool>,
    mut room_panel_open: Signal<bool>,
) {
    let Some((first, rest)) = tracks.split_first() else {
        notice.set("This playlist has no tracks to play.".to_string());
        return;
    };
    let rest = rest.to_vec();
    queue.set(rest.clone());
    save_saved("wavelength-queue", &rest, notice);
    play_track(
        first.clone(),
        current,
        audio_src,
        playing,
        queue,
        history,
        notice,
    );
    room_panel_open.set(false);
    queue_panel_open.set(true);
}

fn play_spotify_playlist(
    playlist: SpotifyPlaylist,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    mut notice: Signal<String>,
    queue_panel_open: Signal<bool>,
    room_panel_open: Signal<bool>,
) {
    spawn(async move {
        match browser::spotify_playlist_tracks(&playlist.id).await {
            Ok(tracks) => play_playlist_tracks(
                tracks,
                current,
                audio_src,
                playing,
                queue,
                history,
                notice,
                queue_panel_open,
                room_panel_open,
            ),
            Err(error) => notice.set(error),
        }
    });
}

fn spotify_track_uri(track: &Track) -> Option<String> {
    if !track.is_spotify() {
        return None;
    }
    let id = track.id.strip_prefix("spotify:track:").unwrap_or(&track.id);
    (!id.is_empty()).then(|| format!("spotify:track:{id}"))
}

fn create_spotify_playlist(
    mut name: Signal<String>,
    mut playlists: Signal<Vec<SpotifyPlaylist>>,
    mut busy: Signal<bool>,
    mut dialog_open: Signal<bool>,
    mut notice: Signal<String>,
) {
    let clean_name = name().trim().to_string();
    if clean_name.is_empty() {
        notice.set("Enter a name for the Spotify playlist.".to_string());
        return;
    }
    busy.set(true);
    spawn(async move {
        match browser::create_spotify_playlist(&clean_name).await {
            Ok(playlist) => {
                let mut all = playlists();
                if !all.iter().any(|item| item.id == playlist.id) {
                    all.push(playlist);
                    playlists.set(all);
                }
                name.set(String::new());
                dialog_open.set(false);
                notice.set("Spotify playlist created.".to_string());
            }
            Err(error) => notice.set(error),
        }
        busy.set(false);
    });
}

async fn send_guest_room_command(
    action: &str,
    details: &serde_json::Value,
    mut notice: Signal<String>,
) -> bool {
    match browser::emit_room_command(action, details).await {
        Ok(status) if status == "sent" => {
            notice.set("Playback request sent to the room host.".to_string());
            true
        }
        Ok(status) if status == "unavailable" => {
            notice.set("The room connection is not ready. Try again shortly.".to_string());
            true
        }
        Ok(status) if status == "permission-denied" => {
            notice.set(room_permission_notice(action).to_string());
            true
        }
        Ok(_) => false,
        Err(error) => {
            notice.set(error);
            true
        }
    }
}

fn room_permission_notice(action: &str) -> &'static str {
    match action {
        "add-track" => "The host has not allowed guests to add tracks.",
        "remove-track" | "clear-queue" | "reorder-queue" => {
            "The host has not allowed guests to manage the queue."
        }
        _ => "The host has not allowed guests to control playback.",
    }
}

fn play_track(
    track: Track,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    notice: Signal<String>,
) {
    spawn(async move {
        if send_guest_room_command(
            "play-track",
            &serde_json::json!({"track":room_track_json(&track)}),
            notice,
        )
        .await
        {
            return;
        }
        play_track_direct(track, current, audio_src, playing, queue, history, notice);
    });
}

fn play_track_direct(
    track: Track,
    mut current: Signal<Option<Track>>,
    mut audio_src: Signal<String>,
    mut playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    mut history: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    let track_id = track.key().to_string();
    let audio_service_id = track.audio_service_id();
    let is_local = track.is_local();
    let shared_audio_id = track
        .audio_id
        .clone()
        .filter(|audio_id| !audio_id.trim().is_empty());
    let track_for_history = track.clone();
    current.set(Some(track));
    playing.set(false);
    spawn(async move {
        let source = if let Some(audio_id) = shared_audio_id.as_deref() {
            browser::room_audio_url(audio_id).await
        } else if is_local {
            browser::local_url(&track_id).await
        } else {
            browser::audio_service_base().map(|base| {
                format!(
                    "{base}/stream/{}?code=solo&scope=permanent",
                    encode_uri_component(&audio_service_id)
                )
            })
        };
        match source {
            Ok(url) => {
                audio_src.set(url.clone());
                notice.set(String::new());
                if let Err(error) = browser::play_source(&url).await {
                    notice.set(error);
                } else {
                    let mut items = history();
                    items.retain(|item| item.key() != track_for_history.key());
                    items.insert(0, track_for_history);
                    items.truncate(100);
                    history.set(items.clone());
                    if let Err(error) = browser::save("wavelength-play-history", &items) {
                        notice.set(error);
                    }
                }
            }
            Err(error) => notice.set(error),
        }
        let items = queue();
        if let Err(error) = browser::save("wavelength-queue", &items) {
            notice.set(error);
        }
    });
}

fn add_to_queue(track: Track, mut queue: Signal<Vec<Track>>, mut notice: Signal<String>) {
    spawn(async move {
        match browser::emit_room_command(
            "add-track",
            &serde_json::json!({"track":room_track_json(&track)}),
        )
        .await
        {
            Ok(status) if status == "sent" => {
                notice.set("Track request sent to the room.".to_string());
                return;
            }
            Ok(status) if status == "unavailable" => {
                notice.set("The room connection is not ready. Try again shortly.".to_string());
                return;
            }
            Ok(status) if status == "permission-denied" => {
                notice.set(room_permission_notice("add-track").to_string());
                return;
            }
            Ok(_) => {}
            Err(error) => {
                notice.set(error);
                return;
            }
        }
        let mut items = queue();
        items.push(track);
        queue.set(items.clone());
        match browser::save("wavelength-queue", &items) {
            Ok(()) => notice.set("Added to queue.".to_string()),
            Err(error) => notice.set(error),
        }
    });
}

fn remove_queue_item(index: usize, mut queue: Signal<Vec<Track>>, mut notice: Signal<String>) {
    spawn(async move {
        let mut items = queue();
        if index >= items.len() {
            return;
        }
        let track = items[index].clone();
        match browser::emit_room_command(
            "remove-track",
            &serde_json::json!({"trackId":track.key()}),
        )
        .await
        {
            Ok(status) if status == "sent" => {
                notice.set("Queue change sent to the room.".to_string());
                return;
            }
            Ok(status) if status == "unavailable" => {
                notice.set("The room connection is not ready. Try again shortly.".to_string());
                return;
            }
            Ok(status) if status == "permission-denied" => {
                notice.set(room_permission_notice("remove-track").to_string());
                return;
            }
            Ok(_) => {}
            Err(error) => {
                notice.set(error);
                return;
            }
        }
        items.remove(index);
        queue.set(items.clone());
        save_saved("wavelength-queue", &items, notice);
    });
}

fn move_queue_item(
    source_index: usize,
    target_index: usize,
    mut queue: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    spawn(async move {
        let items = queue();
        if source_index >= items.len()
            || target_index >= items.len()
            || source_index == target_index
        {
            return;
        }
        let track_id = items[source_index].key().to_string();
        match browser::emit_room_command(
            "reorder-queue",
            &serde_json::json!({"trackId":track_id, "targetIndex":target_index}),
        )
        .await
        {
            Ok(status) if status == "sent" => {
                notice.set("Queue order request sent to the room.".to_string());
                return;
            }
            Ok(status) if status == "unavailable" => {
                notice.set("The room connection is not ready. Try again shortly.".to_string());
                return;
            }
            Ok(status) if status == "permission-denied" => {
                notice.set(room_permission_notice("reorder-queue").to_string());
                return;
            }
            Ok(_) => {}
            Err(error) => {
                notice.set(error);
                return;
            }
        }
        if let Some(items) = reordered_queue(items, source_index, target_index) {
            queue.set(items.clone());
            save_saved("wavelength-queue", &items, notice);
        }
    });
}

fn clear_queue(mut queue: Signal<Vec<Track>>, mut notice: Signal<String>) {
    spawn(async move {
        match browser::emit_room_command("clear-queue", &serde_json::json!({})).await {
            Ok(status) if status == "sent" => {
                notice.set("Queue change sent to the room.".to_string());
                return;
            }
            Ok(status) if status == "unavailable" => {
                notice.set("The room connection is not ready. Try again shortly.".to_string());
                return;
            }
            Ok(status) if status == "permission-denied" => {
                notice.set(room_permission_notice("clear-queue").to_string());
                return;
            }
            Ok(_) => {}
            Err(error) => {
                notice.set(error);
                return;
            }
        }
        queue.set(Vec::new());
        save_saved("wavelength-queue", &Vec::<Track>::new(), notice);
    });
}

fn toggle_favorite(track: Track, mut favorites: Signal<Vec<Track>>, mut notice: Signal<String>) {
    let mut items = favorites();
    if items.iter().any(|item| item.key() == track.key()) {
        items.retain(|item| item.key() != track.key());
    } else {
        items.push(track);
    }
    favorites.set(items.clone());
    if let Err(error) = browser::save("wavelength-favorites", &items) {
        notice.set(error);
    }
}

fn toggle_playback(mut playing: Signal<bool>, mut notice: Signal<String>) {
    spawn(async move {
        if send_guest_room_command("toggle-playback", &serde_json::json!({}), notice).await {
            return;
        }
        if playing() {
            if let Err(error) = browser::pause() {
                notice.set(error);
            } else {
                playing.set(false);
            }
        } else if let Err(error) = browser::play() {
            notice.set(error);
        } else {
            playing.set(true);
        }
    });
}

fn toggle_shuffle(mut shuffle: Signal<bool>, notice: Signal<String>) {
    spawn(async move {
        if send_guest_room_command("toggle-shuffle", &serde_json::json!({}), notice).await {
            return;
        }
        shuffle.set(!shuffle());
    });
}

fn toggle_repeat(mut repeat_mode: Signal<String>, notice: Signal<String>) {
    spawn(async move {
        if send_guest_room_command("toggle-repeat", &serde_json::json!({}), notice).await {
            return;
        }
        repeat_mode.set(next_repeat_mode(&repeat_mode()).to_string());
    });
}

fn next_repeat_mode(mode: &str) -> &'static str {
    match mode {
        "off" => "all",
        "all" => "one",
        _ => "off",
    }
}

fn set_player_volume(
    value: f64,
    mut volume: Signal<f64>,
    mut previous_volume: Signal<f64>,
    mut notice: Signal<String>,
) {
    if !value.is_finite() {
        notice.set("Volume must be a finite number.".to_string());
        return;
    }
    let value = value.clamp(0.0, 1.0);
    volume.set(value);
    if value > 0.0 {
        previous_volume.set(value);
        if let Err(error) = browser::save("wavelength-previous-volume", &value) {
            notice.set(error);
        }
    }
    if let Err(error) = browser::save("wavelength-volume", &value) {
        notice.set(error);
    }
    if let Err(error) = browser::set_volume(value) {
        notice.set(error);
    }
}

fn toggle_volume(volume: Signal<f64>, previous_volume: Signal<f64>, notice: Signal<String>) {
    let next = if volume() == 0.0 {
        if previous_volume() > 0.0 {
            previous_volume().clamp(0.01, 1.0)
        } else {
            0.3
        }
    } else {
        volume()
    };
    set_player_volume(
        if volume() == 0.0 { next } else { 0.0 },
        volume,
        previous_volume,
        notice,
    );
}

fn seek_track(position: f64, mut playhead: Signal<f64>, notice: Signal<String>) {
    spawn(async move {
        if send_guest_room_command(
            "seek",
            &serde_json::json!({"currentTime":position.max(0.0)}),
            notice,
        )
        .await
        {
            return;
        }
        playhead.set(position.max(0.0));
        browser::seek(position.max(0.0));
    });
}

fn next_track(
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    mut queue: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    source: Vec<Track>,
    shuffle: Signal<bool>,
    repeat_mode: Signal<String>,
    mut notice: Signal<String>,
) {
    spawn(async move {
        if send_guest_room_command("next-track", &serde_json::json!({}), notice).await {
            return;
        }
        let mut items = queue();
        if items.is_empty() {
            let active = current();
            let target =
                select_track_from_source(&source, active.as_ref(), 1, shuffle(), random_unit_f64())
                    .or_else(|| (repeat_mode() == "all").then(|| active.clone()).flatten());
            if let Some(track) = target.filter(|track| {
                repeat_mode() == "all"
                    || active
                        .as_ref()
                        .is_none_or(|active| active.key() != track.key())
            }) {
                play_track_direct(track, current, audio_src, playing, queue, history, notice);
            }
            return;
        }
        let next_index = if shuffle() {
            (random_unit_f64() * items.len() as f64) as usize
        } else {
            0
        };
        let track = items.remove(next_index.min(items.len() - 1));
        queue.set(items.clone());
        if let Err(error) = browser::save("wavelength-queue", &items) {
            notice.set(error);
            return;
        }
        play_track_direct(track, current, audio_src, playing, queue, history, notice);
    });
}

fn select_track_from_source(
    source: &[Track],
    current: Option<&Track>,
    direction: isize,
    shuffle: bool,
    random: f64,
) -> Option<Track> {
    if source.is_empty() {
        return None;
    }
    if shuffle && direction > 0 {
        let options = source
            .iter()
            .filter(|track| current.is_none_or(|active| active.key() != track.key()))
            .collect::<Vec<_>>();
        let index = if options.is_empty() {
            0
        } else {
            (random.clamp(0.0, 1.0) * options.len() as f64)
                .floor()
                .min((options.len() - 1) as f64) as usize
        };
        return options
            .get(index)
            .copied()
            .cloned()
            .or_else(|| source.first().cloned());
    }
    let index = current
        .and_then(|active| source.iter().position(|track| track.key() == active.key()))
        .map(|index| (index as isize + direction).rem_euclid(source.len() as isize) as usize)
        .unwrap_or_else(|| if direction > 0 { 0 } else { source.len() - 1 });
    source.get(index).cloned()
}

fn handle_track_ended(
    mut current: Signal<Option<Track>>,
    mut audio_src: Signal<String>,
    mut playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    source: Vec<Track>,
    mut playhead: Signal<f64>,
    mut media_duration: Signal<f64>,
    shuffle: Signal<bool>,
    repeat_mode: Signal<String>,
    room_role: Signal<String>,
    mut notice: Signal<String>,
) {
    if room_role() == "guest" {
        return;
    }
    if repeat_mode() == "one" {
        browser::seek(0.0);
        playhead.set(0.0);
        match browser::play() {
            Ok(()) => playing.set(true),
            Err(error) => notice.set(error),
        }
        return;
    }
    if !queue().is_empty() || repeat_mode() == "all" {
        next_track(
            current,
            audio_src,
            playing,
            queue,
            history,
            source,
            shuffle,
            repeat_mode,
            notice,
        );
        return;
    }
    playing.set(false);
    current.set(None);
    audio_src.set(String::new());
    playhead.set(0.0);
    media_duration.set(0.0);
}

fn previous_track(
    history: Signal<Vec<Track>>,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    source: Vec<Track>,
    notice: Signal<String>,
) {
    spawn(async move {
        if send_guest_room_command("previous-track", &serde_json::json!({}), notice).await {
            return;
        }
        if let Some(track) = select_track_from_source(&source, current().as_ref(), -1, false, 0.0)
            .or_else(|| history().get(1).cloned())
        {
            play_track(track, current, audio_src, playing, queue, history, notice);
        }
    });
}

fn download_track(track: Track, format: String, mut notice: Signal<String>) {
    let id = track.audio_service_id();
    let filename = format!("{} - {}", track.artist, track.title);
    let format = match validate_download_format(&format) {
        Ok(format) => format,
        Err(error) => {
            notice.set(error);
            return;
        }
    };
    spawn(async move {
        match browser::download(&id, &filename, format).await {
            Ok(name) => notice.set(format!("Downloaded {name}")),
            Err(error) => notice.set(error),
        }
    });
}

fn download_tracks(tracks: Vec<Track>, format: String, mut notice: Signal<String>) {
    let tracks = tracks
        .into_iter()
        .filter(|track| !track.is_local())
        .collect::<Vec<_>>();
    if tracks.is_empty() {
        notice.set("This collection has no downloadable tracks.".to_string());
        return;
    }

    let format = match validate_download_format(&format) {
        Ok(format) => format,
        Err(error) => {
            notice.set(error);
            return;
        }
    };
    let total = tracks.len();
    spawn(async move {
        let mut downloaded = 0;
        let mut first_error = None;
        for track in tracks {
            let id = track.audio_service_id();
            let filename = format!("{} - {}", track.artist, track.title);
            match browser::download(&id, &filename, format).await {
                Ok(_) => downloaded += 1,
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        if let Some(error) = first_error {
            notice.set(format!(
                "Downloaded {downloaded} of {total} tracks. {error}"
            ));
        } else {
            notice.set(format!("Downloaded {downloaded} tracks."));
        }
    });
}

fn validate_download_format(format: &str) -> Result<&'static str, String> {
    match format {
        "flac" => Ok("flac"),
        "mp3" => Ok("mp3"),
        _ => Err("Choose FLAC or MP3 as the download format in Settings.".to_string()),
    }
}

fn toggle_room_permission(
    permission: RoomPermission,
    room_url: Signal<String>,
    room_access: Signal<String>,
    room_code: Signal<String>,
    room_name: Signal<String>,
    host_token: Signal<String>,
    mut permissions: Signal<RoomPermissions>,
    mut notice: Signal<String>,
) {
    let previous = permissions();
    let mut updated = previous.clone();
    match permission {
        RoomPermission::AddTracks => updated.can_add = !updated.can_add,
        RoomPermission::ManageQueue => updated.can_manage_queue = !updated.can_manage_queue,
        RoomPermission::ControlPlayback => {
            updated.can_control_playback = !updated.can_control_playback
        }
    }
    permissions.set(updated.clone());

    let url = room_url();
    let access = room_access();
    let code = room_code();
    let name = room_name();
    let host = host_token();
    spawn(async move {
        let client = browser::RoomClient::new(&url, &access);
        match client
            .update(
                &code,
                &serde_json::json!({"username": name, "permissions": updated}),
                &host,
            )
            .await
        {
            Ok(party) => permissions.set(party.permissions),
            Err(error) => {
                permissions.set(previous);
                notice.set(error);
            }
        }
    });
}

fn copy_room_invite(
    room_code: Signal<String>,
    invite_link: bool,
    mut feedback: Signal<String>,
    mut notice: Signal<String>,
) {
    let code = room_code();
    if code.is_empty() {
        return;
    }
    spawn(async move {
        let copied = if invite_link {
            browser::copy_room_invite(&code).await
        } else {
            browser::copy_room_code(&code).await
        };
        match copied {
            Ok(()) => {
                feedback.set(if invite_link { "invite" } else { "code" }.to_string());
                notice.set(if invite_link {
                    "Invite link copied.".to_string()
                } else {
                    "Room code copied.".to_string()
                });
                sleep_ms(1800).await;
                feedback.set(String::new());
            }
            Err(error) => notice.set(error),
        }
    });
}

fn normalize_room_server_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let url = trimmed.strip_suffix('/').unwrap_or(trimmed);
    let (scheme, authority) = if let Some(authority) = url.strip_prefix("https://") {
        ("https://", authority)
    } else if let Some(authority) = url.strip_prefix("http://") {
        ("http://", authority)
    } else {
        return Err("Use an HTTPS server URL, or HTTP on localhost, without a path.".to_string());
    };
    if authority.is_empty()
        || authority.chars().any(char::is_whitespace)
        || authority.contains(['/', '?', '#', '@'])
    {
        return Err(
            "Use a server URL without credentials, a path, or query parameters.".to_string(),
        );
    }

    if scheme == "http://" {
        let host = authority.to_ascii_lowercase();
        let local = ["localhost", "127.0.0.1"].iter().any(|name| {
            host == *name
                || host
                    .strip_prefix(name)
                    .is_some_and(|suffix| suffix.starts_with(':'))
        }) || host == "[::1]"
            || host.starts_with("[::1]:");
        if !local {
            return Err(
                "Use an HTTPS server URL, or HTTP on localhost, without a path.".to_string(),
            );
        }
    }

    Ok(url.to_string())
}

fn save_room_server_url(
    mut room_url_input: Signal<String>,
    mut room_url: Signal<String>,
    mut access: Signal<String>,
    active_room_code: Signal<String>,
    mut notice: Signal<String>,
) -> bool {
    let normalized = match normalize_room_server_url(&room_url_input()) {
        Ok(url) => url,
        Err(error) => {
            notice.set(error);
            return false;
        }
    };
    if !active_room_code().is_empty() && normalized != room_url() {
        room_url_input.set(room_url());
        notice.set("Leave the current room before changing its server.".to_string());
        return false;
    }

    let changed = normalized != room_url();
    if let Err(error) = browser::save("wavelength-room-url", &normalized) {
        notice.set(error);
        return false;
    }
    if changed && !access().is_empty() {
        if let Err(error) = browser::save("wavelength-room-access", "") {
            notice.set(error);
            return false;
        }
        access.set(String::new());
    }

    room_url.set(normalized.clone());
    room_url_input.set(normalized.clone());
    notice.set(if normalized.is_empty() {
        "Room server disconnected.".to_string()
    } else {
        "Room server saved.".to_string()
    });
    true
}

fn connect_room(
    room_url_input: Signal<String>,
    room_url: Signal<String>,
    passcode: Signal<String>,
    mut access: Signal<String>,
    mut connecting: Signal<bool>,
    active_room_code: Signal<String>,
    mut notice: Signal<String>,
) {
    if !save_room_server_url(room_url_input, room_url, access, active_room_code, notice) {
        return;
    }
    let url = room_url();
    if url.is_empty() {
        notice.set("Enter a room server URL before connecting.".to_string());
        return;
    }

    if passcode().trim().is_empty() {
        notice.set(if access().is_empty() {
            "Enter the server passcode to connect.".to_string()
        } else {
            "Server address saved; the connection is still active.".to_string()
        });
        return;
    }

    let passcode_value = passcode();
    if let Err(error) = browser::set_room_server_passcode(Some(&passcode_value)) {
        notice.set(error);
        return;
    }
    connecting.set(true);
    notice.set(String::new());
    spawn(async move {
        let client = browser::RoomClient::new(&url, "");
        match client.authenticate(&passcode_value).await {
            Ok(result) => match browser::save("wavelength-room-access", &result.token) {
                Ok(()) => {
                    access.set(result.token);
                    notice.set("Room server connected.".to_string());
                }
                Err(error) => notice.set(error),
            },
            Err(error) => notice.set(error),
        }
        connecting.set(false);
    });
}

fn disconnect_room(
    mut access: Signal<String>,
    active_room_code: Signal<String>,
    mut notice: Signal<String>,
) {
    if !active_room_code().is_empty() {
        notice.set("Leave the current room before disconnecting.".to_string());
        return;
    }
    match browser::save("wavelength-room-access", "") {
        Ok(()) => {
            access.set(String::new());
            notice.set("Disconnected from the room server.".to_string());
        }
        Err(error) => notice.set(error),
    }
}

#[derive(Deserialize)]
struct AccessResult {
    token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoomResult {
    code: String,
    #[serde(default)]
    host_token: String,
    #[serde(default)]
    party: Option<RoomParty>,
}

fn switch_room(
    session: RoomSession,
    mut room_url: Signal<String>,
    mut room_url_input: Signal<String>,
    mut room_access: Signal<String>,
    room_sessions: Signal<Vec<RoomSession>>,
    mut room_name: Signal<String>,
    mut active_room_code: Signal<String>,
    mut room_role: Signal<String>,
    mut room_host_token: Signal<String>,
    mut room_members: Signal<Vec<String>>,
    mut room_permissions: Signal<RoomPermissions>,
    mut room_shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    queue: Signal<Vec<Track>>,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    mut playing: Signal<bool>,
    mut playhead: Signal<f64>,
    mut media_duration: Signal<f64>,
    mut shuffle: Signal<bool>,
    mut repeat_mode: Signal<String>,
    mut local_tracks: Signal<Vec<Track>>,
    mut history: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    room_url_input.set(session.room_url.clone());
    room_url.set(session.room_url.clone());
    room_access.set(session.access_token.clone());
    room_name.set(session.name.clone());
    active_room_code.set(session.code.clone());
    room_role.set(session.role.clone());
    room_host_token.set(session.host_token.clone());
    room_members.set(Vec::new());
    room_permissions.set(RoomPermissions::default());
    room_shared_playlists.set(Vec::new());
    if let Err(error) = persist_active_room_session(&session) {
        notice.set(error);
    }
    let _ = (
        queue,
        current,
        audio_src,
        playing,
        playhead,
        media_duration,
        shuffle,
        repeat_mode,
        local_tracks,
        history,
        room_sessions,
    );
}

fn join_room(
    room_url: Signal<String>,
    access: Signal<String>,
    room_name: Signal<String>,
    room_code: Signal<String>,
    mut active_room_code: Signal<String>,
    mut room_role: Signal<String>,
    room_host_token: Signal<String>,
    mut room_members: Signal<Vec<String>>,
    mut room_permissions: Signal<RoomPermissions>,
    mut room_shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    mut room_sessions: Signal<Vec<RoomSession>>,
    queue: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    let url = room_url();
    let token = access();
    let name = room_name().trim().to_string();
    let code = room_code().trim().to_ascii_uppercase();
    if name.is_empty() || code.is_empty() {
        notice.set("Enter your name and a room code.".to_string());
        return;
    }
    spawn(async move {
        let client = browser::RoomClient::new(&url, &token);
        match client.join(&code, &name).await {
            Ok(room) => {
                let party = room.party.unwrap_or_default();
                let socket_permissions = party.permissions.clone();
                if let Err(error) = backup_guest_queue(&queue()) {
                    notice.set(error);
                    return;
                }
                if let Err(error) = persist_room_session(
                    room_sessions,
                    &url,
                    &room.code,
                    "guest",
                    &name,
                    "",
                    &token,
                ) {
                    let cleanup_error = browser::remove_local("wavelength-pre-party-queue").err();
                    notice.set(cleanup_error.map_or_else(
                        || error.clone(),
                        |cleanup| {
                            format!(
                                "{error} Could not remove the temporary queue backup: {cleanup}"
                            )
                        },
                    ));
                    return;
                }
                active_room_code.set(room.code.clone());
                room_role.set("guest".to_string());
                room_members.set(party.members);
                room_permissions.set(socket_permissions.clone());
                room_shared_playlists.set(party.playlists);
                match browser::connect_room_socket(
                    &url,
                    &room.code,
                    &name,
                    &token,
                    &room_host_token(),
                    "guest",
                    &socket_permissions,
                ) {
                    Ok(()) => notice.set(format!("Joined room {}.", room.code)),
                    Err(error) => notice.set(format!(
                        "Joined room, but live sync could not start: {error}"
                    )),
                }
            }
            Err(error) => notice.set(error),
        }
    });
}

fn create_room(
    room_url: Signal<String>,
    access: Signal<String>,
    mut room_name: Signal<String>,
    mut room_name_mode: Signal<String>,
    spotify_profile: Signal<Option<SpotifyProfile>>,
    mut room_code: Signal<String>,
    mut active_room_code: Signal<String>,
    mut room_role: Signal<String>,
    mut room_host_token: Signal<String>,
    mut room_members: Signal<Vec<String>>,
    mut room_permissions: Signal<RoomPermissions>,
    mut room_shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    mut room_sessions: Signal<Vec<RoomSession>>,
    mut notice: Signal<String>,
) {
    let url = room_url();
    let token = access();
    let spotify_name = spotify_profile()
        .map(|profile| profile.display_name.unwrap_or(profile.id))
        .unwrap_or_default();
    let name = if room_name_mode() == "spotify" && !spotify_name.trim().is_empty() {
        spotify_name
    } else {
        room_name()
    }
    .trim()
    .to_string();
    if name.is_empty() {
        notice.set("Enter your name before creating a room.".to_string());
        return;
    }
    spawn(async move {
        let client = browser::RoomClient::new(&url, &token);
        match client.create(&name).await {
            Ok(room) => {
                if room.host_token.is_empty() {
                    notice.set("The room server did not return host authorization.".to_string());
                    return;
                }
                let party = room.party.unwrap_or_default();
                let socket_permissions = party.permissions.clone();
                if let Err(error) = persist_room_session(
                    room_sessions,
                    &url,
                    &room.code,
                    "host",
                    &name,
                    &room.host_token,
                    &token,
                ) {
                    notice.set(error);
                    return;
                }
                room_code.set(room.code.clone());
                active_room_code.set(room.code.clone());
                room_role.set("host".to_string());
                room_host_token.set(room.host_token);
                room_members.set(party.members);
                room_permissions.set(socket_permissions.clone());
                room_shared_playlists.set(party.playlists);
                room_name.set(name.clone());
                room_name_mode.set("custom".to_string());
                match browser::connect_room_socket(
                    &url,
                    &room.code,
                    &name,
                    &token,
                    &room_host_token(),
                    "host",
                    &socket_permissions,
                ) {
                    Ok(()) => notice.set(format!("Created room {}.", room.code)),
                    Err(error) => notice.set(format!(
                        "Room created, but live sync could not start: {error}"
                    )),
                }
            }
            Err(error) => notice.set(error),
        }
    });
}

fn upsert_room_session(sessions: &mut Vec<RoomSession>, session: RoomSession) {
    sessions
        .retain(|existing| existing.code != session.code || existing.room_url != session.room_url);
    sessions.push(session);
}

fn remove_room_session(sessions: &mut Vec<RoomSession>, room_url: &str, code: &str) -> bool {
    let previous_len = sessions.len();
    sessions.retain(|session| {
        session.code != code
            || session.room_url.trim_end_matches('/') != room_url.trim_end_matches('/')
    });
    sessions.len() != previous_len
}

fn persist_room_session(
    mut room_sessions: Signal<Vec<RoomSession>>,
    room_url: &str,
    code: &str,
    role: &str,
    name: &str,
    host_token: &str,
    access_token: &str,
) -> Result<(), String> {
    let session = RoomSession {
        code: code.to_string(),
        role: role.to_string(),
        name: name.to_string(),
        host_token: host_token.to_string(),
        room_url: room_url.to_string(),
        access_token: access_token.to_string(),
    };
    remember_room_session(room_sessions, session.clone())?;
    persist_active_room_session(&session)
}

fn remember_room_session(
    mut room_sessions: Signal<Vec<RoomSession>>,
    session: RoomSession,
) -> Result<(), String> {
    let mut sessions = room_sessions();
    upsert_room_session(&mut sessions, session);
    browser::save("wavelength-party-sessions", &sessions)?;
    room_sessions.set(sessions);
    Ok(())
}

fn persist_active_room_session(session: &RoomSession) -> Result<(), String> {
    browser::save("wavelength-party-code", &session.code)?;
    browser::save("wavelength-party-role", &session.role)?;
    browser::save("wavelength-party-name", &session.name)?;
    browser::save("wavelength-party-host-token", &session.host_token)?;
    browser::save("wavelength-room-url", &session.room_url)?;
    browser::save("wavelength-room-access", &session.access_token)?;
    browser::save("wavelength-username", &session.name)
}

fn clear_active_room_session() -> Result<(), String> {
    browser::save("wavelength-party-code", &String::new())?;
    browser::save("wavelength-party-role", &String::new())?;
    browser::save("wavelength-party-name", &String::new())?;
    browser::save("wavelength-party-host-token", &String::new())
}

fn backup_guest_queue(queue: &[Track]) -> Result<(), String> {
    match browser::load::<Vec<Track>>("wavelength-pre-party-queue")? {
        Some(_) => Ok(()),
        None => browser::save("wavelength-pre-party-queue", &queue.to_vec()),
    }
}

fn restore_guest_queue(mut queue: Signal<Vec<Track>>) -> Result<(), String> {
    let mut restore_error = None;
    match browser::load::<Vec<Track>>("wavelength-pre-party-queue") {
        Ok(Some(saved_queue)) => {
            queue.set(saved_queue.clone());
            if let Err(error) = browser::save("wavelength-queue", &saved_queue) {
                restore_error = Some(error);
            }
        }
        Ok(None) => {}
        Err(error) => {
            queue.set(Vec::new());
            if let Err(save_error) = browser::save("wavelength-queue", &Vec::<Track>::new()) {
                restore_error = Some(save_error);
            }
            if restore_error.is_none() {
                restore_error = Some(format!(
                    "Could not restore the queue from before the room: {error}"
                ));
            }
        }
    }
    if let Err(error) = browser::remove_local("wavelength-pre-party-queue") {
        if restore_error.is_none() {
            restore_error = Some(error);
        }
    }
    restore_error.map_or(Ok(()), Err)
}

fn clear_saved_room_session(
    mut room_sessions: Signal<Vec<RoomSession>>,
    room_url: &str,
    code: &str,
    mut notice: Signal<String>,
) -> bool {
    let mut sessions = room_sessions();
    let removed = remove_room_session(&mut sessions, room_url, code);
    if removed {
        room_sessions.set(sessions.clone());
        if let Err(error) = browser::save("wavelength-party-sessions", &sessions) {
            notice.set(format!("Could not update saved room sessions: {error}"));
        }
    }
    if let Err(error) = clear_active_room_session() {
        notice.set(format!("Could not clear the saved room session: {error}"));
    }
    if let Err(error) = browser::disconnect_room_socket() {
        notice.set(format!("Could not disconnect the room socket: {error}"));
    }
    sessions.is_empty()
}

fn refresh_room(
    room_url: Signal<String>,
    room_access: Signal<String>,
    room_code: Signal<String>,
    host_token: Signal<String>,
    mut members: Signal<Vec<String>>,
    mut permissions: Signal<RoomPermissions>,
    mut shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    mut notice: Signal<String>,
) {
    let url = room_url();
    let code = room_code();
    let access = room_access();
    let host = host_token();
    spawn(async move {
        let client = browser::RoomClient::new(&url, &access);
        match client.read(&code, &host).await {
            Ok(party) => {
                members.set(party.members);
                permissions.set(party.permissions);
                shared_playlists.set(party.playlists);
                notice.set("Room details refreshed.".to_string());
            }
            Err(error) => notice.set(error),
        }
    });
}

fn end_room(
    room_url: Signal<String>,
    room_access: Signal<String>,
    room_name: Signal<String>,
    mut active_room_code: Signal<String>,
    mut room_role: Signal<String>,
    mut host_token: Signal<String>,
    mut members: Signal<Vec<String>>,
    mut permissions: Signal<RoomPermissions>,
    mut shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    mut room_sessions: Signal<Vec<RoomSession>>,
    queue: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    let url = room_url();
    let access = room_access();
    let name = room_name();
    let code = active_room_code();
    let role = room_role();
    let host = host_token();
    if code.is_empty() {
        return;
    }
    spawn(async move {
        let client = browser::RoomClient::new(&url, &access);
        let result = if role == "host" {
            client.end(&code, &host).await
        } else {
            client.leave(&code, &name).await
        };
        match result {
            Ok(_) => {
                let mut cleanup_error = persist_active_room_session(&RoomSession::default()).err();
                let mut sessions = room_sessions();
                if remove_room_session(&mut sessions, &url, &code) {
                    room_sessions.set(sessions.clone());
                    if let Err(error) = browser::save("wavelength-party-sessions", &sessions) {
                        cleanup_error.get_or_insert(format!(
                            "Could not update saved room sessions: {error}"
                        ));
                    }
                }
                if role == "guest" {
                    if let Err(error) = restore_guest_queue(queue) {
                        cleanup_error.get_or_insert(error);
                    }
                }
                if let Err(error) = browser::disconnect_room_socket() {
                    cleanup_error.get_or_insert(error);
                }
                active_room_code.set(String::new());
                room_role.set(String::new());
                host_token.set(String::new());
                members.set(Vec::new());
                permissions.set(RoomPermissions::default());
                shared_playlists.set(Vec::new());
                let success_message = if role == "host" {
                    "Room closed."
                } else {
                    "You left the room."
                };
                notice.set(cleanup_error.map_or_else(
                    || success_message.to_string(),
                    |error| format!("{success_message} {error}"),
                ));
            }
            Err(error) => notice.set(error),
        }
    });
}

fn room_track_json(track: &Track) -> serde_json::Value {
    serde_json::json!({
        "id": track.key(),
        "videoId": track.video_id.as_deref().unwrap_or_default(),
        "audioId": track.audio_id.as_deref().unwrap_or_default(),
        "title": track.title,
        "artist": track.artist,
        "thumbnail": track.thumbnail.as_deref().unwrap_or_default(),
        "duration": track.duration.unwrap_or_default(),
        "source": track.source.as_deref().unwrap_or_default(),
    })
}

fn room_host_state_signature(
    queue: &[serde_json::Value],
    playlists: &[RoomSharedPlaylist],
    permissions: &RoomPermissions,
    repeat_mode: &str,
    shuffle: bool,
) -> String {
    serde_json::to_string(&serde_json::json!({
        "queue": queue,
        "playlists": playlists,
        "permissions": permissions,
        "repeatMode": repeat_mode,
        "shuffle": shuffle,
    }))
    .unwrap_or_default()
}

fn room_track_platform(track: &Track) -> String {
    track
        .source
        .as_deref()
        .filter(|source| !source.trim().is_empty())
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| {
            if track.is_local() {
                "local".to_string()
            } else if track.is_spotify() {
                "spotify".to_string()
            } else if track.key().starts_with("yt-") || track.video_id.is_some() {
                "youtube".to_string()
            } else {
                "audio".to_string()
            }
        })
}

fn room_playlist_track_id(track: &Track) -> String {
    let platform = room_track_platform(track);
    if platform == "youtube" {
        let id = track.video_id.as_deref().unwrap_or_else(|| track.key());
        if id.starts_with("yt-") {
            id.to_string()
        } else {
            format!("yt-{id}")
        }
    } else if platform == "spotify" {
        track
            .key()
            .strip_prefix("spotify:track:")
            .unwrap_or_else(|| track.key())
            .to_string()
    } else {
        track.key().to_string()
    }
}

fn room_playlist_references(playlists: &[Playlist]) -> Vec<RoomSharedPlaylist> {
    playlists
        .iter()
        .map(|playlist| RoomSharedPlaylist {
            id: playlist.id.clone(),
            name: playlist.name.clone(),
            tracks: playlist
                .tracks
                .iter()
                .map(|track| RoomPlaylistTrackReference {
                    id: room_playlist_track_id(track),
                    platform: room_track_platform(track),
                })
                .filter(|reference| !reference.id.is_empty())
                .collect(),
        })
        .collect()
}

fn resolve_room_playlist(
    playlist: &RoomSharedPlaylist,
    library: &[Track],
    favorites: &[Track],
    history: &[Track],
    room_queue: &[Track],
) -> Playlist {
    let tracks = playlist
        .tracks
        .iter()
        .map(|reference| {
            library
                .iter()
                .chain(favorites)
                .chain(history)
                .chain(room_queue)
                .find(|track| {
                    room_playlist_track_id(track) == reference.id
                        && room_track_platform(track).eq_ignore_ascii_case(&reference.platform)
                })
                .cloned()
                .unwrap_or_else(|| {
                    let youtube_id = reference
                        .id
                        .strip_prefix("yt-")
                        .unwrap_or(&reference.id)
                        .to_string();
                    Track {
                        id: reference.id.clone(),
                        video_id: (reference.platform.eq_ignore_ascii_case("youtube"))
                            .then_some(youtube_id),
                        title: reference.id.clone(),
                        artist: if reference.platform.is_empty() {
                            "Unknown platform".to_string()
                        } else {
                            reference.platform.clone()
                        },
                        album: Some(playlist.name.clone()),
                        source: Some(reference.platform.clone()),
                        ..Track::default()
                    }
                })
        })
        .collect();
    Playlist {
        id: playlist.id.clone(),
        name: playlist.name.clone(),
        thumbnail: None,
        tracks,
    }
}

async fn upload_room_queue_tracks(
    url: &str,
    code: &str,
    host_token: &str,
    mut queue: Signal<Vec<Track>>,
    mut current: Signal<Option<Track>>,
    mut local_tracks: Signal<Vec<Track>>,
) -> Result<(), String> {
    let mut candidates = queue();
    if let Some(track) = current() {
        if !candidates.iter().any(|item| item.key() == track.key()) {
            candidates.push(track);
        }
    }
    let mut uploaded = Vec::<String>::new();
    for mut track in candidates {
        if !track.is_local()
            || track.audio_id.is_some()
            || uploaded.iter().any(|id| id == track.key())
        {
            continue;
        }
        let track_id = track.key().to_string();
        if !browser::has_local_audio_file(&track_id)? {
            continue;
        }
        let audio_id = browser::upload_room_track(url, code, &track_id, host_token).await?;
        uploaded.push(track_id.clone());
        track.audio_id = Some(audio_id);

        let mut queued_tracks = queue();
        for queued in &mut queued_tracks {
            if queued.key() == track_id {
                queued.audio_id = track.audio_id.clone();
            }
        }
        queue.set(queued_tracks.clone());
        browser::save("wavelength-queue", &queued_tracks)?;

        let mut library_tracks = local_tracks();
        for library_track in &mut library_tracks {
            if library_track.key() == track_id {
                library_track.audio_id = track.audio_id.clone();
            }
        }
        local_tracks.set(library_tracks);

        if current()
            .as_ref()
            .is_some_and(|active| active.key() == track_id)
        {
            current.set(Some(track));
        }
    }
    Ok(())
}

async fn room_sync_loop(
    room_url: Signal<String>,
    _room_access: Signal<String>,
    room_name: Signal<String>,
    mut active_room_code: Signal<String>,
    mut room_role: Signal<String>,
    mut room_host_token: Signal<String>,
    mut room_members: Signal<Vec<String>>,
    mut room_permissions: Signal<RoomPermissions>,
    mut room_shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    room_sessions: Signal<Vec<RoomSession>>,
    playlists: Signal<Vec<Playlist>>,
    mut queue: Signal<Vec<Track>>,
    mut current: Signal<Option<Track>>,
    mut audio_src: Signal<String>,
    mut playing: Signal<bool>,
    mut playhead: Signal<f64>,
    mut media_duration: Signal<f64>,
    mut shuffle: Signal<bool>,
    mut repeat_mode: Signal<String>,
    mut notice: Signal<String>,
    local_tracks: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
) {
    let mut last_error = String::new();
    let mut last_host_state = String::new();
    let mut next_host_state_retry = 0.0;
    let mut next_upload_retry = 0.0;
    let mut next_playback_control_retry = 0.0;
    let mut last_playback_track = String::new();
    let mut last_playback_state = false;
    let mut last_progress_sent = 0.0;
    loop {
        sleep_ms(250).await;
        let code = active_room_code();
        if code.is_empty() {
            last_error.clear();
            last_host_state.clear();
            next_host_state_retry = 0.0;
            next_upload_retry = 0.0;
            next_playback_control_retry = 0.0;
            last_playback_track.clear();
            continue;
        }
        let url = room_url();
        let role = room_role();
        let host_token = room_host_token();
        let name = room_name();
        let now = now_epoch_millis() as f64;
        let mut pushed_party = None;

        match browser::drain_room_events().await {
            Ok(events) => {
                for event in events {
                    if event.event == "connect" {
                        last_playback_track.clear();
                        last_host_state.clear();
                        next_host_state_retry = 0.0;
                        next_playback_control_retry = 0.0;
                    } else if event.event == "room-state" {
                        if let Ok(party) = serde_json::from_value::<RoomParty>(event.data) {
                            room_members.set(party.members.clone());
                            room_permissions.set(party.permissions.clone());
                            room_shared_playlists.set(party.playlists.clone());
                            pushed_party = Some(party);
                        }
                    } else if role == "guest"
                        && matches!(
                            event.event.as_str(),
                            "playback-control" | "playback-progress"
                        )
                    {
                        let Some(track_id) = event
                            .data
                            .get("currentTrackId")
                            .and_then(serde_json::Value::as_str)
                        else {
                            continue;
                        };
                        if !current()
                            .as_ref()
                            .is_some_and(|track| track.key() == track_id)
                        {
                            continue;
                        }
                        let Some(position) = event
                            .data
                            .get("currentTime")
                            .and_then(serde_json::Value::as_f64)
                            .filter(|position| {
                                position.is_finite() && (0.0..=86_400.0).contains(position)
                            })
                        else {
                            continue;
                        };
                        if event.event == "playback-control" {
                            let seek_required = (playhead() - position).abs() > 0.35;
                            playhead.set(position);
                            if seek_required {
                                browser::seek(position);
                            }
                            let should_play = event
                                .data
                                .get("isPlaying")
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false);
                            if should_play != playing() {
                                if should_play {
                                    match browser::play() {
                                        Ok(()) => playing.set(true),
                                        Err(error) => notice.set(error),
                                    }
                                } else {
                                    let _ = browser::pause();
                                    playing.set(false);
                                }
                            }
                        } else {
                            if !playing() {
                                continue;
                            }
                            if let Some(duration) = event
                                .data
                                .get("duration")
                                .and_then(serde_json::Value::as_f64)
                                .filter(|duration| duration.is_finite() && *duration >= 0.0)
                            {
                                media_duration.set(duration);
                            }
                            let seek_required = (playhead() - position).abs() > 1.5;
                            playhead.set(position);
                            if seek_required {
                                browser::seek(position);
                            }
                        }
                    } else if role == "host" && event.event == "room-command" {
                        apply_host_room_command(
                            event.data,
                            &room_members(),
                            &room_permissions(),
                            local_tracks(),
                            queue,
                            current,
                            audio_src,
                            playing,
                            playhead,
                            shuffle,
                            repeat_mode,
                            history,
                            notice,
                        );
                    } else if event.event == "room-closed" {
                        let queue_restore_error = if role == "guest" {
                            restore_guest_queue(queue).err()
                        } else {
                            None
                        };
                        active_room_code.set(String::new());
                        room_role.set(String::new());
                        room_host_token.set(String::new());
                        room_members.set(Vec::new());
                        room_permissions.set(RoomPermissions::default());
                        room_shared_playlists.set(Vec::new());
                        notice.set(queue_restore_error.map_or_else(
                            || {
                                if role == "guest" {
                                    "The room was closed by its host.".to_string()
                                } else {
                                    "The room expired after its last listener disconnected."
                                        .to_string()
                                }
                            },
                            |error| format!("The room ended. Queue restoration failed: {error}"),
                        ));
                        clear_saved_room_session(room_sessions, &url, &code, notice);
                        break;
                    } else if event.event == "error" {
                        let error = event
                            .data
                            .as_str()
                            .unwrap_or("Room socket connection failed.");
                        if last_error != error {
                            notice.set(error.to_string());
                            last_error = error.to_string();
                        }
                        if role == "host" {
                            last_host_state.clear();
                            next_host_state_retry = now + 5000.0;
                        }
                    }
                }
            }
            Err(error) if last_error != error => {
                notice.set(error.clone());
                last_error = error.clone();
            }
            Err(_) => {}
        }

        if active_room_code().is_empty() {
            continue;
        }

        if role == "host" {
            if let Some(party) = pushed_party.as_ref() {
                last_error.clear();
                if !party.members.iter().any(|member| member == &name) {
                    active_room_code.set(String::new());
                    room_role.set(String::new());
                    room_host_token.set(String::new());
                    room_members.set(Vec::new());
                    room_permissions.set(RoomPermissions::default());
                    room_shared_playlists.set(Vec::new());
                    notice.set("You are no longer a member of this room.".to_string());
                    clear_saved_room_session(room_sessions, &url, &code, notice);
                    continue;
                }
                room_members.set(party.members.clone());
                room_permissions.set(party.permissions.clone());
                room_shared_playlists.set(party.playlists.clone());
                if queue() != party.queue {
                    queue.set(party.queue.clone());
                    if let Err(error) = browser::save("wavelength-queue", &party.queue) {
                        notice.set(error);
                    }
                }
                shuffle.set(party.shuffle);
                repeat_mode.set(
                    if matches!(party.repeat_mode.as_str(), "off" | "all" | "one") {
                        party.repeat_mode.clone()
                    } else {
                        "off".to_string()
                    },
                );
                let shared_queue = party.queue.iter().map(room_track_json).collect::<Vec<_>>();
                last_host_state = room_host_state_signature(
                    &shared_queue,
                    &party.playlists,
                    &party.permissions,
                    &party.repeat_mode,
                    party.shuffle,
                );
            }

            if let Some(track) = current().filter(|track| !track.key().is_empty()) {
                let track_id = track.key().to_string();
                if (track_id != last_playback_track || playing() != last_playback_state)
                    && now >= next_playback_control_retry
                {
                    let control = serde_json::json!({
                        "currentTrackId": track_id,
                        "currentTime": playhead().max(0.0),
                        "isPlaying": playing(),
                        "sentAtEpochMs": now as i64,
                    });
                    match browser::emit_room_playback_control(&control).await {
                        Ok(status) if status == "sent" => {
                            last_playback_track = track_id.clone();
                            last_playback_state = playing();
                            last_progress_sent = now;
                            next_playback_control_retry = 0.0;
                        }
                        Ok(status) if status == "unavailable" => {
                            last_playback_track = track_id.clone();
                            last_playback_state = playing();
                            next_playback_control_retry = now + 1000.0;
                        }
                        Ok(_) => {}
                        Err(error) => {
                            if last_error != error {
                                notice.set(error.clone());
                                last_error = error;
                            }
                            next_playback_control_retry = now + 2000.0;
                        }
                    }
                }
                if track_id == last_playback_track && playing() && now - last_progress_sent >= 250.0
                {
                    let progress = serde_json::json!({
                        "currentTrackId": track_id,
                        "currentTime": playhead().max(0.0),
                        "duration": media_duration().max(0.0),
                    });
                    match browser::emit_room_playback_progress(&progress).await {
                        Ok(status) if status == "sent" || status == "unavailable" => {
                            last_progress_sent = now;
                        }
                        Ok(_) => {}
                        Err(error) if last_error != error => {
                            notice.set(error.clone());
                            last_error = error;
                        }
                        Err(_) => {}
                    }
                }
            } else {
                last_playback_track.clear();
            }

            let mut upload_candidates = queue();
            if let Some(track) = current() {
                upload_candidates.push(track);
            }
            if now >= next_upload_retry
                && upload_candidates
                    .iter()
                    .any(|track| track.is_local() && track.audio_id.is_none())
            {
                match upload_room_queue_tracks(
                    &url,
                    &code,
                    &host_token,
                    queue,
                    current,
                    local_tracks,
                )
                .await
                {
                    Ok(()) => {
                        next_upload_retry = now + 5000.0;
                        last_host_state.clear();
                    }
                    Err(error) => {
                        let error = format!("Room track upload failed: {error}");
                        if last_error != error {
                            notice.set(error.clone());
                            last_error = error;
                        }
                        next_upload_retry = now + 5000.0;
                    }
                }
            }

            let mut shared_queue = queue()
                .into_iter()
                .filter(|track| !track.is_local() || track.audio_id.is_some())
                .collect::<Vec<_>>();
            if let Some(track) =
                current().filter(|track| !track.is_local() || track.audio_id.is_some())
            {
                if !shared_queue.iter().any(|item| item.key() == track.key()) {
                    shared_queue.insert(0, track);
                }
            }
            let shared_queue = shared_queue.iter().map(room_track_json).collect::<Vec<_>>();
            let shared_playlists = room_playlist_references(&playlists());
            let permissions = room_permissions();
            let repeat = repeat_mode();
            let is_shuffle = shuffle();
            let signature = room_host_state_signature(
                &shared_queue,
                &shared_playlists,
                &permissions,
                &repeat,
                is_shuffle,
            );
            if signature != last_host_state && now >= next_host_state_retry {
                let update = serde_json::json!({
                    "queue": shared_queue,
                    "playlists": shared_playlists,
                    "repeatMode": repeat,
                    "shuffle": is_shuffle,
                    "permissions": permissions,
                    "syncRevision": now as i64,
                });
                match browser::emit_room_host_state(&update).await {
                    Ok(status) if status == "sent" => {
                        last_host_state = signature;
                        next_host_state_retry = 0.0;
                        last_error.clear();
                    }
                    Ok(status) if status == "unavailable" => {
                        next_host_state_retry = now + 1000.0;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        if last_error != error {
                            notice.set(error.clone());
                            last_error = error;
                        }
                        next_host_state_retry = now + 5000.0;
                    }
                }
            }
            continue;
        }

        let Some(party) = pushed_party else {
            continue;
        };
        last_error.clear();
        if !party.members.iter().any(|member| member == &name) {
            let queue_restore_error = if role == "guest" {
                restore_guest_queue(queue).err()
            } else {
                None
            };
            active_room_code.set(String::new());
            room_role.set(String::new());
            room_host_token.set(String::new());
            room_members.set(Vec::new());
            room_permissions.set(RoomPermissions::default());
            room_shared_playlists.set(Vec::new());
            notice.set(queue_restore_error.map_or_else(
                || "You are no longer a member of this room.".to_string(),
                |error| {
                    format!(
                        "You are no longer a member of this room. Queue restoration failed: {error}"
                    )
                },
            ));
            clear_saved_room_session(room_sessions, &url, &code, notice);
            continue;
        }
        room_members.set(party.members.clone());
        room_permissions.set(party.permissions);
        room_shared_playlists.set(party.playlists);
        if role != "guest" {
            continue;
        }
        shuffle.set(party.shuffle);
        if matches!(party.repeat_mode.as_str(), "off" | "all" | "one") {
            repeat_mode.set(party.repeat_mode);
        } else {
            repeat_mode.set("off".to_string());
        }

        queue.set(party.queue.clone());
        if let Err(error) = browser::save("wavelength-queue", &party.queue) {
            notice.set(error);
        }
        if party.current_track_id.is_empty() {
            if current().is_some() {
                let _ = browser::pause();
                current.set(None);
                audio_src.set(String::new());
                playing.set(false);
                playhead.set(0.0);
            }
            continue;
        }
        let track = party
            .queue
            .iter()
            .chain(local_tracks().iter())
            .find(|track| track.key() == party.current_track_id)
            .cloned();
        let Some(track) = track else {
            if last_error != "The room's current track is unavailable." {
                notice.set("The room's current track is unavailable.".to_string());
                last_error = "The room's current track is unavailable.".to_string();
            }
            continue;
        };
        let needs_track = current()
            .as_ref()
            .is_none_or(|active| active.key() != party.current_track_id);
        if needs_track {
            play_track_direct(track, current, audio_src, playing, queue, history, notice);
            continue;
        }
        let target_time = if party.is_playing {
            party
                .start_epoch
                .map(|start| ((now_epoch_millis() - start).max(0)) as f64 / 1000.0)
                .unwrap_or(party.paused_at.max(0.0))
        } else {
            party.paused_at.max(0.0)
        };
        if (playhead() - target_time).abs() > 0.75 {
            playhead.set(target_time);
            browser::seek(target_time);
        }
        if party.is_playing != playing() {
            if party.is_playing {
                match browser::play() {
                    Ok(()) => playing.set(true),
                    Err(error) => notice.set(error),
                }
            } else {
                let _ = browser::pause();
                playing.set(false);
            }
        }
    }
}

fn apply_host_room_command(
    envelope: serde_json::Value,
    members: &[String],
    permissions: &RoomPermissions,
    source: Vec<Track>,
    mut queue: Signal<Vec<Track>>,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    mut playing: Signal<bool>,
    mut playhead: Signal<f64>,
    mut shuffle: Signal<bool>,
    mut repeat_mode: Signal<String>,
    history: Signal<Vec<Track>>,
    mut notice: Signal<String>,
) {
    let Some(username) = envelope.get("username").and_then(serde_json::Value::as_str) else {
        return;
    };
    if !members.iter().any(|member| member == username) {
        return;
    }
    let Some(command) = envelope.get("command") else {
        return;
    };
    let action = command
        .get("action")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    match action {
        "add-track" if permissions.can_add => {
            let Some(value) = command.get("track") else {
                return;
            };
            let Ok(track) = serde_json::from_value::<Track>(value.clone()) else {
                notice.set("A room member sent an invalid track.".to_string());
                return;
            };
            let mut tracks = queue();
            if tracks.iter().any(|item| item.key() == track.key())
                || current()
                    .as_ref()
                    .is_some_and(|item| item.key() == track.key())
            {
                return;
            }
            tracks.push(track);
            queue.set(tracks.clone());
            save_saved("wavelength-queue", &tracks, notice);
        }
        "remove-track" if permissions.can_manage_queue => {
            let id = command
                .get("trackId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let mut tracks = queue();
            tracks.retain(|track| track.key() != id);
            queue.set(tracks.clone());
            save_saved("wavelength-queue", &tracks, notice);
        }
        "clear-queue" if permissions.can_manage_queue => {
            queue.set(Vec::new());
            save_saved("wavelength-queue", &Vec::<Track>::new(), notice);
        }
        "reorder-queue" if permissions.can_manage_queue => {
            let id = command
                .get("trackId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let Some(target) = command
                .get("targetIndex")
                .and_then(serde_json::Value::as_u64)
                .map(|index| index as usize)
            else {
                return;
            };
            let mut tracks = queue();
            if let Some(source) = tracks.iter().position(|track| track.key() == id) {
                if target < tracks.len() && source != target {
                    let moved = tracks.remove(source);
                    tracks.insert(target.min(tracks.len()), moved);
                    queue.set(tracks.clone());
                    save_saved("wavelength-queue", &tracks, notice);
                }
            }
        }
        "play-track" if permissions.can_control_playback => {
            let Some(value) = command.get("track") else {
                return;
            };
            match serde_json::from_value::<Track>(value.clone()) {
                Ok(track) => {
                    play_track_direct(track, current, audio_src, playing, queue, history, notice)
                }
                Err(error) => notice.set(format!("Room track could not be played: {error}")),
            }
        }
        "toggle-playback" if permissions.can_control_playback => {
            if current().is_none() {
                return;
            }
            if playing() {
                if let Err(error) = browser::pause() {
                    notice.set(error);
                } else {
                    playing.set(false);
                }
            } else if let Err(error) = browser::play() {
                notice.set(error);
            } else {
                playing.set(true);
            }
        }
        "previous-track" if permissions.can_control_playback => {
            previous_track(
                history,
                current,
                audio_src,
                playing,
                queue,
                source.clone(),
                notice,
            );
        }
        "next-track" if permissions.can_control_playback => {
            next_track(
                current,
                audio_src,
                playing,
                queue,
                history,
                source,
                shuffle,
                repeat_mode,
                notice,
            );
        }
        "seek" if permissions.can_control_playback => {
            let Some(position) = command
                .get("currentTime")
                .and_then(serde_json::Value::as_f64)
                .filter(|value| value.is_finite() && *value >= 0.0)
            else {
                return;
            };
            playhead.set(position);
            browser::seek(position);
        }
        "toggle-shuffle" if permissions.can_control_playback => {
            shuffle.set(!shuffle());
        }
        "toggle-repeat" if permissions.can_control_playback => {
            repeat_mode.set(next_repeat_mode(&repeat_mode()).to_string());
        }
        _ => {}
    }
}

fn format_activity_time(timestamp: &serde_json::Value) -> String {
    use chrono::TimeZone;
    let Some(milliseconds) = timestamp_millis(timestamp).filter(|value| value.is_finite()) else {
        return "—".to_string();
    };
    chrono::Local
        .timestamp_millis_opt(milliseconds as i64)
        .single()
        .map(|date| date.format("%I:%M:%S %p").to_string())
        .unwrap_or_else(|| "—".to_string())
}

fn record_play_event(
    track: Track,
    mut play_events: Signal<Vec<PlayEvent>>,
    mut play_counts: Signal<std::collections::HashMap<String, u64>>,
    notice: Signal<String>,
) {
    let id = track.key().to_string();
    let mut counts = play_counts();
    *counts.entry(id.clone()).or_default() += 1;
    play_counts.set(counts.clone());
    save_saved("wavelength-play-counts", &counts, notice);

    let mut events = play_events();
    events.push(PlayEvent {
        id,
        title: track.title,
        artist: if track.artist.trim().is_empty() {
            "Unknown artist".to_string()
        } else {
            track.artist
        },
        played_at: serde_json::json!(now_epoch_millis()),
    });
    if events.len() > 5_000 {
        events.drain(..events.len() - 5_000);
    }
    play_events.set(events.clone());
    save_saved("wavelength-play-events", &events, notice);
}

fn home_statistics(
    history: &[Track],
    events: &[PlayEvent],
    play_counts: &std::collections::HashMap<String, u64>,
    period: &str,
) -> (Vec<(String, u64)>, Vec<(String, String, u64)>) {
    let cutoff = stats_cutoff(period);
    let mut artists = std::collections::HashMap::<String, u64>::new();
    let mut songs = std::collections::HashMap::<String, (String, String, u64)>::new();

    if period == "all" {
        for track in history {
            let count = *play_counts.get(track.key()).unwrap_or(&1);
            let artist = if track.artist.trim().is_empty() {
                "Unknown artist"
            } else {
                &track.artist
            };
            *artists.entry(artist.to_string()).or_default() += count;
            songs.insert(
                track.key().to_string(),
                (track.title.clone(), artist.to_string(), count),
            );
        }
    } else {
        for event in events.iter().filter(|event| {
            timestamp_millis(&event.played_at).is_some_and(|played_at| played_at >= cutoff)
        }) {
            let artist = if event.artist.trim().is_empty() {
                "Unknown artist"
            } else {
                &event.artist
            };
            *artists.entry(artist.to_string()).or_default() += 1;
            let song = songs.entry(event.id.clone()).or_insert_with(|| {
                let track = history.iter().find(|track| track.key() == event.id);
                (
                    track
                        .map(|track| track.title.clone())
                        .filter(|title| !title.is_empty())
                        .unwrap_or_else(|| event.title.clone()),
                    artist.to_string(),
                    0,
                )
            });
            song.2 += 1;
        }
    }

    let mut artist_rows = artists.into_iter().collect::<Vec<_>>();
    artist_rows.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.to_lowercase().cmp(&right.0.to_lowercase()))
    });
    artist_rows.truncate(5);

    let mut song_rows = songs.into_values().collect::<Vec<_>>();
    song_rows.sort_by(|left, right| {
        right
            .2
            .cmp(&left.2)
            .then_with(|| left.0.to_lowercase().cmp(&right.0.to_lowercase()))
    });
    song_rows.truncate(5);
    (artist_rows, song_rows)
}

fn stats_cutoff(period: &str) -> f64 {
    if period == "all" {
        return 0.0;
    }
    use chrono::Datelike;
    let now = chrono::Local::now();
    if period == "week" {
        return (now.timestamp_millis() - 7 * 24 * 60 * 60 * 1000) as f64;
    }
    let months = match period {
        "month" => 1,
        "3months" => 3,
        "6months" => 6,
        "year" => 12,
        _ => return 0.0,
    };
    let day = now.day();
    let first_of_month = now.with_day(1).unwrap_or(now);
    let cutoff = first_of_month
        .checked_sub_months(chrono::Months::new(months))
        .unwrap_or(first_of_month);
    let first_of_next_month = cutoff
        .checked_add_months(chrono::Months::new(1))
        .unwrap_or(cutoff);
    let last_day = first_of_next_month
        .checked_sub_days(chrono::Days::new(1))
        .unwrap_or(cutoff)
        .day();
    cutoff
        .with_day(day.min(last_day))
        .unwrap_or(cutoff)
        .timestamp_millis() as f64
}

fn save_qobuz_credentials(
    app_id: Signal<String>,
    app_secret: Signal<String>,
    user_token: Signal<String>,
    mut configured: Signal<bool>,
    mut modal_open: Signal<bool>,
    mut saving: Signal<bool>,
    mut notice: Signal<String>,
) {
    let app_id = app_id().trim().to_string();
    let app_secret = app_secret().trim().to_string();
    let user_token = user_token().trim().to_string();
    if app_id.is_empty() || app_secret.is_empty() || user_token.is_empty() {
        notice.set("Enter the Qobuz App ID, App secret, and User token.".to_string());
        return;
    }

    saving.set(true);
    spawn(async move {
        let body = serde_json::json!({
            "app_id": app_id,
            "app_secret": app_secret,
            "user_token": user_token,
        });
        match browser::request_json_method::<QobuzSettings>(
            &format!("{AUDIO_SERVICE}/settings/qobuz"),
            "PUT",
            &body,
        )
        .await
        {
            Ok(settings) => {
                configured.set(settings.configured);
                modal_open.set(false);
                notice.set("Qobuz credentials saved.".to_string());
            }
            Err(error) => notice.set(error),
        }
        saving.set(false);
    });
}

fn parse_lyric_time(value: &str) -> Option<f64> {
    let (minutes, seconds) = value.split_once(':')?;
    let minutes = minutes.parse::<u64>().ok()?;
    let seconds = seconds.parse::<f64>().ok()?;
    if seconds >= 60.0 || !seconds.is_finite() {
        return None;
    }
    Some(minutes as f64 * 60.0 + seconds)
}

fn parse_synced_lyrics(source: &str) -> Vec<LyricLine> {
    let mut result = Vec::new();
    for line in source.lines() {
        let mut remaining = line.trim();
        if remaining.is_empty() {
            continue;
        }
        let mut timestamps = Vec::new();
        while let Some(tag) = remaining.strip_prefix('[') {
            let Some(end) = tag.find(']') else {
                break;
            };
            if let Some(time) = parse_lyric_time(&tag[..end]) {
                timestamps.push(time);
            }
            remaining = &tag[end + 1..];
        }
        let text = remaining.trim();
        if text.is_empty() || (timestamps.is_empty() && line.trim().starts_with('[')) {
            continue;
        }
        if timestamps.is_empty() {
            result.push(LyricLine {
                time: None,
                text: text.to_string(),
            });
        } else {
            result.extend(timestamps.into_iter().map(|time| LyricLine {
                time: Some(time),
                text: text.to_string(),
            }));
        }
    }
    result
}

fn lyric_track_key(track: &Track) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{:?}",
        track.source.as_deref().unwrap_or_default(),
        track.key(),
        track.title.trim(),
        track.artist.trim(),
        track.album.as_deref().unwrap_or_default(),
        track.duration
    )
}

fn load_track_lyrics(
    track: Track,
    lyric_track_id: Signal<String>,
    mut lyrics: Signal<String>,
    mut lines: Signal<Vec<LyricLine>>,
    mut loading: Signal<bool>,
) {
    lyrics.set(String::new());
    lines.set(Vec::new());
    if track.artist.trim().is_empty() || track.artist.eq_ignore_ascii_case("local file") {
        lyrics.set("Lyrics are not available for this track.".to_string());
        loading.set(false);
        return;
    }
    loading.set(true);
    let request_id = lyric_track_key(&track);
    spawn(async move {
        let response = browser::lyrics(
            &track.title,
            &track.artist,
            track.album.as_deref().unwrap_or_default(),
            track.duration.unwrap_or_default(),
            track.source.as_deref().unwrap_or_default(),
        )
        .await;
        if lyric_track_id() != request_id {
            return;
        }
        match response {
            Ok(Some(response)) => {
                let text = response
                    .synced_lyrics
                    .filter(|text| !text.trim().is_empty())
                    .or_else(|| response.plain_lyrics.filter(|text| !text.trim().is_empty()))
                    .unwrap_or_default();
                if text.trim().is_empty() {
                    lyrics.set("No lyrics found.".to_string());
                } else {
                    lines.set(parse_synced_lyrics(&text));
                    lyrics.set(text);
                }
            }
            Ok(None) => lyrics.set("No lyrics found.".to_string()),
            Err(_) => lyrics.set("Lyrics are unavailable for this track.".to_string()),
        }
        loading.set(false);
    });
}

fn load_activity(
    mut logs: Signal<Vec<ActivityLog>>,
    mut loading: Signal<bool>,
    mut error_message: Signal<String>,
) {
    if loading() {
        return;
    }
    loading.set(true);
    error_message.set(String::new());
    spawn(async move {
        match browser::get_json::<ActivityLogsResponse>(&format!("{AUDIO_SERVICE}/logs?limit=200"))
            .await
        {
            Ok(response) => {
                if logs() != response.logs {
                    logs.set(response.logs);
                }
            }
            Err(error) => error_message.set(error),
        }
        loading.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
fn format_error(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&value, &JsValue::from_str("message"))
                .ok()?
                .as_string()
        })
        .unwrap_or_else(|| "The browser request failed.".to_string())
}

fn format_clock(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "0:00".to_string();
    }
    let seconds = seconds as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
