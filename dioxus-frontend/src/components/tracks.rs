use super::Icon;
use crate::*;
use dioxus::prelude::*;

#[component]
pub(crate) fn QueueDownloadIndicator(status: DownloadStatus) -> Element {
    let progress = status
        .progress
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.0, 100.0));
    let label = status.error.clone().unwrap_or_else(|| {
        progress
            .map(|value| format!("{} {:.0}%", status.status, value))
            .unwrap_or_else(|| status.status.clone())
    });

    rsx! {
        span {
            class: if status.status == "error" {
                "queue-download-indicator error"
            } else if status.status == "ready" {
                "queue-download-indicator ready"
            } else {
                "queue-download-indicator"
            },
            role: "img",
            aria_label: "{label}",
            title: "{label}",
            span {
                class: "queue-download-ring",
                style: "--download-progress: {progress.unwrap_or(0.0)}%;"
            }
            if status.status == "ready" {
                svg {
                    class: "queue-download-mark",
                    view_box: "0 0 16 16",
                    path { d: "m3.5 8.5 3 3 6-7" }
                }
            } else if status.status == "error" {
                span { class: "queue-download-mark", "!" }
            } else if let Some(progress) = progress {
                span { class: "queue-download-percent", "{progress.round() as u32}%" }
            } else {
                span { class: "queue-download-spinner" }
            }
        }
    }
}

#[component]
pub(crate) fn TrackList(
    tracks: Vec<Track>,
    current: Signal<Option<Track>>,
    audio_src: Signal<String>,
    playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    history: Signal<Vec<Track>>,
    favorites: Signal<Vec<Track>>,
    notice: Signal<String>,
    context_track: Signal<Option<TrackContextMenu>>,
    #[props(default)] context_menu_only: bool,
    #[props(default)] history_actions: bool,
    #[props(default)] download_statuses: Option<std::collections::HashMap<String, DownloadStatus>>,
    #[props(default)] on_download: EventHandler<Track>,
) -> Element {
    rsx! {
        div { class: "track-table",
            for (index, track) in tracks.iter().cloned().enumerate() {
                {
                    let download_status = download_statuses
                        .as_ref()
                        .and_then(|statuses| statuses.get(&track.audio_service_id()).cloned());
                    rsx! {
                        TrackRow {
                            favorite_active: history_actions && favorites().iter().any(|favorite| favorite.key() == track.key()),
                            key: "{track.key()}-{index}",
                            track,
                            download_status,
                            context_menu_only,
                            history_actions,
                            on_play: move |track| play_track(track, current, audio_src, playing, queue, history, notice),
                            on_queue: move |track| add_to_queue(track, queue, notice),
                            on_favorite: move |track| toggle_favorite(track, favorites, notice),
                            on_download: move |track| on_download.call(track),
                            on_context: move |menu| context_track.set(Some(menu)),
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn TrackRow(
    track: Track,
    #[props(default)] context_menu_only: bool,
    #[props(default)] is_current: bool,
    #[props(default)] history_actions: bool,
    #[props(default)] favorite_active: bool,
    #[props(default)] show_add_to_playlist: bool,
    #[props(default)] on_add_to_playlist: EventHandler<Track>,
    #[props(default)] download_status: Option<DownloadStatus>,
    on_play: EventHandler<Track>,
    on_queue: EventHandler<Track>,
    on_favorite: EventHandler<Track>,
    on_download: EventHandler<Track>,
    on_context: EventHandler<TrackContextMenu>,
) -> Element {
    let title = track.title.clone();
    let artist = track.artist.trim();
    let artist = (!artist.is_empty() && !artist.eq_ignore_ascii_case("unknown artist"))
        .then(|| artist.to_string());
    let length = track.duration_label();
    let play_track = track.clone();
    let select_track = track.clone();
    let queue_track = track.clone();
    let favorite_track = track.clone();
    let download_track = track.clone();
    let contextual_track = track.clone();
    let more_track = track.clone();
    let add_track = track.clone();
    let is_local = track.is_local();
    let favorite_aria_label = if history_actions {
        if favorite_active {
            format!("Unlike {title}")
        } else {
            format!("Like {title}")
        }
    } else {
        format!("Favorite {title}")
    };
    let favorite_title = if history_actions {
        if favorite_active {
            "Unlike song"
        } else {
            "Like song"
        }
    } else {
        "Add to favorites"
    };
    let favorite_icon = if history_actions && favorite_active {
        "favorite-filled"
    } else {
        "favorite"
    };
    let row_class = if history_actions {
        "track-row history-track-row"
    } else if context_menu_only {
        "track-row context-menu-only"
    } else {
        "track-row"
    };
    let row_class = if is_current {
        format!("{row_class} now-playing")
    } else {
        row_class.to_string()
    };
    rsx! {
        article { class: "{row_class}",
            oncontextmenu: move |event| {
                event.prevent_default();
                let position = event.client_coordinates();
                on_context.call(TrackContextMenu {
                    track: contextual_track.clone(),
                    x: position.x,
                    y: position.y,
                });
            },
            button { class: "track-select", aria_label: "Play {title}", onclick: move |_| on_play.call(select_track.clone()),
                span { class: "track-name",
                    if is_current {
                        span { class: "track-playing-waveform", role: "img", aria_label: "Now playing",
                            i {}
                            i {}
                            i {}
                        }
                    }
                    span { class: "mini-art",
                        if let Some(image) = track.thumbnail.as_deref() {
                            img { src: "{image}", alt: "" }
                        } else {
                            Icon { name: "disc" }
                        }
                        if let Some(status) = download_status.clone() {
                            QueueDownloadIndicator { status }
                        }
                    }
                    span { class: "track-copy",
                        strong { "{title}" }
                        if let Some(artist) = artist {
                            small { "{artist}" }
                        }
                    }
                }
                span { class: "album-name", "{track.album.as_deref().unwrap_or(\"Single\")}" }
                span { class: "track-length", "{length}" }
            }
            div { class: "track-actions",
                button { class: "row-action play-row-action", aria_label: "Play {title}", title: "Play now", onclick: move |_| on_play.call(play_track.clone()), Icon { name: "play" } }
                if history_actions {
                    button {
                        class: if favorite_active { "row-action active" } else { "row-action" },
                        aria_label: "{favorite_aria_label}",
                        title: "{favorite_title}",
                        onclick: move |_| on_favorite.call(favorite_track.clone()),
                        Icon { name: favorite_icon }
                    }
                } else if !context_menu_only {
                    if show_add_to_playlist {
                        button {
                            class: "row-action",
                            aria_label: "Add {title} to playlist",
                            title: "Add to playlist",
                            onclick: move |_| on_add_to_playlist.call(add_track.clone()),
                            Icon { name: "plus" }
                        }
                    }
                    button { class: "row-action", aria_label: "Add {title} to queue", title: "Add to queue", onclick: move |_| on_queue.call(queue_track.clone()), Icon { name: "list" } }
                    button { class: "row-action", aria_label: "Favorite {title}", title: "Add to favorites", onclick: move |_| on_favorite.call(favorite_track.clone()), Icon { name: "favorite" } }
                    button { class: "row-action", aria_label: "More actions for {title}", title: "More actions", onclick: move |event| {
                        let position = event.client_coordinates();
                        on_context.call(TrackContextMenu {
                            track: more_track.clone(),
                            x: position.x,
                            y: position.y,
                        });
                    }, Icon { name: "more" } }
                    if !is_local {
                        button { class: "row-action", aria_label: "Download {title}", title: "Download", onclick: move |_| on_download.call(download_track.clone()), Icon { name: "download" } }
                    }
                }
            }
        }
    }
}
