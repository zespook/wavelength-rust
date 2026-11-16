use super::super::AppState;
use super::PageData;
use crate::components::{CollectionDetailHeader, TrackRow};
use crate::*;
use dioxus::prelude::*;

#[component]
pub(super) fn AlbumDetailPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let album_name = data.album_detail_name.clone();
    let tracks = data.album_detail_tracks.clone();
    let current_track = (app_state.current)();
    let is_playing = (app_state.playing)();

    rsx! {
        if let Some(name) = album_name {
            CollectionDetailHeader {
                title: name,
                kind: "Album".to_string(),
                track_count: tracks.len(),
                tracks: tracks.clone(),
                artist: None,
                artwork_class: String::new(),
                back_label: "Back to Library".to_string(),
                play_label: "Play album".to_string(),
                extra_action_label: None,
                on_back: move |_| app_state.page.set(Page::Playlists),
                on_play: {
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
                on_extra_action: move |_| {},
            }
            if tracks.is_empty() {
                p { class: "library-empty", "This album has no available tracks" }
            } else {
                section { class: "collection-detail-track-section",
                    div { class: "collection-detail-track-heading",
                        h2 { "Tracks" }
                    }
                    div { class: "track-table collection-detail-track-list",
                    for (index, track) in tracks.iter().cloned().enumerate() {
                        TrackRow {
                            key: "{track.key()}-{index}",
                            track: track.clone(),
                            context_menu_only: true,
                            is_current: is_playing && current_track.as_ref().is_some_and(|current| current.key() == track.key()),
                            on_play: move |track| play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                            on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                            on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                            on_download: move |track| download_track(track, app_state.notice),
                            on_context: move |menu| app_state.context_track.set(Some(menu)),
                        }
                    }
                    }
                }
            }
        } else {
            header { class: "library-page-heading",
                h1 { "Album not found" }
            }
        }
    }
}
