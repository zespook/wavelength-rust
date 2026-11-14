use super::super::AppState;
use super::PageData;
use crate::browser;
use crate::components::{CustomSelect, Icon, TrackRow};
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn LibraryPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let library_folder_rows = data.library_folder_rows.clone();
    let library_groups = data.library_groups.clone();
    match Page::Library {
        Page::Library => rsx! {
            header { class: "library-page-heading",
                h1 { "Local library" }
                button { class: "outline-button compact-action", onclick: move |_| {
                    app_state.folder_manager_open.set(true);
                    spawn(async move {
                        match browser::saved_library_folders().await {
                            Ok(folders) => app_state.saved_folder_names.set(folders),
                            Err(error) => app_state.notice.set(error),
                        }
                    });
                }, "Manage folders" }
            }
            div { class: "library-toolbar",
                label { class: "local-library-search",
                    input { r#type: "search", placeholder: "Filter your library", value: "{(app_state.library_query)()}", oninput: move |event| app_state.library_query.set(event.value()) }
                }
                label { class: "library-sort-control", "Group by"
                    CustomSelect {
                        value: (app_state.library_sort)(),
                        options: vec![
                            ("folders".to_string(), "Folders".to_string()),
                            ("album".to_string(), "Album".to_string()),
                            ("artist".to_string(), "Artist".to_string()),
                        ],
                        aria_label: "Group library by",
                        class_name: "library-sort-select",
                        on_change: move |value: String| {
                            app_state.library_sort.set(value);
                            app_state.collapsed_library_groups.set(Vec::new());
                        }
                    }
                }
            }
            if (app_state.library_sort)() == "folders" {
                if library_folder_rows.is_empty() {
                    p { class: "library-empty", "Choose a music folder to import local tracks." }
                }
                for folder in library_folder_rows.clone() {
                    {
                        let folder_path = folder.path.clone();
                        let folder_name = folder.name.clone();
                        let folder_depth = folder.depth;
                        let folder_count = folder.track_count;
                        let folder_expanded = folder.expanded;
                        let folder_tracks = folder.tracks.clone();
                        let indent = folder_depth * 16;
                        rsx! {
                            section {
                                class: "imported-folder-section",
                                key: "{folder_path}",
                                style: "--folder-indent: {indent}px",
                                button {
                                    class: "imported-folder-heading",
                                    aria_expanded: folder_expanded,
                                    onclick: {
                                        let path = folder_path.clone();
                                        move |_| {
                                            let mut expanded = (app_state.expanded_library_folders)();
                                            if let Some(index) = expanded.iter().position(|item| item == &path) {
                                                expanded.remove(index);
                                            } else {
                                                expanded.push(path.clone());
                                            }
                                            app_state.expanded_library_folders.set(expanded);
                                        }
                                    },
                                    if folder_expanded {
                                        Icon { name: "chevron-down" }
                                    } else {
                                        Icon { name: "chevron-right" }
                                    }
                                    Icon { name: "folder" }
                                    span { class: "imported-folder-name", "{folder_name}" }
                                    span { "{folder_count}" }
                                }
                                if folder_expanded && !folder_tracks.is_empty() {
                                    div { class: "track-table",
                                        for (index, track) in folder_tracks.into_iter().enumerate() {
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
                            }
                        }
                    }
                }
            } else {
                if library_groups.is_empty() { p { class: "library-empty", "Choose a music folder to import local tracks." } }
                for (group_index, (name, tracks)) in library_groups.clone().into_iter().enumerate() {
                    {
                        let group_key = format!("{}:{name}", (app_state.library_sort)());
                        let collapsed = (app_state.collapsed_library_groups)().contains(&group_key);
                        rsx! {
                            section { class: "imported-folder-section", key: "{group_index}",
                                div { class: "library-group-heading-row",
                                    button {
                                        class: "imported-folder-heading",
                                        aria_expanded: !collapsed,
                                        onclick: move |_| {
                                            let mut groups = (app_state.collapsed_library_groups)();
                                            if let Some(index) = groups.iter().position(|group| group == &group_key) {
                                                groups.remove(index);
                                            } else {
                                                groups.push(group_key.clone());
                                            }
                                            app_state.collapsed_library_groups.set(groups);
                                        },
                                        span { class: "imported-folder-name", "{name}" }
                                        span { "{tracks.len()}" }
                                    }
                                    if (app_state.library_sort)() == "album" {
                                        button {
                                            class: "outline-button compact-action album-open-action",
                                            aria_label: "Open album {name}",
                                            onclick: {
                                                let album_name = name.clone();
                                                move |_| {
                                                    app_state.album_detail_name.set(Some(album_name.clone()));
                                                    app_state.page.set(Page::AlbumDetail);
                                                }
                                            },
                                            "Open album"
                                        }
                                    }
                                }
                                if !collapsed {
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
                            }
                        }
                    }
                }
            }
        },
        _ => rsx! {},
    }
}
