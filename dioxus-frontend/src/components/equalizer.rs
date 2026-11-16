use super::CustomSelect;
use crate::browser;
use crate::components::Icon;
use crate::*;
use dioxus::prelude::*;

#[component]
pub(crate) fn EqualizerPage(notice: Signal<String>) -> Element {
    let bands = [
        60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0, 14000.0, 16000.0,
    ];
    let mut gains = use_signal(|| vec![0.0; 10]);
    let mut frequencies = use_signal(|| bands.to_vec());
    let mut qs = use_signal(|| vec![1.1; 10]);
    let mut types = use_signal(|| vec!["peaking".to_string(); 10]);
    let mut selected_band = use_signal(|| 0usize);
    let mut placement_band = use_signal(|| Option::<usize>::None);
    let mut preset_id = use_signal(|| "flat".to_string());
    let mut custom_presets = use_signal(Vec::<EqualizerPreset>::new);
    let mut preset_name = use_signal(String::new);
    let mut unavailable = use_signal(|| false);

    use_effect(move || {
        let mut errors = Vec::new();
        let loaded_gains =
            load_equalizer_values("wavelength-equalizer-gains", vec![0.0; 10], -18.0, 18.0);
        let loaded_frequencies = load_equalizer_values(
            "wavelength-equalizer-frequencies",
            bands.to_vec(),
            20.0,
            20000.0,
        );
        let loaded_qs = load_equalizer_values("wavelength-equalizer-qs", vec![1.1; 10], 0.3, 10.0);
        let gains_for_preset = loaded_gains.as_ref().ok().cloned();
        let loaded_types = browser::load::<Vec<String>>("wavelength-equalizer-types");
        let loaded_preset = browser::load::<String>("wavelength-equalizer-preset");
        let loaded_custom = browser::load::<Vec<EqualizerPreset>>("wavelength-equalizer-presets");

        match loaded_gains {
            Ok(values) => gains.set(values),
            Err(error) => errors.push(error),
        }
        match loaded_frequencies {
            Ok(values) => frequencies.set(values),
            Err(error) => errors.push(error),
        }
        match loaded_qs {
            Ok(values) => qs.set(values),
            Err(error) => errors.push(error),
        }
        match loaded_types {
            Ok(Some(values))
                if values.len() == 10 && values.iter().all(|value| equalizer_type_valid(value)) =>
            {
                types.set(values);
            }
            Ok(Some(_)) => errors.push("Saved equalizer filter types are invalid.".to_string()),
            Ok(None) => {}
            Err(error) => errors.push(error),
        }
        match loaded_preset {
            Ok(Some(value)) => preset_id.set(value),
            Ok(None) => {
                let initial_gains = gains_for_preset.unwrap_or_else(|| vec![0.0; 10]);
                let matching_preset = ["flat", "bass", "vocal", "treble", "rock"]
                    .into_iter()
                    .find(|name| {
                        equalizer_preset(name)
                            .is_some_and(|values| values.to_vec() == initial_gains)
                    })
                    .unwrap_or("custom");
                preset_id.set(matching_preset.to_string());
            }
            Err(error) => errors.push(error),
        }
        match loaded_custom {
            Ok(Some(values)) => custom_presets.set(values),
            Ok(None) => {}
            Err(error) => errors.push(error),
        }

        let state = EqualizerSettings {
            gains: gains(),
            frequencies: frequencies(),
            qs: qs(),
            types: types(),
        };
        if let Err(error) = browser::set_equalizer(&state) {
            unavailable.set(true);
            errors.push(error);
        } else if let Err(error) =
            browser::start_equalizer_visualization("equalizer-response-canvas", 0)
        {
            unavailable.set(true);
            errors.push(error);
        }
        if !errors.is_empty() {
            notice.set(errors.join(" "));
        }
    });

    use_effect(move || {
        if let Err(error) = browser::set_equalizer_selected_band(selected_band() as u32) {
            notice.set(error);
        }
    });

    let band_index = selected_band().min(9);
    let selected_frequency = frequencies()
        .get(band_index)
        .copied()
        .unwrap_or(bands[band_index]);
    let selected_gain = gains().get(band_index).copied().unwrap_or(0.0);
    let frequency_input_value = selected_frequency.round().to_string();
    let gain_input_value = format!("{selected_gain:.1}");
    let selected_q = qs().get(band_index).copied().unwrap_or(1.1);
    let selected_type = types()
        .get(band_index)
        .cloned()
        .unwrap_or_else(|| "peaking".to_string());
    let mut preset_options = vec![
        ("flat".to_string(), "Flat".to_string()),
        ("bass".to_string(), "Bass boost".to_string()),
        ("vocal".to_string(), "Vocal".to_string()),
        ("treble".to_string(), "Treble boost".to_string()),
        ("rock".to_string(), "Rock".to_string()),
    ];
    preset_options.extend(
        custom_presets()
            .iter()
            .map(|preset| (preset.id.clone(), preset.name.clone())),
    );
    preset_options.push(("custom".to_string(), "Custom".to_string()));

    rsx! {
        section { class: "equalizer-page",
            header { class: "equalizer-page-heading",
                h1 { "Equalizer" }
                p { "Shape playback across ten parametric bands" }
            }
            div { class: "equalizer-workbench",
                div { class: "equalizer-toolbar",
                    div { class: "equalizer-preset-section",
                        label { class: "equalizer-preset-control",
                            span { "Preset" }
                            CustomSelect {
                                value: preset_id(),
                                options: preset_options,
                                class_name: "form-select",
                                aria_label: "Equalizer preset",
                                disabled: unavailable(),
                                on_change: move |id| {
                            placement_band.set(None);
                            if let Err(error) = browser::clear_equalizer_preview() {
                                notice.set(error);
                            }
                            if id == "custom" { preset_id.set(id); return; }
                            let preset = custom_presets().into_iter()
                                .find(|item| item.id == id)
                                .or_else(|| equalizer_builtin_preset(&id));
                            if let Some(preset) = preset {
                                let next_gains = normalize_eq_values(&preset.gains, &vec![0.0; 10], -18.0, 18.0);
                                let next_frequencies = normalize_eq_values(&preset.frequencies, &bands, 20.0, 20000.0);
                                let next_qs = normalize_eq_values(&preset.qs, &vec![1.1; 10], 0.3, 10.0);
                                let next_types = normalize_eq_types(&preset.types);
                                gains.set(next_gains);
                                frequencies.set(next_frequencies);
                                qs.set(next_qs);
                                types.set(next_types);
                                preset_id.set(id);
                                persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                            }
                                    }
                                }
                            }
                            form { class: "equalizer-save-form", onsubmit: move |event| {
                                event.prevent_default();
                                let name = preset_name().trim().to_string();
                                if name.is_empty() {
                                    notice.set("Enter a name for this equalizer preset.".to_string());
                                    return;
                                }
                                let preset = EqualizerPreset {
                                    id: format!("custom-{}", crate::now_epoch_millis()),
                                    name: name.clone(),
                                    gains: gains(),
                                    frequencies: frequencies(),
                                    qs: qs(),
                                    types: types(),
                                };
                                let mut all = custom_presets();
                                all.push(preset.clone());
                                match browser::save("wavelength-equalizer-presets", &all) {
                                    Ok(()) => {
                                        custom_presets.set(all);
                                        preset_id.set(preset.id.clone());
                                        persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset.id, notice);
                                        preset_name.set(String::new());
                                        notice.set(format!("{name} preset saved."));
                                    }
                                    Err(error) => notice.set(error),
                                }
                            },
                                input {
                                    value: "{preset_name()}",
                                    placeholder: "Preset name",
                                    aria_label: "Preset name",
                                    disabled: unavailable(),
                                    oninput: move |event| preset_name.set(event.value())
                                }
                                button {
                                    class: "outline-button compact-action",
                                    r#type: "submit",
                                    disabled: unavailable(),
                                    "Save preset"
                                }
                            }
                        }
                        div { class: "equalizer-toolbar-actions",
                            button { class: "outline-button compact-action", disabled: unavailable(), onclick: move |_| {
                            placement_band.set(None);
                            if let Err(error) = browser::clear_equalizer_preview() {
                                notice.set(error);
                            }
                            gains.set(vec![0.0; 10]);
                            frequencies.set(bands.to_vec());
                            qs.set(vec![1.1; 10]);
                            types.set(vec!["peaking".to_string(); 10]);
                            preset_id.set("flat".to_string());
                            persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                            unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                                            }, "Reset" }
                                        }
                                    }
                div { class: "equalizer-graph-panel",
                    canvas {
                        id: "equalizer-response-canvas",
                        class: "equalizer-spectrum-canvas",
                        aria_label: "Equalizer response and live spectrum. Click a band node, then click the graph to place it.",
                        onclick: move |event| {
                            if unavailable() { return; }
                            let point = event.element_coordinates();
                            spawn(async move {
                                let (width, height) = match browser::equalizer_canvas_size().await {
                                    Ok(size) => size,
                                    Err(error) => { notice.set(error); return; }
                                };
                                if let Some(index) = placement_band() {
                                    set_equalizer_graph_point(
                                        index, point.x, point.y, width, height,
                                        gains, frequencies, qs, types, preset_id, unavailable, notice,
                                    );
                                    persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                    placement_band.set(None);
                                    if let Err(error) = browser::clear_equalizer_preview() {
                                        notice.set(error);
                                    }
                                } else if let Some(index) = equalizer_band_hit_test(
                                    point.x, point.y, width, height, &frequencies(), &gains(),
                                ) {
                                    selected_band.set(index);
                                    placement_band.set(Some(index));
                                    if let Err(error) = browser::set_equalizer_preview(index as u32, point.x, point.y) {
                                        notice.set(error);
                                    }
                                } else {
                                    placement_band.set(None);
                                    if let Err(error) = browser::clear_equalizer_preview() {
                                        notice.set(error);
                                    }
                                }
                            });
                        },
                        onpointermove: move |event| {
                            let Some(index) = placement_band() else { return; };
                            let point = event.element_coordinates();
                            if let Err(error) = browser::set_equalizer_preview(index as u32, point.x, point.y) {
                                notice.set(error);
                            }
                        }
                    }
                }
                p { class: "equalizer-graph-hint",
                    if let Some(index) = placement_band() {
                        "Band {index + 1} selected. Move the pointer and click to place it."
                    } else {
                        "Click a band dot, then click its new position."
                    }
                }
                if unavailable() {
                    p { class: "equalizer-unavailable", "The browser audio engine could not initialize the equalizer." }
                }
                div { class: "equalizer-band-strip", role: "tablist", aria_label: "Equalizer bands",
                    for (index, frequency) in bands.iter().enumerate() {
                        button {
                            class: if selected_band() == index { "equalizer-band-tab selected" } else { "equalizer-band-tab" },
                            role: "tab", aria_selected: selected_band() == index,
                            onclick: move |_| {
                                selected_band.set(index);
                                placement_band.set(None);
                                if let Err(error) = browser::clear_equalizer_preview() {
                                    notice.set(error);
                                }
                            },
                            span { class: "equalizer-band-number", "Band {index + 1}" }
                            strong { "{format_frequency(frequencies().get(index).copied().unwrap_or(*frequency))}" }
                            small { "{gains().get(index).copied().unwrap_or(0.0):+.1} dB" }
                        }
                    }
                }
                div { class: "equalizer-band-editor",
                    header { class: "equalizer-band-editor-heading",
                        div { class: "equalizer-band-title",
                            h2 { "Band {band_index + 1}" }
                            small { "{format_frequency(selected_frequency)}" }
                        }
                        button { class: "text-button equalizer-band-reset", disabled: unavailable(), onclick: move |_| {
                            placement_band.set(None);
                            if let Err(error) = browser::clear_equalizer_preview() {
                                notice.set(error);
                            }
                            let mut next_gains = gains();
                            let mut next_frequencies = frequencies();
                            let mut next_qs = qs();
                            let mut next_types = types();
                            next_gains[band_index] = 0.0;
                            next_frequencies[band_index] = bands[band_index];
                            next_qs[band_index] = 1.1;
                            next_types[band_index] = "peaking".to_string();
                            gains.set(next_gains);
                            frequencies.set(next_frequencies);
                            qs.set(next_qs);
                            types.set(next_types);
                            preset_id.set("custom".to_string());
                            persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                            unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                        }, "Reset band" }
                    }
                    label { class: "equalizer-parameter",
                        span { "Frequency" }
                        input { r#type: "number", min: "20", max: "20000", step: "1",
                            value: "{frequency_input_value}",
                            aria_label: "Band {band_index + 1} frequency in hertz",
                            disabled: unavailable(),
                            oninput: move |event| if let Some(value) = equalizer_frequency_from_input(&event.value()) {
                                let mut next = frequencies();
                                next[band_index] = value;
                                frequencies.set(next);
                                preset_id.set("custom".to_string());
                                persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                            }
                        }
                        input { r#type: "range", min: "0", max: "1000", step: "1",
                            value: "{equalizer_frequency_slider(selected_frequency)}",
                            aria_label: "Band {band_index + 1} frequency",
                            disabled: unavailable(),
                            oninput: move |event| if let Ok(value) = event.value().parse::<f64>() {
                                let mut next = frequencies();
                                next[band_index] = equalizer_frequency_from_slider(value);
                                frequencies.set(next);
                                preset_id.set("custom".to_string());
                                persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                            }
                        }
                    }
                    label { class: "equalizer-parameter",
                        span { "Gain" }
                        input { r#type: "number", min: "-18", max: "18", step: "0.1",
                            value: "{gain_input_value}",
                            aria_label: "Band {band_index + 1} gain in decibels",
                            disabled: unavailable(),
                            oninput: move |event| if let Some(value) = equalizer_gain_from_input(&event.value()) {
                                let mut next = gains();
                                next[band_index] = value;
                                gains.set(next);
                                preset_id.set("custom".to_string());
                                persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                            }
                        }
                        input { r#type: "range", min: "-18", max: "18", step: "0.1",
                            value: "{selected_gain}", aria_label: "Band {band_index + 1} gain", disabled: unavailable(),
                            oninput: move |event| if let Ok(value) = event.value().parse::<f64>() {
                                let mut next = gains();
                                next[band_index] = value;
                                gains.set(next);
                                preset_id.set("custom".to_string());
                                persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                            }
                        }
                    }
                    div { class: "equalizer-parameter",
                        div { class: "equalizer-parameter-heading",
                            span { "Q" }
                            button {
                                class: "equalizer-parameter-help-trigger",
                                r#type: "button",
                                aria_label: "Q factor help",
                                aria_describedby: "equalizer-q-tooltip-{band_index}",
                                title: "Q controls band width: higher values narrow the effect; lower values broaden it.",
                                Icon { name: "help" }
                                span {
                                    id: "equalizer-q-tooltip-{band_index}",
                                    class: "equalizer-parameter-tooltip",
                                    role: "tooltip",
                                    "Q controls band width: higher values narrow the effect; lower values broaden it."
                                }
                            }
                        }
                        output { "{selected_q:.1}" }
                        input { r#type: "range", min: "0.3", max: "10", step: "0.1",
                            value: "{selected_q}", aria_label: "Band {band_index + 1} Q factor", disabled: unavailable(),
                            oninput: move |event| if let Ok(value) = event.value().parse::<f64>() {
                                let mut next = qs();
                                next[band_index] = value;
                                qs.set(next);
                                preset_id.set("custom".to_string());
                                persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                            }
                        }
                    }
                    label { class: "equalizer-type-control",
                        span { "Shape" }
                        CustomSelect {
                            class_name: "form-select",
                            value: selected_type,
                            options: vec![
                                ("peaking".to_string(), "Bell".to_string()),
                                ("lowshelf".to_string(), "Low shelf".to_string()),
                                ("highshelf".to_string(), "High shelf".to_string()),
                            ],
                            aria_label: format!("Band {} filter shape", band_index + 1),
                            disabled: unavailable(),
                            on_change: move |value: String| {
                                if !equalizer_type_valid(&value) { return; }
                                let mut next = types();
                                next[band_index] = value;
                                types.set(next);
                                preset_id.set("custom".to_string());
                                persist_equalizer(&gains(), &frequencies(), &qs(), &types(), &preset_id(), notice);
                                unavailable.set(set_equalizer_state(&gains(), &frequencies(), &qs(), &types(), notice).is_err());
                            }
                        }
                    }
                }
            }
        }
    }
}
