use super::super::AppState;
use super::PageData;
use crate::components::TrackRow;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn PlaylistDetailPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let favorite_playlist_selected = data.favorite_playlist_selected.clone();
    let playlist_detail_is_local = data.playlist_detail_is_local.clone();
    let playlist_detail = data.playlist_detail.clone();
    match Page::PlaylistDetail {
        Page::PlaylistDetail => rsx! {
            if let Some(selected) = playlist_detail.clone() {
                header { class: "library-page-heading playlist-detail-heading",
                    button { class: "outline-button compact-action", onclick: move |_| app_state.page.set(Page::Playlists), "Back to playlists" }
                    h1 { "{selected.name}" }
                    p { "{selected.tracks.len()} tracks" }
                    button { class: "solid-button", disabled: selected.tracks.is_empty(), onclick: {
                        let tracks = selected.tracks.clone();
                        move |_| {
                            if let Some((first, rest)) = tracks.split_first() {
                                app_state.queue.set(rest.to_vec());
                                save_saved("wavelength-queue", &rest.to_vec(), app_state.notice);
                                play_track(first.clone(), app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice);
                            }
                        }
                    }, "Play playlist" }
                    if !favorite_playlist_selected && (playlist_detail_is_local || (app_state.room_role)() == "host") {
                        button {
                            class: "outline-button compact-action",
                            onclick: {
                                let playlist_id = selected.id.clone();
                                move |_| {
                                    app_state.playlist_add_target.set(Some(playlist_id.clone()));
                                    app_state.page.set(Page::Search);
                                }
                            },
                            "Add songs"
                        }
                    }
                }
                if selected.tracks.is_empty() {
                    p { class: "library-empty",
                        if favorite_playlist_selected {
                            "Like songs to add them to Favorite Songs."
                        } else if playlist_detail_is_local {
                            "Add tracks from search or the player to build this playlist."
                        } else {
                            "This shared playlist has no tracks."
                        }
                    }
                }
                div { class: "track-table",
                    for (index, track) in selected.tracks.iter().cloned().enumerate() {
                        div { class: "playlist-track-row", key: "{track.key()}-{index}",
                            TrackRow {
                                track: track.clone(),
                                on_play: move |track| play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                                on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                                on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                                on_download: move |track| download_track(track, app_state.notice),
                                on_context: move |menu| app_state.context_track.set(Some(menu)),
                            }
                            if playlist_detail_is_local {
                                button { class: "row-action", aria_label: "Remove {track.title} from playlist", onclick: {
                                    let playlist_id = selected.id.clone();
                                    let track_key = track.key().to_string();
                                    move |_| {
                                        let mut all = (app_state.playlists)();
                                        if let Some(target) = all.iter_mut().find(|item| item.id == playlist_id) {
                                            target.tracks.retain(|item| item.key() != track_key);
                                            app_state.playlists.set(all.clone());
                                            save_saved("wavelength-playlists", &all, app_state.notice);
                                        }
                                    }
                                }, "Remove" }
                            }
                        }
                    }
                }
            } else {
                header { class: "library-page-heading",
                    h1 { "Playlist not found" }
                    button { class: "outline-button compact-action", onclick: move |_| app_state.page.set(Page::Playlists), "Back to playlists" }
                }
            }
        },
        _ => rsx! {},
    }
}
