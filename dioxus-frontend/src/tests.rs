use super::{
    ACCENT_PRESETS, DownloadStatusResponse, Playlist, Track, accent_color_has_contrast,
    accessible_accent_color, contrast_adjusted_accent, default_background_color,
    equalizer_band_hit_test, equalizer_frequency_from_input, equalizer_frequency_from_slider,
    equalizer_frequency_slider, equalizer_gain_from_input, equalizer_graph_values,
    group_local_tracks, home_statistics, local_folder_rows, merge_local_tracks,
    nearest_equalizer_band, next_repeat_mode, normalize_hex_color, normalize_room_server_url,
    parse_synced_lyrics, playlist_matches, queue_source_label, reordered_queue,
    resolve_room_playlist, room_playlist_references, select_track_from_source,
    selected_favorite_tracks,
};

#[test]
fn default_background_colors_match_dark_and_light_themes() {
    assert_eq!(default_background_color("dark"), "#0a0a0a");
    assert_eq!(default_background_color("light"), "#fff5d2");
    assert_eq!(default_background_color("system"), "#0a0a0a");
}

#[test]
fn accent_colors_must_have_readable_contrast_on_theme_surfaces() {
    assert!(accent_color_has_contrast("#d7c7a4", "dark", ""));
    assert!(!accent_color_has_contrast("#0a0a0a", "dark", ""));
    assert!(!accent_color_has_contrast("#d7c7a4", "light", ""));
    assert!(accent_color_has_contrast("#4f621e", "light", ""));
}

#[test]
fn fallback_accent_stays_readable_on_custom_backgrounds() {
    for background in ["#fff5d2", "#0a0a0a", "#39798c"] {
        let accent = accessible_accent_color("dark", background);
        assert!(accent_color_has_contrast(&accent, "dark", background));
    }
}

#[test]
fn accent_presets_are_adjusted_for_each_theme_and_custom_background() {
    for (name, base_color) in ACCENT_PRESETS {
        for theme in ["dark", "light"] {
            let adjusted = contrast_adjusted_accent(base_color, theme, "");
            assert!(
                accent_color_has_contrast(&adjusted, theme, ""),
                "{name} did not meet contrast in {theme} mode"
            );
        }

        for background in ["#fff5d2", "#0a0a0a", "#39798c", "#808080"] {
            let adjusted = contrast_adjusted_accent(base_color, "dark", background);
            assert!(
                accent_color_has_contrast(&adjusted, "dark", background),
                "{name} did not meet contrast for background {background}"
            );
        }
    }

    assert_eq!(contrast_adjusted_accent("#d7c7a4", "dark", ""), "#d7c7a4");
}

#[test]
fn hex_color_input_accepts_optional_hash_and_requires_six_digits() {
    assert_eq!(
        normalize_hex_color(" #Ab12ef "),
        Some("#ab12ef".to_string())
    );
    assert_eq!(normalize_hex_color("#abc"), None);
    assert_eq!(normalize_hex_color("#12gh56"), None);
}

#[test]
fn formats_track_duration_from_seconds() {
    let track = Track {
        duration: Some(125.0),
        ..Track::default()
    };
    assert_eq!(track.duration_label(), "2:05");
}

#[test]
fn repeat_mode_cycles_through_all_three_states() {
    assert_eq!(next_repeat_mode("off"), "all");
    assert_eq!(next_repeat_mode("all"), "one");
    assert_eq!(next_repeat_mode("one"), "off");
    assert_eq!(next_repeat_mode("invalid"), "off");
}

#[test]
fn player_track_selection_wraps_and_shuffle_skips_current_track() {
    let tracks = ["first", "second", "third"].map(|id| Track {
        id: id.to_string(),
        ..Track::default()
    });
    assert_eq!(
        select_track_from_source(&tracks, Some(&tracks[0]), 1, false, 0.0)
            .unwrap()
            .key(),
        "second"
    );
    assert_eq!(
        select_track_from_source(&tracks, Some(&tracks[2]), 1, false, 0.0)
            .unwrap()
            .key(),
        "first"
    );
    assert_eq!(
        select_track_from_source(&tracks, Some(&tracks[0]), -1, false, 0.0)
            .unwrap()
            .key(),
        "third"
    );
    assert_eq!(
        select_track_from_source(&tracks, Some(&tracks[1]), 1, true, 0.99)
            .unwrap()
            .key(),
        "third"
    );
}

#[test]
fn prefers_source_duration_label_and_rejects_invalid_seconds() {
    let labeled = Track {
        length: Some("4:02".to_string()),
        duration: Some(125.0),
        ..Track::default()
    };
    assert_eq!(labeled.duration_label(), "4:02");

    let invalid = Track {
        duration: Some(f64::NAN),
        ..Track::default()
    };
    assert_eq!(invalid.duration_label(), "—");
}

#[test]
fn recognizes_local_tracks_from_source_or_folder_metadata() {
    let source = Track {
        source: Some("local".to_string()),
        ..Track::default()
    };
    let folder = Track {
        folder_path: Some("Music".to_string()),
        ..Track::default()
    };
    assert!(source.is_local());
    assert!(folder.is_local());
    assert!(!Track::default().is_local());
}

#[test]
fn replacing_an_imported_folder_keeps_other_folders() {
    let existing = vec![
        Track {
            id: "old-a".to_string(),
            folder_path: Some("Music/old".to_string()),
            ..Track::default()
        },
        Track {
            id: "keep".to_string(),
            folder_path: Some("Audiobooks".to_string()),
            ..Track::default()
        },
    ];
    let incoming = vec![Track {
        id: "new-a".to_string(),
        folder_path: Some("Music/new".to_string()),
        ..Track::default()
    }];

    let merged = merge_local_tracks(existing, incoming);
    let ids = merged.into_iter().map(|track| track.id).collect::<Vec<_>>();
    assert_eq!(ids, ["keep", "new-a"]);
}

#[test]
fn groups_local_tracks_and_sorts_songs_within_groups() {
    let tracks = vec![
        Track {
            id: "two".into(),
            title: "Zebra".into(),
            album: Some("Album B".into()),
            ..Track::default()
        },
        Track {
            id: "one".into(),
            title: "Alpha".into(),
            album: Some("Album A".into()),
            ..Track::default()
        },
        Track {
            id: "three".into(),
            title: "Beta".into(),
            album: Some("Album A".into()),
            ..Track::default()
        },
    ];

    let groups = group_local_tracks(tracks, "album");
    assert_eq!(
        groups
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["Album A", "Album B"]
    );
    assert_eq!(
        groups[0]
            .1
            .iter()
            .map(|track| track.title.as_str())
            .collect::<Vec<_>>(),
        ["Alpha", "Beta"]
    );
}

#[test]
fn local_folder_rows_nest_children_and_count_descendant_tracks() {
    let tracks = vec![
        Track {
            id: "root-track".into(),
            title: "Root track".into(),
            folder_path: Some("Music".into()),
            ..Track::default()
        },
        Track {
            id: "nested-track".into(),
            title: "Nested track".into(),
            folder_path: Some("Music/Albums".into()),
            ..Track::default()
        },
        Track {
            id: "other-root".into(),
            folder_path: Some("Podcasts".into()),
            ..Track::default()
        },
    ];

    let expanded = local_folder_rows(&tracks, &["Music".into()]);
    assert_eq!(
        expanded
            .iter()
            .map(|folder| folder.path.as_str())
            .collect::<Vec<_>>(),
        ["Music", "Music/Albums", "Podcasts"]
    );
    assert_eq!(expanded[0].track_count, 2);
    assert_eq!(expanded[0].tracks.len(), 1);
    assert!(expanded[0].expanded);
    assert_eq!(expanded[1].depth, 1);
    assert_eq!(expanded[1].track_count, 1);
    assert!(!expanded[1].expanded);

    let collapsed = local_folder_rows(&tracks, &[]);
    assert_eq!(
        collapsed
            .iter()
            .map(|folder| folder.path.as_str())
            .collect::<Vec<_>>(),
        ["Music", "Podcasts"]
    );
}

#[test]
fn equalizer_presets_and_clock_format_are_stable() {
    assert_eq!(super::equalizer_preset("flat"), Some([0.0; 10]));
    assert_eq!(super::equalizer_preset("unknown"), None);
    assert_eq!(super::format_clock(125.0), "2:05");
    assert_eq!(super::format_clock(f64::NAN), "0:00");
    assert_eq!(equalizer_frequency_slider(20.0), 0);
    assert_eq!(equalizer_frequency_slider(20_000.0), 1000);
    assert_eq!(equalizer_frequency_from_slider(0.0), 20.0);
    assert_eq!(equalizer_frequency_from_slider(1000.0), 20_000.0);
    assert!(super::equalizer_type_valid("lowshelf"));
    assert!(!super::equalizer_type_valid("not-a-filter"));
}

#[test]
fn equalizer_numeric_inputs_are_bounded_and_reject_non_finite_values() {
    assert_eq!(equalizer_frequency_from_input("200"), Some(200.0));
    assert_eq!(equalizer_frequency_from_input("10"), Some(20.0));
    assert_eq!(equalizer_frequency_from_input("25000"), Some(20_000.0));
    assert_eq!(equalizer_frequency_from_input("NaN"), None);
    assert_eq!(equalizer_gain_from_input("-4.26"), Some(-4.3));
    assert_eq!(equalizer_gain_from_input("-22"), Some(-18.0));
    assert_eq!(equalizer_gain_from_input("20"), Some(18.0));
    assert_eq!(equalizer_gain_from_input("inf"), None);
}

#[test]
fn equalizer_graph_maps_pointer_edges_to_valid_frequency_and_gain() {
    assert_eq!(
        equalizer_graph_values(44.0, 16.0, 600.0, 300.0),
        (20.0, 18.0)
    );
    assert_eq!(
        equalizer_graph_values(586.0, 270.0, 600.0, 300.0),
        (20_000.0, -18.0)
    );
    assert_eq!(
        nearest_equalizer_band(44.0, 143.0, 600.0, 300.0, &[20.0, 20_000.0], &[0.0, 0.0],),
        0
    );
}

#[test]
fn equalizer_graph_only_selects_nodes_inside_the_hit_radius() {
    let frequencies = [
        60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0, 14000.0, 16000.0,
    ];
    let gains = [0.0; 10];
    let x = super::equalizer_frequency_to_x(1000.0, 600.0);
    let y = super::equalizer_gain_to_y(0.0, 300.0);

    assert_eq!(
        equalizer_band_hit_test(x, y, 600.0, 300.0, &frequencies, &gains),
        Some(4)
    );
    assert_eq!(
        equalizer_band_hit_test(x + 20.0, y + 20.0, 600.0, 300.0, &frequencies, &gains),
        None
    );
}

#[test]
fn room_server_urls_are_restricted_to_safe_origins() {
    assert_eq!(
        normalize_room_server_url(" https://music.example.com/ "),
        Ok("https://music.example.com".to_string())
    );
    assert_eq!(
        normalize_room_server_url("http://127.0.0.1:8787"),
        Ok("http://127.0.0.1:8787".to_string())
    );
    assert!(normalize_room_server_url("http://music.example.com").is_err());
    assert!(normalize_room_server_url("https://music.example.com/api").is_err());
    assert!(normalize_room_server_url("https://user:pass@music.example.com").is_err());
}

#[test]
fn shared_room_playlists_preserve_platform_ids_and_resolve_known_tracks() {
    let playlist = Playlist {
        id: "playlist-1".into(),
        name: "Room mix".into(),
        tracks: vec![
            Track {
                id: "spotify:track:abc123".into(),
                title: "Known track".into(),
                source: Some("spotify".into()),
                ..Track::default()
            },
            Track {
                id: "yt-video456".into(),
                video_id: Some("video456".into()),
                title: "Video track".into(),
                source: Some("youtube".into()),
                ..Track::default()
            },
        ],
    };
    let references = room_playlist_references(&[playlist]);
    assert_eq!(references[0].tracks[0].id, "abc123");
    assert_eq!(references[0].tracks[0].platform, "spotify");
    assert_eq!(references[0].tracks[1].id, "yt-video456");
    assert_eq!(references[0].tracks[1].platform, "youtube");

    let known_track = Track {
        id: "spotify:track:abc123".into(),
        title: "Known track".into(),
        source: Some("spotify".into()),
        ..Track::default()
    };
    let resolved = resolve_room_playlist(&references[0], &[known_track], &[], &[], &[]);
    assert_eq!(resolved.tracks[0].title, "Known track");
    assert_eq!(resolved.tracks[1].video_id.as_deref(), Some("video456"));
    assert_eq!(resolved.tracks[1].source.as_deref(), Some("youtube"));
}

#[test]
fn playlist_search_ignores_case_and_accepts_empty_query() {
    assert!(playlist_matches("  jazz ", "Sunday Jazz"));
    assert!(playlist_matches("", "Any playlist"));
    assert!(!playlist_matches("rock", "Sunday Jazz"));
}

#[test]
fn new_playlists_include_only_selected_favorite_tracks() {
    let favorites = ["first", "second", "third"].map(|id| Track {
        id: id.to_string(),
        ..Track::default()
    });
    let selected = vec!["third".to_string(), "missing".to_string()];

    assert_eq!(
        selected_favorite_tracks(&favorites, &selected),
        vec![favorites[2].clone()]
    );
}

#[test]
fn parses_synced_lyrics_and_multiple_timestamp_tags() {
    let lines =
        parse_synced_lyrics("[ar:Artist]\n[00:01.50][00:03.00]First line\n[00:05.25]Second line");
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].time, Some(1.5));
    assert_eq!(lines[1].time, Some(3.0));
    assert_eq!(lines[2].text, "Second line");
}

#[test]
fn home_statistics_rank_saved_play_counts() {
    let history = vec![
        Track {
            id: "track-a".into(),
            title: "First".into(),
            artist: "Artist A".into(),
            ..Track::default()
        },
        Track {
            id: "track-b".into(),
            title: "Second".into(),
            artist: "Artist A".into(),
            ..Track::default()
        },
        Track {
            id: "track-c".into(),
            title: "Third".into(),
            artist: "Artist B".into(),
            ..Track::default()
        },
    ];
    let play_counts = std::collections::HashMap::from([
        ("track-a".to_string(), 5),
        ("track-b".to_string(), 2),
        ("track-c".to_string(), 4),
    ]);

    let (artists, songs) = home_statistics(&history, &[], &play_counts, "all");

    assert_eq!(artists, [("Artist A".into(), 7), ("Artist B".into(), 4)]);
    assert_eq!(songs.first(), Some(&("First".into(), "Artist A".into(), 5)));
}

#[test]
fn queue_reordering_moves_items_and_rejects_invalid_indices() {
    assert_eq!(
        reordered_queue(vec!["first", "second", "third"], 2, 0),
        Some(vec!["third", "first", "second"])
    );
    assert_eq!(reordered_queue(vec!["only"], 0, 0), None);
    assert_eq!(reordered_queue(vec!["first"], 0, 1), None);
}

#[test]
fn download_status_response_preserves_progress_and_error_details() {
    let response: DownloadStatusResponse = serde_json::from_str(
        r#"{"tracks":{"track-a":{"status":"downloading","progress":42.5},"track-b":{"status":"error","progress":null,"error":"unavailable"}}}"#,
    )
    .expect("download status response should deserialize");

    assert_eq!(response.tracks["track-a"].status, "downloading");
    assert_eq!(response.tracks["track-a"].progress, Some(42.5));
    assert_eq!(response.tracks["track-b"].status, "error");
    assert_eq!(
        response.tracks["track-b"].error.as_deref(),
        Some("unavailable")
    );
}

#[test]
fn queue_source_labels_distinguish_local_flac_youtube_and_streaming_tracks() {
    let local = Track {
        id: "local-library/track.mp3".into(),
        source: Some("local".into()),
        ..Track::default()
    };
    let flac = Track {
        id: "local-library/track.FLAC".into(),
        source: Some("local".into()),
        ..Track::default()
    };
    let youtube = Track {
        id: "youtube-track".into(),
        source: Some("youtube".into()),
        ..Track::default()
    };
    let spotify = Track {
        id: "spotify:track:abc".into(),
        source: Some("spotify".into()),
        ..Track::default()
    };

    assert_eq!(queue_source_label(&local), "Local");
    assert_eq!(queue_source_label(&flac), "FLAC");
    assert_eq!(queue_source_label(&youtube), "YouTube");
    assert_eq!(queue_source_label(&spotify), "Spotify");
}
