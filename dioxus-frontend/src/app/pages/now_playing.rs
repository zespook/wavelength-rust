use super::super::AppState;
use super::PageData;
use crate::components::Icon;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn NowPlayingPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let can_control_playback = data.can_control_playback.clone();
    let now_playing_previous_source = data.now_playing_previous_source.clone();
    let now_playing_next_source = data.now_playing_next_source.clone();
    let active_track_is_favorite = data.active_track_is_favorite.clone();
    let active_lyric_index = data.active_lyric_index.clone();
    let progress_max = data.progress_max.clone();
    let displayed_playhead = data.displayed_playhead.clone();
    let elapsed_label = data.elapsed_label.clone();
    let duration_label = data.duration_label.clone();
    match Page::NowPlaying {
        Page::NowPlaying => rsx! {
            section { class: "now-playing-page",
                if let Some(track) = (app_state.current)() {
                    section { class: "now-playing-track-section", aria_label: "Now playing",
                        header { class: "now-playing-toolbar",
                            h1 { "Now playing" }
                            div { class: "now-playing-actions",
                                button {
                                    class: if active_track_is_favorite { "player-tool active" } else { "player-tool" },
                                    aria_label: if active_track_is_favorite { "Remove from favorites" } else { "Add to favorites" },
                                    title: if active_track_is_favorite { "Remove from favorites" } else { "Add to favorites" },
                                    onclick: move |_| {
                                        if let Some(track) = (app_state.current)() { toggle_favorite(track, app_state.favorites, app_state.notice); }
                                    },
                                    if active_track_is_favorite {
                                        Icon { name: "favorite-filled" }
                                    } else {
                                        Icon { name: "favorite" }
                                    }
                                }
                                button {
                                    class: "player-tool",
                                    aria_label: "Song actions",
                                    title: "Song actions",
                                    onclick: move |event| {
                                        if let Some(track) = (app_state.current)() {
                                            let position = event.client_coordinates();
                                            app_state.context_track.set(Some(TrackContextMenu {
                                                track,
                                                x: position.x,
                                                y: position.y,
                                            }));
                                        }
                                    },
                                    Icon { name: "more" }
                                }
                                button {
                                    class: "player-tool",
                                    aria_label: "Open queue",
                                    title: "Open queue",
                                    onclick: move |_| {
                                        app_state.room_panel_open.set(false);
                                        app_state.queue_panel_open.set(true);
                                    },
                                    Icon { name: "queue" }
                                }
                            }
                        }
                        div { class: "now-playing-art",
                            span { class: "vinyl-grooves", aria_hidden: "true" }
                            span { class: "vinyl-label",
                                if let Some(art) = track.thumbnail.clone() {
                                    img { src: "{art}", alt: "Artwork for {track.title}" }
                                } else {
                                    Icon { name: "disc" }
                                }
                            }
                            span { class: "vinyl-spindle", aria_hidden: "true" }
                        }
                        div { class: "now-playing-copy",
                            h1 { "{track.title}" }
                            p { "{track.artist}" }
                            small { "{track.album.as_deref().unwrap_or(\"Single\")} · {track.source.as_deref().unwrap_or(\"Audio\")}" }
                        }
                        div { class: "now-playing-progress",
                            input {
                                class: "progress-slider",
                                r#type: "range",
                                min: "0",
                                max: "{progress_max}",
                                step: "0.1",
                                value: "{displayed_playhead}",
                                aria_label: "Track progress",
                                disabled: !can_control_playback,
                                oninput: move |event| {
                                    if let Ok(value) = event.value().parse::<f64>() {
                                        seek_track(value, app_state.playhead, app_state.notice);
                                    }
                                }
                            }
                            div {
                                span { "{elapsed_label}" }
                                span { "{duration_label}" }
                            }
                        }
                        div { class: "now-playing-controls",
                            button {
                                class: if (app_state.shuffle)() { "player-tool active" } else { "player-tool" },
                                aria_label: "Toggle shuffle",
                                title: "Shuffle",
                                aria_pressed: (app_state.shuffle)(),
                                disabled: !can_control_playback,
                                onclick: move |_| toggle_shuffle(app_state.shuffle, app_state.notice),
                                Icon { name: "shuffle" }
                            }
                            button { class: "player-tool", aria_label: "Previous track", disabled: !can_control_playback, onclick: move |_| previous_track(app_state.history, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, now_playing_previous_source.clone(), app_state.notice),
                                Icon { name: "skip-back" }
                            }
                            button { class: "play-main", aria_label: if (app_state.playing)() { "Pause" } else { "Play" }, disabled: !can_control_playback, onclick: move |_| toggle_playback(app_state.playing, app_state.notice),
                                if (app_state.playing)() {
                                    Icon { name: "pause" }
                                } else {
                                    Icon { name: "play" }
                                }
                            }
                            button { class: "player-tool", aria_label: "Next track", disabled: !can_control_playback, onclick: move |_| next_track(app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, now_playing_next_source.clone(), app_state.shuffle, app_state.repeat_mode, app_state.notice),
                                Icon { name: "skip-forward" }
                            }
                            button {
                                class: if (app_state.repeat_mode)() == "off" { "player-tool repeat-control" } else { "player-tool active repeat-control" },
                                aria_label: "Toggle repeat",
                                title: if (app_state.repeat_mode)() == "off" { "Repeat all" } else if (app_state.repeat_mode)() == "all" { "Repeat current song" } else { "Turn repeat off" },
                                aria_pressed: (app_state.repeat_mode)() != "off",
                                disabled: !can_control_playback,
                                onclick: move |_| toggle_repeat(app_state.repeat_mode, app_state.notice),
                                Icon { name: "repeat" }
                                if (app_state.repeat_mode)() == "one" { span { class: "repeat-one-mark", "1" } }
                            }
                        }
                    }
                    section { class: "now-playing-extra-panel lyrics-panel", aria_label: "Lyrics",
                        header { class: "lyrics-heading",
                            h2 { "Lyrics" }
                            button { class: "compact-action", aria_label: "Open fullscreen lyrics", onclick: move |_| app_state.lyrics_fullscreen.set(true),
                                Icon { name: "expand" }
                                "Fullscreen"
                            }
                        }
                        div { class: "lyrics-lines", aria_label: "Lyrics",
                            if (app_state.lyric_loading)() {
                                p { "Loading lyrics…" }
                            } else if (app_state.lyric_lines)().is_empty() {
                                p { "{(app_state.lyrics)()}" }
                            } else {
                                for (index, line) in (app_state.lyric_lines)().iter().enumerate() {
                                    p {
                                        class: if index as isize == active_lyric_index { "active" } else if active_lyric_index >= 0 && (index as isize) < active_lyric_index { "passed" } else { "upcoming" },
                                        "{line.text}"
                                    }
                                }
                            }
                        }
                    }
                    section { class: "now-playing-extra-panel song-info-panel",
                        header { class: "now-playing-extra-heading", h2 { "Song information" } }
                        dl { class: "song-info-list",
                            div { dt { "Artist" } dd { "{track.artist}" } }
                            div { dt { "Album" } dd { "{track.album.as_deref().unwrap_or(\"Single\")}" } }
                            div { dt { "Source" } dd { "{track.source.as_deref().unwrap_or(\"Audio\")}" } }
                            div { dt { "Duration" } dd { "{track.duration_label()}" } }
                        }
                    }
                } else {
                    div { class: "now-playing-copy",
                        h1 { "Nothing playing" }
                        p { "Choose a track from your library or queue." }
                    }
                }
            }
        },
        _ => rsx! {},
    }
}
