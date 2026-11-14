use super::super::AppState;
use super::PageData;
use crate::components::TrackList;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn HistoryPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();

    match Page::History {
        Page::History => rsx! {
            header { class: "library-page-heading", h1 { "Listening history" }
                button { class: "text-button", onclick: move |_| { app_state.history.set(Vec::new()); save_saved("wavelength-play-history", &Vec::<Track>::new(), app_state.notice); }, "Clear history" }
            }
            TrackList {
                tracks: (app_state.history)(),
                current: app_state.current,
                audio_src: app_state.audio_src,
                playing: app_state.playing,
                queue: app_state.queue,
                history: app_state.history,
                favorites: app_state.favorites,
                notice: app_state.notice,
                context_track: app_state.context_track,
                history_actions: true
            }
        },
        _ => rsx! {},
    }
}
