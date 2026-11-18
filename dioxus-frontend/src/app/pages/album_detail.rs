use super::super::AppState;
use super::PageData;
use crate::components::{CollectionDetailHeader, TrackRow};
use crate::*;
use dioxus::prelude::*;

#[component]
pub(super) fn AlbumDetailPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let album_name = data.album_detail_name.clone();
    let spotify_album = (app_state.spotify_album_selected)();
    let tracks = if spotify_album.is_some() {
        (app_state.spotify_album_tracks)()
    } else {
        data.album_detail_tracks.clone()
    };
    let tracks_busy = spotify_album.is_some() && (app_state.spotify_album_tracks_busy)();
    let tracks_error = (app_state.spotify_album_tracks_error)();
    let current_track = (app_state.current)();
    let is_playing = (app_state.playing)();
    let track_count = if tracks_busy {
        spotify_album
            .as_ref()
            .map_or(tracks.len(), |album| album.tracks_total)
    } else {
        tracks.len()
    };
    let kind = if spotify_album.is_some() {
        "Spotify album"
    } else {
        "Album"
    }
    .to_string();

    rsx! {
        if let Some(name) = album_name {
            CollectionDetailHeader {
                title: name,
                kind: kind.clone(),
                track_count,
                tracks: tracks.clone(),
                artist: spotify_album.as_ref().map(|album| album.artist.clone()),
                artwork_url: spotify_album.as_ref().and_then(|album| album.thumbnail.clone()),
                artwork_class: String::new(),
                artwork_icon: None,
                play_label: "Play album".to_string(),
                extra_action_label: None,
                extra_action_disabled: false,
                rename_enabled: false,
                rename_editing: false,
                rename_value: String::new(),
                rename_busy: false,
                rename_notice: None,
                save_action_label: None,
                save_action_disabled: true,
                download_action_label: None,
                download_action_disabled: true,
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
                        app_state.queue_panel_open,
                        app_state.room_panel_open,
                    )
                },
                on_extra_action: move |_| {},
                on_download_action: move |_| {},
                on_rename_start: move |_| {},
                on_rename_change: move |_| {},
                on_rename_save: move |_| {},
                on_rename_cancel: move |_| {},
                on_save_action: move |_| {},
            }
            if tracks_busy {
                p { class: "library-empty", "Loading Spotify album tracks…" }
            } else if let Some(error) = tracks_error.filter(|_| spotify_album.is_some()) {
                div { class: "collection-detail-load-error", role: "alert",
                    p { "{error}" }
                    if let Some(album) = spotify_album.clone() {
                        button {
                            class: "outline-button compact-action",
                            onclick: move |_| load_spotify_album(
                                album.clone(),
                                app_state.spotify_album_selected,
                                app_state.spotify_album_tracks,
                                app_state.spotify_album_tracks_busy,
                                app_state.spotify_album_tracks_error,
                                app_state.spotify_detail_load_id,
                                app_state.album_detail_name,
                                app_state.page,
                                app_state.notice,
                            ),
                            "Try again"
                        }
                    }
                }
            } else if tracks.is_empty() {
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
                            on_download: move |track| download_track(track, (app_state.download_format)(), app_state.notice),
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
