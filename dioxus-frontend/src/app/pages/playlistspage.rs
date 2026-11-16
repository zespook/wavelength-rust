use super::super::AppState;
use super::PageData;
use crate::components::Icon;
use crate::*;
use dioxus::prelude::*;

#[component]
fn AlbumCollectionCard(name: String, tracks: Vec<Track>) -> Element {
    let mut app_state = use_context::<AppState>();
    rsx! {
        article { class: "collection-card playlist-card album-collection-card",
            oncontextmenu: {
                let menu_name = name.clone();
                let menu_tracks = tracks.clone();
                move |event| {
                    event.prevent_default();
                    let position = event.client_coordinates();
                    app_state.context_playlist.set(Some(PlaylistContextMenu {
                        target: PlaylistMenuTarget::Album {
                            name: menu_name.clone(),
                            tracks: menu_tracks.clone(),
                        },
                        x: position.x,
                        y: position.y,
                    }));
                }
            },
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
            }
        }
    }
}

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn PlaylistsPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let favorite_playlist_id = data.favorite_playlist_id.clone();
    let albums = data.home_albums.clone();
    let query = (app_state.playlist_query)();
    let list_view = (app_state.playlist_list_view)();
    let local_playlists = (app_state.playlists)();
    let shared_playlists = (app_state.room_shared_playlists)();
    let spotify_playlists = (app_state.spotify_playlists)();
    let show_spotify = (app_state.spotify_profile)().is_some();
    let visible_album_count = albums
        .iter()
        .filter(|(name, tracks)| album_matches(&query, name, tracks))
        .count();
    let visible_collection_item_count = visible_album_count
        + if playlist_matches(&query, "Favorite Songs") {
            1
        } else {
            0
        }
        + local_playlists
            .iter()
            .filter(|item| playlist_matches(&query, &item.name))
            .count()
        + shared_playlists
            .iter()
            .filter(|item| playlist_matches(&query, &item.name))
            .count()
        + if show_spotify {
            spotify_playlists
                .iter()
                .filter(|item| playlist_matches(&query, &item.name))
                .count()
        } else {
            0
        };
    let result_label = if visible_collection_item_count == 1 {
        "item"
    } else {
        "items"
    };
    match Page::Playlists {
        Page::Playlists => rsx! {
            header { class: "library-page-heading",
                h1 { "Library" }
                button { class: "solid-button playlist-create-trigger", onclick: move |_| {
                    app_state.playlist_name.set(String::new());
                    app_state.playlist_create_favorites.set((app_state.favorites)().iter().map(|track| track.key().to_string()).collect());
                    app_state.playlist_create_open.set(true);
                },
                    Icon { name: "plus" }
                    "Create playlist"
                }
            }
            div { class: "collection-search-section",
            div { class: "playlists-toolbar",
                label { class: "playlist-search",
                    input {
                        r#type: "search",
                            value: "{query}",
                            placeholder: "Search albums and playlists",
                            aria_label: "Search albums and playlists",
                        oninput: move |event| app_state.playlist_query.set(event.value())
                    }
                }
                    div { class: "playlist-toolbar-actions",
                        if show_spotify {
                            if (app_state.spotify_playlists_busy)() {
                                span { class: "playlist-result-count", "Loading Spotify" }
                            }
                            button {
                                class: "outline-button compact-action playlist-refresh",
                                aria_label: "Refresh Spotify playlists",
                                title: "Refresh Spotify playlists",
                                disabled: (app_state.spotify_playlists_busy)(),
                                onclick: move |_| {
                                    app_state.spotify_playlists_page_loaded.set(true);
                                    load_spotify_playlists(app_state.spotify_playlists, app_state.spotify_playlists_busy, app_state.notice);
                                },
                                Icon { name: "refresh" }
                                "Refresh"
                            }
                        }
                        div { class: "collection-view-toggle", role: "group", aria_label: "Collection view",
                            button {
                                class: if list_view { "icon-button collection-view-button" } else { "icon-button collection-view-button active" },
                                aria_label: "Grid view",
                                title: "Grid view",
                                aria_pressed: !list_view,
                                onclick: move |_| app_state.playlist_list_view.set(false),
                                Icon { name: "grid" }
                            }
                            button {
                                class: if list_view { "icon-button collection-view-button active" } else { "icon-button collection-view-button" },
                                aria_label: "List view",
                                title: "List view",
                                aria_pressed: list_view,
                                onclick: move |_| app_state.playlist_list_view.set(true),
                                Icon { name: "list" }
                            }
                        }
                    }
                }
                div { class: "collection-results-meta",
                    span { class: "playlist-result-count", "{visible_collection_item_count} {result_label}" }
                }
            }
            div { class: if list_view { "collection-grid list-view" } else { "collection-grid" },
                if !albums.is_empty() {
                    for (album_index, (name, tracks)) in albums.iter().cloned()
                        .filter(|(name, tracks)| album_matches(&query, name, tracks))
                        .enumerate()
                    {
                        AlbumCollectionCard { key: "album-{album_index}-{name}", name, tracks }
                    }
                }
                if playlist_matches(&query, "Favorite Songs") {
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
                        div { class: "playlist-card-copy favorite-playlist-card-copy",
                            span { class: "mini-art favorite-playlist-art", Icon { name: "favorite-filled" } }
                            div { class: "favorite-playlist-card-details",
                                div { class: "playlist-card-info",
                                    h2 { "Favorite Songs" }
                                    p { "{(app_state.favorites)().len()} tracks" }
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
                                }
                            }
                        }
                    }
                }
                for playlist in (app_state.playlists)().into_iter().filter(|item| playlist_matches(&query, &item.name)) {
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
                        }
                }
                    }
            for playlist in shared_playlists.into_iter().filter(|item| playlist_matches(&query, &item.name)) {
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
                                }
                            }
                }
            if show_spotify {
                for playlist in spotify_playlists.into_iter().filter(|item| playlist_matches(&query, &item.name)) {
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
                        }
                        }
                    }
                }
            }
            if visible_collection_item_count == 0
                && !(show_spotify && (app_state.spotify_playlists_busy)())
            {
                p { class: "playlist-empty", "No albums or playlists match that search" }
            }
        },
        _ => rsx! {},
    }
}
