use crate::components::Icon;
use crate::*;
use dioxus::prelude::*;

#[component]
pub(crate) fn CollectionDetailHeader(
    title: String,
    kind: String,
    tracks: Vec<Track>,
    track_count: usize,
    artist: Option<String>,
    artwork_url: Option<String>,
    artwork_class: String,
    artwork_icon: Option<&'static str>,
    play_label: String,
    extra_action_label: Option<String>,
    extra_action_disabled: bool,
    rename_enabled: bool,
    rename_editing: bool,
    rename_value: String,
    rename_busy: bool,
    rename_notice: Option<String>,
    save_action_label: Option<String>,
    save_action_disabled: bool,
    download_action_label: Option<String>,
    download_action_disabled: bool,
    on_play: EventHandler<()>,
    on_extra_action: EventHandler<()>,
    on_download_action: EventHandler<()>,
    on_rename_start: EventHandler<()>,
    on_rename_change: EventHandler<String>,
    on_rename_save: EventHandler<()>,
    on_rename_cancel: EventHandler<()>,
    on_save_action: EventHandler<()>,
) -> Element {
    let artwork = artwork_url
        .as_deref()
        .filter(|url| !url.trim().is_empty())
        .or_else(|| {
            tracks.iter().find_map(|track| {
                track
                    .thumbnail
                    .as_deref()
                    .filter(|url| !url.trim().is_empty())
            })
        });
    let artist = artist
        .filter(|artist| {
            let artist = artist.trim();
            !artist.is_empty() && !artist.eq_ignore_ascii_case("unknown artist")
        })
        .or_else(|| collection_artist(&tracks));
    let duration = collection_total_time(&tracks);
    let fallback_icon = artwork_icon.unwrap_or(if kind == "Playlist" { "list" } else { "disc" });
    let artwork_alt = format!("Artwork for {title}");
    let track_count_label = format!(
        "{} {}",
        track_count,
        if track_count == 1 { "track" } else { "tracks" }
    );
    let no_tracks = tracks.is_empty();
    let playlist_kind = matches!(kind.as_str(), "Playlist" | "Spotify playlist" | "Favorites");

    rsx! {
        section { class: "collection-detail-hero",
            div { class: "collection-detail-main",
                div { class: "collection-detail-cover {artwork_class}",
                    if let Some(image) = artwork {
                        img { src: "{image}", alt: "{artwork_alt}" }
                    } else {
                        div { class: "collection-detail-art-fallback", aria_hidden: "true",
                            Icon { name: fallback_icon }
                        }
                    }
                }
                div { class: "collection-detail-copy",
                    if !playlist_kind {
                        p { class: "collection-detail-type", "{kind}" }
                    }
                    div { class: "collection-detail-title-row",
                        if rename_enabled && rename_editing {
                            form {
                                class: "collection-detail-title-edit",
                                onsubmit: move |event| {
                                    event.prevent_default();
                                    on_rename_save.call(());
                                },
                                input {
                                    value: "{rename_value}",
                                    maxlength: "100",
                                    aria_label: "Playlist name",
                                    autofocus: true,
                                    disabled: rename_busy,
                                    oninput: move |event| on_rename_change.call(event.value())
                                }
                                button {
                                    class: "text-button",
                                    r#type: "submit",
                                    disabled: rename_busy || rename_value.trim().is_empty(),
                                    "Save"
                                }
                                button {
                                    class: "text-button",
                                    r#type: "button",
                                    disabled: rename_busy,
                                    onclick: move |_| on_rename_cancel.call(()),
                                    "Cancel"
                                }
                            }
                        } else {
                            h1 { class: "collection-detail-title", "{title}" }
                            if rename_enabled {
                                button {
                                    class: "collection-detail-title-edit-button",
                                    aria_label: "Edit playlist name",
                                    title: "Edit playlist name",
                                    disabled: rename_busy,
                                    onclick: move |_| on_rename_start.call(()),
                                    Icon { name: "pencil" }
                                }
                            }
                        }
                    }
                    if rename_enabled && rename_editing {
                        if let Some(notice) = rename_notice {
                            p { class: "collection-detail-rename-notice", "{notice}" }
                        }
                    }
                    if let Some(artist) = artist {
                        p { class: "collection-detail-artist", "{artist}" }
                    }
                    div { class: "collection-detail-meta",
                        span { "{track_count_label}" }
                        if let Some(duration) = duration {
                            span { class: "collection-detail-meta-separator", "·" }
                            span { "Total time {duration}" }
                        }
                        if playlist_kind {
                            span { class: "collection-detail-meta-separator", "·" }
                            span { class: "collection-detail-meta-type", "{kind}" }
                        }
                    }
                    div { class: "collection-detail-actions",
                        button {
                            class: "icon-button collection-detail-play collection-detail-play-icon",
                            disabled: no_tracks,
                            aria_label: "{play_label}",
                            title: "{play_label}",
                            onclick: move |_| on_play.call(()),
                            Icon { name: "play" }
                        }
                        if let Some(label) = extra_action_label {
                            button {
                                class: "outline-button compact-action",
                                disabled: extra_action_disabled,
                                onclick: move |_| on_extra_action.call(()),
                                "{label}"
                            }
                        }
                        if let Some(label) = download_action_label {
                            button {
                                class: "outline-button compact-action",
                                disabled: download_action_disabled,
                                aria_label: "Download playlist tracks",
                                onclick: move |_| on_download_action.call(()),
                                Icon { name: "download" }
                                "{label}"
                            }
                        }
                        if let Some(label) = save_action_label {
                            button {
                                class: "outline-button compact-action",
                                disabled: save_action_disabled,
                                onclick: move |_| on_save_action.call(()),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn collection_artist(tracks: &[Track]) -> Option<String> {
    let mut artists = tracks
        .iter()
        .map(|track| track.artist.trim())
        .filter(|artist| !artist.is_empty() && !artist.eq_ignore_ascii_case("unknown artist"));
    let first = artists.next()?;

    Some(if artists.all(|artist| artist == first) {
        first.to_string()
    } else {
        "Various artists".to_string()
    })
}

fn collection_total_time(tracks: &[Track]) -> Option<String> {
    if tracks.is_empty() {
        return None;
    }

    let total_seconds = tracks.iter().try_fold(0_u64, |total, track| {
        total.checked_add(track_duration_seconds(track)?)
    })?;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    Some(if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{}:{seconds:02}", total_seconds / 60)
    })
}

fn track_duration_seconds(track: &Track) -> Option<u64> {
    let numeric_duration = track
        .duration
        .filter(|duration| duration.is_finite() && *duration >= 0.0)
        .and_then(|duration| (duration <= u64::MAX as f64).then_some(duration as u64));
    if let Some(duration) = numeric_duration {
        return Some(duration);
    }

    parse_duration_label(track.length.as_deref()?)
}

fn parse_duration_label(label: &str) -> Option<u64> {
    let parts = label.trim().split(':').collect::<Vec<_>>();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }

    parts
        .iter()
        .enumerate()
        .try_fold(0_u64, |total, (index, part)| {
            let value = part.parse::<u64>().ok()?;
            if index > 0 && value >= 60 {
                return None;
            }
            total.checked_mul(60)?.checked_add(value)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_duration_sums_numeric_and_labeled_track_lengths() {
        let tracks = vec![
            Track {
                duration: Some(65.0),
                ..Track::default()
            },
            Track {
                length: Some("1:15".to_string()),
                ..Track::default()
            },
        ];

        assert_eq!(collection_total_time(&tracks).as_deref(), Some("2:20"));
    }

    #[test]
    fn collection_duration_is_omitted_when_a_track_length_is_unknown() {
        let tracks = vec![
            Track {
                duration: Some(65.0),
                ..Track::default()
            },
            Track::default(),
        ];

        assert_eq!(collection_total_time(&tracks), None);
    }

    #[test]
    fn collection_artist_ignores_placeholder_artist_names() {
        let tracks = vec![Track {
            artist: "Unknown artist".to_string(),
            ..Track::default()
        }];

        assert_eq!(collection_artist(&tracks), None);
    }
}
