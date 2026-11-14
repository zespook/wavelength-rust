use super::super::AppState;
use super::PageData;
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn ActivityPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();

    match Page::Activity {
        Page::Activity => rsx! {
            header { class: "library-page-heading",
                h1 { "Activity" }
                button { class: "outline-button compact-action", disabled: (app_state.activity_loading)(), onclick: move |_| load_activity(app_state.activity_logs, app_state.activity_loading, app_state.activity_error), "Refresh activity" }
            }
            div { class: "activity-tabs", role: "tablist", aria_label: "Activity type",
                button { class: if (app_state.activity_tab)() == "downloads" { "active" } else { "" }, role: "tab", aria_selected: (app_state.activity_tab)() == "downloads", onclick: move |_| app_state.activity_tab.set("downloads".to_string()), "Downloads" }
                button { class: if (app_state.activity_tab)() == "notifications" { "active" } else { "" }, role: "tab", aria_selected: (app_state.activity_tab)() == "notifications", onclick: move |_| app_state.activity_tab.set("notifications".to_string()), "Notifications" }
            }
            if (app_state.activity_tab)() == "downloads" {
                if !(app_state.activity_error)().is_empty() {
                    p { class: "download-logs-error", role: "alert", "{(app_state.activity_error)()}" }
                } else if (app_state.activity_logs)().is_empty() {
                    p { class: "download-log-empty",
                        if (app_state.activity_loading)() { "Loading activity…" } else { "No download activity recorded." }
                    }
                } else {
                    div { class: "download-log-list", role: "table", aria_label: "Download activity",
                        div { class: "download-log-columns", aria_hidden: "true",
                            span { "Time" } span { "Level" } span { "Source" } span { "Event" } span { "Progress" }
                        }
                        for (index, entry) in (app_state.activity_logs)().iter().cloned().enumerate() {
                            article { class: if entry.level == "error" { "download-log-entry log-error" } else if entry.level == "warning" { "download-log-entry log-warning" } else { "download-log-entry" }, key: "{index}",
                                time { "{format_activity_time(&entry.timestamp)}" }
                                span { class: "download-log-level", "{entry.level}" }
                                span { class: "download-log-source", "{entry.source.as_deref().unwrap_or(\"—\")}" }
                                div { class: "download-log-message",
                                    strong { "{entry.message}" }
                                    if let Some(track_id) = entry.track_id { small { "{track_id}" } }
                                }
                                if let Some(progress) = entry.progress {
                                    progress { max: "100", value: "{progress.clamp(0.0, 100.0)}", aria_label: "Download progress {progress:.0}%" }
                                }
                            }
                        }
                    }
                }
            } else if (app_state.notification_logs)().is_empty() {
                p { class: "download-log-empty", "No popup notifications recorded." }
            } else {
                div { class: "notification-log-list", role: "list", aria_label: "Popup notifications",
                    for (index, entry) in (app_state.notification_logs)().iter().cloned().enumerate() {
                        article { class: "notification-log-entry", role: "listitem", key: "{index}",
                            time { "{format_activity_time(&entry.timestamp)}" }
                            p { "{entry.message}" }
                        }
                    }
                }
            }
        },
        _ => rsx! {},
    }
}
