use super::super::AppState;
use super::PageData;
use crate::browser;
use crate::components::{CustomSelect, Icon, TrackList};
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn HomePage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let home_featured_track = data.home_featured_track.clone();
    let recently_added = data.recently_added.clone();
    let home_playlists = data.home_playlists.clone();
    let home_albums = data.home_albums.clone();
    let artist_stats = data.artist_stats.clone();
    let song_stats = data.song_stats.clone();
    let show_home_demo = data.show_home_demo.clone();
    match Page::Home {
        Page::Home => rsx! {
            section { class: "home-hero",
                div { class: "home-hero-copy",
                    section { class: "home-greeting",
                        p { class: "eyebrow", "Your library" }
                        h1 { "Your music, within reach." }
                        p { "Search across music services or bring your own collection together." }
                    }
                    div { class: "home-actions",
                        button { class: "solid-button", onclick: move |_| app_state.page.set(Page::Search),
                            Icon { name: "search" }
                            "Find music"
                        }
                        button { class: "outline-button", onclick: move |_| {
                            spawn(async move {
                                match browser::choose_folder().await {
                                    Ok(Some(tracks)) => {
                                        app_state.local_tracks.set(merge_local_tracks((app_state.local_tracks)(), tracks));
                                        app_state.page.set(Page::Library);
                                    }
                                    Ok(None) => {}
                                    Err(error) => app_state.notice.set(error),
                                }
                            });
                        },
                            Icon { name: "folder" }
                            "Add a folder"
                        }
                    }
                    p { class: "home-library-count",
                        "{(app_state.local_tracks)().len()} local tracks"
                        span { "·" }
                        "{(app_state.playlists)().len()} playlists"
                    }
                }
                article { class: "home-library-preview",
                    div { class: "home-preview-cover",
                        if let Some(track) = home_featured_track.as_ref() {
                            if let Some(image) = track.thumbnail.as_ref() {
                                img { src: "{image}", alt: "" }
                            } else {
                                Icon { name: "disc" }
                            }
                        } else {
                            Icon { name: "disc" }
                        }
                    }
                    div { class: "home-preview-copy",
                        p { class: "eyebrow",
                            if home_featured_track.is_some() { "In your library" } else { "Start your library" }
                        }
                        if let Some(track) = home_featured_track.as_ref() {
                            h2 { "{track.title}" }
                            p { "{track.artist}" }
                        } else {
                            h2 { "Nothing added yet" }
                            p { "Add a music folder to browse albums and tracks here." }
                        }
                    }
                    if home_featured_track.is_some() {
                        button { class: "home-preview-link", onclick: move |_| app_state.page.set(Page::Library),
                            "Browse local library"
                            Icon { name: "arrow-right" }
                        }
                    }
                }
            }
            section { class: "home-recently-played",
                div { class: "section-head",
                    div { h2 { "Recently played" } }
                    if show_home_demo {
                        span { class: "home-demo-badge", "Sample data" }
                    }
                }
                if show_home_demo {
                    div { class: "home-demo-track-list", role: "list",
                        for (index, (title, artist, played_at)) in HOME_DEMO_RECENT.iter().enumerate() {
                            div { class: "home-demo-track", key: "{index}", role: "listitem",
                                span { class: "home-demo-art", Icon { name: "disc" } }
                                div { class: "home-demo-track-copy",
                                    strong { "{title}" }
                                    small { "{artist} · {played_at}" }
                                }
                            }
                        }
                    }
                } else if !(app_state.history)().is_empty() {
                    TrackList {
                        tracks: (app_state.history)().into_iter().take(6).collect(),
                        current: app_state.current, audio_src: app_state.audio_src, playing: app_state.playing, queue: app_state.queue, history: app_state.history, favorites: app_state.favorites, notice: app_state.notice, context_track: app_state.context_track,
                        context_menu_only: true
                    }
                } else {
                    p { class: "settings-empty", "Recent plays will appear here." }
                }
            }
            if !home_playlists.is_empty() || !home_albums.is_empty() {
                section { class: "home-section home-collections",
                    div { class: "section-head",
                        div {
                            p { class: "eyebrow", "Your library" }
                            h2 { "Recent playlists and albums" }
                        }
                        button { class: "text-button", onclick: move |_| app_state.page.set(Page::Playlists), "View library" }
                    }
                    div { class: "home-collection-grid",
                        for playlist in home_playlists.iter().cloned() {
                            button {
                                class: "home-collection-card",
                                key: "{playlist.id}",
                                onclick: {
                                    let id = playlist.id.clone();
                                    move |_| {
                                        app_state.selected_playlist_id.set(Some(id.clone()));
                                        app_state.page.set(Page::PlaylistDetail);
                                    }
                                },
                                span { class: "mini-art",
                                    if let Some(track) = playlist.tracks.first() {
                                        if let Some(image) = track.thumbnail.as_deref() {
                                            img { src: "{image}", alt: "" }
                                        } else {
                                            Icon { name: "list" }
                                        }
                                    } else {
                                        Icon { name: "list" }
                                    }
                                }
                                span {
                                    strong { "{playlist.name}" }
                                    small { "Playlist · {playlist.tracks.len()} songs" }
                                }
                            }
                        }
                        for (name, tracks) in home_albums.iter().take(6) {
                            button {
                                class: "home-collection-card",
                                key: "album-{name}",
                                onclick: {
                                    let album_name = name.clone();
                                    move |_| {
                                        app_state.album_detail_name.set(Some(album_name.clone()));
                                        app_state.page.set(Page::AlbumDetail);
                                    }
                                },
                                span { class: "mini-art",
                                    if let Some(image) = tracks.first().and_then(|track| track.thumbnail.as_deref()) {
                                        img { src: "{image}", alt: "" }
                                    } else {
                                        Icon { name: "disc" }
                                    }
                                }
                                span {
                                    strong { "{name}" }
                                    small { "Album · {tracks.len()} songs" }
                                }
                            }
                        }
                    }
                }
            }
            if !recently_added.is_empty() {
                section { class: "home-section",
                    div { class: "section-head",
                        div { h2 { "Recently added" } }
                        button {
                            class: "icon-button recent-added-library-link",
                            aria_label: "Open local library",
                            title: "Open local library",
                            onclick: move |_| app_state.page.set(Page::Library),
                            Icon { name: "arrow-right" }
                        }
                    }
                    div { class: "recent-added-grid",
                        for track in recently_added.iter().cloned() {
                            {
                                let play_target = track.clone();
                                let queue_target = track.clone();
                                let favorite_target = track.clone();
                                rsx! {
                                    article { class: "recent-added-card", key: "{track.key()}",
                                        button {
                                            class: "recent-added-play",
                                            aria_label: "Play {track.title}",
                                            title: "Play {track.title}",
                                            onclick: move |_| play_track(play_target.clone(), app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                                            span { class: "mini-art",
                                                if let Some(image) = track.thumbnail.as_deref() {
                                                    img { src: "{image}", alt: "" }
                                                } else {
                                                    Icon { name: "disc" }
                                                }
                                            }
                                            span { class: "recent-added-copy",
                                                strong { "{track.title}" }
                                                small { "{track.artist} · {track.source.as_deref().unwrap_or(\"Local\")}" }
                                            }
                                        }
                                        button {
                                            class: "recent-added-queue",
                                            aria_label: "Add {track.title} to queue",
                                            title: "Add to queue",
                                            onclick: move |_| add_to_queue(queue_target.clone(), app_state.queue, app_state.notice),
                                            Icon { name: "list" }
                                        }
                                        button {
                                            class: if (app_state.favorites)().iter().any(|favorite| favorite.key() == track.key()) { "recent-added-like active" } else { "recent-added-like" },
                                            aria_label: "Toggle favorite for {track.title}",
                                            title: "Add to favorites",
                                            onclick: move |_| toggle_favorite(favorite_target.clone(), app_state.favorites, app_state.notice),
                                            Icon { name: "favorite" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            section { class: "home-stats",
                    div { class: "section-head home-stats-heading",
                        div { class: "home-stats-title",
                            h2 { "Listening Stats" }
                            if show_home_demo {
                                span { class: "home-demo-badge", "Sample data" }
                            }
                        }
                        label { class: "stats-period-label",
                            CustomSelect {
                                value: (app_state.stats_period)(),
                                options: vec![
                                    ("week".to_string(), "1 week".to_string()),
                                    ("month".to_string(), "1 month".to_string()),
                                    ("3months".to_string(), "3 months".to_string()),
                                    ("6months".to_string(), "6 months".to_string()),
                                    ("year".to_string(), "1 year".to_string()),
                                    ("all".to_string(), "All time".to_string()),
                                ],
                                aria_label: "Listening stats period",
                                class_name: "stats-period-select",
                                on_change: move |value: String| app_state.stats_period.set(value)
                            }
                        }
                    }
                    div { class: "stats-grid",
                        section { class: "stat-panel",
                            h3 { "Top artists" }
                            if show_home_demo {
                                for (index, (artist, count)) in HOME_DEMO_ARTISTS.iter().enumerate() {
                                    div { class: "stat-row", key: "{artist}",
                                        span { "{index + 1:02}" }
                                        strong { "{artist}" }
                                        b { "{count}" }
                                    }
                                }
                            } else if artist_stats.is_empty() {
                                p { class: "settings-empty", "Play music to build your stats." }
                            } else {
                                for (index, (artist, count)) in artist_stats.iter().enumerate() {
                                    div { class: "stat-row", key: "{artist}",
                                        span { "{index + 1:02}" }
                                        strong { "{artist}" }
                                        b { "{count}" }
                                    }
                                }
                            }
                        }
                        section { class: "stat-panel",
                            h3 { "Top tracks" }
                            if show_home_demo {
                                for (index, (title, count)) in HOME_DEMO_TRACKS.iter().enumerate() {
                                    div { class: "stat-row", key: "{title}",
                                        span { "{index + 1:02}" }
                                        strong { "{title}" }
                                        b { "{count}" }
                                    }
                                }
                            } else if song_stats.is_empty() {
                                p { class: "settings-empty", "Your most-played tracks will appear here." }
                            } else {
                                for (index, (title, _, count)) in song_stats.iter().enumerate() {
                                    div { class: "stat-row", key: "{title}-{index}",
                                        span { "{index + 1:02}" }
                                        strong { "{title}" }
                                        b { "{count}" }
                                    }
                                }
                            }
                        }
                    }
                }
        },
        _ => rsx! {},
    }
}
