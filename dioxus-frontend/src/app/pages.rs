use super::AppState;
use crate::*;
use dioxus::prelude::*;
use std::rc::Rc;

mod activitypage;
mod album_detail;
mod customization;
mod equalizer;
mod historypage;
mod homepage;
mod librarypage;
mod now_playing;
mod playlist_detail;
mod playlistspage;
mod queuepage;
mod searchpage;
mod settingspage;
mod spotify_library;
mod spotify_playlists;

use self::activitypage::ActivityPage;
use self::album_detail::AlbumDetailPage;
use self::customization::CustomizationPage;
use self::equalizer::EqualizerRoutePage;
use self::historypage::HistoryPage;
use self::homepage::HomePage;
use self::librarypage::LibraryPage;
use self::now_playing::NowPlayingPage;
use self::playlist_detail::PlaylistDetailPage;
use self::playlistspage::PlaylistsPage;
use self::queuepage::QueuePage;
use self::searchpage::SearchPage;
use self::settingspage::SettingsPage;
use self::spotify_library::SpotifyLibraryPage;
use self::spotify_playlists::SpotifyPlaylistsPage;

#[derive(Clone, PartialEq)]
pub(super) struct PageData {
    can_control_playback: bool,
    favorite_playlist_selected: bool,
    playlist_detail_is_local: bool,
    playlist_detail: Option<Playlist>,
    album_detail_name: Option<String>,
    album_detail_tracks: Vec<Track>,
    visible_local_playlist_count: usize,
    favorite_playlist_id: String,
    now_playing_previous_source: Vec<Track>,
    now_playing_next_source: Vec<Track>,
    active_track_is_favorite: bool,
    home_featured_track: Option<Track>,
    recently_added: Vec<Track>,
    home_playlists: Vec<Playlist>,
    home_albums: Vec<(String, Vec<Track>)>,
    artist_stats: Vec<(String, u64)>,
    song_stats: Vec<(String, String, u64)>,
    show_home_demo: bool,
    background_color_value: String,
    background_picker_color: String,
    library_folder_rows: Vec<LocalFolderRow>,
    library_groups: Vec<(String, Vec<Track>)>,
    active_lyric_index: isize,
    progress_max: f64,
    displayed_playhead: f64,
    elapsed_label: String,
    duration_label: String,
}

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn PageContent() -> Element {
    let state = use_context::<AppState>();
    let mut page = state.page;
    let mut page_back_stack = state.page_back_stack;
    let mut page_forward_stack = state.page_forward_stack;
    let mut last_navigation_page = state.last_navigation_page;
    let mut navigating_history = state.navigating_history;
    let mut sidebar_collapsed = state.sidebar_collapsed;
    let mut narrow_window = state.narrow_window;
    let mut narrow_right_panel = state.narrow_right_panel;
    let mut sidebar_auto_collapsed = state.sidebar_auto_collapsed;
    let mut window_controls_available = state.window_controls_available;
    let mut search_query = state.search_query;
    let mut search_source = state.search_source;
    let mut topbar_search_open = state.topbar_search_open;
    let results = state.results;
    let mut local_tracks = state.local_tracks;
    let spotify_tracks = state.spotify_tracks;
    let spotify_busy = state.spotify_busy;
    let mut spotify_playlists = state.spotify_playlists;
    let mut spotify_playlist_tracks = state.spotify_playlist_tracks;
    let mut spotify_playlist_selected = state.spotify_playlist_selected;
    let mut spotify_playlist_name = state.spotify_playlist_name;
    let mut spotify_playlist_create_name = state.spotify_playlist_create_name;
    let spotify_playlist_create_busy = state.spotify_playlist_create_busy;
    let mut spotify_playlist_save_busy = state.spotify_playlist_save_busy;
    let spotify_playlists_busy = state.spotify_playlists_busy;
    let mut spotify_playlists_page_loaded = state.spotify_playlists_page_loaded;
    let spotify_playlist_tracks_busy = state.spotify_playlist_tracks_busy;
    let mut queue = state.queue;
    let mut dragging_queue_item = state.dragging_queue_item;
    let mut history = state.history;
    let play_counts = state.play_counts;
    let play_events = state.play_events;
    let mut last_stats_track = state.last_stats_track;
    let favorites = state.favorites;
    let mut playlists = state.playlists;
    let mut playlist_name = state.playlist_name;
    let mut playlist_create_open = state.playlist_create_open;
    let mut playlist_create_favorites = state.playlist_create_favorites;
    let mut playlist_query = state.playlist_query;
    let mut selected_playlist_id = state.selected_playlist_id;
    let album_detail_name = (state.album_detail_name)();
    let mut folder_manager_open = state.folder_manager_open;
    let mut saved_folder_names = state.saved_folder_names;
    let mut context_track = state.context_track;
    let mut context_playlist = state.context_playlist;
    let mut detail_track = state.detail_track;
    let mut lyrics_fullscreen = state.lyrics_fullscreen;
    let shuffle = state.shuffle;
    let repeat_mode = state.repeat_mode;
    let volume = state.volume;
    let previous_volume = state.previous_volume;
    let current = state.current;
    let audio_src = state.audio_src;
    let playing = state.playing;
    let mut lyrics = state.lyrics;
    let mut lyric_lines = state.lyric_lines;
    let mut lyric_loading = state.lyric_loading;
    let mut lyric_track_id = state.lyric_track_id;
    let mut playhead = state.playhead;
    let mut media_duration = state.media_duration;
    let busy = state.busy;
    let mut notice = state.notice;
    let activity_logs = state.activity_logs;
    let activity_error = state.activity_error;
    let activity_loading = state.activity_loading;
    let mut activity_loaded = state.activity_loaded;
    let mut activity_tab = state.activity_tab;
    let notification_logs = state.notification_logs;
    let mut spotify_profile = state.spotify_profile;
    let mut local_avatar = state.local_avatar;
    let mut library_query = state.library_query;
    let mut library_sort = state.library_sort;
    let mut expanded_library_folders = state.expanded_library_folders;
    let mut collapsed_library_groups = state.collapsed_library_groups;
    let mut theme = state.theme;
    let mut accent = state.accent;
    let mut accent_preset = state.accent_preset;
    let mut effective_appearance_theme = state.effective_appearance_theme;
    let mut background_color = state.background_color;
    let mut background_hex_input = state.background_hex_input;
    let mut background_hex_error = state.background_hex_error;
    let mut background_picker_open = state.background_picker_open;
    let mut accent_picker_open = state.accent_picker_open;
    let mut accent_hex_input = state.accent_hex_input;
    let mut accent_hex_error = state.accent_hex_error;
    let mut stats_period = state.stats_period;
    let mut qobuz_configured = state.qobuz_configured;
    let mut qobuz_settings_loaded = state.qobuz_settings_loaded;
    let mut qobuz_modal_open = state.qobuz_modal_open;
    let mut qobuz_app_id = state.qobuz_app_id;
    let mut qobuz_app_secret = state.qobuz_app_secret;
    let mut qobuz_user_token = state.qobuz_user_token;
    let qobuz_saving = state.qobuz_saving;
    let room_url = state.room_url;
    let mut room_url_input = state.room_url_input;
    let mut room_passcode = state.room_passcode;
    let room_connecting = state.room_connecting;
    let room_access = state.room_access;
    let mut room_code = state.room_code;
    let mut room_name = state.room_name;
    let mut room_name_mode = state.room_name_mode;
    let active_room_code = state.active_room_code;
    let room_role = state.room_role;
    let room_host_token = state.room_host_token;
    let room_members = state.room_members;
    let room_permissions = state.room_permissions;
    let room_shared_playlists = state.room_shared_playlists;
    let mut room_panel_open = state.room_panel_open;
    let mut queue_panel_open = state.queue_panel_open;
    let room_copy_feedback = state.room_copy_feedback;

    let active_track = current();
    let can_control_playback = room_role() != "guest" || room_permissions().can_control_playback;
    let can_manage_queue = room_role() != "guest" || room_permissions().can_manage_queue;
    let player_track_caption = active_track
        .as_ref()
        .map(|track| {
            format!(
                "{} · {}",
                track.artist,
                track.source.as_deref().unwrap_or("Audio")
            )
        })
        .unwrap_or_else(|| "Select music to begin".to_string());
    let selected_playlist = selected_playlist_id();
    let favorite_playlist_selected = selected_playlist.as_deref() == Some("favorite-songs");
    let local_playlist = playlists()
        .into_iter()
        .find(|playlist| Some(playlist.id.clone()) == selected_playlist);
    let shared_playlist = room_shared_playlists()
        .into_iter()
        .find(|playlist| Some(playlist.id.clone()) == selected_playlist);
    let playlist_detail_is_local = local_playlist.is_some();
    let favorite_playlist = favorite_playlist_selected.then(|| Playlist {
        id: "favorite-songs".to_string(),
        name: "Favorite Songs".to_string(),
        tracks: favorites(),
    });
    let playlist_detail = favorite_playlist
        .clone()
        .or_else(|| local_playlist.clone())
        .or_else(|| {
            shared_playlist.as_ref().map(|playlist| {
                resolve_room_playlist(
                    playlist,
                    &local_tracks(),
                    &favorites(),
                    &history(),
                    &queue(),
                )
            })
        });
    let playlist_tracks = playlist_detail
        .as_ref()
        .map(|playlist| playlist.tracks.clone())
        .unwrap_or_default();
    let album_detail_tracks = album_detail_name
        .as_ref()
        .and_then(|name| {
            group_local_tracks(local_tracks(), "album")
                .into_iter()
                .find(|(group_name, _)| group_name == name)
                .map(|(_, tracks)| tracks)
        })
        .unwrap_or_default();
    let visible_local_playlist_count = playlists()
        .iter()
        .filter(|item| playlist_matches(&playlist_query(), &item.name))
        .count()
        + if playlist_matches(&playlist_query(), "Favorite Songs") {
            1
        } else {
            0
        };
    let favorite_playlist_id = "favorite-songs".to_string();
    let page_tracks = match page() {
        Page::Search => results(),
        Page::Library => local_tracks(),
        Page::SpotifyLibrary => spotify_tracks(),
        Page::SpotifyPlaylists => spotify_playlist_tracks(),
        Page::PlaylistDetail => playlist_tracks.clone(),
        Page::Queue => queue(),
        Page::History => history(),
        _ => Vec::new(),
    };
    let fallback_tracks = active_track.as_ref().and_then(|track| {
        [
            results(),
            spotify_playlist_tracks(),
            spotify_tracks(),
            favorites(),
            local_tracks(),
            history(),
            playlist_tracks.clone(),
        ]
        .into_iter()
        .find(|tracks| {
            tracks
                .iter()
                .any(|candidate| candidate.key() == track.key())
        })
    });
    let playback_source = if !queue().is_empty() {
        queue()
    } else if !page_tracks.is_empty() {
        page_tracks.clone()
    } else {
        fallback_tracks.clone().unwrap_or_default()
    };
    let player_end_source = playback_source.clone();
    let player_previous_source = playback_source.clone();
    let player_next_source = playback_source.clone();
    let now_playing_previous_source = playback_source.clone();
    let now_playing_next_source = playback_source.clone();
    let active_track_is_favorite = active_track.as_ref().is_some_and(|track| {
        favorites()
            .iter()
            .any(|favorite| favorite.key() == track.key())
    });
    let home_featured_track = local_tracks().first().cloned();
    let recently_added = local_tracks().into_iter().take(9).collect::<Vec<_>>();
    let home_playlists = playlists()
        .into_iter()
        .filter(|playlist| !playlist.tracks.is_empty())
        .take(8)
        .collect::<Vec<_>>();
    let home_albums = group_local_tracks(local_tracks(), "album");
    let (artist_stats, song_stats) =
        home_statistics(&history(), &play_events(), &play_counts(), &stats_period());
    let show_home_demo = history().is_empty() && play_events().is_empty();
    let background_color_value = if background_color().is_empty() {
        default_background_color(&effective_appearance_theme()).to_string()
    } else {
        background_color()
    };
    let background_picker_color = normalize_hex_color(&background_hex_input())
        .unwrap_or_else(|| background_color_value.clone());
    let filtered_local = local_tracks()
        .into_iter()
        .filter(|track| {
            let query = library_query().to_lowercase();
            query.is_empty()
                || format!(
                    "{} {} {}",
                    track.title,
                    track.artist,
                    track.album.as_deref().unwrap_or_default()
                )
                .to_lowercase()
                .contains(&query)
        })
        .collect::<Vec<_>>();
    let (library_folder_rows, library_groups) = if library_sort() == "folders" {
        (
            local_folder_rows(&filtered_local, &expanded_library_folders()),
            Vec::new(),
        )
    } else {
        (
            Vec::new(),
            group_local_tracks(filtered_local.clone(), &library_sort()),
        )
    };
    let active_lyric_index = lyric_lines()
        .iter()
        .rposition(|line| line.time.is_some_and(|time| time <= playhead()))
        .map(|index| index as isize)
        .unwrap_or(-1);

    let fallback_duration = active_track
        .as_ref()
        .and_then(|track| track.duration)
        .unwrap_or_default();
    let displayed_duration = if media_duration() > 0.0 {
        media_duration()
    } else {
        fallback_duration
    };
    let progress_max = media_duration().max(fallback_duration).max(1.0);
    let displayed_playhead = if active_track.is_some() {
        playhead()
    } else {
        0.0
    };
    let player_duration = if active_track.is_some() {
        displayed_duration
    } else {
        0.0
    };
    let player_progress = if displayed_duration > 0.0 {
        (playhead() / displayed_duration * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let elapsed_label = format_clock(displayed_playhead);
    let duration_label = format_clock(player_duration);
    let search_text = search_query().trim().to_lowercase();
    let topbar_suggestions = if search_text.is_empty() {
        Vec::new()
    } else {
        local_tracks()
            .into_iter()
            .filter(|track| {
                format!(
                    "{} {} {}",
                    track.title,
                    track.artist,
                    track.album.as_deref().unwrap_or_default()
                )
                .to_lowercase()
                .contains(&search_text)
            })
            .take(10)
            .collect::<Vec<_>>()
    };

    let data = Rc::new(PageData {
        can_control_playback: can_control_playback,
        favorite_playlist_selected: favorite_playlist_selected,
        playlist_detail_is_local: playlist_detail_is_local,
        playlist_detail: playlist_detail,
        album_detail_name: album_detail_name,
        album_detail_tracks: album_detail_tracks,
        visible_local_playlist_count: visible_local_playlist_count,
        favorite_playlist_id: favorite_playlist_id,
        now_playing_previous_source: now_playing_previous_source,
        now_playing_next_source: now_playing_next_source,
        active_track_is_favorite: active_track_is_favorite,
        home_featured_track: home_featured_track,
        recently_added: recently_added,
        home_playlists: home_playlists,
        home_albums: home_albums,
        artist_stats: artist_stats,
        song_stats: song_stats,
        show_home_demo: show_home_demo,
        background_color_value: background_color_value,
        background_picker_color: background_picker_color,
        library_folder_rows: library_folder_rows,
        library_groups: library_groups,
        active_lyric_index: active_lyric_index,
        progress_max: progress_max,
        displayed_playhead: displayed_playhead,
        elapsed_label: elapsed_label,
        duration_label: duration_label,
    });

    rsx! {
                        match page() {
                            Page::Home => rsx! { HomePage { data: data.clone() } },
                            Page::NowPlaying => rsx! { NowPlayingPage { data: data.clone() } },
                            Page::Search => rsx! { SearchPage { data: data.clone() } },
                            Page::Library => rsx! { LibraryPage { data: data.clone() } },
                            Page::Playlists => rsx! { PlaylistsPage { data: data.clone() } },
                            Page::PlaylistDetail => rsx! { PlaylistDetailPage { data: data.clone() } },
                            Page::AlbumDetail => rsx! { AlbumDetailPage { data: data.clone() } },
                            Page::SpotifyLibrary => rsx! { SpotifyLibraryPage { data: data.clone() } },
                            Page::SpotifyPlaylists => rsx! { SpotifyPlaylistsPage { data: data.clone() } },
                            Page::Queue => rsx! { QueuePage { data: data.clone() } },
                            Page::History => rsx! { HistoryPage { data: data.clone() } },
                            Page::Settings => rsx! { SettingsPage { data: data.clone() } },
                            Page::Activity => rsx! { ActivityPage { data: data.clone() } },
                            Page::Equalizer => rsx! { EqualizerRoutePage { data: data.clone() } },
                            Page::Customization => rsx! { CustomizationPage { data: data.clone() } },
                        }
    }
}
