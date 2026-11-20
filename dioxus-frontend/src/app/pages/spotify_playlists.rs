use super::super::AppState;
use super::PageData;
use crate::browser;
use crate::components::{CollectionDetailHeader, Icon, TrackRow};
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn SpotifyPlaylistsPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let mut copy_local_open = use_signal(|| false);
    let mut copy_local_name = use_signal(String::new);
    let mut copy_local_thumbnail = use_signal(|| Option::<String>::None);
    let mut copy_local_track_indexes = use_signal(Vec::<usize>::new);
    let mut rename_editing = use_signal(|| false);
    let mut rename_name = use_signal(String::new);

    match (app_state.page)() {
        Page::SpotifyPlaylists => rsx! {
            header { class: "library-page-heading",
                h1 { "Spotify playlists" }
                button { class: "outline-button compact-action", disabled: (app_state.spotify_playlists_busy)(), onclick: move |_| load_spotify_playlists(app_state.spotify_playlists, app_state.spotify_playlists_busy, app_state.notice), "Refresh" }
            }
            if (app_state.spotify_playlists_busy)() { p { class: "library-empty", "Loading Spotify playlists…" } }
            else if (app_state.spotify_playlists)().is_empty() { p { class: "library-empty", "No Spotify playlists loaded yet" } }
            div { class: "collection-grid",
                for playlist in (app_state.spotify_playlists)() {
                    article { class: "collection-card", key: "{playlist.id}",
                        h2 { "{playlist.name}" }
                        p { "{playlist.tracks_total} tracks" }
                        button { class: "text-button", onclick: {
                            let selected = playlist.clone();
                            move |_| {
                                load_spotify_playlist(
                                    selected.clone(),
                                    app_state.spotify_playlist_tracks,
                                    app_state.spotify_playlist_selected,
                                    app_state.spotify_playlist_name,
                                    app_state.spotify_playlist_tracks_busy,
                                    app_state.spotify_playlist_tracks_error,
                                    app_state.spotify_detail_load_id,
                                    app_state.notice,
                                    app_state.page,
                                );
                            }
                        }, "Open playlist" }
                    }
                }
            }
        },
        Page::SpotifyPlaylistDetail => {
            if let Some(selected) = (app_state.spotify_playlist_selected)() {
                let tracks = (app_state.spotify_playlist_tracks)();
                let loaded_track_count = tracks.len();
                let tracks_busy = (app_state.spotify_playlist_tracks_busy)();
                let track_count = if tracks_busy {
                    selected.tracks_total
                } else {
                    tracks.len()
                };
                let selected_id = selected.id.clone();
                let save_selected_id = selected.id.clone();
                let current_track = (app_state.current)();
                let is_playing = (app_state.playing)();
                let can_edit =
                    spotify_playlist_is_owned((app_state.spotify_profile)().as_ref(), &selected);
                let owner_label = (!selected.owner_name.is_empty())
                    .then(|| format!("By {}", selected.owner_name));

                rsx! {
                    CollectionDetailHeader {
                        title: selected.name.clone(),
                        kind: "Spotify playlist".to_string(),
                        tracks: tracks.clone(),
                        track_count,
                        artist: owner_label,
                        artwork_url: selected.thumbnail.clone(),
                        artwork_class: String::new(),
                        artwork_icon: Some("list"),
                        play_label: "Play playlist".to_string(),
                        extra_action_label: Some("Copy to local playlists".to_string()),
                        extra_action_disabled: tracks_busy,
                        rename_enabled: can_edit,
                        rename_editing: rename_editing(),
                        rename_value: rename_name(),
                        rename_busy: (app_state.spotify_playlist_save_busy)(),
                        rename_notice: Some("Name changes will sync to Spotify.".to_string()),
                        save_action_label: can_edit.then(|| "Save track changes".to_string()),
                        save_action_disabled: (app_state.spotify_playlist_save_busy)() || tracks_busy,
                        download_action_label: Some("Download".to_string()),
                        download_action_disabled: tracks_busy || tracks.is_empty(),
                        on_play: {
                            let playlist = selected.clone();
                            move |_| play_spotify_playlist(
                                playlist.clone(),
                                app_state.current,
                                app_state.audio_src,
                                app_state.playing,
                                app_state.queue,
                                app_state.history,
                                app_state.notice,
                                app_state.queue_panel_open,
                                app_state.room_panel_open,
                            )
                        },
                        on_download_action: {
                            let tracks = tracks.clone();
                            move |_| {
                                download_tracks(
                                    tracks.clone(),
                                    (app_state.download_format)(),
                                    app_state.notice,
                                )
                            }
                        },
                        on_extra_action: {
                            let source = selected.clone();
                            move |_| {
                                let tracks = (app_state.spotify_playlist_tracks)();
                                copy_local_name.set(source.name.clone());
                                copy_local_thumbnail.set(source.thumbnail.clone());
                                copy_local_track_indexes.set((0..tracks.len()).collect());
                                copy_local_open.set(true);
                            }
                        },
                        on_rename_start: {
                            let name = selected.name.clone();
                            move |_| {
                                rename_name.set(name.clone());
                                rename_editing.set(true);
                            }
                        },
                        on_rename_change: move |name| rename_name.set(name),
                        on_rename_save: {
                            let id = selected_id.clone();
                            let original_name = selected.name.clone();
                            move |_| {
                                if !can_edit {
                                    app_state.notice.set("Only the Spotify playlist owner can rename it.".to_string());
                                    return;
                                }
                                let name = rename_name().trim().to_string();
                                if name.is_empty() {
                                    app_state.notice.set("Enter a playlist name.".to_string());
                                    return;
                                }
                                if name == original_name {
                                    rename_editing.set(false);
                                    return;
                                }
                                if (app_state.spotify_playlist_save_busy)() {
                                    return;
                                }
                                app_state.spotify_playlist_save_busy.set(true);
                                let request_id = id.clone();
                                spawn(async move {
                                    match browser::rename_spotify_playlist(&request_id, &name).await {
                                        Ok(()) => {
                                            if let Some(mut item) = (app_state.spotify_playlist_selected)()
                                                .filter(|item| item.id == request_id)
                                            {
                                                item.name = name.clone();
                                                app_state.spotify_playlist_selected.set(Some(item));
                                            }
                                            let mut all = (app_state.spotify_playlists)();
                                            if let Some(item) = all.iter_mut().find(|item| item.id == request_id) {
                                                item.name = name.clone();
                                            }
                                            app_state.spotify_playlists.set(all);
                                            app_state.spotify_playlist_name.set(name.clone());
                                            rename_editing.set(false);
                                            app_state.notice.set("Spotify playlist renamed.".to_string());
                                        }
                                        Err(error) => app_state.notice.set(error),
                                    }
                                    app_state.spotify_playlist_save_busy.set(false);
                                });
                            }
                        },
                        on_rename_cancel: {
                            let name = selected.name.clone();
                            move |_| {
                                rename_name.set(name.clone());
                                rename_editing.set(false);
                            }
                        },
                        on_save_action: move |_| {
                            if !can_edit {
                                app_state.notice.set("Only the Spotify playlist owner can edit tracks.".to_string());
                                return;
                            }
                            if (app_state.spotify_playlist_save_busy)() || (app_state.spotify_playlist_tracks_busy)() {
                                return;
                            }
                            let id = save_selected_id.clone();
                            let tracks = (app_state.spotify_playlist_tracks)();
                            let uris = tracks.iter().filter_map(spotify_track_uri).collect::<Vec<_>>();
                            if uris.len() != tracks.len() {
                                app_state.notice.set("Some tracks are not Spotify tracks and will be omitted.".to_string());
                            }
                            app_state.spotify_playlist_save_busy.set(true);
                            spawn(async move {
                                match browser::save_spotify_playlist(&id, &uris).await {
                                    Ok(()) => {
                                        if let Some(mut item) = (app_state.spotify_playlist_selected)() {
                                            item.tracks_total = uris.len();
                                            app_state.spotify_playlist_selected.set(Some(item.clone()));
                                            let mut all = (app_state.spotify_playlists)();
                                            if let Some(existing) = all.iter_mut().find(|playlist| playlist.id == item.id) {
                                                existing.tracks_total = item.tracks_total;
                                                app_state.spotify_playlists.set(all);
                                            }
                                        }
                                        app_state.notice.set("Spotify playlist tracks updated.".to_string());
                                    }
                                    Err(error) => app_state.notice.set(error),
                                }
                                app_state.spotify_playlist_save_busy.set(false);
                            });
                        },
                    }
                    if tracks_busy {
                        p { class: "library-empty", "Loading playlist tracks…" }
                    } else if let Some(error) = (app_state.spotify_playlist_tracks_error)() {
                        div { class: "collection-detail-load-error", role: "alert",
                            p { "{error}" }
                            button {
                                class: "outline-button compact-action",
                                onclick: {
                                    let selected = selected.clone();
                                    move |_| load_spotify_playlist(
                                        selected.clone(),
                                        app_state.spotify_playlist_tracks,
                                        app_state.spotify_playlist_selected,
                                        app_state.spotify_playlist_name,
                                        app_state.spotify_playlist_tracks_busy,
                                        app_state.spotify_playlist_tracks_error,
                                        app_state.spotify_detail_load_id,
                                        app_state.notice,
                                        app_state.page,
                                    )
                                },
                                "Try again"
                            }
                        }
                    } else if tracks.is_empty() {
                        div { class: "library-empty",
                            p { "This playlist has no available tracks" }
                            if let Some(track) = current_track.as_ref() {
                                button {
                                    class: "text-button",
                                    disabled: !track.is_spotify(),
                                    onclick: move |_| {
                                        if let Some(track) = (app_state.current)() {
                                            let mut tracks = (app_state.spotify_playlist_tracks)();
                                            if !tracks.iter().any(|item| item.key() == track.key()) {
                                                tracks.push(track);
                                                app_state.spotify_playlist_tracks.set(tracks);
                                                app_state.notice.set("Track added. Save track changes to sync it to Spotify.".to_string());
                                            }
                                        }
                                    },
                                    "Add current track"
                                }
                            }
                        }
                    } else {
                        section { class: "collection-detail-track-section",
                            div { class: "collection-detail-track-heading",
                                h2 { "Tracks" }
                                if let Some(track) = current_track.as_ref() {
                                    button {
                                        class: "text-button",
                                        disabled: !track.is_spotify(),
                                        onclick: move |_| {
                                            if let Some(track) = (app_state.current)() {
                                                let mut tracks = (app_state.spotify_playlist_tracks)();
                                                if !tracks.iter().any(|item| item.key() == track.key()) {
                                                    tracks.push(track);
                                                    app_state.spotify_playlist_tracks.set(tracks);
                                                    app_state.notice.set("Track added. Save track changes to sync it to Spotify.".to_string());
                                                }
                                            }
                                        },
                                        "Add current track"
                                    }
                                }
                            }
                            div { class: "track-table collection-detail-track-list",
                                for (index, track) in tracks.iter().cloned().enumerate() {
                                    div { class: "playlist-track-row", key: "{track.key()}-{index}",
                                        TrackRow {
                                            track: track.clone(),
                                            context_menu_only: true,
                                            is_current: is_playing && current_track.as_ref().is_some_and(|current| current.key() == track.key()),
                                            on_play: move |track| play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                                            on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                                            on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                                            on_download: move |track| download_track(track, (app_state.download_format)(), app_state.notice),
                                            on_context: move |menu| app_state.context_track.set(Some(menu)),
                                        }
                                        button {
                                            class: "row-action playlist-detail-remove",
                                            aria_label: "Remove {track.title} from playlist",
                                            title: "Remove from playlist",
                                            onclick: move |_| {
                                                let mut tracks = (app_state.spotify_playlist_tracks)();
                                                if index < tracks.len() {
                                                    tracks.remove(index);
                                                    app_state.spotify_playlist_tracks.set(tracks);
                                                }
                                            },
                                            Icon { name: "close" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if copy_local_open() {
                        div { class: "modal-backdrop playlist-create-backdrop",
                            div {
                                class: "modal folder-modal playlist-create-modal spotify-playlist-copy-modal",
                                role: "dialog",
                                aria_modal: "true",
                                aria_labelledby: "spotify-playlist-copy-title",
                                button {
                                    class: "modal-close",
                                    aria_label: "Close local playlist copy dialog",
                                    onclick: move |_| copy_local_open.set(false),
                                    Icon { name: "close" }
                                }
                                div { class: "folder-modal-heading",
                                    h2 { id: "spotify-playlist-copy-title", "Copy to local playlists" }
                                    p { "Choose a name, cover, and the songs to include in your local copy." }
                                }
                                form {
                                    class: "playlist-create-form spotify-playlist-copy-form",
                                    onsubmit: move |event| {
                                        event.prevent_default();
                                        let name = copy_local_name().trim().to_string();
                                        if name.is_empty() {
                                            app_state.notice.set("Enter a playlist name.".to_string());
                                            return;
                                        }
                                        let mut all = (app_state.playlists)();
                                        if name.eq_ignore_ascii_case("Favorite Songs")
                                            || all.iter().any(|playlist| playlist.name.eq_ignore_ascii_case(&name))
                                        {
                                            app_state.notice.set("A playlist with that name already exists.".to_string());
                                            return;
                                        }
                                        let selected_indexes = copy_local_track_indexes();
                                        let tracks = (app_state.spotify_playlist_tracks)()
                                            .into_iter()
                                            .enumerate()
                                            .filter_map(|(index, track)| selected_indexes.contains(&index).then_some(track))
                                            .collect::<Vec<_>>();
                                        let track_count = tracks.len();
                                        all.push(Playlist {
                                            id: format!("playlist-{}", crate::now_epoch_millis()),
                                            name: name.clone(),
                                            thumbnail: copy_local_thumbnail(),
                                            tracks,
                                        });
                                        match browser::save("wavelength-playlists", &all) {
                                            Ok(()) => {
                                                app_state.playlists.set(all);
                                                copy_local_open.set(false);
                                                app_state.notice.set(format!("Copied {track_count} songs to {name}."));
                                            }
                                            Err(error) => app_state.notice.set(error),
                                        }
                                    },
                                    label { class: "playlist-create-name-field", "Playlist name"
                                        input {
                                            id: "spotify-playlist-copy-name",
                                            value: "{copy_local_name()}",
                                            maxlength: "80",
                                            aria_label: "Local playlist name",
                                            oninput: move |event| copy_local_name.set(event.value())
                                        }
                                    }
                                    div { class: "spotify-playlist-copy-artwork",
                                        if let Some(image) = copy_local_thumbnail() {
                                            img { class: "spotify-playlist-copy-artwork-preview", src: "{image}", alt: "Playlist cover preview" }
                                        } else {
                                            span { class: "spotify-playlist-copy-artwork-preview spotify-playlist-copy-artwork-empty",
                                                Icon { name: "list" }
                                            }
                                        }
                                        div { class: "spotify-playlist-copy-artwork-controls",
                                            strong { "Playlist cover" }
                                            div { class: "spotify-playlist-copy-artwork-actions",
                                                button {
                                                    class: "outline-button compact-action",
                                                    r#type: "button",
                                                    onclick: move |_| {
                                                        let mut thumbnail = copy_local_thumbnail;
                                                        let mut notice = app_state.notice;
                                                        spawn(async move {
                                                            match browser::choose_avatar().await {
                                                                Ok(Some(image)) => thumbnail.set(Some(image)),
                                                                Ok(None) => {}
                                                                Err(error) => notice.set(error),
                                                            }
                                                        });
                                                    },
                                                    "Choose image"
                                                }
                                                if selected.thumbnail.is_some() {
                                                    button {
                                                        class: "text-button",
                                                        r#type: "button",
                                                        disabled: copy_local_thumbnail() == selected.thumbnail,
                                                        onclick: {
                                                            let spotify_thumbnail = selected.thumbnail.clone();
                                                            move |_| copy_local_thumbnail.set(spotify_thumbnail.clone())
                                                        },
                                                        "Use Spotify image"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "favorite-playlist-picker", aria_label: "Choose songs for the local playlist",
                                        header { class: "favorite-playlist-picker-heading",
                                            div {
                                                strong { "Songs" }
                                                small { "{copy_local_track_indexes().len()} of {loaded_track_count} selected" }
                                            }
                                            if !tracks.is_empty() {
                                                button {
                                                    class: "text-button",
                                                    r#type: "button",
                                                    onclick: move |_| {
                                                        if copy_local_track_indexes().len() == loaded_track_count {
                                                            copy_local_track_indexes.set(Vec::new());
                                                        } else {
                                                            copy_local_track_indexes.set((0..loaded_track_count).collect());
                                                        }
                                                    },
                                                    if copy_local_track_indexes().len() == loaded_track_count { "Clear selection" } else { "Select all" }
                                                }
                                            }
                                        }
                                        if tracks.is_empty() {
                                            p { class: "favorite-playlist-empty", "There are no tracks to copy." }
                                        } else {
                                            div { class: "favorite-playlist-selection-list",
                                                for (index, track) in tracks.iter().cloned().enumerate() {
                                                    {
                                                        let selected = copy_local_track_indexes().contains(&index);
                                                        rsx! {
                                                            button {
                                                                key: "spotify-copy-{index}-{track.key()}",
                                                                class: if selected { "favorite-playlist-track selected" } else { "favorite-playlist-track" },
                                                                r#type: "button",
                                                                aria_pressed: selected,
                                                                aria_label: if selected { "Remove {track.title} from selection" } else { "Add {track.title} to selection" },
                                                                onclick: move |_| {
                                                                    let mut indexes = copy_local_track_indexes();
                                                                    if let Some(position) = indexes.iter().position(|item| *item == index) {
                                                                        indexes.remove(position);
                                                                    } else {
                                                                        indexes.push(index);
                                                                        indexes.sort_unstable();
                                                                    }
                                                                    copy_local_track_indexes.set(indexes);
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
                                        button {
                                            class: "outline-button",
                                            r#type: "button",
                                            onclick: move |_| copy_local_open.set(false),
                                            "Cancel"
                                        }
                                        button {
                                            class: "solid-button",
                                            r#type: "submit",
                                            disabled: copy_local_name().trim().is_empty(),
                                            "Create local copy"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {
                    header { class: "library-page-heading",
                        h1 { "Spotify playlist not found" }
                    }
                }
            }
        }
        _ => rsx! {},
    }
}
