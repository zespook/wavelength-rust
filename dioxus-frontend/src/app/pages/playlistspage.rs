use super::super::AppState;
use super::PageData;
use crate::components::{Icon, TrackList};
use crate::*;
use dioxus::prelude::*;

fn collection_track_count(count: usize) -> String {
    format!("{count} {}", if count == 1 { "track" } else { "tracks" })
}

fn track_matches_query(track: &Track, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || track.title.to_lowercase().contains(&query)
        || track.artist.to_lowercase().contains(&query)
        || track
            .album
            .as_deref()
            .is_some_and(|album| album.to_lowercase().contains(&query))
}

#[component]
fn AlbumCollectionCard(name: String, tracks: Vec<Track>, sort_order: usize) -> Element {
    let mut app_state = use_context::<AppState>();
    let is_single = name.eq_ignore_ascii_case("Singles");
    rsx! {
        article {
            class: if is_single {
                "collection-card playlist-card album-collection-card singles-collection-card"
            } else {
                "collection-card playlist-card album-collection-card"
            },
            style: if is_single { "order: -1".to_string() } else { format!("order: {sort_order}") },
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
                        app_state.spotify_album_selected.set(None);
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
                    p { "{collection_track_count(tracks.len())} · Album" }
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
                            app_state.queue_panel_open,
                            app_state.room_panel_open,
                        )
                    },
                    Icon { name: "play" }
                }
            }
        }
    }
}

#[component]
fn SpotifyAlbumCollectionCard(album: SpotifyAlbum, sort_order: usize) -> Element {
    let mut app_state = use_context::<AppState>();
    let artist = album.artist.trim();
    let metadata = if artist.is_empty() || artist.eq_ignore_ascii_case("unknown artist") {
        format!("{} · Spotify", collection_track_count(album.tracks_total))
    } else {
        format!(
            "{} · {} · Spotify",
            artist,
            collection_track_count(album.tracks_total)
        )
    };
    rsx! {
        article {
            class: "collection-card playlist-card album-collection-card spotify-album-card",
            style: "order: {sort_order}",
            oncontextmenu: {
                let target = album.clone();
                move |event| {
                    event.prevent_default();
                    let position = event.client_coordinates();
                    app_state.context_playlist.set(Some(PlaylistContextMenu {
                        target: PlaylistMenuTarget::SpotifyAlbum(target.clone()),
                        x: position.x,
                        y: position.y,
                    }));
                }
            },
            button {
                class: "playlist-card-open",
                aria_label: "Open Spotify album {album.name}",
                onclick: {
                    let album = album.clone();
                    move |_| load_spotify_album(
                        album.clone(),
                        app_state.spotify_album_selected,
                        app_state.spotify_album_tracks,
                        app_state.spotify_album_tracks_busy,
                        app_state.spotify_album_tracks_error,
                        app_state.spotify_detail_load_id,
                        app_state.album_detail_name,
                        app_state.page,
                        app_state.notice,
                    )
                }
            }
            div { class: "playlist-card-copy",
                span { class: "mini-art",
                    if let Some(image) = album.thumbnail.as_deref() {
                        img { src: "{image}", alt: "" }
                    } else {
                        Icon { name: "disc" }
                    }
                }
                div { class: "playlist-card-info",
                    h2 { "{album.name}" }
                    p { "{metadata}" }
                }
            }
            div { class: "playlist-card-actions",
                button {
                    class: "row-action playlist-card-play",
                    aria_label: "Play Spotify album {album.name}",
                    title: "Play album",
                    disabled: album.tracks_total == 0,
                    onclick: {
                        let album = album.clone();
                        move |_| play_spotify_album(
                            album.clone(),
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
    let spotify_albums = (app_state.spotify_albums)();
    let show_spotify = (app_state.spotify_profile)().is_some();
    let mut collection_order = albums
        .iter()
        .map(|(name, tracks)| (format!("album-{name}"), latest_track_modified_at(tracks)))
        .collect::<Vec<_>>();
    collection_order.extend(local_playlists.iter().map(|playlist| {
        (
            format!("local-{}", playlist.id),
            latest_track_modified_at(&playlist.tracks),
        )
    }));
    collection_order.sort_by_cached_key(|(_, modified_at)| std::cmp::Reverse(*modified_at));
    let mut collection_orders = collection_order
        .into_iter()
        .enumerate()
        .map(|(order, (key, _))| (key, order))
        .collect::<std::collections::HashMap<_, _>>();
    let mut next_collection_order = collection_orders.len();
    for playlist in &shared_playlists {
        collection_orders.insert(format!("room-{}", playlist.id), next_collection_order);
        next_collection_order += 1;
    }
    for album in &spotify_albums {
        collection_orders.insert(format!("spotify-album-{}", album.id), next_collection_order);
        next_collection_order += 1;
    }
    for playlist in &spotify_playlists {
        collection_orders.insert(format!("spotify-{}", playlist.id), next_collection_order);
        next_collection_order += 1;
    }
    let download_filter = (app_state.library_download_filter)();
    let download_statuses = (app_state.library_download_status)();
    let local_tracks = (app_state.local_tracks)();
    let favorites = (app_state.favorites)();
    let history = (app_state.history)();
    let queue = (app_state.queue)();
    let mut library_tracks = local_tracks.clone();
    library_tracks.extend(favorites.clone());
    if show_spotify {
        library_tracks.extend((app_state.spotify_tracks)());
    }
    for playlist in &local_playlists {
        library_tracks.extend(playlist.tracks.clone());
    }
    for (_, tracks) in &albums {
        library_tracks.extend(tracks.clone());
    }
    for playlist in &shared_playlists {
        library_tracks.extend(
            resolve_room_playlist(playlist, &local_tracks, &favorites, &history, &queue).tracks,
        );
    }
    let mut seen_library_tracks = std::collections::HashSet::new();
    library_tracks.retain(|track| {
        !track.key().is_empty() && seen_library_tracks.insert(track.key().to_string())
    });
    library_tracks.sort_by_cached_key(|track| track.title.to_lowercase());
    let visible_song_count = library_tracks
        .iter()
        .filter(|track| {
            matches_download_filter(track, download_filter, &download_statuses)
                && track_matches_query(track, &query)
        })
        .count();
    let visible_album_count = albums
        .iter()
        .filter(|(name, tracks)| album_matches(&query, name, tracks))
        .count()
        + if show_spotify {
            spotify_albums
                .iter()
                .filter(|album| {
                    playlist_matches(&query, &album.name) || playlist_matches(&query, &album.artist)
                })
                .count()
        } else {
            0
        };
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
    let header_count = if download_filter == DownloadFilter::All {
        format!("{visible_collection_item_count} {result_label}")
    } else {
        format!(
            "{visible_song_count} {}",
            if visible_song_count == 1 {
                "song"
            } else {
                "songs"
            }
        )
    };
    let favorite_keys = favorites
        .iter()
        .map(|track| track.key().to_string())
        .collect::<Vec<_>>();
    let refresh_busy = (app_state.library_download_status_busy)()
        || (show_spotify
            && ((app_state.spotify_playlists_busy)() || (app_state.spotify_albums_busy)()));
    match Page::Playlists {
        Page::Playlists => rsx! {
            header { class: "library-page-heading",
                div { class: "library-heading-title",
                    h1 { "Library" }
                    span { class: "library-heading-count", aria_live: "polite", "{header_count}" }
                }
                div { class: "playlist-heading-actions",
                    div {
                        class: "library-create-menu-wrap",
                        onkeydown: move |event| {
                            if event.key().to_string() == "Escape" {
                                app_state.playlist_create_menu_open.set(false);
                            }
                        },
                        button {
                            class: "icon-button library-plus-button",
                            aria_label: "Create playlist",
                            aria_expanded: (app_state.playlist_create_menu_open)(),
                            title: "Create playlist",
                            onclick: {
                                let favorite_keys = favorite_keys.clone();
                                move |_| {
                                    if show_spotify {
                                        app_state.download_filter_menu_open.set(false);
                                        app_state.playlist_create_menu_open.toggle();
                                    } else {
                                        app_state.playlist_create_menu_open.set(false);
                                        app_state.download_filter_menu_open.set(false);
                                        app_state.playlist_name.set(String::new());
                                        app_state.playlist_create_favorites.set(favorite_keys.clone());
                                        app_state.playlist_create_open.set(true);
                                    }
                                }
                            },
                            Icon { name: "plus" }
                        }
                        if (app_state.playlist_create_menu_open)() && show_spotify {
                            button {
                                class: "library-menu-dismiss",
                                r#type: "button",
                                aria_label: "Close playlist creation menu",
                                tabindex: "-1",
                                onclick: move |_| app_state.playlist_create_menu_open.set(false)
                            }
                            div { class: "library-create-menu", role: "menu",
                                button { r#type: "button", role: "menuitem", onclick: {
                                    let favorite_keys = favorite_keys.clone();
                                    move |_| {
                                        app_state.playlist_create_menu_open.set(false);
                                        app_state.playlist_name.set(String::new());
                                        app_state.playlist_create_favorites.set(favorite_keys.clone());
                                        app_state.playlist_create_open.set(true);
                                    }
                                }, "Create local playlist" }
                                button { r#type: "button", role: "menuitem", onclick: move |_| {
                                    app_state.playlist_create_menu_open.set(false);
                                    app_state.spotify_playlist_create_name.set(String::new());
                                    app_state.spotify_playlist_create_open.set(true);
                                }, "Create Spotify playlist" }
                            }
                        }
                    }
                    button {
                        class: "icon-button library-refresh-button",
                        aria_label: "Refresh library",
                        title: "Refresh library",
                        disabled: refresh_busy,
                        onclick: {
                            let tracks = library_tracks.clone();
                            move |_| {
                                if show_spotify {
                                    app_state.spotify_collections_page_loaded.set(true);
                                    load_spotify_collections(
                                        app_state.spotify_playlists,
                                        app_state.spotify_playlists_busy,
                                        app_state.spotify_albums,
                                        app_state.spotify_albums_busy,
                                        app_state.notice,
                                    );
                                }
                                refresh_library_download_status(
                                    tracks.clone(),
                                    show_spotify,
                                    app_state.spotify_tracks,
                                    app_state.spotify_busy,
                                    app_state.library_download_status,
                                    app_state.library_download_status_busy,
                                    app_state.library_download_status_error,
                                );
                            }
                        },
                        Icon { name: "refresh" }
                    }
                }
            }
            div { class: "collection-search-section",
                div { class: "playlists-toolbar",
                    label { class: "playlist-search",
                        input {
                            r#type: "search",
                            value: "{query}",
                            placeholder: if download_filter == DownloadFilter::All { "Search albums and playlists" } else { "Search songs" },
                            aria_label: if download_filter == DownloadFilter::All { "Search albums and playlists" } else { "Search songs" },
                            oninput: move |event| app_state.playlist_query.set(event.value())
                        }
                    }
                    div { class: "playlist-toolbar-actions",
                        div {
                            class: "download-filter-wrap",
                            onkeydown: move |event| {
                                if event.key().to_string() == "Escape" {
                                    app_state.download_filter_menu_open.set(false);
                                }
                            },
                            button {
                                class: "outline-button compact-action download-filter-trigger",
                                aria_expanded: (app_state.download_filter_menu_open)(),
                                aria_label: "Filter songs by download status",
                                onclick: move |_| {
                                    app_state.playlist_create_menu_open.set(false);
                                    app_state.download_filter_menu_open.toggle();
                                },
                                "Downloaded"
                                Icon { name: "chevron-down" }
                            }
                            if (app_state.download_filter_menu_open)() {
                                button {
                                    class: "library-menu-dismiss",
                                    r#type: "button",
                                    aria_label: "Close download filter menu",
                                    tabindex: "-1",
                                    onclick: move |_| app_state.download_filter_menu_open.set(false)
                                }
                                div { class: "download-filter-menu", role: "menu",
                                    for (filter, label) in [
                                        (DownloadFilter::All, "All"),
                                        (DownloadFilter::Downloaded, "Downloaded"),
                                        (DownloadFilter::NotDownloaded, "Not downloaded"),
                                    ] {
                                        button {
                                            r#type: "button",
                                            role: "menuitemradio",
                                            aria_checked: download_filter == filter,
                                            disabled: (app_state.library_download_status_busy)() && filter != DownloadFilter::All,
                                            onclick: {
                                                let tracks = library_tracks.clone();
                                                move |_| {
                                                    app_state.library_download_filter.set(filter);
                                                    app_state.download_filter_menu_open.set(false);
                                                    if filter != DownloadFilter::All {
                                                        refresh_library_download_status(
                                                            tracks.clone(),
                                                            show_spotify,
                                                            app_state.spotify_tracks,
                                                            app_state.spotify_busy,
                                                            app_state.library_download_status,
                                                            app_state.library_download_status_busy,
                                                            app_state.library_download_status_error,
                                                        );
                                                    }
                                                }
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                        if download_filter == DownloadFilter::All {
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
                }
            }
            if download_filter != DownloadFilter::All {
                if (app_state.library_download_status_busy)() {
                    p { class: "playlist-empty", "Loading download status" }
                } else if !(app_state.library_download_status_error)().is_empty() {
                    div { class: "library-download-error", role: "alert",
                        p { "{(app_state.library_download_status_error)()}" }
                        button {
                            class: "text-button",
                            r#type: "button",
                            onclick: {
                                let tracks = library_tracks.clone();
                                move |_| refresh_library_download_status(
                                    tracks.clone(),
                                    show_spotify,
                                    app_state.spotify_tracks,
                                    app_state.spotify_busy,
                                    app_state.library_download_status,
                                    app_state.library_download_status_busy,
                                    app_state.library_download_status_error,
                                )
                            },
                            "Retry"
                        }
                    }
                } else if visible_song_count == 0 {
                    p { class: "playlist-empty", if download_filter == DownloadFilter::Downloaded { "No downloaded songs match your search" } else { "No not-downloaded songs match your search" } }
                } else {
                    TrackList {
                        tracks: library_tracks.iter().filter(|track| {
                            matches_download_filter(track, download_filter, &download_statuses)
                                && track_matches_query(track, &query)
                        }).cloned().collect(),
                        current: app_state.current,
                        audio_src: app_state.audio_src,
                        playing: app_state.playing,
                        queue: app_state.queue,
                        history: app_state.history,
                        favorites: app_state.favorites,
                        notice: app_state.notice,
                        context_track: app_state.context_track,
                        download_statuses: Some(download_statuses.clone()),
                        on_download: move |track| download_track(track, (app_state.download_format)(), app_state.notice),
                    }
                }
            } else {
            div { class: if list_view { "collection-grid list-view" } else { "collection-grid" },
                if !albums.is_empty() {
                    for (album_index, (name, tracks)) in albums.iter().cloned()
                        .filter(|(name, tracks)| album_matches(&query, name, tracks))
                        .enumerate()
                    {
                        AlbumCollectionCard {
                            key: "album-{album_index}-{name}",
                            sort_order: collection_orders.get(&format!("album-{name}")).copied().unwrap_or(album_index),
                            name,
                            tracks
                        }
                    }
                }
                if show_spotify {
                    for album in spotify_albums.iter().cloned()
                        .filter(|album| {
                            playlist_matches(&query, &album.name)
                                || playlist_matches(&query, &album.artist)
                        })
                    {
                        SpotifyAlbumCollectionCard {
                            key: "spotify-album-{album.id}",
                            sort_order: collection_orders.get(&format!("spotify-album-{}", album.id)).copied().unwrap_or_default(),
                            album
                        }
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
                        div { class: "playlist-card-copy",
                            span { class: "mini-art favorite-playlist-art", Icon { name: "favorite-filled" } }
                            div { class: "playlist-card-info",
                                h2 { "Favorite Songs" }
                                p { "{collection_track_count((app_state.favorites)().len())} · Favorites" }
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
                                    app_state.queue_panel_open,
                                    app_state.room_panel_open,
                                ),
                                Icon { name: "play" }
                            }
                        }
                    }
                }
                for playlist in local_playlists.iter().cloned().filter(|item| playlist_matches(&query, &item.name)) {
                    article {
                        class: "collection-card playlist-card",
                        style: format!(
                            "order: {}",
                            collection_orders.get(&format!("local-{}", playlist.id)).copied().unwrap_or_default()
                        ),
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
                                if let Some(image) = playlist.thumbnail.as_deref() {
                                    img { src: "{image}", alt: "" }
                                } else {
                                    Icon { name: "list" }
                                }
                            }
                            div { class: "playlist-card-info",
                                h2 { "{playlist.name}" }
                                p { "{collection_track_count(playlist.tracks.len())} · Playlist" }
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
                                        app_state.queue_panel_open,
                                        app_state.room_panel_open,
                                    )
                                },
                                Icon { name: "play" }
                            }
                        }
                }
                    }
            for playlist in shared_playlists.iter().cloned().filter(|item| playlist_matches(&query, &item.name)) {
                            article {
                                class: "collection-card playlist-card shared-playlist-card",
                                style: format!(
                                    "order: {}",
                                    collection_orders.get(&format!("room-{}", playlist.id)).copied().unwrap_or_default()
                                ),
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
                                        p { "{collection_track_count(playlist.tracks.len())} · Shared playlist" }
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
                                                    app_state.queue_panel_open,
                                                    app_state.room_panel_open,
                                                );
                                            }
                                        },
                                        Icon { name: "play" }
                                    }
                                }
                            }
                }
            if show_spotify {
                for playlist in spotify_playlists.iter().cloned().filter(|item| playlist_matches(&query, &item.name)) {
                            article {
                                class: "collection-card playlist-card spotify-playlist-card",
                                style: format!(
                                    "order: {}",
                                    collection_orders.get(&format!("spotify-{}", playlist.id)).copied().unwrap_or_default()
                                ),
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
                                                app_state.spotify_playlist_tracks_error,
                                                app_state.spotify_detail_load_id,
                                                app_state.notice,
                                                app_state.page,
                                            );
                                        }
                                    }
                                }
                                div { class: "playlist-card-copy",
                                    span { class: "mini-art",
                                        if let Some(image) = playlist.thumbnail.as_deref() {
                                            img { src: "{image}", alt: "" }
                                        } else {
                                            Icon { name: "list" }
                                        }
                                    }
                                    div { class: "playlist-card-info",
                                        h2 { "{playlist.name}" }
                                        p { "{collection_track_count(playlist.tracks_total)} · Spotify" }
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
                                                    app_state.queue_panel_open,
                                                    app_state.room_panel_open,
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
                && !(show_spotify
                    && ((app_state.spotify_playlists_busy)() || (app_state.spotify_albums_busy)()))
            {
                p { class: "playlist-empty", "No albums or playlists match that search" }
            }
            }
        },
        _ => rsx! {},
    }
}
