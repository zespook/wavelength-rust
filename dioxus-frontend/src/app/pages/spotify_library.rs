use super::super::AppState;
use super::PageData;
use crate::components::TrackList;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn SpotifyLibraryPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();

    match Page::SpotifyLibrary {
        Page::SpotifyLibrary => rsx! {
            header { class: "library-page-heading",
                h1 { "Spotify library" }
                button { class: "outline-button compact-action", disabled: (app_state.spotify_busy)(), onclick: move |_| load_spotify_library(app_state.spotify_tracks, app_state.spotify_busy, app_state.notice), "Refresh" }
            }
            if (app_state.spotify_busy)() { p { class: "library-empty", "Loading saved tracks…" } }
            else if (app_state.spotify_tracks)().is_empty() { p { class: "library-empty", "No saved Spotify tracks loaded yet" } }
            if !(app_state.spotify_tracks)().is_empty() {
                TrackList { tracks: (app_state.spotify_tracks)(), current: app_state.current, audio_src: app_state.audio_src, playing: app_state.playing, queue: app_state.queue, history: app_state.history, favorites: app_state.favorites, notice: app_state.notice, context_track: app_state.context_track, context_menu_only: true }
            }
        },
        _ => rsx! {},
    }
}
