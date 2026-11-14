use super::super::AppState;
use super::PageData;
use crate::components::EqualizerPage;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn EqualizerRoutePage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();

    match Page::Equalizer {
        Page::Equalizer => rsx! {
            EqualizerPage { notice: app_state.notice }
        },
        _ => rsx! {},
    }
}
