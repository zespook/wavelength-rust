use super::*;
use dioxus::desktop;
use serde::de::DeserializeOwned;
use serde_json::Value;

async fn call_js<T: DeserializeOwned>(function: &str, arguments: &[Value]) -> Result<T, String> {
    let function = serde_json::to_string(function).map_err(|error| error.to_string())?;
    let arguments = serde_json::to_string(arguments).map_err(|error| error.to_string())?;
    let script = format!(
        "(async()=>{{try{{const value=await globalThis.__WAVELENGTH_BROWSER__[{function}](...{arguments});const refreshToken=globalThis.__WAVELENGTH_BROWSER__.getSpotifyRefreshToken?.()||null;dioxus.send(JSON.stringify({{ok:true,value:value??null,refreshToken}}));}}catch(error){{dioxus.send(JSON.stringify({{ok:false,error:String(error?.message??error)}}));}}}})();"
    );
    let mut evaluation = dioxus::document::eval(&script);
    let response = evaluation
        .recv::<String>()
        .await
        .map_err(|error| format!("The app bridge failed: {error}"))?;
    let response: Value = serde_json::from_str(&response)
        .map_err(|error| format!("The app bridge returned invalid data: {error}"))?;
    if response.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(response
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("The app bridge request failed.")
            .to_string());
    }
    if let Some(token) = response
        .get("refreshToken")
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty())
    {
        crate::desktop::set_spotify_refresh_token(Some(token))?;
    }
    serde_json::from_value(response.get("value").cloned().unwrap_or(Value::Null))
        .map_err(|error| format!("The app bridge returned invalid data: {error}"))
}

async fn call_js_for_side_effect(function: &str, arguments: &[Value]) -> Result<(), String> {
    call_js::<Value>(function, arguments).await.map(|_| ())
}

fn call_js_sync(function: &str, arguments: &[Value]) -> Result<(), String> {
    let function = serde_json::to_string(function).map_err(|error| error.to_string())?;
    let arguments = serde_json::to_string(arguments).map_err(|error| error.to_string())?;
    let script = format!("globalThis.__WAVELENGTH_BROWSER__[{function}](...{arguments});");
    desktop::window()
        .webview
        .evaluate_script(&script)
        .map_err(|error| format!("The app bridge failed: {error}"))
}

fn js_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| error.to_string())
}

pub fn load<T: for<'de> Deserialize<'de>>(key: &str) -> Result<Option<T>, String> {
    crate::desktop::load_local(key)
}

pub fn save<T: Serialize + ?Sized>(key: &str, value: &T) -> Result<(), String> {
    crate::desktop::save_local(key, value)
}

pub fn remove_local(key: &str) -> Result<(), String> {
    crate::desktop::remove_local(key)
}

pub fn room_server_passcode() -> Result<Option<String>, String> {
    crate::desktop::room_server_passcode()
}

pub fn set_room_server_passcode(passcode: Option<&str>) -> Result<(), String> {
    crate::desktop::set_room_server_passcode(passcode)
}

pub fn install_hover_marquee() -> Result<(), String> {
    call_js_sync("installHoverMarquee", &[])
}

pub fn install_app_behaviors() -> Result<(), String> {
    call_js_sync("installAppBehaviors", &[])
}

pub fn set_appearance(theme: &str, accent: &str, background: &str) -> Result<(), String> {
    call_js_sync(
        "setAppearance",
        &[
            Value::String(theme.to_string()),
            Value::String(accent.to_string()),
            Value::String(background.to_string()),
        ],
    )
}

pub fn set_font_scale(value: f64) -> Result<(), String> {
    call_js_sync("setFontScale", &[Value::from(value)])
}

pub fn is_desktop_window() -> Result<bool, String> {
    Ok(true)
}

pub fn audio_service_base() -> Result<String, String> {
    crate::desktop::audio_service_base()
}

pub async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    crate::desktop::check_for_updates().await
}

pub async fn install_update(version: &str) -> Result<(), String> {
    let window = desktop::window();
    crate::desktop::install_update(version).await?;
    window.close();
    Ok(())
}

pub fn drag_window() {
    desktop::window().drag();
}

pub fn effective_theme(theme: &str) -> Result<String, String> {
    if theme == "system" {
        return match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => Ok("dark".to_string()),
            Ok(dark_light::Mode::Light) => Ok("light".to_string()),
            Ok(dark_light::Mode::Unspecified) => Ok("dark".to_string()),
            Err(error) => Err(format!("Could not detect the system appearance: {error}")),
        };
    }
    Ok(theme.to_string())
}

pub async fn window_action(action: &str) -> Result<(), String> {
    let window = desktop::window();
    match action {
        "minimize" => window.set_minimized(true),
        "toggle-maximize" => window.toggle_maximized(),
        "close" => window.close(),
        _ => return Err(format!("Unknown window action: {action}")),
    }
    Ok(())
}

pub fn start_rail_resize() -> Result<(), String> {
    call_js_sync("startRailResize", &[])
}

pub fn watch_narrow_window(mut callback: impl FnMut(bool) + 'static) -> Result<(), String> {
    let window = desktop::window();
    let mut scale_factor = window.scale_factor();
    let mut is_narrow = window.inner_size().width as f64 / scale_factor < 948.0;
    callback(is_narrow);
    window.create_wry_event_handler(move |event, _| {
        let width = match event {
            dioxus::desktop::tao::event::Event::WindowEvent {
                event: dioxus::desktop::WindowEvent::Resized(size),
                ..
            } => Some(size.width),
            dioxus::desktop::tao::event::Event::WindowEvent {
                event:
                    dioxus::desktop::WindowEvent::ScaleFactorChanged {
                        scale_factor: updated_scale_factor,
                        new_inner_size,
                    },
                ..
            } => {
                scale_factor = *updated_scale_factor;
                Some(new_inner_size.width)
            }
            _ => None,
        };
        if let Some(width) = width {
            let next_is_narrow = width as f64 / scale_factor < 948.0;
            if next_is_narrow != is_narrow {
                is_narrow = next_is_narrow;
                callback(is_narrow);
            }
        }
    });
    Ok(())
}

pub fn room_invite_code() -> Result<Option<String>, String> {
    crate::desktop::room_invite_code()
}

pub async fn copy_room_code(code: &str) -> Result<(), String> {
    call_js_for_side_effect("copyRoomCode", &[Value::String(code.to_string())]).await
}

pub async fn copy_room_invite(code: &str) -> Result<(), String> {
    call_js_for_side_effect("copyRoomInvite", &[Value::String(code.to_string())]).await
}

pub async fn restore_spotify() -> Result<Option<SpotifyRestore>, String> {
    if let Some(token) = crate::desktop::spotify_refresh_token()? {
        call_js::<bool>("setSpotifyRefreshToken", &[Value::String(token)]).await?;
    }
    call_js("restoreSpotify", &[]).await
}

pub async fn connect_spotify() -> Result<(), String> {
    let token = crate::desktop::connect_spotify().await?;
    call_js_for_side_effect("setSpotifyRefreshToken", &[Value::String(token)]).await
}

pub async fn spotify_library() -> Result<Vec<Track>, String> {
    call_js("spotifyLibrary", &[]).await
}

pub async fn spotify_playlists() -> Result<Vec<SpotifyPlaylist>, String> {
    call_js("spotifyPlaylists", &[]).await
}

pub async fn spotify_playlist_tracks(id: &str) -> Result<Vec<Track>, String> {
    call_js("spotifyPlaylistTracks", &[Value::String(id.to_string())]).await
}

pub async fn save_spotify_playlist(
    id: &str,
    name: &str,
    track_uris: &[String],
) -> Result<(), String> {
    call_js_for_side_effect(
        "saveSpotifyPlaylist",
        &[
            Value::String(id.to_string()),
            Value::String(name.to_string()),
            js_value(track_uris)?,
        ],
    )
    .await
}

pub async fn create_spotify_playlist(name: &str) -> Result<SpotifyPlaylist, String> {
    call_js("createSpotifyPlaylist", &[Value::String(name.to_string())]).await
}

pub fn disconnect_spotify() -> Result<(), String> {
    crate::desktop::disconnect_spotify()?;
    call_js_sync("disconnectSpotify", &[])
}

pub async fn search(query: &str, source: &str) -> Result<Vec<Track>, String> {
    call_js(
        "searchAudio",
        &[
            Value::String(query.to_string()),
            Value::String(source.to_string()),
        ],
    )
    .await
}

pub fn play() -> Result<(), String> {
    call_js_sync("playAudio", &[])
}

pub async fn play_source(source: &str) -> Result<(), String> {
    call_js::<bool>("playAudioSource", &[Value::String(source.to_string())])
        .await
        .map(|_| ())
}

pub fn pause() -> Result<(), String> {
    call_js_sync("pauseAudio", &[])
}

pub fn seek(value: f64) {
    let _ = call_js_sync("seekAudio", &[Value::from(value)]);
}

pub async fn audio_time() -> Result<(f64, f64), String> {
    call_js("readAudioTime", &[]).await
}

pub fn set_volume(value: f64) -> Result<(), String> {
    call_js_sync("setAudioVolume", &[Value::from(value)])
}

pub async fn download(id: &str, filename: &str) -> Result<String, String> {
    crate::desktop::download(id, filename).await
}

pub async fn saved_download_folder_name() -> Result<Option<String>, String> {
    crate::desktop::saved_download_folder_name()
}

pub async fn choose_download_folder() -> Result<Option<String>, String> {
    crate::desktop::choose_download_folder().await
}

pub async fn clear_download_folder() -> Result<(), String> {
    crate::desktop::clear_download_folder()
}

pub async fn scan_library() -> Result<Vec<Track>, String> {
    crate::desktop::scan_library().await
}

pub async fn saved_library_folders() -> Result<Vec<String>, String> {
    crate::desktop::saved_library_folders()
}

pub async fn remove_saved_library_folder(name: &str) -> Result<(), String> {
    crate::desktop::remove_saved_library_folder(name)
}

pub async fn choose_folder() -> Result<Option<Vec<Track>>, String> {
    crate::desktop::choose_folder().await
}

pub async fn choose_avatar() -> Result<Option<String>, String> {
    call_js("chooseAvatar", &[]).await
}

pub async fn local_url(id: &str) -> Result<String, String> {
    crate::desktop::local_url(id)
}

pub async fn room_request<T: DeserializeOwned>(
    url: &str,
    path: &str,
    method: &str,
    body: &Value,
    access: &str,
    host: &str,
) -> Result<T, String> {
    let body = serde_json::to_string(body).map_err(|error| error.to_string())?;
    call_js(
        "roomRequest",
        &[
            Value::String(url.to_string()),
            Value::String(path.to_string()),
            Value::String(method.to_string()),
            Value::String(body),
            Value::String(access.to_string()),
            Value::String(host.to_string()),
        ],
    )
    .await
}

pub async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let url = resolve_audio_url(url)?;
    call_js("requestJson", &[Value::String(url)]).await
}

fn resolve_audio_url(url: &str) -> Result<String, String> {
    if let Some(path) = url.strip_prefix(crate::AUDIO_SERVICE) {
        return Ok(format!("{}{path}", crate::desktop::audio_service_base()?));
    }
    Ok(url.to_string())
}

pub async fn download_status(
    track_ids: &[String],
) -> Result<std::collections::HashMap<String, DownloadStatus>, String> {
    if track_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let ids = url::form_urlencoded::byte_serialize(track_ids.join(",").as_bytes())
        .collect::<String>()
        .replace('+', "%20");
    let response: DownloadStatusResponse = get_json(&format!(
        "{}/status?video_ids={ids}",
        crate::desktop::audio_service_base()?
    ))
    .await?;
    Ok(response.tracks)
}

pub async fn request_json_method<T: DeserializeOwned>(
    url: &str,
    method: &str,
    body: &Value,
) -> Result<T, String> {
    let url = resolve_audio_url(url)?;
    let body = serde_json::to_string(body).map_err(|error| error.to_string())?;
    call_js(
        "requestJsonMethod",
        &[
            Value::String(url),
            Value::String(method.to_string()),
            Value::String(body),
        ],
    )
    .await
}

pub async fn lyrics(title: &str, artist: &str) -> Result<Option<LyricsResponse>, String> {
    call_js(
        "fetchLyrics",
        &[
            Value::String(title.to_string()),
            Value::String(artist.to_string()),
        ],
    )
    .await
}

pub fn connect_room_socket(
    url: &str,
    code: &str,
    username: &str,
    access_token: &str,
    role: &str,
    permissions: &RoomPermissions,
) -> Result<(), String> {
    call_js_sync(
        "connectRoomSocket",
        &[
            Value::String(url.to_string()),
            Value::String(code.to_string()),
            Value::String(username.to_string()),
            Value::String(access_token.to_string()),
            Value::String(role.to_string()),
            js_value(permissions)?,
        ],
    )
}

pub fn disconnect_room_socket() -> Result<(), String> {
    call_js_sync("disconnectRoomSocket", &[])
}

pub async fn emit_room_command(action: &str, details: &Value) -> Result<String, String> {
    call_js(
        "emitRoomCommand",
        &[Value::String(action.to_string()), details.clone()],
    )
    .await
}

pub async fn emit_room_playback_control(control: &Value) -> Result<String, String> {
    call_js("emitRoomPlaybackControl", &[control.clone()]).await
}

pub async fn emit_room_playback_progress(progress: &Value) -> Result<String, String> {
    call_js("emitRoomPlaybackProgress", &[progress.clone()]).await
}

pub async fn drain_room_events() -> Result<Vec<RoomSocketEvent>, String> {
    call_js("drainRoomEvents", &[]).await
}

pub async fn upload_room_track(
    url: &str,
    code: &str,
    track_id: &str,
    host_token: &str,
) -> Result<String, String> {
    crate::desktop::upload_room_track(url, code, track_id, host_token).await
}

pub fn has_local_audio_file(track_id: &str) -> Result<bool, String> {
    crate::desktop::has_local_audio_file(track_id)
}

pub async fn room_audio_url(audio_id: &str) -> Result<String, String> {
    call_js("roomAudioUrl", &[Value::String(audio_id.to_string())]).await
}

pub fn open_external_track(source: &str, id: &str) -> Result<(), String> {
    crate::desktop::open_external_track(source, id)
}

pub fn set_equalizer(settings: &EqualizerSettings) -> Result<(), String> {
    call_js_sync("setEqualizer", &[js_value(settings)?])
}

pub fn start_equalizer_visualization(canvas_id: &str, selected_band: u32) -> Result<(), String> {
    call_js_sync(
        "startEqualizerVisualization",
        &[
            Value::String(canvas_id.to_string()),
            Value::from(selected_band),
        ],
    )
}

pub fn set_equalizer_selected_band(selected_band: u32) -> Result<(), String> {
    call_js_sync("setEqualizerSelectedBand", &[Value::from(selected_band)])
}

pub fn set_equalizer_preview(index: u32, x: f64, y: f64) -> Result<(), String> {
    call_js_sync(
        "setEqualizerPreview",
        &[Value::from(index), Value::from(x), Value::from(y)],
    )
}

pub fn clear_equalizer_preview() -> Result<(), String> {
    call_js_sync("clearEqualizerPreview", &[])
}

pub async fn equalizer_canvas_size() -> Result<(f64, f64), String> {
    let values: Vec<f64> = call_js("equalizerCanvasSize", &[]).await?;
    if values.len() != 2 || !values.iter().all(|value| value.is_finite() && *value > 0.0) {
        return Err("Equalizer graph size is invalid.".to_string());
    }
    Ok((values[0], values[1]))
}
