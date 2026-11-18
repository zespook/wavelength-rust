use super::super::AppState;
use super::PageData;
use crate::browser;
use crate::components::{CollectionDetailHeader, Icon, TrackRow};
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn PlaylistDetailPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let mut rename_editing = use_signal(|| false);
    let mut rename_name = use_signal(String::new);
    let favorite_playlist_selected = data.favorite_playlist_selected.clone();
    let playlist_detail_is_local = data.playlist_detail_is_local.clone();
    let playlist_detail = data.playlist_detail.clone();
    let current_track = (app_state.current)();
    let is_playing = (app_state.playing)();
    match Page::PlaylistDetail {
        Page::PlaylistDetail => rsx! {
            if let Some(selected) = playlist_detail.clone() {
                {
                    let can_add_songs = !favorite_playlist_selected
                        && (playlist_detail_is_local || (app_state.room_role)() == "host");
                    let can_rename = playlist_detail_is_local && !favorite_playlist_selected;
                    let extra_action_label = can_add_songs.then(|| "Add songs".to_string());
                    let playlist_id = selected.id.clone();
                    rsx! {
                        CollectionDetailHeader {
                            title: selected.name.clone(),
                            kind: if favorite_playlist_selected {
                                "Favorites".to_string()
                            } else {
                                "Playlist".to_string()
                            },
                            track_count: selected.tracks.len(),
                            tracks: selected.tracks.clone(),
                            artist: None,
                            artwork_url: selected.thumbnail.clone(),
                            artwork_class: if favorite_playlist_selected {
                                "favorite-playlist-cover".to_string()
                            } else {
                                String::new()
                            },
                            artwork_icon: favorite_playlist_selected.then_some("favorite-filled"),
                            play_label: "Play playlist".to_string(),
                            extra_action_label,
                            extra_action_disabled: false,
                            rename_enabled: can_rename,
                            rename_editing: rename_editing(),
                            rename_value: rename_name(),
                            rename_busy: false,
                            rename_notice: can_rename.then(|| {
                                "Name changes apply to your local library only.".to_string()
                            }),
                            save_action_label: None,
                            save_action_disabled: true,
                            download_action_label: Some("Download".to_string()),
                            download_action_disabled: !selected
                                .tracks
                                .iter()
                                .any(|track| !track.is_local()),
                            on_play: {
                                let tracks = selected.tracks.clone();
                                move |_| {
                                    if let Some((first, rest)) = tracks.split_first() {
                                        app_state.queue.set(rest.to_vec());
                                        save_saved("wavelength-queue", &rest.to_vec(), app_state.notice);
                                        play_track(first.clone(), app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice);
                                    }
                                }
                            },
                            on_download_action: {
                                let tracks = selected.tracks.clone();
                                move |_| {
                                    download_tracks(
                                        tracks.clone(),
                                        (app_state.download_format)(),
                                        app_state.notice,
                                    )
                                }
                            },
                            on_extra_action: move |_| {
                                app_state.playlist_add_target.set(Some(playlist_id.clone()));
                                app_state.page.set(Page::Search);
                            },
                            on_rename_start: {
                                let name = selected.name.clone();
                                move |_| {
                                    rename_name.set(name.clone());
                                    rename_editing.set(true);
                                }
                            },
                            on_rename_change: move |name| rename_name.set(name),
                            on_rename_save: {
                                let playlist_id = selected.id.clone();
                                let original_name = selected.name.clone();
                                move |_| {
                                    let name = rename_name().trim().to_string();
                                    if name.is_empty() {
                                        app_state.notice.set("Enter a playlist name.".to_string());
                                        return;
                                    }
                                    if name == original_name {
                                        rename_editing.set(false);
                                        return;
                                    }
                                    let mut all = (app_state.playlists)();
                                    if name.eq_ignore_ascii_case("Favorite Songs")
                                        || all.iter().any(|playlist| {
                                            playlist.id != playlist_id
                                                && playlist.name.eq_ignore_ascii_case(&name)
                                        })
                                    {
                                        app_state.notice.set("A playlist with that name already exists.".to_string());
                                        return;
                                    }
                                    let Some(target) = all.iter_mut().find(|playlist| playlist.id == playlist_id) else {
                                        app_state.notice.set("This playlist no longer exists.".to_string());
                                        return;
                                    };
                                    target.name = name.clone();
                                    match browser::save("wavelength-playlists", &all) {
                                        Ok(()) => {
                                            app_state.playlists.set(all);
                                            rename_editing.set(false);
                                            app_state.notice.set(format!("Renamed playlist to {name}."));
                                        }
                                        Err(error) => app_state.notice.set(error),
                                    }
                                }
                            },
                            on_rename_cancel: {
                                let name = selected.name.clone();
                                move |_| {
                                    rename_name.set(name.clone());
                                    rename_editing.set(false);
                                }
                            },
                            on_save_action: move |_| {},
                        }
                    }
                }
                if selected.tracks.is_empty() {
                    p { class: "library-empty",
                        if favorite_playlist_selected {
                            "Like songs to add them to Favorite Songs"
                        } else if playlist_detail_is_local {
                            "Add tracks from search or the player to build this playlist"
                        } else {
                            "This shared playlist has no tracks"
                        }
                    }
                }
                if !selected.tracks.is_empty() {
                    section { class: "collection-detail-track-section",
                        div { class: "collection-detail-track-heading",
                            h2 { "Tracks" }
                        }
                        div { class: "track-table collection-detail-track-list",
                        for (index, track) in selected.tracks.iter().cloned().enumerate() {
                            div { class: "playlist-track-row", key: "{track.key()}-{index}",
                                TrackRow {
                                    track: track.clone(),
                                    context_menu_only: true,
                                    is_current: is_playing && current_track.as_ref().is_some_and(|current| current.key() == track.key()),
                                    on_play: move |track| play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                                    on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                                    on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                                    on_download: move |track| download_track(track, (app_state.download_format)(), app_state.notice),
                                    on_context: move |menu| app_state.context_track.set(Some(menu)),
                                }
                                if playlist_detail_is_local {
                                    button { class: "row-action playlist-detail-remove", aria_label: "Remove {track.title} from playlist", title: "Remove from playlist", onclick: {
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
                                    }, Icon { name: "close" } }
                                }
                            }
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
