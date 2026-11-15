use super::super::AppState;
use super::PageData;
use crate::browser;
use crate::components::TrackRow;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn SpotifyPlaylistsPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();

    match Page::SpotifyPlaylists {
        Page::SpotifyPlaylists => rsx! {
            header { class: "library-page-heading",
                h1 { "Spotify playlists" }
                button { class: "outline-button compact-action", disabled: (app_state.spotify_playlists_busy)(), onclick: move |_| load_spotify_playlists(app_state.spotify_playlists, app_state.spotify_playlists_busy, app_state.notice), "Refresh" }
            }
            if (app_state.spotify_playlists_busy)() { p { class: "library-empty", "Loading Spotify playlists…" } }
            else if (app_state.spotify_playlists)().is_empty() { p { class: "library-empty", "No Spotify playlists loaded yet." } }
            div { class: "collection-grid",
                for playlist in (app_state.spotify_playlists)() {
                    article { class: "collection-card", key: "{playlist.id}",
                        h2 { "{playlist.name}" }
                        p { "{playlist.tracks_total} tracks" }
                        button { class: "text-button", onclick: {
                            let selected = playlist.clone();
                                move |_| load_spotify_playlist(selected.clone(), app_state.spotify_playlist_tracks, app_state.spotify_playlist_selected, app_state.spotify_playlist_name, app_state.spotify_playlist_tracks_busy, app_state.notice)
                        }, "Open playlist" }
                    }
                }
            }
            if let Some(selected) = (app_state.spotify_playlist_selected)() {
                section { class: "spotify-playlist-tracks",
                    header { class: "library-page-heading",
                        h2 { "{selected.name}" }
                        input { value: "{(app_state.spotify_playlist_name)()}", aria_label: "Spotify playlist name", oninput: move |event| app_state.spotify_playlist_name.set(event.value()) }
                        button { class: "outline-button compact-action", disabled: (app_state.spotify_playlist_save_busy)() || (app_state.spotify_playlist_tracks_busy)(), onclick: move |_| {
                            let id = selected.id.clone();
                            let name = (app_state.spotify_playlist_name)().trim().to_string();
                            let uris = (app_state.spotify_playlist_tracks)().iter().filter_map(spotify_track_uri).collect::<Vec<_>>();
                            if uris.len() != (app_state.spotify_playlist_tracks)().len() {
                                app_state.notice.set("Some tracks are not Spotify tracks and will be omitted.".to_string());
                            }
                            app_state.spotify_playlist_save_busy.set(true);
                            spawn(async move {
                                match browser::save_spotify_playlist(&id, &name, &uris).await {
                                    Ok(()) => {
                                        if let Some(mut item) = (app_state.spotify_playlist_selected)() {
                                            item.name = name;
                                            item.tracks_total = uris.len();
                                            app_state.spotify_playlist_selected.set(Some(item.clone()));
                                            app_state.spotify_playlist_name.set(item.name.clone());
                                            let mut all = (app_state.spotify_playlists)();
                                            if let Some(existing) = all.iter_mut().find(|playlist| playlist.id == item.id) {
                                                *existing = item;
                                                app_state.spotify_playlists.set(all);
                                            }
                                        }
                                        app_state.notice.set("Spotify playlist updated.".to_string());
                                    }
                                    Err(error) => app_state.notice.set(error),
                                }
                                app_state.spotify_playlist_save_busy.set(false);
                            });
                        }, "Save changes to Spotify" }
                        if let Some(track) = (app_state.current)() {
                            button { class: "text-button", disabled: !track.is_spotify(), onclick: move |_| {
                                if let Some(track) = (app_state.current)() {
                                    let mut tracks = (app_state.spotify_playlist_tracks)();
                                    if !tracks.iter().any(|item| item.key() == track.key()) {
                                        tracks.push(track);
                                        app_state.spotify_playlist_tracks.set(tracks);
                                        app_state.notice.set("Track added. Save changes to sync it to Spotify.".to_string());
                                    }
                                }
                            }, "Add current track" }
                        }
                        button { class: "text-button", disabled: (app_state.spotify_playlist_tracks_busy)(), onclick: move |_| {
                            let selected = (app_state.spotify_playlist_selected)();
                            if let Some(source) = selected {
                                let local = Playlist {
                                    id: format!("playlist-{}", crate::now_epoch_millis()),
                                    name: source.name.clone(),
                                    tracks: (app_state.spotify_playlist_tracks)(),
                                };
                                let mut all = (app_state.playlists)();
                                all.push(local);
                                app_state.playlists.set(all.clone());
                                match browser::save("wavelength-playlists", &all) {
                                    Ok(()) => app_state.notice.set(format!("Copied {} tracks to your playlists.", (app_state.spotify_playlist_tracks)().len())),
                                    Err(error) => app_state.notice.set(error),
                                }
                            }
                        }, "Copy to local playlists" }
                    }
                    if (app_state.spotify_playlist_tracks_busy)() {
                        p { class: "library-empty", "Loading playlist tracks…" }
                    }
                    div { class: "track-table",
                        for (index, track) in (app_state.spotify_playlist_tracks)().iter().cloned().enumerate() {
                            TrackRow {
                                key: "{track.key()}-{index}", track,
                                on_play: move |track| play_track(track, app_state.current, app_state.audio_src, app_state.playing, app_state.queue, app_state.history, app_state.notice),
                                on_queue: move |track| add_to_queue(track, app_state.queue, app_state.notice),
                                on_favorite: move |track| toggle_favorite(track, app_state.favorites, app_state.notice),
                                on_download: move |track| download_track(track, app_state.notice),
                                on_context: move |menu| app_state.context_track.set(Some(menu)),
                            }
                            button { class: "text-button", onclick: move |_| {
                                let mut tracks = (app_state.spotify_playlist_tracks)();
                                if index < tracks.len() { tracks.remove(index); app_state.spotify_playlist_tracks.set(tracks); }
                            }, "Remove from playlist" }
                        }
                    }
                }
            }
        },
        _ => rsx! {},
    }
}
