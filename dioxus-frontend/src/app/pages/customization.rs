use super::super::AppState;
use super::PageData;
use crate::browser;
use crate::components::{CustomSelect, Icon};
use crate::*;
use dioxus::prelude::*;

#[allow(unused_mut, unused_variables)]
#[component]
pub(super) fn CustomizationPage(data: std::rc::Rc<PageData>) -> Element {
    let mut app_state = use_context::<AppState>();
    let background_color_value = data.background_color_value.clone();
    let background_picker_color = data.background_picker_color.clone();
    match Page::Customization {
        Page::Customization => {
            let preset_theme = (app_state.effective_appearance_theme)();
            let preset_background = (app_state.background_color)();
            let selected_preset = (app_state.accent_preset)();
            let background_value_for_picker = background_color_value.clone();
            let preset_colors = if (app_state.accent_picker_open)() {
                ACCENT_PRESETS
                    .iter()
                    .map(|(name, base_color)| {
                        (
                            *name,
                            *base_color,
                            contrast_adjusted_accent(base_color, &preset_theme, &preset_background),
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            rsx! {
                header { class: "library-page-heading", h1 { "Customization" } }
                section { class: "customization-panel",
                    label { class: "settings-row", "Theme"
                        CustomSelect {
                            value: (app_state.theme)(),
                            options: vec![
                                ("dark".to_string(), "Dark".to_string()),
                                ("light".to_string(), "Light".to_string()),
                                ("system".to_string(), "System".to_string()),
                            ],
                            aria_label: "Theme",
                            class_name: "theme-select",
                            on_change: move |value: String| {
                                app_state.theme.set(value.clone());
                                save_string("wavelength-theme", &value, app_state.notice);
                            }
                        }
                    }
                    div { class: "customization-background-control",
                        div { class: "settings-row-copy",
                            strong { "Background color" }
                            small { "Choose a custom page background." }
                        }
                        div { class: "background-color-actions",
                            div { class: "accent-current",
                                span { class: "accent-current-swatch", style: "background: {background_color_value}" }
                                code { "{background_color_value.to_uppercase()}" }
                            }
                            button {
                                class: "outline-button compact-action",
                                r#type: "button",
                                onclick: {
                                    let initial_background = background_value_for_picker.clone();
                                    move |_| {
                                        app_state.background_hex_input.set(initial_background.clone());
                                        app_state.background_hex_error.set(String::new());
                                        app_state.background_picker_open.set(true);
                                    }
                                },
                                "Change"
                            }
                            if !(app_state.background_color)().is_empty() {
                                button {
                                    class: "outline-button compact-action",
                                    r#type: "button",
                                    onclick: move |_| {
                                        app_state.background_color.set(String::new());
                                        app_state.background_hex_error.set(String::new());
                                        app_state.background_hex_input.set(
                                            default_background_color(&(app_state.effective_appearance_theme)()).to_string()
                                        );
                                        save_string("wavelength-background-color", "", app_state.notice);
                                    },
                                    "Default"
                                }
                            }
                        }
                    }
                    if (app_state.background_picker_open)() {
                        div { class: "modal-backdrop", role: "presentation",
                            div { class: "modal color-picker-modal", role: "dialog", aria_modal: "true", aria_labelledby: "background-picker-heading",
                                button {
                                    class: "modal-close",
                                    aria_label: "Close background color picker",
                                    r#type: "button",
                                    onclick: move |_| app_state.background_picker_open.set(false),
                                    Icon { name: "close" }
                                }
                                p { class: "eyebrow", "Customization" }
                                h2 { id: "background-picker-heading", "Background color" }
                                p { "Choose a color with the picker or enter a six-digit hex value." }
                                label { class: "background-picker-control",
                                    span { "Color" }
                                    input {
                                        id: "background-color-picker-input",
                                        class: "background-color-picker",
                                        r#type: "color",
                                        value: "{background_picker_color}",
                                        aria_label: "Background color",
                                        oninput: move |event| {
                                            app_state.background_hex_input.set(event.value());
                                            app_state.background_hex_error.set(String::new());
                                        }
                                    }
                                }
                                form {
                                    class: "custom-color-form",
                                    onsubmit: move |event| {
                                        event.prevent_default();
                                        let Some(value) = normalize_hex_color(&(app_state.background_hex_input)()) else {
                                            app_state.background_hex_error.set("Enter a six-digit hex color, such as #0a0a0a.".to_string());
                                            return;
                                        };
                                        app_state.background_hex_error.set(String::new());
                                        app_state.background_hex_input.set(value.clone());
                                        app_state.background_color.set(value.clone());
                                        save_string("wavelength-background-color", &value, app_state.notice);
                                        app_state.background_picker_open.set(false);
                                    },
                                    label { r#for: "background-hex-input", "Hex color" }
                                    div {
                                        input {
                                            id: "background-hex-input",
                                            r#type: "text",
                                            value: "{(app_state.background_hex_input)()}",
                                            maxlength: "7",
                                            placeholder: "#0a0a0a",
                                            aria_label: "Background color hex",
                                            aria_invalid: !(app_state.background_hex_error)().is_empty(),
                                            aria_describedby: "background-hex-error",
                                            oninput: move |event| {
                                                app_state.background_hex_input.set(event.value());
                                                app_state.background_hex_error.set(String::new());
                                            }
                                        }
                                        button { class: "outline-button compact-action", r#type: "submit", "Apply" }
                                    }
                                    if !(app_state.background_hex_error)().is_empty() {
                                        p { id: "background-hex-error", role: "alert", "{(app_state.background_hex_error)()}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "settings-row",
                        div { class: "settings-row-copy",
                            strong { "Accent color" }
                            small { "Choose a preset or enter a hex color." }
                        }
                        div { class: "accent-actions",
                            div { class: "accent-current",
                                span { class: "accent-current-swatch", style: "background: {(app_state.accent)()}" }
                                code { "{(app_state.accent)().to_uppercase()}" }
                            }
                            button {
                                class: "outline-button compact-action",
                                r#type: "button",
                                onclick: move |_| {
                                    app_state.accent_hex_input.set((app_state.accent)().to_uppercase());
                                    app_state.accent_hex_error.set(String::new());
                                    app_state.accent_picker_open.set(true);
                                },
                                "Change"
                            }
                        }
                    }
                    if (app_state.accent_picker_open)() {
                        div { class: "modal-backdrop", role: "presentation",
                            div { class: "modal color-picker-modal", role: "dialog", aria_modal: "true", aria_labelledby: "accent-picker-heading",
                                button {
                                    class: "modal-close",
                                    aria_label: "Close accent picker",
                                    r#type: "button",
                                    onclick: move |_| app_state.accent_picker_open.set(false),
                                    Icon { name: "close" }
                                }
                                p { class: "eyebrow", "Customization" }
                                h2 { id: "accent-picker-heading", "Accent color" }
                                div { class: "accent-picker-list", role: "group", aria_label: "Accent color presets",
                                    for (name, base_color, preview_color) in preset_colors {
                                        button {
                                            class: if selected_preset == name { "active" } else { "" },
                                            r#type: "button",
                                            aria_label: "{name} accent",
                                            title: "{name}",
                                            aria_pressed: selected_preset == name,
                                            onclick: move |_| {
                                                let color = contrast_adjusted_accent(
                                                    base_color,
                                                    &(app_state.effective_appearance_theme)(),
                                                    &(app_state.background_color)(),
                                                );
                                                app_state.accent_preset.set(name.to_string());
                                                save_string("wavelength-accent-preset", name, app_state.notice);
                                                app_state.accent.set(color.clone());
                                                app_state.accent_hex_input.set(color.to_uppercase());
                                                app_state.accent_hex_error.set(String::new());
                                                save_string("wavelength-accent", &color, app_state.notice);
                                                app_state.accent_picker_open.set(false);
                                            },
                                            span { class: "accent-option-swatch", style: "background: {preview_color}" }
                                        }
                                    }
                                }
                                form {
                                    class: "custom-color-form",
                                    onsubmit: move |event| {
                                        event.prevent_default();
                                        let Some(value) = normalize_hex_color(&(app_state.accent_hex_input)()) else {
                                            app_state.accent_hex_error.set("Enter a six-digit hex color, such as #D7C7A4.".to_string());
                                            return;
                                        };
                                        let resolved_theme = match browser::effective_theme(&(app_state.theme)()) {
                                            Ok(value) => value,
                                            Err(error) => { app_state.notice.set(error); return; }
                                        };
                                        if !accent_color_has_contrast(&value, &resolved_theme, &(app_state.background_color)()) {
                                            app_state.accent_hex_error.set(format!("{value} does not meet the 4.5:1 contrast requirement for {resolved_theme} mode."));
                                            return;
                                        }
                                        app_state.accent_hex_error.set(String::new());
                                        app_state.accent_preset.set(String::new());
                                        save_string("wavelength-accent-preset", "", app_state.notice);
                                        app_state.accent.set(value.clone());
                                        app_state.accent_hex_input.set(value.to_uppercase());
                                        save_string("wavelength-accent", &value, app_state.notice);
                                        app_state.accent_picker_open.set(false);
                                    },
                                    label { r#for: "accent-hex-input", "Custom hex color" }
                                    div {
                                        input {
                                            id: "accent-hex-input",
                                            r#type: "text",
                                            value: "{(app_state.accent_hex_input)()}",
                                            maxlength: "7",
                                            placeholder: "#D7C7A4",
                                            aria_label: "Accent color hex",
                                            aria_invalid: !(app_state.accent_hex_error)().is_empty(),
                                            aria_describedby: "accent-hex-error",
                                            oninput: move |event| {
                                                app_state.accent_hex_input.set(event.value());
                                                app_state.accent_hex_error.set(String::new());
                                            }
                                        }
                                        button { class: "outline-button compact-action", r#type: "submit", "Apply" }
                                    }
                                    if !(app_state.accent_hex_error)().is_empty() {
                                        p { id: "accent-hex-error", role: "alert", "{(app_state.accent_hex_error)()}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => rsx! {},
    }
}
