use super::super::AppState;
use super::PageData;
use crate::components::TrackRow;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn SearchPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let local_results =
        search_local_tracks(&(app_state.local_tracks)(), &(app_state.search_query)());
    let online_results = (app_state.results)();
    let busy = (app_state.busy)();
    let playlist_add_target = (app_state.playlist_add_target)();
    let can_add_to_playlist = playlist_add_target.as_ref().is_some_and(|playlist_id| {
        (app_state.playlists)()
            .iter()
            .any(|playlist| playlist.id == *playlist_id)
            || ((app_state.room_role)() == "host"
                && (app_state.room_shared_playlists)()
                    .iter()
                    .any(|playlist| playlist.id == *playlist_id))
    });

    match Page::Search {
        Page::Search => rsx! {
            header { class: "library-page-heading", h1 { "Search" } }
            if busy && local_results.is_empty() && online_results.is_empty() {
                p { class: "library-empty", "Searching…" }
            } else if !busy && local_results.is_empty() && online_results.is_empty() {
                p { class: "library-empty", "Search Spotify or YouTube to find something to play" }
            }
            for (local_section, tracks) in [
                (true, local_results.clone()),
                (false, online_results.clone()),
            ] {
                if !tracks.is_empty() {
                    if local_section || !local_results.is_empty() {
                        h2 { class: "search-result-section-title",
                            if local_section { "Local files" } else { "Online" }
                        }
                    }
                    div { class: "track-table",
                        for (index, track) in tracks.into_iter().enumerate() {
                            {
                                let playlist_id = playlist_add_target.clone();
                                let use_local_copy = !local_section;
                                rsx! {
                                    TrackRow {
                                        key: "{track.key()}-{index}", track,
                                        context_menu_only: false,
                                        search_result_actions: true,
                                        show_add_to_playlist: can_add_to_playlist,
                                        on_add_to_playlist: move |track| {
                                            if let Some(playlist_id) = playlist_id.as_deref() {
                                                add_track_to_playlist(
                                                    track,
                                                    playlist_id,
                                                    app_state.playlists,
                                                    app_state.room_shared_playlists,
                                                    app_state.room_role,
                                                    app_state.local_tracks,
                                                    app_state.favorites,
                                                    app_state.history,
                                                    app_state.queue,
                                                    app_state.notice,
                                                );
                                                app_state.playlist_add_target.set(None);
                                            }
                                        },
                                        on_play: move |track| {
                                            let track = if use_local_copy && (app_state.room_code)().is_empty() {
                                                preferred_local_track(track, &(app_state.local_tracks)())
                                            } else {
                                                track
                                            };
                                            play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice);
                                        },
                                        on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                                        on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                                        on_download: move |track| download_track(track, (app_state.download_format)(), app_state.notice),
                                        on_context: move |menu| app_state.context_track.set(Some(menu)),
                                    }
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
