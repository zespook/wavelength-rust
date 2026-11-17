use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/browser.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = getLocal)]
    fn get_local(key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = setLocal)]
    fn set_local(key: &str, value: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = removeLocal)]
    fn remove_local_js(key: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = installHoverMarquee)]
    fn install_hover_marquee_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = installAppBehaviors)]
    fn install_app_behaviors_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = watchNarrowWindow)]
    fn watch_narrow_window_js(callback: &js_sys::Function) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = getRoomInviteCode)]
    fn get_room_invite_code_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = copyRoomCode)]
    async fn copy_room_code_js(code: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = copyRoomInvite)]
    async fn copy_room_invite_js(code: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = setAppearance)]
    fn set_appearance_js(theme: &str, accent: &str, background: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = setFontScale)]
    fn set_font_scale_js(value: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = isDesktopWindow)]
    fn is_desktop_window_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = checkForUpdates)]
    async fn check_for_updates_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = installUpdate)]
    async fn install_update_js(version: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = effectiveTheme)]
    fn effective_theme_js(theme: &str) -> Result<String, JsValue>;
    #[wasm_bindgen(catch, js_name = windowAction)]
    async fn window_action_js(action: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = startRailResize)]
    fn start_rail_resize_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = restoreSpotify)]
    async fn restore_spotify_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = connectSpotify)]
    async fn connect_spotify_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = spotifyLibrary)]
    async fn spotify_library_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = spotifyPlaylists)]
    async fn spotify_playlists_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = spotifyPlaylistTracks)]
    async fn spotify_playlist_tracks_js(id: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = saveSpotifyPlaylist)]
    async fn save_spotify_playlist_js(
        id: &str,
        name: &str,
        track_uris: JsValue,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = createSpotifyPlaylist)]
    async fn create_spotify_playlist_js(name: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = disconnectSpotify)]
    fn disconnect_spotify_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAudio)]
    async fn search_audio_js(query: &str, source: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = playAudio)]
    fn play_audio_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = playAudioSource)]
    async fn play_audio_source_js(source: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = pauseAudio)]
    fn pause_audio_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = seekAudio)]
    fn seek_audio_js(value: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = readAudioTime)]
    fn read_audio_time_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = setAudioVolume)]
    fn set_audio_volume_js(value: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = downloadAudio)]
    async fn download_audio_js(id: &str, filename: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = savedDownloadFolderName)]
    async fn saved_download_folder_name_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = chooseDownloadFolder)]
    async fn choose_download_folder_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = clearDownloadFolder)]
    async fn clear_download_folder_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scanLibrary)]
    async fn scan_library_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = savedLibraryFolders)]
    async fn saved_library_folders_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = removeSavedLibraryFolder)]
    async fn remove_saved_library_folder_js(name: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = chooseFolder)]
    async fn choose_folder_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = chooseAvatar)]
    async fn choose_avatar_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = localUrl)]
    fn local_url_js(id: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = roomRequest)]
    async fn room_request_js(
        url: &str,
        path: &str,
        method: &str,
        body: &str,
        access: &str,
        host: &str,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = requestJson)]
    async fn request_json_js(url: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = requestJsonMethod)]
    async fn request_json_method_js(
        url: &str,
        method: &str,
        body: &str,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = fetchLyrics)]
    async fn fetch_lyrics_js(title: &str, artist: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = connectRoomSocket)]
    fn connect_room_socket_js(
        url: &str,
        code: &str,
        username: &str,
        access_token: &str,
        host_token: &str,
        role: &str,
        permissions: JsValue,
    ) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = disconnectRoomSocket)]
    fn disconnect_room_socket_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = emitRoomCommand)]
    fn emit_room_command_js(action: &str, details: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = emitRoomPlaybackControl)]
    fn emit_room_playback_control_js(control: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = emitRoomPlaybackProgress)]
    fn emit_room_playback_progress_js(progress: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = emitRoomHostState)]
    fn emit_room_host_state_js(state: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = drainRoomEvents)]
    fn drain_room_events_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = uploadRoomTrack)]
    async fn upload_room_track_js(
        url: &str,
        code: &str,
        track_id: &str,
        host_token: &str,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = hasLocalAudioFile)]
    fn has_local_audio_file_js(track_id: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = roomAudioUrl)]
    async fn room_audio_url_js(audio_id: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = openExternalTrack)]
    fn open_external_track_js(source: &str, id: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = setEqualizer)]
    fn set_equalizer_js(settings: JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = startEqualizerVisualization)]
    fn start_equalizer_visualization_js(
        canvas_id: &str,
        selected_band: u32,
    ) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = setEqualizerSelectedBand)]
    fn set_equalizer_selected_band_js(selected_band: u32) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = setEqualizerPreview)]
    fn set_equalizer_preview_js(index: u32, x: f64, y: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = clearEqualizerPreview)]
    fn clear_equalizer_preview_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = equalizerCanvasSize)]
    fn equalizer_canvas_size_js() -> Result<JsValue, JsValue>;
}

fn error(error: JsValue) -> String {
    super::format_error(error)
}

pub fn load<T: for<'de> Deserialize<'de>>(key: &str) -> Result<Option<T>, String> {
    let value = get_local(key).map_err(error)?;
    let Some(value) = value.as_string() else {
        return Ok(None);
    };
    match serde_json::from_str(&value) {
        Ok(saved) => Ok(Some(saved)),
        Err(json_error) => serde_wasm_bindgen::from_value(JsValue::from_str(&value))
            .map(Some)
            .map_err(|_| format!("Saved {key} data is invalid: {json_error}")),
    }
}

pub fn save<T: Serialize + ?Sized>(key: &str, value: &T) -> Result<(), String> {
    let json = serde_json::to_string(value).map_err(|error| error.to_string())?;
    set_local(key, &json).map(|_| ()).map_err(error)
}

pub fn remove_local(key: &str) -> Result<(), String> {
    remove_local_js(key).map(|_| ()).map_err(error)
}

pub fn room_server_passcode() -> Result<Option<String>, String> {
    load("wavelength-room-passcode")
}

pub fn set_room_server_passcode(passcode: Option<&str>) -> Result<(), String> {
    match passcode.filter(|passcode| !passcode.is_empty()) {
        Some(passcode) if passcode.len() <= 128 => save("wavelength-room-passcode", &passcode),
        Some(_) => Err("The room server passcode is invalid.".to_string()),
        None => remove_local("wavelength-room-passcode"),
    }
}

pub fn install_hover_marquee() -> Result<(), String> {
    install_hover_marquee_js().map(|_| ()).map_err(error)
}

pub fn install_app_behaviors() -> Result<(), String> {
    install_app_behaviors_js().map(|_| ()).map_err(error)
}

pub fn set_appearance(theme: &str, accent: &str, background: &str) -> Result<(), String> {
    set_appearance_js(theme, accent, background)
        .map(|_| ())
        .map_err(error)
}

pub fn set_font_scale(value: f64) -> Result<(), String> {
    set_font_scale_js(value).map(|_| ()).map_err(error)
}

pub fn is_desktop_window() -> Result<bool, String> {
    is_desktop_window_js().map_err(error)
}

pub fn audio_service_base() -> Result<String, String> {
    Ok(AUDIO_SERVICE.to_string())
}

pub async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    let value = check_for_updates_js().await.map_err(error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(value)
        .map(Some)
        .map_err(|error| format!("The update service returned invalid data: {error}"))
}

pub async fn install_update(version: &str) -> Result<(), String> {
    install_update_js(version).await.map(|_| ()).map_err(error)
}

pub fn drag_window() {}

pub fn effective_theme(theme: &str) -> Result<String, String> {
    effective_theme_js(theme).map_err(error)
}

pub async fn window_action(action: &str) -> Result<(), String> {
    window_action_js(action).await.map(|_| ()).map_err(error)
}

pub fn start_rail_resize() -> Result<(), String> {
    start_rail_resize_js().map(|_| ()).map_err(error)
}

pub fn watch_narrow_window(callback: impl FnMut(bool) + 'static) -> Result<(), String> {
    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(callback) as Box<dyn FnMut(bool)>);
    watch_narrow_window_js(callback.as_ref().unchecked_ref())
        .map(|_| {
            callback.forget();
        })
        .map_err(error)
}

pub fn room_invite_code() -> Result<Option<String>, String> {
    get_room_invite_code_js()
        .map(|value| value.as_string().filter(|code| !code.trim().is_empty()))
        .map_err(error)
}

pub async fn copy_room_code(code: &str) -> Result<(), String> {
    copy_room_code_js(code).await.map(|_| ()).map_err(error)
}

pub async fn copy_room_invite(code: &str) -> Result<(), String> {
    copy_room_invite_js(code).await.map(|_| ()).map_err(error)
}

pub async fn restore_spotify() -> Result<Option<SpotifyRestore>, String> {
    let value = restore_spotify_js().await.map_err(error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(value)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub async fn connect_spotify() -> Result<(), String> {
    connect_spotify_js().await.map(|_| ()).map_err(error)
}

pub async fn spotify_library() -> Result<Vec<Track>, String> {
    let value = spotify_library_js().await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn spotify_playlists() -> Result<Vec<SpotifyPlaylist>, String> {
    let value = spotify_playlists_js().await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn spotify_playlist_tracks(id: &str) -> Result<Vec<Track>, String> {
    let value = spotify_playlist_tracks_js(id).await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn save_spotify_playlist(
    id: &str,
    name: &str,
    track_uris: &[String],
) -> Result<(), String> {
    let track_uris = serde_wasm_bindgen::to_value(track_uris).map_err(|error| error.to_string())?;
    save_spotify_playlist_js(id, name, track_uris)
        .await
        .map(|_| ())
        .map_err(error)
}

pub async fn create_spotify_playlist(name: &str) -> Result<SpotifyPlaylist, String> {
    let value = create_spotify_playlist_js(name).await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub fn disconnect_spotify() -> Result<(), String> {
    disconnect_spotify_js().map(|_| ()).map_err(error)
}

pub async fn search(query: &str, source: &str) -> Result<Vec<Track>, String> {
    let value = search_audio_js(query, source).await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub fn play() -> Result<(), String> {
    play_audio_js().map(|_| ()).map_err(error)
}
pub async fn play_source(source: &str) -> Result<(), String> {
    play_audio_source_js(source)
        .await
        .map(|_| ())
        .map_err(error)
}
pub fn pause() -> Result<(), String> {
    pause_audio_js().map(|_| ()).map_err(error)
}
pub fn seek(value: f64) {
    let _ = seek_audio_js(value);
}

pub async fn audio_time() -> Result<(f64, f64), String> {
    let value = read_audio_time_js().map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}
pub fn set_volume(value: f64) -> Result<(), String> {
    set_audio_volume_js(value).map(|_| ()).map_err(error)
}

pub async fn download(id: &str, filename: &str) -> Result<String, String> {
    download_audio_js(id, filename)
        .await
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Download did not return a filename.".to_string())
}

pub async fn saved_download_folder_name() -> Result<Option<String>, String> {
    let value = saved_download_folder_name_js().await.map_err(error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .as_string()
        .map(Some)
        .ok_or_else(|| "Saved download folder did not return a folder name.".to_string())
}

pub async fn choose_download_folder() -> Result<Option<String>, String> {
    let value = choose_download_folder_js().await.map_err(error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .as_string()
        .map(Some)
        .ok_or_else(|| "Selected download folder did not return a folder name.".to_string())
}

pub async fn clear_download_folder() -> Result<(), String> {
    clear_download_folder_js().await.map(|_| ()).map_err(error)
}

pub async fn scan_library() -> Result<Vec<Track>, String> {
    let value = scan_library_js().await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn saved_library_folders() -> Result<Vec<String>, String> {
    let value = saved_library_folders_js().await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn remove_saved_library_folder(name: &str) -> Result<(), String> {
    remove_saved_library_folder_js(name)
        .await
        .map(|_| ())
        .map_err(error)
}

pub async fn choose_folder() -> Result<Option<Vec<Track>>, String> {
    let value = choose_folder_js().await.map_err(error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(value)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub async fn choose_avatar() -> Result<Option<String>, String> {
    let value = choose_avatar_js().await.map_err(error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .as_string()
        .map(Some)
        .ok_or_else(|| "Photo selection did not return an image.".to_string())
}

pub async fn local_url(id: &str) -> Result<String, String> {
    local_url_js(id)
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Local audio URL was not available.".to_string())
}

pub async fn room_request<T: for<'de> Deserialize<'de>>(
    url: &str,
    path: &str,
    method: &str,
    body: &serde_json::Value,
    access: &str,
    host: &str,
) -> Result<T, String> {
    let body = serde_json::to_string(body).map_err(|error| error.to_string())?;
    let value = room_request_js(url, path, method, &body, access, host)
        .await
        .map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, String> {
    let value = request_json_js(url).await.map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn download_status(
    track_ids: &[String],
) -> Result<std::collections::HashMap<String, DownloadStatus>, String> {
    if track_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let encoded_ids = js_sys::encode_uri_component(&track_ids.join(","))
        .as_string()
        .ok_or_else(|| "Download status query could not be encoded.".to_string())?;
    let response: DownloadStatusResponse =
        get_json(&format!("{AUDIO_SERVICE}/status?video_ids={encoded_ids}")).await?;
    Ok(response.tracks)
}

pub async fn request_json_method<T: for<'de> Deserialize<'de>>(
    url: &str,
    method: &str,
    body: &serde_json::Value,
) -> Result<T, String> {
    let body = serde_json::to_string(body).map_err(|error| error.to_string())?;
    let value = request_json_method_js(url, method, &body)
        .await
        .map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn lyrics(title: &str, artist: &str) -> Result<Option<LyricsResponse>, String> {
    let value = fetch_lyrics_js(title, artist).await.map_err(error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(value)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub fn connect_room_socket(
    url: &str,
    code: &str,
    username: &str,
    access_token: &str,
    host_token: &str,
    role: &str,
    permissions: &RoomPermissions,
) -> Result<(), String> {
    let permissions =
        serde_wasm_bindgen::to_value(permissions).map_err(|error| error.to_string())?;
    connect_room_socket_js(url, code, username, access_token, host_token, role, permissions)
        .map(|_| ())
        .map_err(error)
}

pub fn disconnect_room_socket() -> Result<(), String> {
    disconnect_room_socket_js().map(|_| ()).map_err(error)
}

pub async fn emit_room_command(
    action: &str,
    details: &serde_json::Value,
) -> Result<String, String> {
    let value = serde_wasm_bindgen::to_value(details).map_err(|error| error.to_string())?;
    emit_room_command_js(action, value)
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Room command returned an invalid status.".to_string())
}

pub async fn emit_room_playback_control(control: &serde_json::Value) -> Result<String, String> {
    let value = serde_wasm_bindgen::to_value(control).map_err(|error| error.to_string())?;
    emit_room_playback_control_js(value)
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Room playback control returned an invalid status.".to_string())
}

pub async fn emit_room_playback_progress(progress: &serde_json::Value) -> Result<String, String> {
    let value = serde_wasm_bindgen::to_value(progress).map_err(|error| error.to_string())?;
    emit_room_playback_progress_js(value)
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Room playback progress returned an invalid status.".to_string())
}

pub async fn emit_room_host_state(state: &serde_json::Value) -> Result<String, String> {
    let value = serde_wasm_bindgen::to_value(state).map_err(|error| error.to_string())?;
    emit_room_host_state_js(value)
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Room state returned an invalid status.".to_string())
}

pub async fn drain_room_events() -> Result<Vec<RoomSocketEvent>, String> {
    let value = drain_room_events_js().map_err(error)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn upload_room_track(
    url: &str,
    code: &str,
    track_id: &str,
    host_token: &str,
) -> Result<String, String> {
    upload_room_track_js(url, code, track_id, host_token)
        .await
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Room audio upload returned an invalid ID.".to_string())
}

pub fn has_local_audio_file(track_id: &str) -> Result<bool, String> {
    has_local_audio_file_js(track_id).map_err(error)
}

pub async fn room_audio_url(audio_id: &str) -> Result<String, String> {
    room_audio_url_js(audio_id)
        .await
        .map_err(error)?
        .as_string()
        .ok_or_else(|| "Room audio access returned an invalid URL.".to_string())
}

pub fn open_external_track(source: &str, id: &str) -> Result<(), String> {
    open_external_track_js(source, id).map_err(error)
}

pub fn set_equalizer(settings: &EqualizerSettings) -> Result<(), String> {
    let value = serde_wasm_bindgen::to_value(settings).map_err(|error| error.to_string())?;
    set_equalizer_js(value).map(|_| ()).map_err(error)
}

pub fn start_equalizer_visualization(canvas_id: &str, selected_band: u32) -> Result<(), String> {
    start_equalizer_visualization_js(canvas_id, selected_band)
        .map(|_| ())
        .map_err(error)
}

pub fn set_equalizer_selected_band(selected_band: u32) -> Result<(), String> {
    set_equalizer_selected_band_js(selected_band)
        .map(|_| ())
        .map_err(error)
}

pub fn set_equalizer_preview(index: u32, x: f64, y: f64) -> Result<(), String> {
    set_equalizer_preview_js(index, x, y)
        .map(|_| ())
        .map_err(error)
}

pub fn clear_equalizer_preview() -> Result<(), String> {
    clear_equalizer_preview_js().map(|_| ()).map_err(error)
}

pub async fn equalizer_canvas_size() -> Result<(f64, f64), String> {
    let value = equalizer_canvas_size_js().map_err(error)?;
    let values = js_sys::Array::from(&value);
    let width = values
        .get(0)
        .as_f64()
        .ok_or_else(|| "Equalizer graph width is invalid.".to_string())?;
    let height = values
        .get(1)
        .as_f64()
        .ok_or_else(|| "Equalizer graph height is invalid.".to_string())?;
    Ok((width, height))
}
