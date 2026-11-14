use super::super::AppState;
use super::PageData;
use crate::components::TrackRow;
use crate::*;
use dioxus::prelude::*;

#[component]
pub(super) fn AlbumDetailPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let album_name = data.album_detail_name.clone();
    let tracks = data.album_detail_tracks.clone();

    rsx! {
        if let Some(name) = album_name {
            header { class: "library-page-heading playlist-detail-heading",
                button {
                    class: "outline-button compact-action",
                    onclick: move |_| app_state.page.set(Page::Library),
                    "Back to library"
                }
                h1 { "{name}" }
                p { "{tracks.len()} tracks · Album" }
                button {
                    class: "solid-button",
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
                    "Play album"
                }
            }
            if tracks.is_empty() {
                p { class: "library-empty", "This album has no available tracks." }
            } else {
                div { class: "track-table",
                    for (index, track) in tracks.iter().cloned().enumerate() {
                        TrackRow {
                            key: "{track.key()}-{index}",
                            track,
                            context_menu_only: true,
                            on_play: move |track| play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                            on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                            on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                            on_download: move |track| download_track(track, app_state.notice),
                            on_context: move |menu| app_state.context_track.set(Some(menu)),
                        }
                    }
                }
            }
        } else {
            header { class: "library-page-heading",
                h1 { "Album not found" }
                button {
                    class: "outline-button compact-action",
                    onclick: move |_| app_state.page.set(Page::Library),
                    "Back to library"
                }
            }
        }
    }
}
