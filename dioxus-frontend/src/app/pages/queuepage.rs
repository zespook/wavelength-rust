use super::super::AppState;
use super::PageData;
use crate::components::{Icon, TrackRow};
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn QueuePage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let can_manage_queue =
        (app_state.room_role)() != "guest" || (app_state.room_permissions)().can_manage_queue;

    match Page::Queue {
        Page::Queue => rsx! {
            header { class: "library-page-heading", h1 { "Queue" }
                if !(app_state.queue)().is_empty() {
                    button {
                        class: "text-button",
                        disabled: !can_manage_queue,
                        onclick: move |_| clear_queue(app_state.queue, app_state.notice),
                        "Clear queue"
                    }
                }
            }
            if !(app_state.queue_download_error)().is_empty() {
                p { class: "queue-download-error", role: "status",
                    "Download status unavailable: {(app_state.queue_download_error)()}"
                }
            }
            if (app_state.queue)().is_empty() { p { class: "library-empty", "Your queue is empty" } }
            div { class: "queue-page-list",
                for (index, track) in (app_state.queue)().iter().cloned().enumerate() {
                    {
                        let download_status =
                            (app_state.queue_download_status)().get(track.key()).cloned();
                        rsx! {
                            div {
                                class: "queue-page-item",
                                key: "{track.key()}-{index}",
                                onpointerup: move |_| {
                                    if can_manage_queue {
                                        if let Some(source_index) = (app_state.dragging_queue_item)() {
                                            move_queue_item(source_index, index, app_state.queue, app_state.notice);
                                        }
                                    }
                                    app_state.dragging_queue_item.set(None);
                                },
                                onpointercancel: move |_| app_state.dragging_queue_item.set(None),
                                button {
                                    class: "queue-page-reorder queue-drag-handle icon-button",
                                    aria_label: "Reorder {track.title}; use arrow keys to move",
                                    aria_keyshortcuts: "ArrowUp ArrowDown",
                                    title: "Drag to reorder or use arrow keys",
                                    disabled: !can_manage_queue,
                                    onpointerdown: move |event| {
                                        if can_manage_queue {
                                            event.prevent_default();
                                            app_state.dragging_queue_item.set(Some(index));
                                        }
                                    },
                                    onkeydown: move |event| {
                                        match event.key().to_string().as_str() {
                                            "ArrowUp" if index > 0 => {
                                                event.prevent_default();
                                                move_queue_item(index, index - 1, app_state.queue, app_state.notice);
                                            }
                                            "ArrowDown" if index + 1 < (app_state.queue)().len() => {
                                                event.prevent_default();
                                                move_queue_item(index, index + 1, app_state.queue, app_state.notice);
                                            }
                                            _ => {}
                                        }
                                    },
                                    Icon { name: "grip" }
                                }
                                TrackRow {
                                    track: track.clone(),
                                    download_status,
                                    on_play: move |track| play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                                    on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                                    on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                                    on_download: move |track| download_track(track, app_state.notice),
                                    on_context: move |menu| app_state.context_track.set(Some(menu)),
                                }
                                button {
                                    class: "text-button",
                                    disabled: !can_manage_queue,
                                    aria_label: "Remove {track.title} from queue",
                                    onclick: move |_| remove_queue_item(index, app_state.queue, app_state.notice),
                                    "Remove"
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
