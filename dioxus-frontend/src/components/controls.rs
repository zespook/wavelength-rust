use crate::{NotificationLog, UpdateInfo, record_notification};
use dioxus::prelude::*;

#[component]
pub(crate) fn CustomSelect(
    value: String,
    options: Vec<(String, String)>,
    on_change: EventHandler<String>,
    aria_label: String,
    #[props(default)] disabled: bool,
    #[props(default)] class_name: String,
) -> Element {
    let mut open = use_signal(|| false);
    let selected_label = options
        .iter()
        .find(|(option_value, _)| option_value == &value)
        .map(|(_, label)| label.clone())
        .or_else(|| options.first().map(|(_, label)| label.clone()))
        .unwrap_or_default();
    let root_class = if open() {
        format!("select-menu open {class_name}")
    } else {
        format!("select-menu {class_name}")
    };

    rsx! {
        div {
            class: "{root_class}",
            onkeydown: move |event| {
                if event.key().to_string() == "Escape" {
                    open.set(false);
                }
            },
            if open() {
                button {
                    class: "select-menu-dismiss",
                    r#type: "button",
                    aria_label: "Close {aria_label} menu",
                    tabindex: "-1",
                    onclick: move |_| open.set(false)
                }
            }
            button {
                class: "select-menu-trigger",
                r#type: "button",
                aria_label: "{aria_label}",
                aria_haspopup: "listbox",
                aria_expanded: open(),
                disabled,
                onclick: move |_| open.set(!open()),
                span { class: "select-menu-value", "{selected_label}" }
                span { class: "select-menu-chevron", aria_hidden: "true", Icon { name: "chevron-down" } }
            }
            if open() {
                div {
                    class: "select-menu-options",
                    role: "listbox",
                    aria_label: "{aria_label}",
                    for (option_value, option_label) in options.iter() {
                        {
                            let option_value = option_value.clone();
                            let option_label = option_label.clone();
                            let selected = option_value == value;
                            rsx! {
                                button {
                                    class: if selected { "select-menu-option selected" } else { "select-menu-option" },
                                    r#type: "button",
                                    role: "option",
                                    aria_selected: selected,
                                    onclick: move |_| {
                                        on_change.call(option_value.clone());
                                        open.set(false);
                                    },
                                    "{option_label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn AppNotice(
    mut notice: Signal<String>,
    notification_logs: Signal<Vec<NotificationLog>>,
) -> Element {
    use_effect(move || {
        let message = notice();
        if message.is_empty() {
            return;
        }

        record_notification(notification_logs, &message);
        spawn(async move {
            crate::sleep_ms(3800).await;
            if notice() == message {
                notice.set(String::new());
            }
        });
    });

    let message = notice();
    if message.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "app-notice", role: "status", aria_live: "polite",
            span { class: "app-notice-message", "{message}" }
            button {
                class: "icon-button app-notice-dismiss",
                r#type: "button",
                aria_label: "Dismiss notification",
                onclick: move |_| notice.set(String::new()),
                Icon { name: "close" }
            }
        }
    }
}

#[component]
pub(crate) fn UpdateNotice(
    update: UpdateInfo,
    mut available: Signal<Option<UpdateInfo>>,
    mut dismissed_version: Signal<Option<String>>,
    mut installing: Signal<bool>,
    mut notice: Signal<String>,
) -> Element {
    let version = update.version.clone();
    let install_version = version.clone();
    let dismiss_version = version.clone();
    let notes = update.notes.filter(|notes| !notes.trim().is_empty());

    rsx! {
        div { class: "app-notice update-notice", role: "status", aria_live: "polite",
            div { class: "update-notice-copy",
                strong { "Wavelength {version} is available" }
                if let Some(notes) = notes {
                    small { "{notes}" }
                }
            }
            div { class: "update-notice-actions",
                button {
                    class: "solid-button compact-action",
                    disabled: installing(),
                    onclick: move |_| {
                        let version = install_version.clone();
                        installing.set(true);
                        spawn(async move {
                            match crate::browser::install_update(&version).await {
                                Ok(()) => {}
                                Err(error) => {
                                    installing.set(false);
                                    notice.set(format!("Update installation failed: {error}"));
                                }
                            }
                        });
                    },
                    if installing() { "Installing…" } else { "Install update" }
                }
                button {
                    class: "icon-button app-notice-dismiss",
                    r#type: "button",
                    aria_label: "Dismiss update notification",
                    disabled: installing(),
                    onclick: move |_| {
                        dismissed_version.set(Some(dismiss_version.clone()));
                        available.set(None);
                    },
                    Icon { name: "close" }
                }
            }
        }
    }
}

#[component]
pub(crate) fn Icon(name: &'static str) -> Element {
    let path = match name {
        "home" => "M3 10l9-7 9 7M5 9v12h14V9M9 21v-7h6v7",
        "chevron-left" => "m15 18-6-6 6-6",
        "chevron-right" => "m9 18 6-6-6-6",
        "chevron-up" => "m18 15-6-6-6 6",
        "chevron-down" => "m6 9 6 6 6-6",
        "minus" => "M5 12h14",
        "plus" => "M12 5v14M5 12h14",
        "arrow-right" => "M5 12h14m-7-7 7 7-7 7",
        "more" => "M5 12h.01M12 12h.01M19 12h.01",
        "disc" => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Zm0 6a3 3 0 1 0 0 6 3 3 0 0 0 0-6Zm0 3h.01",
        "help" => {
            "M9.09 9a3 3 0 0 1 5.82 1c0 2-3 3-3 3m.09 4h.01M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Z"
        }
        "expand" => {
            "M8 3H5a2 2 0 0 0-2 2v3m13-5h3a2 2 0 0 1 2 2v3M3 16v3a2 2 0 0 0 2 2h3m13-5v3a2 2 0 0 1-2 2h-3"
        }
        "minimize" => "M5 12h14",
        "maximize" => "M4 4h16v16H4z",
        "refresh" => "M20 11a8 8 0 0 0-14.9-3M4 5v4h4m-4 4a8 8 0 0 0 14.9 3M20 19v-4h-4",
        "favorite" | "favorite-filled" => {
            "M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8l1.1 1.1L12 21l7.8-7.5 1.1-1.1a5.5 5.5 0 0 0-.1-7.8Z"
        }
        "pencil" => "M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L8 18l-4 1 1-4Z",
        "folder" => "M3 6h7l2 2h9v11H3zM3 10h18",
        "music" => {
            "M9 18V5l12-2v13M9 18c0 1.7-1.6 3-3.5 3S2 19.7 2 18s1.6-3 3.5-3S9 16.3 9 18Zm12-2c0 1.7-1.6 3-3.5 3S14 17.7 14 16s1.6-3 3.5-3S21 14.3 21 16Z"
        }
        "pause" => "M8 5v14M16 5v14",
        "play" => "M8 5v14l11-7z",
        "search" => "M11 3a8 8 0 1 0 0 16 8 8 0 0 0 0-16Zm10 18-4.3-4.3",
        "library" => "M4 5v14M9 5v14M15 5l4 14",
        "list" => "M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01",
        "grid" => "M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z",
        "queue" => "M4 6h16M4 12h16M4 18h10",
        "shuffle" => "M16 3h5v5M4 20l5.5-5.5M21 3l-7 7M15 14l6 7v-5m0 5h-5M4 4l5 5",
        "repeat" => "M17 1l4 4-4 4M3 11V9a4 4 0 0 1 4-4h14M7 23l-4-4 4-4m14-2v2a4 4 0 0 1-4 4H3",
        "lyrics" => "M9 10V6a3 3 0 1 1 6 0v4a3 3 0 1 1-6 0Zm-4 0a7 7 0 0 0 14 0m-7 7v5m-4 0h8",
        "close" => "M18 6 6 18M6 6l12 12",
        "trash" => "M3 6h18m-2 0-.9 14H5.9L5 6m4 0V4h6v2m-5 4v6m4-6v6",
        "users" => {
            "M16 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2m6-10a4 4 0 1 0 0-8 4 4 0 0 0 0 8Zm6-7.5a4 4 0 0 1 0 7.75M20 21v-2a4 4 0 0 0-3-3.87"
        }
        "skip-back" => "M5 5v14m14-13-9 6 9 6V6Z",
        "skip-forward" => "M19 5v14M5 6l9 6-9 6V6Z",
        "history" => "M3 12a9 9 0 1 0 2.64-6.36L3 8m0-5v5h5m4-1v5l3 2",
        "download" => "M21 15v4H3v-4m4-5 5 5 5-5m-5 5V3",
        "volume" => "M11 5 6 9H3v6h3l5 4V5Zm4 4a5 5 0 0 1 0 6m3-9a9 9 0 0 1 0 12",
        "volume-muted" => "M11 5 6 9H3v6h3l5 4V5Zm5 4 5 6m0-6-5 6",
        "settings" => {
            "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Zm0-6v3m0 14v3m10-10h-3M5 12H2m17.07-7.07-2.12 2.12M7.05 16.95l-2.12 2.12m14.14 0-2.12-2.12M7.05 7.05 4.93 4.93"
        }
        "user" => "M20 21a8 8 0 0 0-16 0M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8",
        "grip" => "M9 5h.01M9 12h.01M9 19h.01M15 5h.01M15 12h.01M15 19h.01",
        _ => panic!("Missing icon mapping for `{name}`"),
    };
    let fill = if matches!(name, "favorite-filled" | "play") {
        "currentColor"
    } else {
        "none"
    };
    rsx! { svg { width: 18, height: 18, view_box: "0 0 24 24", fill: fill, stroke: "currentColor", stroke_width: "1.7", stroke_linecap: "round", stroke_linejoin: "round", path { d: "{path}" } } }
}
