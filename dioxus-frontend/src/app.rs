use crate::browser;
use crate::components::{AppNotice, Icon, QueueDownloadIndicator, UpdateNotice};
use crate::*;
use dioxus::html::{input_data::MouseButton, point_interaction::PointerInteraction};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct AppState {
    page: Signal<Page>,
    page_back_stack: Signal<Vec<Page>>,
    page_forward_stack: Signal<Vec<Page>>,
    last_navigation_page: Signal<Page>,
    navigating_history: Signal<bool>,
    sidebar_collapsed: Signal<bool>,
    narrow_window: Signal<bool>,
    narrow_right_panel: Signal<Option<RightPanel>>,
    sidebar_auto_collapsed: Signal<bool>,
    window_controls_available: Signal<bool>,
    search_query: Signal<String>,
    search_source: Signal<String>,
    topbar_search_open: Signal<bool>,
    results: Signal<Vec<Track>>,
    local_tracks: Signal<Vec<Track>>,
    spotify_tracks: Signal<Vec<Track>>,
    spotify_busy: Signal<bool>,
    spotify_playlists: Signal<Vec<SpotifyPlaylist>>,
    spotify_playlist_tracks: Signal<Vec<Track>>,
    spotify_playlist_selected: Signal<Option<SpotifyPlaylist>>,
    spotify_playlist_name: Signal<String>,
    spotify_playlist_create_name: Signal<String>,
    spotify_playlist_create_busy: Signal<bool>,
    spotify_playlist_save_busy: Signal<bool>,
    spotify_playlists_busy: Signal<bool>,
    spotify_playlists_page_loaded: Signal<bool>,
    spotify_playlist_tracks_busy: Signal<bool>,
    queue: Signal<Vec<Track>>,
    queue_download_status: Signal<std::collections::HashMap<String, DownloadStatus>>,
    queue_download_error: Signal<String>,
    dragging_queue_item: Signal<Option<usize>>,
    history: Signal<Vec<Track>>,
    play_counts: Signal<std::collections::HashMap<String, u64>>,
    play_events: Signal<Vec<PlayEvent>>,
    last_stats_track: Signal<String>,
    favorites: Signal<Vec<Track>>,
    playlists: Signal<Vec<Playlist>>,
    playlist_name: Signal<String>,
    playlist_create_open: Signal<bool>,
    playlist_create_favorites: Signal<Vec<String>>,
    playlist_query: Signal<String>,
    playlist_list_view: Signal<bool>,
    selected_playlist_id: Signal<Option<String>>,
    playlist_add_target: Signal<Option<String>>,
    album_detail_name: Signal<Option<String>>,
    folder_manager_open: Signal<bool>,
    saved_folder_names: Signal<Vec<String>>,
    download_folder_name: Signal<String>,
    context_track: Signal<Option<TrackContextMenu>>,
    context_playlist: Signal<Option<PlaylistContextMenu>>,
    detail_track: Signal<Option<Track>>,
    lyrics_fullscreen: Signal<bool>,
    shuffle: Signal<bool>,
    repeat_mode: Signal<String>,
    volume: Signal<f64>,
    previous_volume: Signal<f64>,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    lyrics: Signal<String>,
    lyric_lines: Signal<Vec<LyricLine>>,
    lyric_loading: Signal<bool>,
    lyric_track_id: Signal<String>,
    playhead: Signal<f64>,
    media_duration: Signal<f64>,
    busy: Signal<bool>,
    notice: Signal<String>,
    update_available: Signal<Option<UpdateInfo>>,
    update_dismissed_version: Signal<Option<String>>,
    update_checking: Signal<bool>,
    update_installing: Signal<bool>,
    activity_logs: Signal<Vec<ActivityLog>>,
    activity_error: Signal<String>,
    activity_loading: Signal<bool>,
    activity_loaded: Signal<bool>,
    activity_tab: Signal<String>,
    notification_logs: Signal<Vec<NotificationLog>>,
    spotify_profile: Signal<Option<SpotifyProfile>>,
    local_avatar: Signal<Option<String>>,
    library_query: Signal<String>,
    library_sort: Signal<String>,
    expanded_library_folders: Signal<Vec<String>>,
    collapsed_library_groups: Signal<Vec<String>>,
    theme: Signal<String>,
    accent: Signal<String>,
    accent_preset: Signal<String>,
    effective_appearance_theme: Signal<String>,
    background_color: Signal<String>,
    font_scale: Signal<f64>,
    background_hex_input: Signal<String>,
    background_hex_error: Signal<String>,
    background_picker_open: Signal<bool>,
    accent_picker_open: Signal<bool>,
    accent_hex_input: Signal<String>,
    accent_hex_error: Signal<String>,
    stats_period: Signal<String>,
    qobuz_configured: Signal<bool>,
    qobuz_settings_loaded: Signal<bool>,
    qobuz_modal_open: Signal<bool>,
    qobuz_app_id: Signal<String>,
    qobuz_app_secret: Signal<String>,
    qobuz_user_token: Signal<String>,
    qobuz_saving: Signal<bool>,
    room_url: Signal<String>,
    room_url_input: Signal<String>,
    room_passcode: Signal<String>,
    room_connecting: Signal<bool>,
    room_access: Signal<String>,
    room_code: Signal<String>,
    room_name: Signal<String>,
    room_name_mode: Signal<String>,
    active_room_code: Signal<String>,
    room_role: Signal<String>,
    room_host_token: Signal<String>,
    room_members: Signal<Vec<String>>,
    room_permissions: Signal<RoomPermissions>,
    room_shared_playlists: Signal<Vec<RoomSharedPlaylist>>,
    room_sessions: Signal<Vec<RoomSession>>,
    room_panel_open: Signal<bool>,
    queue_panel_open: Signal<bool>,
    room_copy_feedback: Signal<String>,
}

mod pages;

#[component]
pub(crate) fn App() -> Element {
    let mut page = use_signal(|| Page::Home);
    let mut page_back_stack = use_signal(Vec::<Page>::new);
    let mut page_forward_stack = use_signal(Vec::<Page>::new);
    let mut last_navigation_page = use_signal(|| Page::Home);
    let mut navigating_history = use_signal(|| false);
    let mut sidebar_collapsed = use_signal(|| false);
    let mut narrow_window = use_signal(|| false);
    let mut narrow_right_panel = use_signal(|| Option::<RightPanel>::None);
    let mut sidebar_auto_collapsed = use_signal(|| false);
    let mut window_controls_available = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let mut search_source = use_signal(|| "spotify".to_string());
    let mut topbar_search_open = use_signal(|| false);
    let results = use_signal(Vec::<Track>::new);
    let mut local_tracks = use_signal(Vec::<Track>::new);
    let spotify_tracks = use_signal(Vec::<Track>::new);
    let spotify_busy = use_signal(|| false);
    let spotify_playlists = use_signal(Vec::<SpotifyPlaylist>::new);
    let spotify_playlist_tracks = use_signal(Vec::<Track>::new);
    let spotify_playlist_selected = use_signal(|| Option::<SpotifyPlaylist>::None);
    let spotify_playlist_name = use_signal(String::new);
    let spotify_playlist_create_name = use_signal(String::new);
    let spotify_playlist_create_busy = use_signal(|| false);
    let spotify_playlist_save_busy = use_signal(|| false);
    let spotify_playlists_busy = use_signal(|| false);
    let mut spotify_playlists_page_loaded = use_signal(|| false);
    let spotify_playlist_tracks_busy = use_signal(|| false);
    let queue = use_signal(Vec::<Track>::new);
    let mut queue_download_status =
        use_signal(std::collections::HashMap::<String, DownloadStatus>::new);
    let mut queue_download_error = use_signal(String::new);
    let mut dragging_queue_item = use_signal(|| Option::<usize>::None);
    let history = use_signal(Vec::<Track>::new);
    let play_counts = use_signal(std::collections::HashMap::<String, u64>::new);
    let play_events = use_signal(Vec::<PlayEvent>::new);
    let mut last_stats_track = use_signal(String::new);
    let favorites = use_signal(Vec::<Track>::new);
    let mut playlists = use_signal(Vec::<Playlist>::new);
    let mut playlist_name = use_signal(String::new);
    let mut playlist_create_open = use_signal(|| false);
    let mut playlist_create_favorites = use_signal(Vec::<String>::new);
    let mut playlist_query = use_signal(String::new);
    let playlist_list_view = use_signal(|| false);
    let mut selected_playlist_id = use_signal(|| Option::<String>::None);
    let mut playlist_add_target = use_signal(|| Option::<String>::None);
    let mut album_detail_name = use_signal(|| Option::<String>::None);
    let mut folder_manager_open = use_signal(|| false);
    let mut saved_folder_names = use_signal(Vec::<String>::new);
    let download_folder_name = use_signal(String::new);
    let mut context_track = use_signal(|| Option::<TrackContextMenu>::None);
    let mut context_playlist = use_signal(|| Option::<PlaylistContextMenu>::None);
    let mut detail_track = use_signal(|| Option::<Track>::None);
    let mut lyrics_fullscreen = use_signal(|| false);
    let shuffle = use_signal(|| false);
    let repeat_mode = use_signal(|| "off".to_string());
    let volume = use_signal(|| 0.3f64);
    let previous_volume = use_signal(|| 0.3f64);
    let current = use_signal(|| Option::<Track>::None);
    let audio_src = use_signal(String::new);
    let playing = use_signal(|| false);
    let mut lyrics = use_signal(String::new);
    let mut lyric_lines = use_signal(Vec::<LyricLine>::new);
    let mut lyric_loading = use_signal(|| false);
    let mut lyric_track_id = use_signal(String::new);
    let mut playhead = use_signal(|| 0.0f64);
    let mut media_duration = use_signal(|| 0.0f64);
    let busy = use_signal(|| false);
    let mut notice = use_signal(String::new);
    let mut update_available = use_signal(|| Option::<UpdateInfo>::None);
    let update_dismissed_version = use_signal(|| Option::<String>::None);
    let mut update_checking = use_signal(|| false);
    let update_installing = use_signal(|| false);
    let activity_logs = use_signal(Vec::<ActivityLog>::new);
    let activity_error = use_signal(String::new);
    let activity_loading = use_signal(|| false);
    let mut activity_loaded = use_signal(|| false);
    let activity_tab = use_signal(|| "downloads".to_string());
    let notification_logs = use_signal(Vec::<NotificationLog>::new);
    let mut spotify_profile = use_signal(|| Option::<SpotifyProfile>::None);
    let local_avatar = use_signal(|| Option::<String>::None);
    let library_query = use_signal(String::new);
    let library_sort = use_signal(|| "folders".to_string());
    let expanded_library_folders = use_signal(Vec::<String>::new);
    let collapsed_library_groups = use_signal(Vec::<String>::new);
    let theme = use_signal(|| "dark".to_string());
    let mut accent = use_signal(|| "#d7c7a4".to_string());
    let mut accent_preset = use_signal(String::new);
    let mut effective_appearance_theme = use_signal(|| "dark".to_string());
    let background_color = use_signal(String::new);
    let font_scale = use_signal(|| 1.0f64);
    let mut background_hex_input = use_signal(|| default_background_color("dark").to_string());
    let background_hex_error = use_signal(String::new);
    let background_picker_open = use_signal(|| false);
    let accent_picker_open = use_signal(|| false);
    let accent_hex_input = use_signal(String::new);
    let accent_hex_error = use_signal(String::new);
    let stats_period = use_signal(|| "all".to_string());
    let mut qobuz_configured = use_signal(|| false);
    let mut qobuz_settings_loaded = use_signal(|| false);
    let mut qobuz_modal_open = use_signal(|| false);
    let mut qobuz_app_id = use_signal(String::new);
    let mut qobuz_app_secret = use_signal(String::new);
    let mut qobuz_user_token = use_signal(String::new);
    let qobuz_saving = use_signal(|| false);
    let room_url = use_signal(String::new);
    let mut room_url_input = use_signal(String::new);
    let room_passcode = use_signal(String::new);
    let room_connecting = use_signal(|| false);
    let room_access = use_signal(String::new);
    let mut room_code = use_signal(String::new);
    let mut room_name = use_signal(String::new);
    let mut room_name_mode = use_signal(|| "custom".to_string());
    let active_room_code = use_signal(String::new);
    let room_role = use_signal(String::new);
    let room_host_token = use_signal(String::new);
    let room_members = use_signal(Vec::<String>::new);
    let room_permissions = use_signal(RoomPermissions::default);
    let room_shared_playlists = use_signal(Vec::<RoomSharedPlaylist>::new);
    let room_sessions = use_signal(Vec::<RoomSession>::new);
    let mut room_panel_open = use_signal(|| false);
    let mut queue_panel_open = use_signal(|| false);
    let room_copy_feedback = use_signal(String::new);

    use_context_provider(|| AppState {
        page,
        page_back_stack,
        page_forward_stack,
        last_navigation_page,
        navigating_history,
        sidebar_collapsed,
        narrow_window,
        narrow_right_panel,
        sidebar_auto_collapsed,
        window_controls_available,
        search_query,
        search_source,
        topbar_search_open,
        results,
        local_tracks,
        spotify_tracks,
        spotify_busy,
        spotify_playlists,
        spotify_playlist_tracks,
        spotify_playlist_selected,
        spotify_playlist_name,
        spotify_playlist_create_name,
        spotify_playlist_create_busy,
        spotify_playlist_save_busy,
        spotify_playlists_busy,
        spotify_playlists_page_loaded,
        spotify_playlist_tracks_busy,
        queue,
        queue_download_status,
        queue_download_error,
        dragging_queue_item,
        history,
        play_counts,
        play_events,
        last_stats_track,
        favorites,
        playlists,
        playlist_name,
        playlist_create_open,
        playlist_create_favorites,
        playlist_query,
        playlist_list_view,
        selected_playlist_id,
        playlist_add_target,
        album_detail_name,
        folder_manager_open,
        saved_folder_names,
        download_folder_name,
        context_track,
        context_playlist,
        detail_track,
        lyrics_fullscreen,
        shuffle,
        repeat_mode,
        volume,
        previous_volume,
        current,
        audio_src,
        playing,
        lyrics,
        lyric_lines,
        lyric_loading,
        lyric_track_id,
        playhead,
        media_duration,
        busy,
        notice,
        update_available,
        update_dismissed_version,
        update_checking,
        update_installing,
        activity_logs,
        activity_error,
        activity_loading,
        activity_loaded,
        activity_tab,
        notification_logs,
        spotify_profile,
        local_avatar,
        library_query,
        library_sort,
        expanded_library_folders,
        collapsed_library_groups,
        theme,
        accent,
        accent_preset,
        effective_appearance_theme,
        background_color,
        font_scale,
        background_hex_input,
        background_hex_error,
        background_picker_open,
        accent_picker_open,
        accent_hex_input,
        accent_hex_error,
        stats_period,
        qobuz_configured,
        qobuz_settings_loaded,
        qobuz_modal_open,
        qobuz_app_id,
        qobuz_app_secret,
        qobuz_user_token,
        qobuz_saving,
        room_url,
        room_url_input,
        room_passcode,
        room_connecting,
        room_access,
        room_code,
        room_name,
        room_name_mode,
        active_room_code,
        room_role,
        room_host_token,
        room_members,
        room_permissions,
        room_shared_playlists,
        room_sessions,
        room_panel_open,
        queue_panel_open,
        room_copy_feedback,
    });

    use_effect(move || {
        if let Err(error) = browser::install_hover_marquee() {
            notice.set(error);
        }
        if let Err(error) = browser::install_app_behaviors() {
            notice.set(error);
        }
    });

    use_effect(move || {
        let mut download_folder_name = download_folder_name;
        let mut notice = notice;
        spawn(async move {
            match browser::saved_download_folder_name().await {
                Ok(Some(name)) => download_folder_name.set(name),
                Ok(None) => {}
                Err(error) => notice.set(error),
            }
        });
    });

    use_effect(move || {
        if let Err(error) = browser::watch_narrow_window(move |is_narrow: bool| {
            narrow_window.set(is_narrow);
        }) {
            notice.set(error);
        }
    });

    use_effect(move || {
        let is_narrow = narrow_window();
        let active_panel = if room_panel_open() {
            Some(RightPanel::Room)
        } else if queue_panel_open() {
            Some(RightPanel::Queue)
        } else {
            None
        };
        let is_collapsed = sidebar_collapsed();
        let return_panel = narrow_right_panel();

        if is_narrow {
            if let Some(panel) = active_panel {
                if !is_collapsed {
                    sidebar_collapsed.set(true);
                    narrow_right_panel.set(Some(panel));
                    sidebar_auto_collapsed.set(true);
                } else if return_panel != Some(panel) {
                    narrow_right_panel.set(Some(panel));
                }
            } else if is_collapsed && return_panel.is_some() {
                sidebar_collapsed.set(false);
                narrow_right_panel.set(None);
                sidebar_auto_collapsed.set(false);
            }
        } else {
            if sidebar_auto_collapsed() && active_panel.is_some() {
                sidebar_collapsed.set(false);
            }
            if narrow_right_panel().is_some() {
                narrow_right_panel.set(None);
            }
            if sidebar_auto_collapsed() {
                sidebar_auto_collapsed.set(false);
            }
        }
    });

    use_effect(move || match browser::is_desktop_window() {
        Ok(available) => window_controls_available.set(available),
        Err(error) => notice.set(error),
    });

    use_effect(move || {
        match browser::is_desktop_window() {
            Ok(false) => return,
            Err(error) => {
                notice.set(error);
                return;
            }
            Ok(true) => {}
        }
        spawn(async move {
            crate::sleep_ms(10_000).await;
            loop {
                update_checking.set(true);
                match browser::check_for_updates().await {
                    Ok(update) => {
                        if update.as_ref().map(|update| update.version.as_str())
                            != update_dismissed_version().as_deref()
                        {
                            update_available.set(update);
                        }
                    }
                    Err(error) => notice.set(format!("Unable to check for updates: {error}")),
                }
                update_checking.set(false);
                crate::sleep_ms(6 * 60 * 60 * 1000).await;
            }
        });
    });

    use_effect(move || {
        let active_page = page();
        let previous_page = last_navigation_page();
        if active_page != Page::Search {
            playlist_add_target.set(None);
        }
        if active_page == previous_page {
            return;
        }

        if navigating_history() {
            navigating_history.set(false);
        } else {
            let mut back_stack = page_back_stack();
            back_stack.push(previous_page);
            page_back_stack.set(back_stack);
            page_forward_stack.set(Vec::new());
        }
        last_navigation_page.set(active_page);
    });

    use_effect(move || {
        spawn(async move {
            let mut poll_delay_ms = 2500;
            loop {
                let mut track_ids = queue()
                    .iter()
                    .filter(|track| !track.is_local())
                    .map(|track| track.audio_service_id())
                    .filter(|id| !id.is_empty())
                    .collect::<Vec<_>>();
                if let Some(track) = current().filter(|track| !track.is_local()) {
                    let id = track.audio_service_id();
                    if !id.is_empty() && !track_ids.contains(&id) {
                        track_ids.push(id);
                    }
                }
                if track_ids.is_empty() {
                    queue_download_status.set(std::collections::HashMap::new());
                    queue_download_error.set(String::new());
                    poll_delay_ms = 2500;
                } else {
                    match browser::download_status(&track_ids).await {
                        Ok(statuses) => {
                            queue_download_status.set(statuses);
                            queue_download_error.set(String::new());
                            poll_delay_ms = 2500;
                        }
                        Err(error) => {
                            queue_download_error.set(error);
                            poll_delay_ms = (poll_delay_ms * 2).min(30_000);
                        }
                    }
                }
                if page() == Page::Activity {
                    load_activity(activity_logs, activity_loading, activity_error);
                    poll_delay_ms = poll_delay_ms.min(5000);
                }
                crate::sleep_ms(poll_delay_ms as u64).await;
            }
        });
    });

    use_effect(move || {
        load_saved("wavelength-playlists", playlists, notice);
        load_saved("wavelength-favorites", favorites, notice);
        load_saved("wavelength-play-history", history, notice);
        load_saved("wavelength-play-counts", play_counts, notice);
        load_saved("wavelength-play-events", play_events, notice);
        load_saved("wavelength-queue", queue, notice);
        load_saved("wavelength-notification-logs", notification_logs, notice);
        load_saved("wavelength-avatar", local_avatar, notice);
        load_saved("wavelength-theme", theme, notice);
        load_saved("wavelength-accent", accent, notice);
        load_saved("wavelength-accent-preset", accent_preset, notice);
        load_saved("wavelength-background-color", background_color, notice);
        load_accessibility_font_scale(font_scale, notice);
        load_saved("wavelength-volume", volume, notice);
        load_saved("wavelength-previous-volume", previous_volume, notice);
        load_saved("wavelength-room-url", room_url, notice);
        room_url_input.set(room_url());
        load_room_server_passcode(room_passcode, notice);
        load_saved("wavelength-room-access", room_access, notice);
        load_saved("wavelength-party-code", active_room_code, notice);
        load_saved("wavelength-party-role", room_role, notice);
        load_saved("wavelength-party-host-token", room_host_token, notice);
        load_saved("wavelength-party-name", room_name, notice);
        load_saved("wavelength-party-sessions", room_sessions, notice);
        if !active_room_code().is_empty()
            && !room_sessions().iter().any(|session| {
                session.code == active_room_code() && session.room_url == room_url()
            })
        {
            let session = RoomSession {
                code: active_room_code(),
                role: room_role(),
                name: room_name(),
                host_token: room_host_token(),
                room_url: room_url(),
                access_token: room_access(),
            };
            if let Err(error) = remember_room_session(room_sessions, session) {
                notice.set(error);
            }
        }
        match browser::room_invite_code() {
            Ok(Some(code)) => {
                room_code.set(code.to_ascii_uppercase());
                room_panel_open.set(true);
            }
            Ok(None) => {}
            Err(error) => notice.set(error),
        }
        if !active_room_code().is_empty() && !room_url().is_empty() {
            let url = room_url();
            let code = active_room_code();
            let access = room_access();
            let host = room_host_token();
            let name = room_name();
            let saved_role = room_role();
            let mut active_code = active_room_code;
            let mut role = room_role;
            let mut members = room_members;
            let mut permissions = room_permissions;
            let mut shared_playlists = room_shared_playlists;
            let queue = queue;
            spawn(async move {
                match browser::room_request::<RoomParty>(
                    &url,
                    &format!("/parties/{}", code),
                    "GET",
                    &serde_json::json!({}),
                    &access,
                    &host,
                )
                .await
                {
                    Ok(party) => {
                        if !party.members.contains(&name) {
                            let queue_restore_error = if saved_role == "guest" {
                                restore_guest_queue(queue).err()
                            } else {
                                None
                            };
                            active_code.set(String::new());
                            role.set(String::new());
                            members.set(Vec::new());
                            permissions.set(RoomPermissions::default());
                            shared_playlists.set(Vec::new());
                            notice.set(queue_restore_error.map_or_else(
                                || "The saved room session has expired.".to_string(),
                                |error| format!(
                                    "The saved room session has expired. Queue restoration failed: {error}"
                                ),
                            ));
                            clear_saved_room_session(room_sessions, &url, &code, notice);
                        } else if party.host == name && !party.host_authorized {
                            active_code.set(String::new());
                            role.set(String::new());
                            members.set(Vec::new());
                            permissions.set(RoomPermissions::default());
                            shared_playlists.set(Vec::new());
                            notice.set(
                                "This room needs to be recreated before host controls can be used."
                                    .to_string(),
                            );
                            clear_saved_room_session(room_sessions, &url, &code, notice);
                        } else {
                            let restored_role = if party.host == name { "host" } else { "guest" };
                            members.set(party.members);
                            permissions.set(party.permissions.clone());
                            shared_playlists.set(party.playlists);
                            role.set(restored_role.to_string());
                            if let Err(error) = browser::connect_room_socket(
                                &url,
                                &code,
                                &name,
                                &access,
                                restored_role,
                                &party.permissions,
                            ) {
                                notice.set(error);
                            }
                        }
                    }
                    Err(error) => {
                        if error == "Too many attempts. Please wait before trying again" {
                            notice.set(format!("Unable to restore the room session: {error}"));
                            return;
                        }
                        let queue_restore_error = if saved_role == "guest" {
                            restore_guest_queue(queue).err()
                        } else {
                            None
                        };
                        active_code.set(String::new());
                        role.set(String::new());
                        members.set(Vec::new());
                        permissions.set(RoomPermissions::default());
                        shared_playlists.set(Vec::new());
                        notice.set(queue_restore_error.map_or_else(
                            || format!("Unable to restore the room session: {error}"),
                            |restore_error| format!(
                                "Unable to restore the room session: {error}. Queue restoration failed: {restore_error}"
                            ),
                        ));
                        clear_saved_room_session(room_sessions, &url, &code, notice);
                    }
                }
            });
        }
        spawn(async move {
            match browser::scan_library().await {
                Ok(tracks) => local_tracks.set(tracks),
                Err(error) => notice.set(error),
            }
        });
        spawn(async move {
            match browser::restore_spotify().await {
                Ok(Some(restore)) => {
                    if restore.connected_now {
                        page.set(Page::Settings);
                        notice.set("Spotify connected.".to_string());
                    }
                    spotify_profile.set(Some(restore.profile));
                }
                Ok(None) => {}
                Err(error) => notice.set(error),
            }
        });
    });

    use_effect(move || {
        if page() == Page::Activity && !activity_loaded() {
            activity_loaded.set(true);
            load_activity(activity_logs, activity_loading, activity_error);
        }
    });

    use_effect(move || {
        if page() == Page::Playlists
            && spotify_profile().is_some()
            && !spotify_playlists_page_loaded()
        {
            spotify_playlists_page_loaded.set(true);
            load_spotify_playlists(spotify_playlists, spotify_playlists_busy, notice);
        }
    });

    use_effect(move || {
        let track = current();
        let next_id = track
            .as_ref()
            .map(|track| track.key().to_string())
            .unwrap_or_default();
        if next_id != lyric_track_id() {
            lyric_track_id.set(next_id);
            match track {
                Some(track) => load_track_lyrics(track, lyrics, lyric_lines, lyric_loading),
                None => {
                    lyrics.set(String::new());
                    lyric_lines.set(Vec::new());
                    lyric_loading.set(false);
                }
            }
        }
    });

    use_effect(move || {
        spawn(room_sync_loop(
            room_url,
            room_access,
            room_name,
            active_room_code,
            room_role,
            room_host_token,
            room_members,
            room_permissions,
            room_shared_playlists,
            room_sessions,
            playlists,
            queue,
            current,
            audio_src,
            playing,
            playhead,
            media_duration,
            shuffle,
            repeat_mode,
            notice,
            local_tracks,
            history,
        ));
    });

    use_effect(move || {
        let value = theme();
        let color = accent();
        let background = background_color();
        let resolved_theme = match browser::effective_theme(&value) {
            Ok(value) => value,
            Err(error) => {
                notice.set(error);
                return;
            }
        };
        effective_appearance_theme.set(resolved_theme.clone());

        let selected_preset = accent_preset();
        let preset = ACCENT_PRESETS
            .iter()
            .find(|(name, _)| *name == selected_preset.as_str())
            .or_else(|| {
                ACCENT_PRESETS
                    .iter()
                    .find(|(_, preset_color)| *preset_color == color.as_str())
            });
        if let Some((name, base_color)) = preset {
            if selected_preset != *name {
                accent_preset.set((*name).to_string());
                save_string("wavelength-accent-preset", name, notice);
            }
            let safe_color = contrast_adjusted_accent(base_color, &resolved_theme, &background);
            if color != safe_color {
                accent.set(safe_color.clone());
                save_string("wavelength-accent", &safe_color, notice);
                return;
            }
        } else if !selected_preset.is_empty() {
            accent_preset.set(String::new());
            save_string("wavelength-accent-preset", "", notice);
        }

        if preset.is_none() && !accent_color_has_contrast(&color, &resolved_theme, &background) {
            let safe_color = accessible_accent_color(&resolved_theme, &background);
            accent.set(safe_color.clone());
            save_string("wavelength-accent", &safe_color, notice);
            return;
        }
        if let Err(error) = browser::set_appearance(&value, &color, &background) {
            notice.set(error);
        }
    });

    use_effect(move || {
        let background = background_color();
        let theme = effective_appearance_theme();
        background_hex_input.set(if background.is_empty() {
            default_background_color(&theme).to_string()
        } else {
            background
        });
    });

    use_effect(move || {
        if let Err(error) = browser::set_font_scale(font_scale()) {
            notice.set(error);
        }
    });

    use_effect(move || {
        if !playing() {
            return;
        }
        let Some(track) = current() else {
            return;
        };
        let track_id = track.key().to_string();
        if track_id == last_stats_track() {
            return;
        }
        last_stats_track.set(track_id.clone());
        record_play_event(track, play_events, play_counts, notice);
    });

    use_effect(move || {
        if page() != Page::Settings || qobuz_settings_loaded() {
            return;
        }
        qobuz_settings_loaded.set(true);
        spawn(async move {
            match browser::get_json::<QobuzSettings>(&format!("{AUDIO_SERVICE}/qobuz/settings"))
                .await
            {
                Ok(settings) => qobuz_configured.set(settings.configured),
                Err(error) => notice.set(error),
            }
        });
    });

    use_effect(move || {
        if let Err(error) = browser::set_volume(volume()) {
            notice.set(error);
        }
    });

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
    let favorite_playlist = favorite_playlist_selected.then(|| Playlist {
        id: "favorite-songs".to_string(),
        name: "Favorite Songs".to_string(),
        tracks: favorites(),
    });
    let playlist_detail = favorite_playlist
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
            playlist_tracks,
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
        page_tracks
    } else {
        fallback_tracks.unwrap_or_default()
    };
    let player_end_source = playback_source.clone();
    let player_previous_source = playback_source.clone();
    let player_next_source = playback_source.clone();
    let active_track_is_favorite = active_track.as_ref().is_some_and(|track| {
        favorites()
            .iter()
            .any(|favorite| favorite.key() == track.key())
    });
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

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/styles/minimal.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/styles/desktop.css") }
        div {
            id: "app",
            class: match (room_panel_open() || queue_panel_open(), sidebar_collapsed()) {
                (true, true) => "app-shell sidebar-collapsed",
                (true, false) => "app-shell",
                (false, true) => "app-shell sidebar-collapsed queue-closed",
                (false, false) => "app-shell queue-closed",
            },
            onpointerup: move |_| {
                if dragging_queue_item().is_some() {
                    dragging_queue_item.set(None);
                }
            },
            onpointercancel: move |_| {
                if dragging_queue_item().is_some() {
                    dragging_queue_item.set(None);
                }
            },
            header {
                class: "topbar",
                onmousedown: move |event| {
                    if event.data.trigger_button() == Some(MouseButton::Primary) {
                        browser::drag_window();
                    }
                },
                div { class: "topbar-start",
                    img { class: "topbar-brand-mark", src: asset!("/assets/images/icon.svg"), alt: "Wavelength" }
                    div {
                        class: "topbar-navigation",
                        aria_label: "Page navigation",
                        onmousedown: move |event| event.stop_propagation(),
                        button {
                            class: "topbar-navigation-button",
                            r#type: "button",
                            aria_label: "Go back",
                            title: "Back",
                            disabled: page_back_stack().is_empty(),
                            onclick: move |_| go_back(page, page_back_stack, page_forward_stack, navigating_history),
                            Icon { name: "chevron-left" }
                        }
                        button {
                            class: "topbar-navigation-button",
                            r#type: "button",
                            aria_label: "Go forward",
                            title: "Forward",
                            disabled: page_forward_stack().is_empty(),
                            onclick: move |_| go_forward(page, page_back_stack, page_forward_stack, navigating_history),
                            Icon { name: "chevron-right" }
                        }
                    }
                }
                div { class: "topbar-center",
                    div { class: "topbar-search-control",
                        form {
                            class: "topbar-search",
                            role: "search",
                            onmousedown: move |event| event.stop_propagation(),
                            onsubmit: move |event| {
                                event.prevent_default();
                                topbar_search_open.set(false);
                                page.set(Page::Search);
                                search_tracks(search_query(), search_source(), results, busy, notice);
                            },
                            div { class: "topbar-source-switch", role: "group", aria_label: "Search source",
                                button {
                                    class: if search_source() == "spotify" { "active" } else { "" },
                                    aria_pressed: search_source() == "spotify",
                                    r#type: "button",
                                    onclick: move |_| search_source.set("spotify".to_string()),
                                    "Spotify"
                                }
                                button {
                                    class: if search_source() == "youtube" { "active" } else { "" },
                                    aria_pressed: search_source() == "youtube",
                                    r#type: "button",
                                    onclick: move |_| search_source.set("youtube".to_string()),
                                    "YouTube"
                                }
                            }
                            label { class: "topbar-search-input",
                                Icon { name: "search" }
                                input {
                                    r#type: "text",
                                    value: "{search_query()}",
                                    placeholder: "Search songs, artists, albums...",
                                    aria_label: "Search songs",
                                    aria_expanded: topbar_search_open() && !search_query().trim().is_empty() && page() != Page::Search,
                                    aria_controls: "topbar-search-results",
                                    autocomplete: "off",
                                    onfocus: move |_| if page() != Page::Search { topbar_search_open.set(true) },
                                    oninput: move |event| {
                                        search_query.set(event.value());
                                        topbar_search_open.set(page() != Page::Search);
                                    },
                                }
                                if !search_query().is_empty() {
                                    button {
                                        class: "topbar-search-clear",
                                        r#type: "button",
                                        aria_label: "Clear search",
                                        onclick: move |_| {
                                            search_query.set(String::new());
                                            topbar_search_open.set(page() != Page::Search);
                                        },
                                        Icon { name: "close" }
                                    }
                                }
                                if page() != Page::Search && topbar_search_open() && !search_text.is_empty() {
                                    div {
                                        id: "topbar-search-results",
                                        class: "topbar-search-results",
                                        role: "region",
                                        aria_label: "Song suggestions",
                                        if topbar_suggestions.is_empty() {
                                            p { class: "topbar-search-empty",
                                                "No library matches. Press Enter to search "
                                                if search_source() == "youtube" { "YouTube." } else { "Spotify." }
                                            }
                                        } else {
                                            for track in topbar_suggestions.iter().cloned() {
                                                div { class: "topbar-search-result",
                                                    span { class: "mini-art",
                                                        if let Some(thumbnail) = track.thumbnail.as_deref() {
                                                            img { src: "{thumbnail}", alt: "" }
                                                        } else {
                                                            Icon { name: "disc" }
                                                        }
                                                    }
                                                    span { class: "topbar-search-copy",
                                                        strong { "{track.title}" }
                                                        small { "{track.artist}" }
                                                    }
                                                    span { class: "topbar-search-length", "{track.duration_label()}" }
                                                    button {
                                                        class: "icon-button topbar-search-play",
                                                        r#type: "button",
                                                        aria_label: "Play local suggestion",
                                                        title: "Play now",
                                                        onclick: move |_| {
                                                            topbar_search_open.set(false);
                                                            play_track(track.clone(), current, audio_src, playing, queue, history, notice);
                                                        },
                                                        Icon { name: "play" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "topbar-end",
                    button {
                        class: "avatar",
                        aria_label: "Settings",
                        title: "Settings",
                        onmousedown: move |event| event.stop_propagation(),
                        onclick: move |_| {
                            room_panel_open.set(false);
                            queue_panel_open.set(false);
                            topbar_search_open.set(false);
                            page.set(Page::Settings);
                        },
                        if let Some(source) = local_avatar().or_else(|| {
                            spotify_profile().and_then(|profile| {
                                profile.images.into_iter().find_map(|image| {
                                    (!image.url.is_empty()).then_some(image.url)
                                })
                            })
                        }) {
                            img { class: "avatar-photo", src: "{source}", alt: "" }
                        } else {
                            Icon { name: "user" }
                        }
                    }
                    if window_controls_available() {
                        div {
                            class: "window-controls",
                            role: "group",
                            aria_label: "Window controls",
                            onmousedown: move |event| event.stop_propagation(),
                            button {
                                class: "window-control",
                                r#type: "button",
                                aria_label: "Minimize window",
                                title: "Minimize",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Err(error) = browser::window_action("minimize").await {
                                            notice.set(error);
                                        }
                                    });
                                },
                                Icon { name: "minimize" }
                            }
                            button {
                                class: "window-control",
                                r#type: "button",
                                aria_label: "Maximize or restore window",
                                title: "Maximize or restore",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Err(error) = browser::window_action("toggle-maximize").await {
                                            notice.set(error);
                                        }
                                    });
                                },
                                Icon { name: "maximize" }
                            }
                            button {
                                class: "window-control window-close-control",
                                r#type: "button",
                                aria_label: "Close window",
                                title: "Close",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Err(error) = browser::window_action("close").await {
                                            notice.set(error);
                                        }
                                    });
                                },
                                Icon { name: "close" }
                            }
                        }
                    }
                }
            }
            div { class: "workspace-cluster",
                aside { class: "sidebar",
                    nav { aria_label: "Main navigation",
                        for item in [
                            (Page::Home, "Home", "home"),
                            (Page::Playlists, "Library", "library"),
                            (Page::Library, "Local files", "folder"),
                            (Page::History, "Listening history", "history"),
                        ] {
                            button {
                                class: if page() == item.0 || (item.0 == Page::Playlists && page() == Page::PlaylistDetail) { "active" } else { "" },
                                aria_label: item.1,
                                title: item.1,
                                onclick: move |_| {
                                    room_panel_open.set(false);
                                    page.set(item.0);
                                },
                                Icon { name: item.2 }
                                span { "{item.1}" }
                            }
                        }
                    }
                    button {
                        class: "sidebar-collapse-hover-zone sidebar-collapse-toggle",
                        aria_label: if sidebar_collapsed() { "Expand sidebar" } else { "Collapse sidebar" },
                        title: if sidebar_collapsed() { "Expand sidebar" } else { "Collapse sidebar" },
                        onclick: move |_| {
                            if narrow_window() {
                                if sidebar_collapsed() {
                                    sidebar_collapsed.set(false);
                                    if room_panel_open() || queue_panel_open() {
                                        room_panel_open.set(false);
                                        queue_panel_open.set(false);
                                        sidebar_auto_collapsed.set(false);
                                    }
                                } else if let Some(panel) = narrow_right_panel() {
                                    sidebar_collapsed.set(true);
                                    sidebar_auto_collapsed.set(false);
                                    match panel {
                                        RightPanel::Queue => {
                                            queue_panel_open.set(true);
                                            room_panel_open.set(false);
                                        }
                                        RightPanel::Room => {
                                            room_panel_open.set(true);
                                            queue_panel_open.set(false);
                                        }
                                    }
                                } else {
                                    sidebar_collapsed.set(true);
                                    sidebar_auto_collapsed.set(false);
                                }
                            } else {
                                sidebar_collapsed.toggle();
                            }
                        },
                        if sidebar_collapsed() {
                            Icon { name: "chevron-right" }
                        } else {
                            Icon { name: "chevron-left" }
                        }
                    }
                }

                main { class: "content",
                    section { class: "content-inner page-layout {page().class()}", aria_label: "Page content",
                        pages::PageContent {}
                        AppNotice { notice, notification_logs }
                        if let Some(update) = update_available() {
                            UpdateNotice {
                                update,
                                available: update_available,
                                dismissed_version: update_dismissed_version,
                                installing: update_installing,
                                notice
                            }
                        }
                    }
                }
                if queue_panel_open() {
                    aside { class: "right-rail queue-rail", aria_label: "Queue",
                        button {
                            class: "rail-resizer",
                            aria_label: "Resize queue panel",
                            onpointerdown: move |_| {
                                if let Err(error) = browser::start_rail_resize() {
                                    notice.set(error);
                                }
                            }
                        }
                        div { class: "rail-heading",
                            div {
                                p { class: "eyebrow", "Up next" }
                                h3 { "Queue " span { "{queue().len()}" } }
                            }
                            div { class: "rail-actions",
                                if !queue().is_empty() {
                                    button {
                                        class: "icon-button danger",
                                        aria_label: "Clear queue",
                                        title: "Clear queue",
                                        disabled: !can_manage_queue,
                                        onclick: move |_| clear_queue(queue, notice),
                                        Icon { name: "trash" }
                                    }
                                }
                            }
                        }
                        if queue().is_empty() {
                            p { class: "rail-empty", "Add tracks to play them next" }
                        } else {
                            div { class: "rail-queue-list",
                                for (index, track) in queue().iter().cloned().enumerate() {
                                    div {
                                        class: if dragging_queue_item() == Some(index) {
                                            "queue-row dragging"
                                        } else if active_track.as_ref().is_some_and(|active| active.key() == track.key()) {
                                            "queue-row now-playing"
                                        } else if index == 0 {
                                            "queue-row up-next"
                                        } else {
                                            "queue-row"
                                        },
                                        key: "{track.key()}-{index}",
                                        role: "listitem",
                                        tabindex: if can_manage_queue { "0" } else { "-1" },
                                        aria_label: "Reorder {track.title}; use arrow keys when focused",
                                        aria_keyshortcuts: "ArrowUp ArrowDown",
                                        onpointerdown: move |event| {
                                            if can_manage_queue
                                                && event.data().trigger_button()
                                                    == Some(MouseButton::Primary)
                                            {
                                                event.prevent_default();
                                                dragging_queue_item.set(Some(index));
                                            }
                                        },
                                        onpointerup: move |_| {
                                            if can_manage_queue {
                                                if let Some(source_index) = dragging_queue_item() {
                                                    move_queue_item(source_index, index, queue, notice);
                                                }
                                            }
                                            dragging_queue_item.set(None);
                                        },
                                        onpointercancel: move |_| dragging_queue_item.set(None),
                                        onkeydown: move |event| {
                                            if can_manage_queue {
                                                match event.key().to_string().as_str() {
                                                    "ArrowUp" if index > 0 => {
                                                        event.prevent_default();
                                                        move_queue_item(index, index - 1, queue, notice);
                                                    }
                                                    "ArrowDown" if index + 1 < queue().len() => {
                                                        event.prevent_default();
                                                        move_queue_item(index, index + 1, queue, notice);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        },
                                        span {
                                            class: "queue-source-dot {queue_source_class(&track)}",
                                            role: "img",
                                            aria_label: "{queue_source_label(&track)}",
                                            title: "{queue_source_label(&track)}"
                                        }
                                        span { class: "queue-art mini-art",
                                            if let Some(thumbnail) = track.thumbnail.as_deref() {
                                                img { src: "{thumbnail}", alt: "" }
                                            } else {
                                                Icon { name: "disc" }
                                            }
                                            if let Some(status) = queue_download_status().get(&track.audio_service_id()).cloned() {
                                                QueueDownloadIndicator { status }
                                            }
                                        }
                                        div { class: "queue-track",
                                            strong { "{track.title}" }
                                            small { "{track.artist}" }
                                        }
                                        button {
                                            class: "row-action",
                                            aria_label: "Play queued track",
                                            title: "Play now",
                                            disabled: !can_control_playback,
                                            onpointerdown: move |event| event.stop_propagation(),
                                            onkeydown: move |event| {
                                                if matches!(event.key().to_string().as_str(), "ArrowUp" | "ArrowDown") {
                                                    event.stop_propagation();
                                                }
                                            },
                                            onclick: move |_| play_track(track.clone(), current, audio_src, playing, queue, history, notice),
                                            Icon { name: "play" }
                                        }
                                        button {
                                            class: "row-action",
                                            aria_label: "Remove track from queue",
                                            title: "Remove from queue",
                                            disabled: !can_manage_queue,
                                            onpointerdown: move |event| event.stop_propagation(),
                                            onkeydown: move |event| {
                                                if matches!(event.key().to_string().as_str(), "ArrowUp" | "ArrowDown") {
                                                    event.stop_propagation();
                                                }
                                            },
                                            onclick: move |_| remove_queue_item(index, queue, notice),
                                            Icon { name: "close" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if room_panel_open() {
                    aside { class: "right-rail party-room-rail", aria_label: "Room settings",
                        button {
                            class: "rail-resizer",
                            aria_label: "Resize room settings panel",
                            onpointerdown: move |_| {
                                if let Err(error) = browser::start_rail_resize() {
                                    notice.set(error);
                                }
                            }
                        }
                        div { class: "rail-heading",
                            h2 { "Room settings" }
                            button { class: "icon-button", aria_label: "Close room settings", onclick: move |_| room_panel_open.set(false),
                                Icon { name: "close" }
                            }
                        }
                        div { class: "room-panel-summary", aria_live: "polite",
                            div { class: "room-panel-status",
                                span {
                                    class: if !active_room_code().is_empty() && !room_access().is_empty() {
                                        "room-status-indicator connected"
                                    } else {
                                        "room-status-indicator"
                                    }
                                }
                                strong {
                                    if !active_room_code().is_empty() && !room_access().is_empty() {
                                        "Room connected"
                                    } else if !active_room_code().is_empty() {
                                        "Reconnect to room"
                                    } else {
                                        "No active room"
                                    }
                                }
                            }
                            p { class: "room-idle-policy",
                                "Rooms close automatically after five minutes without activity. Create a new room if it expires."
                            }
                        }
                        if !room_sessions().is_empty() {
                            section { class: "party-room-section party-room-sessions", aria_label: "Open rooms",
                                h3 { class: "party-room-section-title", "Open rooms" }
                                div { class: "room-session-list",
                                    for session in room_sessions().iter().cloned() {
                                        button {
                                            class: if session.code == active_room_code()
                                                && session.room_url == room_url()
                                            {
                                                "room-session-button active"
                                            } else {
                                                "room-session-button"
                                            },
                                            aria_label: format!("Open room {}", session.code),
                                            disabled: session.code == active_room_code()
                                                && session.room_url == room_url(),
                                            onclick: move |_| switch_room(
                                                session.clone(),
                                                room_url,
                                                room_url_input,
                                                room_access,
                                                room_sessions,
                                                room_name,
                                                active_room_code,
                                                room_role,
                                                room_host_token,
                                                room_members,
                                                room_permissions,
                                                room_shared_playlists,
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
                                                notice,
                                            ),
                                            strong { "{session.code}" }
                                            small { "{session.name} · {session.role}" }
                                        }
                                    }
                                }
                            }
                        }
                        if room_url().is_empty() {
                            p { class: "party-member-empty", "Add a room server in Settings to use shared listening." }
                            button { class: "outline-button", onclick: move |_| {
                                room_panel_open.set(false);
                                page.set(Page::Settings);
                            }, "Open settings" }
                        } else if room_access().is_empty() && !active_room_code().is_empty() {
                            p { class: "party-member-empty", "Connect with the server passcode in Settings to manage this room." }
                        } else if !active_room_code().is_empty() {
                            p { class: "room-session-role", "You are {room_role()} as {room_name()}" }
                            section { class: "party-room-section party-invite-section",
                                h3 { class: "party-room-section-title", "Invite listeners" }
                                button {
                                    class: "party-code-button",
                                    aria_label: if room_copy_feedback() == "code" { "Room code copied" } else { "Copy room code" },
                                    onclick: move |_| copy_room_invite(active_room_code, false, room_copy_feedback, notice),
                                    strong { class: "party-code-value", "{active_room_code()}" }
                                    small { class: "party-code-hint",
                                        if room_copy_feedback() == "code" { "Copied" } else { "Copy room code" }
                                    }
                                }
                                button { class: "outline-button compact-action",
                                    aria_label: if room_copy_feedback() == "invite" { "Invite link copied" } else { "Copy invite link" },
                                    onclick: move |_| copy_room_invite(active_room_code, true, room_copy_feedback, notice),
                                    if room_copy_feedback() == "invite" { "Invite link copied" } else { "Copy invite link" }
                                }
                            }
                            if room_role() == "host" {
                                section { class: "party-room-section",
                                    h3 { class: "party-room-section-title", "Guest permissions" }
                                    p { class: "room-section-hint", "Choose what listeners can change during the session." }
                                    div { class: "guest-controls",
                                        label { class: "guest-control-toggle",
                                            span { "Allow guests to add tracks" }
                                            input { r#type: "checkbox", checked: room_permissions().can_add,
                                                onchange: move |_| toggle_room_permission(RoomPermission::AddTracks, room_url, room_access, active_room_code, room_name, room_host_token, room_permissions, notice)
                                            }
                                        }
                                        label { class: "guest-control-toggle",
                                            span { "Allow guests to manage the queue" }
                                            input { r#type: "checkbox", checked: room_permissions().can_manage_queue,
                                                onchange: move |_| toggle_room_permission(RoomPermission::ManageQueue, room_url, room_access, active_room_code, room_name, room_host_token, room_permissions, notice)
                                            }
                                        }
                                        label { class: "guest-control-toggle",
                                            span { "Allow guests to control playback" }
                                            input { r#type: "checkbox", checked: room_permissions().can_control_playback,
                                                onchange: move |_| toggle_room_permission(RoomPermission::ControlPlayback, room_url, room_access, active_room_code, room_name, room_host_token, room_permissions, notice)
                                            }
                                        }
                                    }
                                }
                            }
                            section { class: "party-room-section party-member-list", aria_label: "Room participants",
                                div { class: "party-member-heading",
                                    strong { "People in this room" }
                                    span { "{room_members().len()}" }
                                }
                                if room_members().is_empty() {
                                    p { class: "party-member-empty", "Waiting for listeners to join." }
                                } else {
                                    ul {
                                        for member in room_members().iter() {
                                            li {
                                                span { class: "party-member-avatar", "{member.chars().next().unwrap_or('?').to_ascii_uppercase()}" }
                                                span { class: "party-member-name", "{member}" }
                                                if member == &room_name() {
                                                    small { if room_role() == "host" { "Host · You" } else { "You" } }
                                                } else if room_role() == "guest" && room_members().first() == Some(member) {
                                                    small { "Host" }
                                                } else if room_role() == "host" && !room_name().is_empty() {
                                                    small { "Listener" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "party-room-end-action",
                                button { class: "outline-button compact-action", onclick: move |_| refresh_room(room_url, room_access, active_room_code, room_host_token, room_members, room_permissions, room_shared_playlists, notice), "Refresh room" }
                                button { class: if room_role() == "host" { "outline-button compact-action danger" } else { "outline-button compact-action" }, onclick: move |_| {
                                    end_room(room_url, room_access, room_name, active_room_code, room_role, room_host_token, room_members, room_permissions, room_shared_playlists, queue, notice);
                                    room_panel_open.set(false);
                                },
                                    if room_role() == "host" { "Close room" } else { "Leave room" }
                                }
                            }
                        } else {
                            p { class: "party-member-empty", "Create a room or join listeners with a room code." }
                        }
                        if !room_url().is_empty() && !room_access().is_empty() {
                            section { class: "party-room-form",
                                if let Some(profile) = spotify_profile() {
                                    fieldset { class: "room-name-options",
                                        legend { "Create room as" }
                                        label {
                                            input {
                                                r#type: "radio",
                                                name: "room-name-mode",
                                                checked: room_name_mode() == "spotify",
                                                onchange: move |_| room_name_mode.set("spotify".to_string())
                                            }
                                            "Spotify · {profile.display_name.as_deref().unwrap_or(&profile.id)}"
                                        }
                                        label {
                                            input {
                                                r#type: "radio",
                                                name: "room-name-mode",
                                                checked: room_name_mode() != "spotify",
                                                onchange: move |_| room_name_mode.set("custom".to_string())
                                            }
                                            "Custom name"
                                        }
                                    }
                                }
                                label {
                                    span { "Your name" }
                                    input { class: "room-server-input", value: "{room_name()}", maxlength: "50", placeholder: "Listener", aria_label: "Your name", oninput: move |event| room_name.set(event.value()) }
                                }
                                label {
                                    span { "Room code" }
                                    input { class: "room-server-input", value: "{room_code()}", maxlength: "12", placeholder: "Enter a code", aria_label: "Room code", oninput: move |event| room_code.set(event.value().to_ascii_uppercase()) }
                                }
                                div { class: "party-room-form-actions",
                                    button { class: "solid-button", onclick: move |_| join_room(room_url, room_access, room_name, room_code, active_room_code, room_role, room_host_token, room_members, room_permissions, room_shared_playlists, room_sessions, queue, notice), "Join room" }
                                    button { class: "outline-button", onclick: move |_| create_room(room_url, room_access, room_name, room_name_mode, spotify_profile, room_code, active_room_code, room_role, room_host_token, room_members, room_permissions, room_shared_playlists, room_sessions, notice), "Create room" }
                                }
                            }
                        }
                    }
                }
            }
            footer { class: "player-bar",
                audio {
                    id: "wavelength-audio",
                    src: if audio_src().is_empty() { None } else { Some(audio_src()) },
                    preload: "metadata",
                    onloadedmetadata: move |_| {
                        spawn(async move {
                            match browser::audio_time().await {
                                Ok((position, duration)) => {
                                    playhead.set(position);
                                    media_duration.set(duration);
                                }
                                Err(error) => notice.set(error),
                            }
                        });
                    },
                    ontimeupdate: move |_| {
                        spawn(async move {
                            match browser::audio_time().await {
                                Ok((position, duration)) => {
                                    playhead.set(position);
                                    media_duration.set(duration);
                                }
                                Err(error) => notice.set(error),
                            }
                        });
                    },
                    onended: move |_| handle_track_ended(
                        current,
                        audio_src,
                        playing,
                        queue,
                        history,
                        player_end_source.clone(),
                        playhead,
                        media_duration,
                        shuffle,
                        repeat_mode,
                        room_role,
                        notice,
                    ),
                    onerror: move |_| {
                        if current().is_some() && notice().is_empty() {
                            notice.set("The audio stream could not be played.".to_string());
                        }
                    },
                }
                div { class: "player-identity",
                    button {
                        class: "vinyl-player",
                        disabled: active_track.is_none(),
                        aria_label: if active_track.is_some() { "Open now playing" } else { "Nothing playing" },
                        onclick: move |_| page.set(Page::NowPlaying),
                        div { class: "player-art",
                            if let Some(track) = active_track.as_ref() {
                                if let Some(image) = track.thumbnail.as_ref() {
                                    img { src: "{image}", alt: "" }
                                } else { Icon { name: "disc" } }
                                if let Some(status) = queue_download_status().get(&track.audio_service_id()).cloned() {
                                    QueueDownloadIndicator { status }
                                }
                            } else {
                                Icon { name: "disc" }
                            }
                        }
                        div { class: "player-track",
                            strong { "{active_track.as_ref().map(|track| track.title.as_str()).unwrap_or(\"Nothing playing\")}" }
                            small { "{player_track_caption}" }
                        }
                    }
                    button {
                        class: if active_track_is_favorite { "player-like-button active" } else { "player-like-button" },
                        aria_label: if active_track.is_some() {
                            if active_track_is_favorite { "Remove from liked songs" } else { "Add to liked songs" }
                        } else { "Like now playing song" },
                        aria_pressed: active_track.is_some() && active_track_is_favorite,
                        title: if active_track.is_none() {
                            "Nothing playing".to_string()
                        } else if active_track_is_favorite {
                            "Remove from liked songs".to_string()
                        } else {
                            "Add to liked songs".to_string()
                        },
                        disabled: active_track.is_none(),
                        onclick: move |_| {
                            if let Some(track) = current() { toggle_favorite(track, favorites, notice); }
                        },
                        if active_track_is_favorite {
                            Icon { name: "favorite-filled" }
                        } else {
                            Icon { name: "favorite" }
                        }
                    }
                }
                div { class: "player-center",
                    div { class: "transport",
                        button {
                            class: if shuffle() { "player-tool active" } else { "player-tool" },
                            aria_label: "Toggle shuffle",
                            title: "Shuffle",
                            aria_pressed: shuffle(),
                            disabled: active_track.is_none() || !can_control_playback,
                            onclick: move |_| toggle_shuffle(shuffle, notice),
                            Icon { name: "shuffle" }
                        }
                        button {
                            class: "icon-button",
                            aria_label: "Previous track",
                            disabled: active_track.is_none() || !can_control_playback,
                            onclick: move |_| previous_track(history, current, audio_src, playing, queue, player_previous_source.clone(), notice),
                            Icon { name: "skip-back" }
                        }
                        button {
                            class: "play-main",
                            aria_label: if playing() { "Pause" } else { "Play" },
                            title: if playing() { "Pause" } else { "Play" },
                            disabled: active_track.is_none() || !can_control_playback,
                            onclick: move |_| toggle_playback(playing, notice),
                            if playing() {
                                Icon { name: "pause" }
                            } else {
                                Icon { name: "play" }
                            }
                        }
                        button {
                            class: "icon-button",
                            aria_label: "Next track",
                            disabled: active_track.is_none() || !can_control_playback,
                            onclick: move |_| next_track(current, audio_src, playing, queue, history, player_next_source.clone(), shuffle, repeat_mode, notice),
                            Icon { name: "skip-forward" }
                        }
                        button {
                            class: if repeat_mode() == "off" { "player-tool repeat-control" } else { "player-tool active repeat-control" },
                            aria_label: "Toggle repeat",
                            title: if repeat_mode() == "off" { "Repeat all" } else if repeat_mode() == "all" { "Repeat current song" } else { "Turn repeat off" },
                            aria_pressed: repeat_mode() != "off",
                            disabled: active_track.is_none() || !can_control_playback,
                            onclick: move |_| toggle_repeat(repeat_mode, notice),
                            Icon { name: "repeat" }
                            if repeat_mode() == "one" { span { class: "repeat-one-mark", "1" } }
                        }
                    }
                    div { class: "progress-row",
                        span { id: "elapsed-time", "{elapsed_label}" }
                        input {
                            id: "track-progress",
                            class: "progress-slider",
                            r#type: "range",
                            min: "0",
                            max: "{progress_max}",
                            step: "0.1",
                            value: if active_track.is_some() { playhead() } else { 0.0 },
                            style: "--progress: {player_progress}%",
                            aria_label: "Track progress",
                            disabled: active_track.is_none() || !can_control_playback,
                            oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f64>() {
                                seek_track(value, playhead, notice);
                            }
                        } }
                        span { id: "track-duration", "{duration_label}" }
                    }
                }
                div { class: "player-tools",
                    button {
                        class: "player-tool",
                        aria_label: "Toggle lyrics",
                        title: if lyrics_fullscreen() { "Hide lyrics" } else { "Show lyrics" },
                        aria_pressed: lyrics_fullscreen(),
                        disabled: active_track.is_none(),
                        onclick: move |_| lyrics_fullscreen.toggle(),
                        Icon { name: "lyrics" }
                    }
                    button {
                        class: if room_panel_open() || !active_room_code().is_empty() { "party-roster-button active" } else { "party-roster-button" },
                        aria_label: if active_room_code().is_empty() {
                            "Open party details".to_string()
                        } else {
                            format!("Party {}, {} listeners", active_room_code(), room_members().len())
                        },
                        aria_pressed: room_panel_open(),
                        title: if active_room_code().is_empty() {
                            "Party details".to_string()
                        } else {
                            format!("Room {}", active_room_code())
                        },
                        onclick: move |_| {
                            room_panel_open.set(!room_panel_open());
                            queue_panel_open.set(false);
                        },
                        Icon { name: "users" }
                    }
                    button {
                        class: if queue_panel_open() { "player-tool active" } else { "player-tool" },
                        aria_label: "Show queue",
                        title: "Queue",
                        aria_pressed: queue_panel_open(),
                        onclick: move |_| {
                            queue_panel_open.set(!queue_panel_open());
                            room_panel_open.set(false);
                        },
                        Icon { name: "queue" }
                    }
                    div { class: "volume-control",
                        button {
                            class: "volume-button",
                            aria_label: if volume() == 0.0 { "Unmute" } else { "Mute" },
                            title: if volume() == 0.0 { "Unmute" } else { "Mute" },
                            onclick: move |_| toggle_volume(volume, previous_volume, notice),
                            if volume() == 0.0 { Icon { name: "volume-muted" } } else { Icon { name: "volume" } }
                        }
                        input {
                            class: "volume-slider",
                            r#type: "range",
                            min: "0",
                            max: "1",
                            step: "0.01",
                            value: "{volume()}",
                            style: " --volume: {volume() * 100.0}%",
                            aria_label: "Volume",
                            oninput: move |event| {
                                if let Ok(value) = event.value().parse::<f64>() {
                                    set_player_volume(value, volume, previous_volume, notice);
                                }
                            }
                        }
                    }
                }
            }
            if playlist_create_open() {
                div { class: "modal-backdrop playlist-create-backdrop",
                    div { class: "modal folder-modal playlist-create-modal", role: "dialog", aria_modal: "true", aria_labelledby: "playlist-create-title",
                        button {
                            class: "modal-close",
                            aria_label: "Close playlist creator",
                            onclick: move |_| playlist_create_open.set(false),
                            Icon { name: "close" }
                        }
                        div { class: "folder-modal-heading",
                            div { class: "folder-modal-title",
                                h2 { id: "playlist-create-title", "Create playlist" }
                            }
                            p { "Choose a name and songs from Favorite Songs." }
                        }
                        form { class: "playlist-create-form", onsubmit: move |event| {
                            event.prevent_default();
                            let name = playlist_name().trim().to_string();
                            if name.is_empty() {
                                notice.set("Enter a playlist name.".to_string());
                                return;
                            }
                            let mut all = playlists();
                            if name.eq_ignore_ascii_case("Favorite Songs")
                                || all.iter().any(|playlist| playlist.name.eq_ignore_ascii_case(&name))
                            {
                                notice.set("A playlist with that name already exists.".to_string());
                                return;
                            }
                            let tracks = selected_favorite_tracks(
                                &favorites(),
                                &playlist_create_favorites(),
                            );
                            let track_count = tracks.len();
                            all.push(Playlist {
                                id: format!("playlist-{}", crate::now_epoch_millis()),
                                name: name.clone(),
                                tracks,
                            });
                            match browser::save("wavelength-playlists", &all) {
                                Ok(()) => {
                                    playlists.set(all);
                                    playlist_name.set(String::new());
                                    playlist_create_favorites.set(Vec::new());
                                    playlist_query.set(String::new());
                                    playlist_create_open.set(false);
                                    notice.set(format!("{name} created with {track_count} songs."));
                                }
                                Err(error) => notice.set(error),
                            }
                        },
                            label { class: "playlist-create-name-field", "Playlist name"
                                input {
                                    value: "{playlist_name()}",
                                    maxlength: "80",
                                    placeholder: "Name your playlist",
                                    aria_label: "Playlist name",
                                    oninput: move |event| playlist_name.set(event.value())
                                }
                            }
                            section { class: "favorite-playlist-picker", aria_label: "Choose songs from Favorite Songs",
                                header { class: "favorite-playlist-picker-heading",
                                    div {
                                        strong { "Favorite Songs" }
                                        small { "{playlist_create_favorites().len()} of {favorites().len()} selected" }
                                    }
                                    if !favorites().is_empty() {
                                        button { class: "text-button", r#type: "button", onclick: move |_| {
                                            let all_selected = favorites().iter().all(|track| {
                                                playlist_create_favorites().iter().any(|key| key.as_str() == track.key())
                                            });
                                            if all_selected {
                                                playlist_create_favorites.set(Vec::new());
                                            } else {
                                                playlist_create_favorites.set(favorites().iter().map(|track| track.key().to_string()).collect());
                                            }
                                        },
                                            if favorites().iter().all(|track| {
                                                playlist_create_favorites().iter().any(|key| key.as_str() == track.key())
                                            }) { "Clear selection" } else { "Select all" }
                                        }
                                    }
                                }
                                if favorites().is_empty() {
                                    p { class: "favorite-playlist-empty", "Favorite songs will appear here when you like tracks." }
                                } else {
                                    div { class: "favorite-playlist-selection-list",
                                        for track in favorites() {
                                            {
                                                let key = track.key().to_string();
                                                let selected = playlist_create_favorites().iter().any(|item| item == &key);
                                                rsx! {
                                                    button {
                                                        key: "{key}",
                                                        class: if selected { "favorite-playlist-track selected" } else { "favorite-playlist-track" },
                                                        r#type: "button",
                                                        aria_pressed: selected,
                                                        aria_label: if selected { "Remove {track.title} from selection" } else { "Add {track.title} to selection" },
                                                        onclick: move |_| {
                                                            let mut selected_keys = playlist_create_favorites();
                                                            if let Some(index) = selected_keys.iter().position(|item| item == &key) {
                                                                selected_keys.remove(index);
                                                            } else {
                                                                selected_keys.push(key.clone());
                                                            }
                                                            playlist_create_favorites.set(selected_keys);
                                                        },
                                                        span { class: "mini-art favorite-playlist-track-art",
                                                            if let Some(image) = track.thumbnail.as_deref() {
                                                                img { src: "{image}", alt: "" }
                                                            } else {
                                                                Icon { name: "music" }
                                                            }
                                                        }
                                                        span { class: "favorite-playlist-track-copy",
                                                            strong { "{track.title}" }
                                                            small { "{track.artist}" }
                                                        }
                                                        span { class: "favorite-playlist-track-state", if selected { "Added" } else { "Add" } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "playlist-create-actions",
                                button { class: "outline-button", r#type: "button", onclick: move |_| playlist_create_open.set(false), "Cancel" }
                                button { class: "solid-button", r#type: "submit", disabled: playlist_name().trim().is_empty(), "Create playlist" }
                            }
                        }
                    }
                }
            }
            if folder_manager_open() {
                div { class: "modal-backdrop folder-manager-backdrop",
                    div { class: "modal folder-modal", role: "dialog", aria_modal: "true", aria_labelledby: "folder-manager-title",
                        button {
                            class: "modal-close",
                            aria_label: "Close music folders",
                            onclick: move |_| folder_manager_open.set(false),
                            Icon { name: "close" }
                        }
                        div { class: "folder-modal-heading",
                            div { class: "folder-modal-title",
                                h2 { id: "folder-manager-title", "Music folders" }
                                button { class: "icon-button folder-add-button", aria_label: "Add music folder", title: "Add folder", onclick: move |_| {
                                    spawn(async move {
                                        match browser::choose_folder().await {
                                            Ok(Some(tracks)) => {
                                                local_tracks.set(merge_local_tracks(local_tracks(), tracks));
                                                match browser::saved_library_folders().await {
                                                    Ok(folders) => saved_folder_names.set(folders),
                                                    Err(error) => notice.set(error),
                                                }
                                            }
                                            Ok(None) => {}
                                            Err(error) => notice.set(error),
                                        }
                                    });
                                }, Icon { name: "plus" } }
                                button { class: "icon-button folder-add-button", aria_label: "Rescan saved music folders", title: "Rescan folders", onclick: move |_| {
                                    spawn(async move {
                                        match browser::scan_library().await {
                                            Ok(tracks) => {
                                                local_tracks.set(merge_local_tracks(local_tracks(), tracks));
                                                notice.set("Imported folders refreshed.".to_string());
                                            }
                                            Err(error) => notice.set(error),
                                        }
                                    });
                                }, Icon { name: "refresh" } }
                            }
                            p { "Saved on this device when the browser supports persistent folder access. Other browsers may keep selected files only until you close the page." }
                        }
                        div { class: "folder-list",
                            if saved_folder_names().is_empty() {
                                p { class: "rail-empty", "No persistent music folders saved." }
                            } else {
                                for folder in saved_folder_names() {
                                    div { class: "folder-row", key: "{folder}",
                                        span { strong { "{folder}" } }
                                        button { class: "icon-button danger", aria_label: "Remove {folder}", onclick: move |_| {
                                            let folder_name = folder.clone();
                                            spawn(async move {
                                                match browser::remove_saved_library_folder(&folder_name).await {
                                                    Ok(()) => {
                                                        saved_folder_names.set(saved_folder_names().into_iter().filter(|name| name != &folder_name).collect());
                                                        local_tracks.set(local_tracks().into_iter().filter(|track| {
                                                            track.folder_path.as_deref().is_none_or(|path| path != folder_name && !path.starts_with(&format!("{folder_name}/")))
                                                        }).collect());
                                                        notice.set(format!("{folder_name} removed from your library."));
                                                    }
                                                    Err(error) => notice.set(error),
                                                }
                                            });
                                        }, Icon { name: "trash" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
                                if qobuz_modal_open() {
                                    div { class: "modal-backdrop folder-manager-backdrop",
                                        div { class: "modal folder-modal qobuz-credentials-modal", role: "dialog", aria_modal: "true", aria_labelledby: "qobuz-credentials-title",
                                            button {
                                                class: "modal-close",
                                                aria_label: "Close Qobuz credentials",
                                                onclick: move |_| qobuz_modal_open.set(false),
                                                Icon { name: "close" }
                                            }
                                            div { class: "folder-modal-heading",
                                                div { class: "folder-modal-title",
                                                    h2 { id: "qobuz-credentials-title", "Qobuz credentials" }
                                                }
                                                p { "Spotify downloads use Qobuz when all three credentials are set." }
                                            }
                                            form { class: "qobuz-credentials-form", onsubmit: move |event| {
                                                event.prevent_default();
                                                save_qobuz_credentials(
                                                    qobuz_app_id,
                                                    qobuz_app_secret,
                                                    qobuz_user_token,
                                                    qobuz_configured,
                                                    qobuz_modal_open,
                                                    qobuz_saving,
                                                    notice,
                                                );
                                            },
                                                label { "App ID"
                                                    input {
                                                        value: "{qobuz_app_id()}",
                                                        autocomplete: "off",
                                                        oninput: move |event| qobuz_app_id.set(event.value())
                                                    }
                                                }
                                                label { "App secret"
                                                    input {
                                                        r#type: "password",
                                                        value: "{qobuz_app_secret()}",
                                                        autocomplete: "new-password",
                                                        oninput: move |event| qobuz_app_secret.set(event.value())
                                                    }
                                                }
                                                label { "User token"
                                                    input {
                                                        r#type: "password",
                                                        value: "{qobuz_user_token()}",
                                                        autocomplete: "new-password",
                                                        oninput: move |event| qobuz_user_token.set(event.value())
                                                    }
                                                }
                                                div { class: "qobuz-credentials-actions",
                                                    button {
                                                        class: "outline-button compact-action",
                                                        r#type: "button",
                                                        onclick: move |_| qobuz_modal_open.set(false),
                                                        "Cancel"
                                                    }
                                                    button {
                                                        class: "outline-button compact-action",
                                                        r#type: "submit",
                                                        disabled: qobuz_saving(),
                                                        if qobuz_saving() { "Saving" } else { "Save" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if context_track().is_some() || context_playlist().is_some() {
                                    button {
                                        class: "context-menu-dismiss",
                                        r#type: "button",
                                        aria_label: "Close context menu",
                                        onclick: move |_| {
                                            context_track.set(None);
                                            context_playlist.set(None);
                                        }
                                    }
                                }
                                if let Some(menu) = context_track() {
                                    {
                                        let track = menu.track.clone();
                                        let details_target = track.clone();
                                        let queue_target = track.clone();
                                        let favorite_target = track.clone();
                                        let download_target = track.clone();
                                        let download_disabled = track.is_local();
                                        rsx! {
                                    div { class: "context-menu", role: "menu", aria_label: "Actions for {track.title}", style: "--context-menu-x: {menu.x}px; --context-menu-y: {menu.y}px;",
                                        button { onclick: move |_| {
                                            detail_track.set(Some(details_target.clone()));
                                            context_track.set(None);
                                        }, "Song details" }
                                        for playlist in playlists() {
                                            button { key: "{playlist.id}", onclick: {
                                                let playlist_id = playlist.id.clone();
                                                let selected_track = track.clone();
                                                move |_| {
                                                    let mut all = playlists();
                                                    if let Some(target) = all.iter_mut().find(|item| item.id == playlist_id) {
                                                        if !target.tracks.iter().any(|item| item.key() == selected_track.key()) {
                                                            target.tracks.push(selected_track.clone());
                                                            playlists.set(all.clone());
                                                            save_saved("wavelength-playlists", &all, notice);
                                                        }
                                                    }
                                                    context_track.set(None);
                                                }
                                            }, "Add to {playlist.name}" }
                                        }
                                        button { onclick: move |_| {
                                            add_to_queue(queue_target.clone(), queue, notice);
                                            context_track.set(None);
                                        }, "Add to queue" }
                                        if page() != Page::History {
                                            button { onclick: move |_| {
                                                toggle_favorite(favorite_target.clone(), favorites, notice);
                                                context_track.set(None);
                                            }, "Toggle favorite" }
                                        }
                                        button { disabled: download_disabled, onclick: move |_| {
                                            download_track(download_target.clone(), notice);
                                            context_track.set(None);
                                        }, "Download track" }
                                    }
                                        }
                                    }
                                }
                                if let Some(menu) = context_playlist() {
                                    {
                                        let target = menu.target.clone();
                                        rsx! {
                                            div {
                                                class: "context-menu",
                                                role: "menu",
                                                aria_label: "Collection actions",
                                                style: "--context-menu-x: {menu.x}px; --context-menu-y: {menu.y}px;",
                                                match target {
                                                    PlaylistMenuTarget::Favorites => {
                                                        let favorite_tracks = favorites();
                                                        let has_tracks = !favorite_tracks.is_empty();
                                                        let play_tracks = favorite_tracks.clone();
                                                        let queue_tracks = favorite_tracks;
                                                        rsx! {
                                                            button { disabled: !has_tracks, onclick: move |_| {
                                                                play_playlist_tracks(
                                                                    play_tracks.clone(),
                                                                    current,
                                                                    audio_src,
                                                                    playing,
                                                                    queue,
                                                                    history,
                                                                    notice,
                                                                    page,
                                                                );
                                                                context_playlist.set(None);
                                                            }, "Play favorites" }
                                                            button { disabled: !has_tracks, onclick: move |_| {
                                                                for track in queue_tracks.clone() {
                                                                    add_to_queue(track, queue, notice);
                                                                }
                                                                context_playlist.set(None);
                                                            }, "Add favorites to queue" }
                                                        }
                                                    },
                                                    PlaylistMenuTarget::Album { name, tracks } => {
                                                        let album_name = name.clone();
                                                        let play_tracks = tracks.clone();
                                                        let queue_tracks = tracks.clone();
                                                        let favorite_tracks = tracks.clone();
                                                        let has_tracks = !tracks.is_empty();
                                                        rsx! {
                                                            button { onclick: move |_| {
                                                                album_detail_name.set(Some(album_name.clone()));
                                                                page.set(Page::AlbumDetail);
                                                                context_playlist.set(None);
                                                            }, "Open album" }
                                                            button { disabled: !has_tracks, onclick: move |_| {
                                                                play_playlist_tracks(
                                                                    play_tracks.clone(),
                                                                    current,
                                                                    audio_src,
                                                                    playing,
                                                                    queue,
                                                                    history,
                                                                    notice,
                                                                    page,
                                                                );
                                                                context_playlist.set(None);
                                                            }, "Play album" }
                                                            button { disabled: !has_tracks, onclick: move |_| {
                                                                for track in queue_tracks.clone() {
                                                                    add_to_queue(track, queue, notice);
                                                                }
                                                                context_playlist.set(None);
                                                            }, "Add album to queue" }
                                                            button { disabled: !has_tracks, onclick: move |_| {
                                                                for track in favorite_tracks.clone() {
                                                                    toggle_favorite(track, favorites, notice);
                                                                }
                                                                context_playlist.set(None);
                                                            }, "Toggle album track favorites" }
                                                        }
                                                    },
                                                    PlaylistMenuTarget::Local(playlist) => rsx! {
                                                        button { onclick: {
                                                            let playlist_id = playlist.id.clone();
                                                            move |_| {
                                                                selected_playlist_id.set(Some(playlist_id.clone()));
                                                                page.set(Page::PlaylistDetail);
                                                                context_playlist.set(None);
                                                            }
                                                        }, "Open playlist" }
                                                        if let Some(track) = current() {
                                                            button { onclick: {
                                                                let playlist_id = playlist.id.clone();
                                                                let track = track.clone();
                                                                move |_| {
                                                                    let mut all = playlists();
                                                                    if let Some(target) = all.iter_mut().find(|item| item.id == playlist_id) {
                                                                        if !target.tracks.iter().any(|item| item.key() == track.key()) {
                                                                            target.tracks.push(track.clone());
                                                                            playlists.set(all.clone());
                                                                            save_saved("wavelength-playlists", &all, notice);
                                                                        }
                                                                    }
                                                                    context_playlist.set(None);
                                                                }
                                                            }, "Add current track" }
                                                        }
                                                        button { onclick: {
                                                            let playlist_id = playlist.id.clone();
                                                            move |_| {
                                                                let mut all = playlists();
                                                                all.retain(|entry| entry.id != playlist_id);
                                                                playlists.set(all.clone());
                                                                save_saved("wavelength-playlists", &all, notice);
                                                                context_playlist.set(None);
                                                            }
                                                        }, "Delete playlist" }
                                                    },
                                                    PlaylistMenuTarget::Shared(playlist) => rsx! {
                                                        button { onclick: {
                                                            let playlist_id = playlist.id.clone();
                                                            move |_| {
                                                                selected_playlist_id.set(Some(playlist_id.clone()));
                                                                page.set(Page::PlaylistDetail);
                                                                context_playlist.set(None);
                                                            }
                                                        }, "Open playlist" }
                                                        button { onclick: {
                                                            let target = playlist.clone();
                                                            move |_| {
                                                                let resolved = resolve_room_playlist(
                                                                    &target,
                                                                    &local_tracks(),
                                                                    &favorites(),
                                                                    &history(),
                                                                    &queue(),
                                                                );
                                                                play_playlist_tracks(
                                                                    resolved.tracks,
                                                                    current,
                                                                    audio_src,
                                                                    playing,
                                                                    queue,
                                                                    history,
                                                                    notice,
                                                                    page,
                                                                );
                                                                context_playlist.set(None);
                                                            }
                                                        }, "Play playlist" }
                                                    },
                                                    PlaylistMenuTarget::Spotify(playlist) => rsx! {
                                                        button { onclick: {
                                                            let selected = playlist.clone();
                                                            move |_| {
                                                                load_spotify_playlist(
                                                                    selected.clone(),
                                                                    spotify_playlist_tracks,
                                                                    spotify_playlist_selected,
                                                                    spotify_playlist_name,
                                                                    spotify_playlist_tracks_busy,
                                                                    notice,
                                                                );
                                                                page.set(Page::SpotifyPlaylists);
                                                                context_playlist.set(None);
                                                            }
                                                        }, "Open playlist" }
                                                        button { onclick: {
                                                            let selected = playlist.clone();
                                                            move |_| {
                                                                play_spotify_playlist(
                                                                    selected.clone(),
                                                                    current,
                                                                    audio_src,
                                                                    playing,
                                                                    queue,
                                                                    history,
                                                                    notice,
                                                                    page,
                                                                );
                                                                context_playlist.set(None);
                                                            }
                                                        }, "Play playlist" }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                                if let Some(track) = detail_track() {
                                    div { class: "modal-backdrop song-details-backdrop",
                                        div { class: "modal song-details-modal", role: "dialog", aria_modal: "true", aria_labelledby: "song-details-title",
                                            button { class: "modal-close", aria_label: "Close song details", onclick: move |_| detail_track.set(None), "Close" }
                                            div { class: "song-details-identity",
                                                div { class: "song-details-art mini-art",
                                                    if let Some(image) = track.thumbnail.as_ref() {
                                                        img { src: "{image}", alt: "" }
                                                    } else {
                                                        Icon { name: "disc" }
                                                    }
                                                }
                                                div { class: "song-details-copy",
                                                    p { class: "eyebrow", "{track.source.as_deref().unwrap_or(\"Audio\")}" }
                                                    h2 { id: "song-details-title", "{track.title}" }
                                                    p { "{track.artist}" }
                                                }
                                            }
                                            dl { class: "song-details-list",
                                                div { dt { "Album" } dd { "{track.album.as_deref().unwrap_or(\"Single\")}" } }
                                                div { dt { "Duration" } dd { "{track.duration_label()}" } }
                                                if let Some(path) = track.folder_path.as_deref() {
                                                    div { dt { "Folder" } dd { "{path}" } }
                                                }
                                                if let Some(source) = track.source.as_deref() {
                                                    div { dt { "Source" } dd { "{source}" } }
                                                }
                                            }
                                            if track.is_spotify() {
                                                {
                                                    let external_id = track.id.strip_prefix("spotify:track:").unwrap_or(&track.id).to_string();
                                                    rsx! {
                                                button { class: "outline-button compact-action song-details-open", onclick: move |_| {
                                                    if let Err(error) = browser::open_external_track("spotify", &external_id) {
                                                        notice.set(error);
                                                    }
                                                }, "Open on Spotify" }
                                                    }
                                                }
                                            } else if let Some(id) = track.video_id.as_deref().or_else(|| {
                                                track.source.as_deref().filter(|source| source.eq_ignore_ascii_case("youtube")).map(|_| track.id.as_str())
                                            }) {
                                                {
                                                    let external_id = id.to_string();
                                                    rsx! {
                                                button { class: "outline-button compact-action song-details-open", onclick: move |_| {
                                                    if let Err(error) = browser::open_external_track("youtube", &external_id) {
                                                        notice.set(error);
                                                    }
                                                }, "Open on YouTube" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if lyrics_fullscreen() {
                                    section { class: "lyrics-panel lyrics-fullscreen", aria_label: "Fullscreen lyrics",
                                        button { class: "modal-close", aria_label: "Close fullscreen lyrics", onclick: move |_| lyrics_fullscreen.set(false), "Close" }
                                        div { class: "lyrics-lines", aria_label: "Lyrics",
                                            if lyric_lines().is_empty() {
                                                p { "{lyrics()}" }
                                            } else {
                                                for (index, line) in lyric_lines().iter().enumerate() {
                                                    p {
                                                        class: if index as isize == active_lyric_index { "active" } else if active_lyric_index >= 0 && (index as isize) < active_lyric_index { "passed" } else { "upcoming" },
                                                        "{line.text}"
                                                    }
                                                }
                        }
                    }
                }
            }
        }
    }
}
