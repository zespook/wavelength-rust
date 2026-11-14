use super::super::AppState;
use super::PageData;
use crate::components::Icon;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn PlaylistsPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let visible_local_playlist_count = data.visible_local_playlist_count.clone();
    let favorite_playlist_id = data.favorite_playlist_id.clone();
    let albums = data.home_albums.clone();
    match Page::Playlists {
        Page::Playlists => rsx! {
            header { class: "library-page-heading",
                h1 { "Playlists" }
                button { class: "solid-button playlist-create-trigger", onclick: move |_| {
                    app_state.playlist_name.set(String::new());
                    app_state.playlist_create_favorites.set((app_state.favorites)().iter().map(|track| track.key().to_string()).collect());
                    app_state.playlist_create_open.set(true);
                },
                    Icon { name: "plus" }
                    "Create playlist"
                }
            }
            if !albums.is_empty() {
                section { class: "library-section album-collections-section",
                    header { class: "library-page-heading",
                        h2 { "Albums" }
                        span { class: "playlist-result-count", "{albums.len()} local" }
                    }
                    div { class: "collection-grid",
                        for (album_index, (name, tracks)) in albums.iter().cloned().enumerate() {
                            article { class: "collection-card album-collection-card", key: "album-{album_index}-{name}",
                                button {
                                    class: "playlist-card-open",
                                    aria_label: "Open album {name}",
                                    onclick: {
                                        let album_name = name.clone();
                                        move |_| {
                                            app_state.album_detail_name.set(Some(album_name.clone()));
                                            app_state.page.set(Page::AlbumDetail);
                                        }
                                    }
                                }
                                div { class: "playlist-card-copy",
                                    span { class: "mini-art",
                                        if let Some(image) = tracks.first().and_then(|track| track.thumbnail.as_deref()) {
                                            img { src: "{image}", alt: "" }
                                        } else {
                                            Icon { name: "disc" }
                                        }
                                    }
                                    div { class: "playlist-card-info",
                                        h2 { "{name}" }
                                        p { "{tracks.len()} songs · Album" }
                                    }
                                }
                                div { class: "playlist-card-actions",
                                    button {
                                        class: "row-action playlist-card-play",
                                        aria_label: "Play album {name}",
                                        title: "Play",
                                        disabled: tracks.is_empty(),
                                        onclick: {
                                            let tracks = tracks.clone();
                                            move |_| play_playlist_tracks(
                                                tracks.clone(),
                                                app_state.current,
                                                app_state.audio_src,
                                                app_state.playing,
                                                app_state.queue,
                                                app_state.history,
                                                app_state.notice,
                                                app_state.page,
                                            )
                                        },
                                        Icon { name: "play" }
                                    }
                                    button {
                                        class: "row-action",
                                        aria_label: "Add album {name} to queue",
                                        title: "Add to queue",
                                        onclick: {
                                            let tracks = tracks.clone();
                                            move |_| {
                                                for track in tracks.clone() {
                                                    add_to_queue(track, app_state.queue, app_state.notice);
                                                }
                                            }
                                        },
                                        Icon { name: "list" }
                                    }
                                    button {
                                        class: "row-action",
                                        aria_label: "Like album {name}",
                                        title: "Like tracks",
                                        onclick: {
                                            let tracks = tracks.clone();
                                            move |_| {
                                                for track in tracks.clone() {
                                                    toggle_favorite(track, app_state.favorites, app_state.notice);
                                                }
                                            }
                                        },
                                        Icon { name: "favorite" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "playlists-toolbar",
                label { class: "playlist-search",
                    input {
                        r#type: "search",
                        value: "{(app_state.playlist_query)()}",
                        placeholder: "Find playlists",
                        aria_label: "Find playlists",
                        oninput: move |event| app_state.playlist_query.set(event.value())
                    }
                }
                span { class: "playlist-result-count",
                    "{visible_local_playlist_count} local"
                }
                if (app_state.spotify_profile)().is_some() {
                    span { class: "playlist-result-count",
                        "{(app_state.spotify_playlists)().iter().filter(|item| playlist_matches(&(app_state.playlist_query)(), &item.name)).count()} Spotify"
                    }
                }
            }
            div { class: "collection-grid",
                if playlist_matches(&(app_state.playlist_query)(), "Favorite Songs") {
                    article {
                        class: "collection-card playlist-card favorite-playlist-card",
                        key: "{favorite_playlist_id}",
                        oncontextmenu: move |event| {
                            event.prevent_default();
                            let position = event.client_coordinates();
                            app_state.context_playlist.set(Some(PlaylistContextMenu {
                                target: PlaylistMenuTarget::Favorites,
                                x: position.x,
                                y: position.y,
                            }));
                        },
                        button {
                            class: "playlist-card-open",
                            aria_label: "Open Favorite Songs",
                            onclick: move |_| {
                                app_state.selected_playlist_id.set(Some("favorite-songs".to_string()));
                                app_state.page.set(Page::PlaylistDetail);
                            }
                        }
                        div { class: "playlist-card-copy",
                            span { class: "mini-art favorite-playlist-art", Icon { name: "favorite-filled" } }
                            div { class: "playlist-card-info",
                                h2 { "Favorite Songs" }
                                p { "{(app_state.favorites)().len()} tracks" }
                            }
                        }
                        div { class: "playlist-card-actions",
                            button {
                                class: "row-action playlist-card-play",
                                aria_label: "Play Favorite Songs",
                                title: "Play",
                                disabled: (app_state.favorites)().is_empty(),
                                onclick: move |_| play_playlist_tracks(
                                    (app_state.favorites)(),
                                    app_state.current,
                                    app_state.audio_src,
                                    app_state.playing,
                                    app_state.queue,
                                    app_state.history,
                                    app_state.notice,
                                    app_state.page,
                                ),
                                Icon { name: "play" }
                            }
                            button {
                                class: "row-action playlist-card-more",
                                aria_label: "More actions for Favorite Songs",
                                title: "More actions",
                                onclick: move |event| {
                                    let position = event.client_coordinates();
                                    app_state.context_playlist.set(Some(PlaylistContextMenu {
                                        target: PlaylistMenuTarget::Favorites,
                                        x: position.x,
                                        y: position.y,
                                    }));
                                },
                                Icon { name: "more" }
                            }
                        }
                    }
                }
                for playlist in (app_state.playlists)().into_iter().filter(|item| playlist_matches(&(app_state.playlist_query)(), &item.name)) {
                    article {
                        class: "collection-card playlist-card",
                        key: "{playlist.id}",
                        oncontextmenu: {
                            let target = playlist.clone();
                            move |event| {
                                event.prevent_default();
                                let position = event.client_coordinates();
                                app_state.context_playlist.set(Some(PlaylistContextMenu {
                                    target: PlaylistMenuTarget::Local(target.clone()),
                                    x: position.x,
                                    y: position.y,
                                }));
                            }
                        },
                        button {
                            class: "playlist-card-open",
                            aria_label: "Open {playlist.name}",
                            onclick: {
                                let playlist_id = playlist.id.clone();
                                move |_| {
                                    app_state.selected_playlist_id.set(Some(playlist_id.clone()));
                                    app_state.page.set(Page::PlaylistDetail);
                                }
                            }
                        }
                        div { class: "playlist-card-copy",
                            span { class: "mini-art",
                                Icon { name: "list" }
                            }
                            div { class: "playlist-card-info",
                                h2 { "{playlist.name}" }
                                p { "{playlist.tracks.len()} tracks" }
                            }
                        }
                        div { class: "playlist-card-actions",
                            button {
                                class: "row-action playlist-card-play",
                                aria_label: "Play {playlist.name}",
                                title: "Play",
                                disabled: playlist.tracks.is_empty(),
                                onclick: {
                                    let tracks = playlist.tracks.clone();
                                    move |_| play_playlist_tracks(
                                        tracks.clone(),
                                        app_state.current,
                                        app_state.audio_src,
                                        app_state.playing,
                                        app_state.queue,
                                        app_state.history,
                                        app_state.notice,
                                        app_state.page,
                                    )
                                },
                                Icon { name: "play" }
                            }
                            button {
                                class: "row-action playlist-card-more",
                                aria_label: "More actions for {playlist.name}",
                                title: "More actions",
                                onclick: {
                                    let target = playlist.clone();
                                    move |event| {
                                        let position = event.client_coordinates();
                                        app_state.context_playlist.set(Some(PlaylistContextMenu {
                                            target: PlaylistMenuTarget::Local(target.clone()),
                                            x: position.x,
                                            y: position.y,
                                        }));
                                    }
                                },
                                Icon { name: "more" }
                            }
                        }
                    }
                }
            }
            if !(app_state.room_shared_playlists)().is_empty() {
                section { class: "library-section room-playlists-section",
                    header { class: "library-page-heading",
                        h2 { "Shared in this room" }
                        span { class: "playlist-result-count",
                            "{(app_state.room_shared_playlists)().iter().filter(|item| playlist_matches(&(app_state.playlist_query)(), &item.name)).count()} shared"
                        }
                    }
                    div { class: "collection-grid",
                        for playlist in (app_state.room_shared_playlists)().into_iter().filter(|item| playlist_matches(&(app_state.playlist_query)(), &item.name)) {
                            article {
                                class: "collection-card playlist-card shared-playlist-card",
                                key: "room-{playlist.id}",
                                oncontextmenu: {
                                    let target = playlist.clone();
                                    move |event| {
                                        event.prevent_default();
                                        let position = event.client_coordinates();
                                        app_state.context_playlist.set(Some(PlaylistContextMenu {
                                            target: PlaylistMenuTarget::Shared(target.clone()),
                                            x: position.x,
                                            y: position.y,
                                        }));
                                    }
                                },
                                button {
                                    class: "playlist-card-open",
                                    aria_label: "Open {playlist.name}",
                                    onclick: {
                                        let playlist_id = playlist.id.clone();
                                        move |_| {
                                            app_state.selected_playlist_id.set(Some(playlist_id.clone()));
                                            app_state.page.set(Page::PlaylistDetail);
                                        }
                                    }
                                }
                                div { class: "playlist-card-copy",
                                    span { class: "mini-art", Icon { name: "list" } }
                                    div { class: "playlist-card-info",
                                        h2 { "{playlist.name}" }
                                        p { "{playlist.tracks.len()} tracks · Shared in this room" }
                                    }
                                }
                                div { class: "playlist-card-actions",
                                    button {
                                        class: "row-action playlist-card-play",
                                        aria_label: "Play {playlist.name}",
                                        title: "Play",
                                        disabled: playlist.tracks.is_empty(),
                                        onclick: {
                                            let target = playlist.clone();
                                            move |_| {
                                                let resolved = resolve_room_playlist(
                                                    &target,
                                                    &(app_state.local_tracks)(),
                                                    &(app_state.favorites)(),
                                                    &(app_state.history)(),
                                                    &(app_state.queue)(),
                                                );
                                                play_playlist_tracks(
                                                    resolved.tracks,
                                                    app_state.current,
                                                    app_state.audio_src,
                                                    app_state.playing,
                                                    app_state.queue,
                                                    app_state.history,
                                                    app_state.notice,
                                                    app_state.page,
                                                );
                                            }
                                        },
                                        Icon { name: "play" }
                                    }
                                    button {
                                        class: "row-action playlist-card-more",
                                        aria_label: "More actions for {playlist.name}",
                                        title: "More actions",
                                        onclick: {
                                            let target = playlist.clone();
                                            move |event| {
                                                let position = event.client_coordinates();
                                                app_state.context_playlist.set(Some(PlaylistContextMenu {
                                                    target: PlaylistMenuTarget::Shared(target.clone()),
                                                    x: position.x,
                                                    y: position.y,
                                                }));
                                            }
                                        },
                                        Icon { name: "more" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if (app_state.spotify_profile)().is_some() {
                section { class: "library-section",
                    header { class: "library-page-heading",
                        h2 { "Spotify playlists" }
                        button {
                            class: "outline-button compact-action",
                            disabled: (app_state.spotify_playlists_busy)(),
                            onclick: move |_| {
                                app_state.spotify_playlists_page_loaded.set(true);
                                load_spotify_playlists(app_state.spotify_playlists, app_state.spotify_playlists_busy, app_state.notice);
                            },
                            "Refresh"
                        }
                    }
                    if (app_state.spotify_playlists_busy)() {
                        p { class: "library-empty", "Loading Spotify playlists…" }
                    } else if (app_state.spotify_playlists)().is_empty() {
                        p { class: "library-empty", "No Spotify playlists found." }
                    }
                    div { class: "collection-grid",
                        for playlist in (app_state.spotify_playlists)().into_iter().filter(|item| playlist_matches(&(app_state.playlist_query)(), &item.name)) {
                            article {
                                class: "collection-card playlist-card spotify-playlist-card",
                                key: "spotify-{playlist.id}",
                                oncontextmenu: {
                                    let target = playlist.clone();
                                    move |event| {
                                        event.prevent_default();
                                        let position = event.client_coordinates();
                                        app_state.context_playlist.set(Some(PlaylistContextMenu {
                                            target: PlaylistMenuTarget::Spotify(target.clone()),
                                            x: position.x,
                                            y: position.y,
                                        }));
                                    }
                                },
                                button {
                                    class: "playlist-card-open",
                                    aria_label: "Open {playlist.name}",
                                    onclick: {
                                        let selected = playlist.clone();
                                        move |_| {
                                            load_spotify_playlist(
                                                selected.clone(),
                                                app_state.spotify_playlist_tracks,
                                                app_state.spotify_playlist_selected,
                                                app_state.spotify_playlist_name,
                                                app_state.spotify_playlist_tracks_busy,
                                                app_state.notice,
                                            );
                                            app_state.page.set(Page::SpotifyPlaylists);
                                        }
                                    }
                                }
                                div { class: "playlist-card-copy",
                                    span { class: "mini-art",
                                        Icon { name: "list" }
                                    }
                                    div { class: "playlist-card-info",
                                        h2 { "{playlist.name}" }
                                        p { "{playlist.tracks_total} tracks" }
                                    }
                                }
                                div { class: "playlist-card-actions",
                                    button {
                                        class: "row-action playlist-card-play",
                                        aria_label: "Play {playlist.name}",
                                        title: "Play",
                                        disabled: playlist.tracks_total == 0,
                                        onclick: {
                                            let selected = playlist.clone();
                                            move |_| {
                                                play_spotify_playlist(
                                                    selected.clone(),
                                                    app_state.current,
                                                    app_state.audio_src,
                                                    app_state.playing,
                                                    app_state.queue,
                                                    app_state.history,
                                                    app_state.notice,
                                                    app_state.page,
                                                );
                                            }
                                        },
                                        Icon { name: "play" }
                                    }
                                    button {
                                        class: "row-action playlist-card-more",
                                        aria_label: "More actions for {playlist.name}",
                                        title: "More actions",
                                        onclick: {
                                            let target = playlist.clone();
                                            move |event| {
                                                let position = event.client_coordinates();
                                                app_state.context_playlist.set(Some(PlaylistContextMenu {
                                                    target: PlaylistMenuTarget::Spotify(target.clone()),
                                                    x: position.x,
                                                    y: position.y,
                                                }));
                                            }
                                        },
                                        Icon { name: "more" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !(app_state.playlist_query)().trim().is_empty()
                && (app_state.playlists)().iter().all(|item| !playlist_matches(&(app_state.playlist_query)(), &item.name))
                && !playlist_matches(&(app_state.playlist_query)(), "Favorite Songs")
                && (app_state.room_shared_playlists)().iter().all(|item| !playlist_matches(&(app_state.playlist_query)(), &item.name))
                && (!(app_state.spotify_profile)().is_some()
                    || (app_state.spotify_playlists)().iter().all(|item| !playlist_matches(&(app_state.playlist_query)(), &item.name)))
            {
                p { class: "playlist-empty", "No playlists match that search." }
            }
        },
        _ => rsx! {},
    }
}
