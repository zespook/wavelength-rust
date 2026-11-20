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
    let artist_stats = data.artist_stats.clone();
    let song_stats = data.song_stats.clone();
    let playlist_count = library_playlist_count(
        &(app_state.playlists)(),
        &(app_state.room_shared_playlists)(),
        &(app_state.spotify_playlists)(),
        (app_state.spotify_profile)().is_some(),
    );
    let playlist_count_label = if playlist_count == 1 {
        "playlist"
    } else {
        "playlists"
    };
    let download_folder_name = (app_state.download_folder_name)();
    match Page::Home {
        Page::Home => rsx! {
            section { class: "home-hero",
                div { class: "home-hero-copy",
                    section { class: "home-greeting",
                        p { class: "eyebrow", "Your library" }
                        h1 { "Your music, within reach" }
                        p { "Search across music services or bring your own collection together" }
                    }
                    if download_folder_name.is_empty() || (app_state.history)().is_empty() {
                        div { class: "home-actions",
                            if download_folder_name.is_empty() {
                                button { class: "outline-button", onclick: move |_| {
                                    let mut folder_name = app_state.download_folder_name;
                                    let mut notice = app_state.notice;
                                    spawn(async move {
                                        match browser::choose_download_folder().await {
                                            Ok(Some(name)) => {
                                                folder_name.set(name);
                                                notice.set(String::new());
                                            }
                                            Ok(None) => {}
                                            Err(error) => notice.set(error),
                                        }
                                    });
                                },
                                    Icon { name: "folder" }
                                    "Set download folder"
                                }
                            }
                            if (app_state.history)().is_empty() {
                                button { class: "solid-button", onclick: move |_| focus_search_bar(app_state.notice),
                                    Icon { name: "search" }
                                    "Find music"
                                }
                            }
                        }
                    }
                    if download_folder_name.is_empty() {
                        p { class: "home-download-disclaimer",
                            "Set a download folder before downloading music. Wavelength does not save downloads without one."
                        }
                    }
                    p { class: "home-library-count",
                        "{(app_state.local_tracks)().len()} local tracks"
                        span { "·" }
                        "{playlist_count} {playlist_count_label}"
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
                            p { "Add a music folder to browse albums and tracks here" }
                        }
                    }
                    if home_featured_track.is_some() {
                        button { class: "home-preview-link", onclick: move |_| app_state.page.set(Page::Library),
                            "Browse local files"
                            Icon { name: "arrow-right" }
                        }
                    }
                }
            }
            section { class: "home-recently-played",
                div { class: "section-head",
                    div { h2 { "Recently played" } }
                }
                if !(app_state.history)().is_empty() {
                    TrackList {
                        tracks: (app_state.history)().into_iter().take(6).collect(),
                        current: app_state.current, audio_src: app_state.audio_src, playing: app_state.playing, queue: app_state.queue, history: app_state.history, favorites: app_state.favorites, notice: app_state.notice, context_track: app_state.context_track,
                        context_menu_only: true
                    }
                } else {
                    p { class: "settings-empty", "Recent plays will appear here" }
                }
            }
            if !recently_added.is_empty() {
                section { class: "home-section",
                    div { class: "section-head",
                        div { h2 { "Recently added" } }
                        button {
                            class: "icon-button recent-added-library-link",
                            aria_label: "Open local files",
                            title: "Open local files",
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
                            h2 { "Listening stats" }
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
                            if artist_stats.is_empty() {
                                p { class: "settings-empty", "Play music to build your stats" }
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
                            if song_stats.is_empty() {
                                p { class: "settings-empty", "Your most-played tracks will appear here" }
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
