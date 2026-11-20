use super::super::{AppState, display_download_folder_path};
use super::PageData;
use crate::browser;
use crate::components::{CustomSelect, Icon};
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn SettingsPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let has_spotify_photo = (app_state.spotify_profile)()
        .as_ref()
        .is_some_and(|profile| profile.images.iter().any(|image| !image.url.is_empty()));
    let font_scale = (app_state.font_scale)();
    let font_scale_percent = (font_scale * 100.0).round() as u32;
    let interface_zoom_percent = ((app_state.interface_zoom)() * 100.0).round() as u32;
    let download_folder_name = (app_state.download_folder_name)();
    let download_folder_display_name = display_download_folder_path(&download_folder_name);
    let download_format = (app_state.download_format)();
    let app_version = format!(
        "Version {} · {}",
        env!("CARGO_PKG_VERSION"),
        if (app_state.window_controls_available)() {
            "Desktop app"
        } else {
            "Web app"
        }
    );

    match Page::Settings {
        Page::Settings => rsx! {
            header { class: "library-page-heading", h1 { "Settings" } }
            div { class: "settings-page settings-dashboard",
                section { class: "settings-section settings-card", aria_label: "Connected services",
                    header { class: "settings-section-heading", h2 { "Connected services" } }
                    div { class: "settings-list",
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Profile picture" }
                                small {
                                    if (app_state.local_avatar)().is_some() {
                                        if has_spotify_photo {
                                            "Using a local photo. You can switch to your Spotify photo."
                                        } else {
                                            "Using a photo saved on this device"
                                        }
                                    } else if has_spotify_photo {
                                        "Choose a local photo or use your Spotify profile photo."
                                    } else {
                                        "Choose a local photo"
                                    }
                                }
                            }
                            div { class: "settings-row-actions",
                                button { class: "outline-button compact-action", onclick: move |_| app_state.profile_picture_open.set(true),
                                    "Set profile picture"
                                }
                            }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Spotify" }
                                small {
                                    if let Some(profile) = (app_state.spotify_profile)() {
                                        "Connected as {profile.display_name.as_deref().filter(|name| !name.is_empty()).unwrap_or(&profile.id)}"
                                    } else {
                                        "Not connected"
                                    }
                                }
                            }
                            div { class: "settings-row-actions",
                                if (app_state.spotify_profile)().is_some() {
                                    button { class: "text-button", onclick: move |_| {
                                        match browser::disconnect_spotify() {
                                            Ok(()) => {
                                                app_state.spotify_profile.set(None);
                                                app_state.spotify_tracks.set(Vec::new());
                                                app_state.spotify_playlists.set(Vec::new());
                                                app_state.spotify_albums.set(Vec::new());
                                                app_state.spotify_playlist_selected.set(None);
                                                app_state.spotify_playlist_tracks.set(Vec::new());
                                                app_state.spotify_album_selected.set(None);
                                                app_state.spotify_album_tracks.set(Vec::new());
                                                app_state.spotify_collections_page_loaded.set(false);
                                                app_state.notice.set("Spotify disconnected.".to_string());
                                            }
                                            Err(error) => app_state.notice.set(error),
                                        }
                                    }, "Disconnect" }
                                } else {
                                    button { class: "outline-button compact-action", onclick: move |_| {
                                        spawn(async move {
                                            match browser::connect_spotify().await {
                                                Ok(()) => match browser::restore_spotify().await {
                                                    Ok(Some(restore)) => {
                                                        app_state.spotify_profile.set(Some(restore.profile));
                                                        app_state.notice.set("Spotify connected.".to_string());
                                                    }
                                                    Ok(None) => app_state.notice.set("Spotify sign-in completed, but the profile could not be restored.".to_string()),
                                                    Err(error) => app_state.notice.set(error),
                                                },
                                                Err(error) => app_state.notice.set(error),
                                            }
                                        });
                                    }, "Connect" }
                                }
                            }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Qobuz" }
                                small { if (app_state.qobuz_configured)() { "Configured for Spotify downloads" } else { "Not configured" } }
                            }
                            button { class: "outline-button compact-action", onclick: move |_| app_state.qobuz_modal_open.set(true), "Set credentials" }
                        }
                    }
                }
                section { class: "settings-section settings-card", aria_label: "Library and downloads",
                    header { class: "settings-section-heading", h2 { "Library and downloads" } }
                    div { class: "settings-list",
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Local folders" }
                                small { "Manage folders saved on this device" }
                            }
                            button { class: "outline-button compact-action", onclick: move |_| {
                                app_state.folder_manager_open.set(true);
                                spawn(async move {
                                    match browser::saved_library_folders().await {
                                        Ok(folders) => app_state.saved_folder_names.set(folders),
                                        Err(error) => app_state.notice.set(error),
                                    }
                                });
                            }, "Manage" }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Download folder" }
                                if download_folder_name.is_empty() {
                                    small { "Choose where downloaded tracks are saved" }
                                } else {
                                    small { class: "download-folder-path", "{download_folder_display_name}" }
                                }
                            }
                            div { class: "settings-row-actions download-folder-actions",
                                div { class: "download-folder-controls",
                                    button {
                                        class: "outline-button compact-action",
                                        onclick: move |_| {
                                            let mut folder_name = app_state.download_folder_name;
                                            let mut notice = app_state.notice;
                                            spawn(async move {
                                                match browser::choose_download_folder().await {
                                                    Ok(Some(name)) => {
                                                        folder_name.set(name);
                                                        notice.set(String::new());
                                                    }
                                                    Ok(None) => {}
                                                    Err(error) => notice.set(error),
                                                }
                                            });
                                        },
                                        if download_folder_name.is_empty() { "Choose folder" } else { "Change" }
                                    }
                                }
                            }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Download format" }
                                small { "Save downloads as FLAC by default, or choose MP3." }
                            }
                            div { class: "settings-row-actions",
                                CustomSelect {
                                    value: download_format,
                                    options: vec![
                                        ("flac".to_string(), "FLAC".to_string()),
                                        ("mp3".to_string(), "MP3".to_string()),
                                        ("wav".to_string(), "WAV".to_string()),
                                        ("m4a".to_string(), "M4A".to_string()),
                                        ("aac".to_string(), "AAC".to_string()),
                                        ("ogg".to_string(), "OGG".to_string()),
                                        ("opus".to_string(), "Opus".to_string()),
                                        ("aiff".to_string(), "AIFF".to_string()),
                                        ("wma".to_string(), "WMA".to_string()),
                                        ("webm".to_string(), "WebM".to_string()),
                                    ],
                                    aria_label: "Download format",
                                    class_name: "download-format-select",
                                    on_change: move |format: String| {
                                        app_state.download_format.set(format.clone());
                                        save_saved("wavelength-download-format", &format, app_state.notice);
                                    }
                                }
                            }
                        }
                    }
                }
                section { class: "settings-section settings-card", aria_label: "Playback and appearance",
                    header { class: "settings-section-heading", h2 { "Playback and appearance" } }
                    div { class: "settings-list",
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Equalizer" }
                                small { "Shape the sound across ten frequency bands" }
                            }
                            button { class: "outline-button compact-action", onclick: move |_| app_state.page.set(Page::Equalizer), "Modify" }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Appearance" }
                                small { "Set a theme and accent color" }
                            }
                            button { class: "outline-button compact-action", onclick: move |_| app_state.page.set(Page::Customization), "Customize" }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                div { class: "settings-row-title",
                                    strong { "Text size" }
                                    span { class: "settings-row-value", aria_live: "polite", "{font_scale_percent}%" }
                                }
                                small { "Adjust text size without changing the layout scale" }
                            }
                            div { class: "settings-row-actions text-scale-controls", role: "group", aria_label: "Text size",
                                button {
                                    class: "outline-button compact-action text-scale-button",
                                    aria_label: "Decrease text size",
                                    title: "Decrease text size",
                                    disabled: font_scale <= 0.8,
                                    onclick: move |_| set_accessibility_font_scale(app_state.font_scale, font_scale - 0.1, app_state.notice),
                                    Icon { name: "minus" }
                                }
                                button {
                                    class: "outline-button compact-action text-scale-button",
                                    aria_label: "Reset text size",
                                    title: "Reset text size",
                                    disabled: font_scale == 1.0,
                                    onclick: move |_| set_accessibility_font_scale(app_state.font_scale, 1.0, app_state.notice),
                                    Icon { name: "refresh" }
                                }
                                button {
                                    class: "outline-button compact-action text-scale-button",
                                    aria_label: "Increase text size",
                                    title: "Increase text size",
                                    disabled: font_scale >= 1.3,
                                    onclick: move |_| set_accessibility_font_scale(app_state.font_scale, font_scale + 0.1, app_state.notice),
                                    Icon { name: "plus" }
                                }
                            }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                div { class: "settings-row-title",
                                    strong { "Interface zoom" }
                                    span { class: "settings-row-value", aria_live: "polite", "{interface_zoom_percent}%" }
                                }
                                small { "Adjust the size of the app interface" }
                            }
                            div { class: "settings-row-actions text-scale-controls", role: "group", aria_label: "Interface zoom",
                                button {
                                    class: "outline-button compact-action text-scale-button",
                                    aria_label: "Decrease interface zoom",
                                    disabled: (app_state.interface_zoom)() <= 0.7,
                                    onclick: move |_| {
                                        let next = (((app_state.interface_zoom)() - 0.1) * 10.0).round() / 10.0;
                                        let next = next.clamp(0.7, 1.5);
                                        app_state.interface_zoom.set(next);
                                        if let Err(error) = browser::set_interface_zoom(next) {
                                            app_state.notice.set(error);
                                        }
                                    },
                                    Icon { name: "minus" }
                                }
                                button {
                                    class: "outline-button compact-action text-scale-button",
                                    aria_label: "Reset interface zoom",
                                    onclick: move |_| {
                                        app_state.interface_zoom.set(1.0);
                                        if let Err(error) = browser::set_interface_zoom(1.0) {
                                            app_state.notice.set(error);
                                        }
                                    },
                                    Icon { name: "refresh" }
                                }
                                button {
                                    class: "outline-button compact-action text-scale-button",
                                    aria_label: "Increase interface zoom",
                                    disabled: (app_state.interface_zoom)() >= 1.5,
                                    onclick: move |_| {
                                        let next = (((app_state.interface_zoom)() + 0.1) * 10.0).round() / 10.0;
                                        let next = next.clamp(0.7, 1.5);
                                        app_state.interface_zoom.set(next);
                                        if let Err(error) = browser::set_interface_zoom(next) {
                                            app_state.notice.set(error);
                                        }
                                    },
                                    Icon { name: "plus" }
                                }
                            }
                        }
                    }
                }
                section { class: "settings-section settings-card", aria_label: "Self-hosted room server",
                    header { class: "settings-section-heading", h2 { "Self-hosted room server" } }
                    div { class: "settings-list",
                        div { class: "settings-row room-server-settings-row",
                            div { class: "room-server-heading",
                                div { class: "settings-row-copy",
                                    strong { "Server address" }
                                    small { "Connect to the Rust room server used for shared listening" }
                                }
                                div { class: "room-server-status",
                                    small { class: if !(app_state.room_access)().is_empty() { "connected" } else { "room-server-state" },
                                        if (app_state.room_connecting)() {
                                            "Connecting…"
                                        } else if !(app_state.room_access)().is_empty() {
                                            "Server connected"
                                        } else if !(app_state.room_url)().is_empty() {
                                            "Passcode required"
                                        } else {
                                            "Not connected"
                                        }
                                    }
                                    if !(app_state.room_access)().is_empty() {
                                        button {
                                            class: "text-button room-server-disconnect",
                                            aria_label: "Disconnect from room server",
                                            disabled: (app_state.room_connecting)() || !(app_state.active_room_code)().is_empty(),
                                            onclick: move |_| disconnect_room(app_state.room_access, app_state.active_room_code, app_state.notice),
                                            "Disconnect"
                                        }
                                    }
                                }
                            }
                            div { class: "room-server-controls",
                                input {
                                    class: "room-server-input",
                                    r#type: "url",
                                    placeholder: "https://music.example.com",
                                    aria_label: "Room server URL",
                                    value: "{(app_state.room_url_input)()}",
                                    oninput: move |event| app_state.room_url_input.set(event.value())
                                }
                                div { class: "room-server-passcode-row",
                                    input {
                                        class: "room-server-input",
                                        r#type: "password",
                                        placeholder: "Server passcode",
                                        aria_label: "Server passcode",
                                        autocomplete: "current-password",
                                        maxlength: "128",
                                        value: "{(app_state.room_passcode)()}",
                                        oninput: move |event| app_state.room_passcode.set(event.value()),
                                        onchange: move |event| {
                                            let value = event.value();
                                            let value = if value.is_empty() { None } else { Some(value.as_str()) };
                                            if let Err(error) = browser::set_room_server_passcode(value) {
                                                app_state.notice.set(error);
                                            }
                                        }
                                    }
                                    button {
                                        class: "solid-button compact-action",
                                        disabled: (app_state.room_connecting)(),
                                        onclick: move |_| connect_room(app_state.room_url_input, app_state.room_url, app_state.room_passcode, app_state.room_access, app_state.room_connecting, app_state.active_room_code, app_state.notice),
                                        if (app_state.room_connecting)() { "Saving…" } else { "Save" }
                                    }
                                }
                            }
                            small { class: "room-server-help", "Saved locally so you can reconnect later" }
                        }
                    }
                }
                section { class: "settings-section settings-card", aria_label: "Activity",
                    header { class: "settings-section-heading", h2 { "Activity" } }
                    div { class: "settings-list",
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Logs" }
                                small { "Review download and notification activity" }
                            }
                            button { class: "outline-button compact-action", onclick: move |_| app_state.page.set(Page::Activity), "View" }
                        }
                    }
                }
                section { class: "settings-section settings-card", aria_label: "About",
                    header { class: "settings-section-heading", h2 { "About" } }
                    div { class: "settings-list",
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Wavelength" }
                                small { "{app_version}" }
                            }
                        }
                        div { class: "settings-row",
                            div { class: "settings-row-copy",
                                strong { "Updates" }
                                small {
                                    if (app_state.window_controls_available)() {
                                        "Check for and install signed desktop updates"
                                    } else {
                                        "Automatic updates are available in the desktop app"
                                    }
                                }
                            }
                            button {
                                class: "outline-button compact-action",
                                disabled: !(app_state.window_controls_available)()
                                    || (app_state.update_checking)()
                                    || (app_state.update_installing)(),
                                onclick: move |_| {
                                    if !(app_state.window_controls_available)() {
                                        return;
                                    }
                                    app_state.update_checking.set(true);
                                    app_state.update_dismissed_version.set(None);
                                    let mut available = app_state.update_available;
                                    let mut checking = app_state.update_checking;
                                    let mut notice = app_state.notice;
                                    spawn(async move {
                                        match browser::check_for_updates().await {
                                            Ok(Some(update)) => {
                                                available.set(Some(update));
                                                notice.set(String::new());
                                            }
                                            Ok(None) => {
                                                available.set(None);
                                                notice.set("Wavelength is up to date.".to_string());
                                            }
                                            Err(error) => notice.set(format!("Update check failed: {error}")),
                                        }
                                        checking.set(false);
                                    });
                                },
                                if (app_state.update_checking)() {
                                    "Checking…"
                                } else if (app_state.window_controls_available)() {
                                    "Check"
                                } else {
                                    "Desktop app only"
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
