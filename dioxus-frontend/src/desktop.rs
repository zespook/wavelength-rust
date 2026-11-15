use crate::{Track, UpdateInfo};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::Element,
};
use sha2::{Digest, Sha256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Cow;
use std::{
    env, fs,
    io::{BufRead, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const APP_ID: &str = "com.wavelength.desktop";
const AUDIO_PORT: u16 = 8787;
const SPOTIFY_TOKEN_KEY: &str = "spotify-refresh-token";
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "ogg", "wav", "aac", "opus"];
static STATE: OnceLock<DesktopState> = OnceLock::new();
static SETTINGS_LOCK: Mutex<()> = Mutex::new(());

struct DesktopState {
    data_dir: PathBuf,
    resource_dir: PathBuf,
    audio_port: Option<u16>,
    service_child: Mutex<Option<Child>>,
}

fn project_dirs() -> Result<directories::ProjectDirs, String> {
    directories::ProjectDirs::from("com", "Wavelength", "Wavelength")
        .ok_or_else(|| "The application data directory is unavailable.".to_string())
}

fn resource_directory() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("WAVELENGTH_RESOURCE_DIR") {
        return Ok(PathBuf::from(path));
    }
    if cfg!(debug_assertions) {
        return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    }
    let executable = env::current_exe().map_err(|error| error.to_string())?;
    let parent = executable
        .parent()
        .ok_or_else(|| "The application installation directory is unavailable.".to_string())?;
    #[cfg(target_os = "macos")]
    {
        return parent
            .parent()
            .map(|contents| contents.join("Resources"))
            .ok_or_else(|| "The application resources directory is unavailable.".to_string());
    }
    #[cfg(not(target_os = "macos"))]
    Ok(parent.to_path_buf())
}

fn app_state() -> Result<&'static DesktopState, String> {
    STATE
        .get()
        .ok_or_else(|| "Native application services have not started.".to_string())
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(project_dirs()?.config_dir().join("settings.json"))
}

fn read_settings() -> Result<Value, String> {
    let path = settings_path()?;
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|_| "Saved settings could not be read.".to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(error) => Err(format!("Could not read saved settings: {error}")),
    }
}

fn write_setting(key: &str, value: Value) -> Result<(), String> {
    let _guard = SETTINGS_LOCK
        .lock()
        .map_err(|error| format!("Could not lock application settings: {error}"))?;
    let path = settings_path()?;
    let directory = path
        .parent()
        .ok_or_else(|| "Settings location is unavailable.".to_string())?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let mut settings = read_settings()?;
    let object = settings
        .as_object_mut()
        .ok_or_else(|| "Saved settings are invalid.".to_string())?;
    object.insert(key.to_string(), value);
    fs::write(path, settings.to_string())
        .map_err(|error| format!("Could not save application settings: {error}"))
}

pub fn load_local<T: DeserializeOwned>(key: &str) -> Result<Option<T>, String> {
    let settings = read_settings()?;
    let Some(value) = settings
        .get("localStorage")
        .and_then(Value::as_object)
        .and_then(|values| values.get(key))
        .and_then(Value::as_str)
    else {
        return Ok(None);
    };
    serde_json::from_str(value)
        .map(Some)
        .map_err(|error| format!("Saved {key} data is invalid: {error}"))
}

pub fn save_local<T: Serialize + ?Sized>(key: &str, value: &T) -> Result<(), String> {
    let value = serde_json::to_string(value).map_err(|error| error.to_string())?;
    let _guard = SETTINGS_LOCK
        .lock()
        .map_err(|error| format!("Could not lock application settings: {error}"))?;
    let path = settings_path()?;
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| "Settings location is unavailable.".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut settings = read_settings()?;
    let values = settings
        .as_object_mut()
        .ok_or_else(|| "Saved settings are invalid.".to_string())?
        .entry("localStorage")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| "Saved app data is invalid.".to_string())?;
    values.insert(key.to_string(), Value::String(value));
    fs::write(path, settings.to_string())
        .map_err(|error| format!("Could not save app data: {error}"))
}

pub fn remove_local(key: &str) -> Result<(), String> {
    let _guard = SETTINGS_LOCK
        .lock()
        .map_err(|error| format!("Could not lock application settings: {error}"))?;
    let path = settings_path()?;
    let mut settings = read_settings()?;
    let values = settings
        .as_object_mut()
        .ok_or_else(|| "Saved settings are invalid.".to_string())?
        .entry("localStorage")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| "Saved app data is invalid.".to_string())?;
    values.remove(key);
    fs::write(path, settings.to_string())
        .map_err(|error| format!("Could not save app data: {error}"))
}

fn saved_window_size(settings: &Value) -> Option<(u32, u32)> {
    let size = settings.get("windowSize")?;
    let width = u32::try_from(size.get("width")?.as_u64()?).ok()?;
    let height = u32::try_from(size.get("height")?.as_u64()?).ok()?;
    if !(750..=8192).contains(&width) || !(600..=8192).contains(&height) {
        return None;
    }
    Some((width, height))
}

fn persist_window_size(window: &dioxus::desktop::tao::window::Window) -> Result<(), String> {
    let logical = window
        .inner_size()
        .to_logical::<f64>(window.scale_factor());
    if !(750.0..=8192.0).contains(&logical.width)
        || !(600.0..=8192.0).contains(&logical.height)
    {
        return Ok(());
    }
    write_setting(
        "windowSize",
        json!({
            "width": logical.width.round() as u32,
            "height": logical.height.round() as u32
        }),
    )
}

pub fn run(app: fn() -> Element) {
    let result = initialize();
    if let Err(error) = result {
        eprintln!("Could not start Wavelength: {error}");
        std::process::exit(1);
    }
    let state = match app_state() {
        Ok(state) => state,
        Err(error) => {
            eprintln!("Could not start Wavelength: {error}");
            std::process::exit(1);
        }
    };
    let settings = match read_settings() {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("Could not start Wavelength: {error}");
            std::process::exit(1);
        }
    };
    let (width, height) = saved_window_size(&settings).unwrap_or((1280, 800));
    let window_slot = std::sync::Arc::new(Mutex::new(None));
    let window_for_events = window_slot.clone();
    let child_handle = &state.service_child;
    let service_base = match audio_service_base() {
        Ok(service_base) => service_base,
        Err(error) => {
            eprintln!("Could not start Wavelength: {error}");
            std::process::exit(1);
        }
    };
    let service_base = serde_json::to_string(&service_base)
        .expect("serializing the local service URL cannot fail");
    let config = Config::new()
        .with_resource_directory(&state.resource_dir)
        .with_data_directory(state.data_dir.join("webview"))
        .with_disable_context_menu(true)
        .with_custom_head(format!(
            "<script>globalThis.__WAVELENGTH_NATIVE__=true;globalThis.__WAVELENGTH_AUDIO_SERVICE__={service_base};</script><script type=\"module\">{}</script>",
            include_str!("browser.js")
        ))
        .with_asynchronous_custom_protocol("wavelength-audio", |_id, request, responder| {
            responder.respond(audio_protocol_response(request));
        })
        .with_window(
            WindowBuilder::new()
                .with_title("Wavelength")
                .with_inner_size(LogicalSize::new(width, height))
                .with_min_inner_size(LogicalSize::new(750.0, 600.0))
                .with_decorations(false),
        )
        .with_on_window(move |window, _| {
            if let Ok(mut slot) = window_slot.lock() {
                *slot = Some(window);
            }
        })
        .with_custom_event_handler(move |event, _| {
            if let dioxus::desktop::tao::event::Event::WindowEvent {
                event: dioxus::desktop::WindowEvent::CloseRequested,
                ..
            } = event
            {
                match window_for_events.lock() {
                    Ok(window) => {
                        if let Some(window) = window.as_ref() {
                            if let Err(error) = persist_window_size(window) {
                                eprintln!("Could not save Wavelength window size: {error}");
                            }
                        }
                    }
                    Err(error) => eprintln!("Could not read Wavelength window state: {error}"),
                }
            }
            if matches!(event, dioxus::desktop::tao::event::Event::LoopDestroyed) {
                if let Err(error) = stop_local_service(child_handle) {
                    eprintln!("Could not stop the local service during app exit: {error}");
                }
            }
        })
        .with_navigation_handler(|url| {
            eprintln!("Blocked navigation away from the Wavelength app: {url}");
            false
        });
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app);
}

fn initialize() -> Result<(), String> {
    let project_dirs = project_dirs()?;
    fs::create_dir_all(project_dirs.data_dir())
        .map_err(|error| format!("Could not create app data directory: {error}"))?;
    fs::create_dir_all(project_dirs.config_dir())
        .map_err(|error| format!("Could not create app settings directory: {error}"))?;
    let resource_dir = resource_directory()?;
    let mut state = DesktopState {
        data_dir: project_dirs.data_dir().to_path_buf(),
        resource_dir,
        audio_port: None,
        service_child: Mutex::new(None),
    };
    start_local_service(&mut state)?;
    STATE
        .set(state)
        .map_err(|_| "Native app services were initialized more than once.".to_string())
}

fn start_local_service(state: &mut DesktopState) -> Result<(), String> {
    let services_dir = service_resources_directory(state)?;
    let executable = services_dir
        .join("api")
        .join(if cfg!(target_os = "windows") {
            "WavelengthLocalService.exe"
        } else {
            "WavelengthLocalService"
        });
    if !executable.exists() {
        if cfg!(debug_assertions) {
            state.audio_port = Some(AUDIO_PORT);
            return Ok(());
        }
        return Err(format!(
            "The bundled local audio service is missing: {}",
            executable.display()
        ));
    }

    let runtime_dir = services_dir.join("runtime");
    let inherited_path = env::var_os("PATH").unwrap_or_default();
    let path = env::join_paths(
        std::iter::once(runtime_dir).chain(env::split_paths(&inherited_path)),
    )
    .map_err(|error| error.to_string())?
    .to_string_lossy()
    .to_string();
    let data_dir = state.data_dir.join("audio-service");
    let download_dir = state.data_dir.join("qobuz-service").join("downloads");
    fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&download_dir).map_err(|error| error.to_string())?;

    for _ in 0..8 {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
        let port = listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port();
        drop(listener);
        let executable_path = executable.to_string_lossy().to_string();
        let working_dir = executable
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let mut command = Command::new(&executable);
        command
            .current_dir(working_dir)
            .env("PORT", port.to_string())
            .env("DOWNLOAD_ROOT", download_dir.to_string_lossy().as_ref())
            .env("WAVELENGTH_DATA_DIR", data_dir.to_string_lossy().as_ref())
            .env("QOBUZ_API_BASE", "https://www.qobuz.com/api.json/0.2")
            .env("QOBUZ_API_URL", format!("http://127.0.0.1:{port}/qobuz"))
            .env("LOG_LEVEL", "WARNING")
            .env("PATH", &path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let child = command.spawn().map_err(|error| {
            format!("Could not start the bundled local audio service ({executable_path}): {error}")
        })?;
        match wait_for_service(child, port) {
            Ok(child) => {
                state.audio_port = Some(port);
                *state
                    .service_child
                    .lock()
                    .map_err(|error| error.to_string())? = Some(child);
                return Ok(());
            }
            Err(error) => {
                if let Some(source) = error.downcast_ref::<std::io::Error>() {
                    if source.kind() == std::io::ErrorKind::AddrInUse {
                        continue;
                    }
                }
                return Err(error.to_string());
            }
        }
    }
    Err("Could not start the local audio service after several attempts.".to_string())
}

fn service_resources_directory(state: &DesktopState) -> Result<PathBuf, String> {
    let development_dir = state.resource_dir.join("resources").join("services");
    if development_dir.is_dir() {
        return Ok(development_dir);
    }
    let extraction_dir = state.data_dir.join("bundled-services");
    let expected_executable = extraction_dir
        .join("api")
        .join(if cfg!(target_os = "windows") {
            "WavelengthLocalService.exe"
        } else {
            "WavelengthLocalService"
        });
    if expected_executable.is_file() {
        return Ok(extraction_dir);
    }
    fs::create_dir_all(&extraction_dir)
        .map_err(|error| format!("Could not prepare packaged services: {error}"))?;
    #[cfg(target_os = "windows")]
    {
        let archive_path = state.resource_dir.join("services.zip");
        let archive_file = fs::File::open(&archive_path).map_err(|error| {
            format!("The packaged local services archive is unavailable: {error}")
        })?;
        let mut archive = zip::ZipArchive::new(archive_file)
            .map_err(|error| format!("The packaged local services archive is invalid: {error}"))?;
        archive
            .extract(&extraction_dir)
            .map_err(|error| format!("Could not unpack packaged local services: {error}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        let archive_path = state.resource_dir.join("services.tar.gz");
        let archive_file = fs::File::open(&archive_path).map_err(|error| {
            format!("The packaged local services archive is unavailable: {error}")
        })?;
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(archive_file));
        archive
            .unpack(&extraction_dir)
            .map_err(|error| format!("Could not unpack packaged local services: {error}"))?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        return Err("Packaged local services are not configured for this platform.".to_string());
    }
    if expected_executable.is_file() {
        Ok(extraction_dir)
    } else {
        Err("The packaged local audio service is missing after extraction.".to_string())
    }
}

fn wait_for_service(mut child: Child, port: u16) -> Result<Child, Box<dyn std::error::Error>> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let started = Instant::now();
    loop {
        if TcpStream::connect_timeout(&address, Duration::from_millis(150)).is_ok() {
            return Ok(child);
        }
        if let Some(status) = child.try_wait()? {
            return Err(std::io::Error::other(format!(
                "Local audio service exited before opening port {port}: {status}"
            ))
            .into());
        }
        if started.elapsed() > Duration::from_secs(45) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("Local audio service did not open port {port} in time."),
            )
            .into());
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn stop_local_service(child: &Mutex<Option<Child>>) -> Result<(), String> {
    let mut child = child
        .lock()
        .map_err(|error| format!("Could not lock the local service process: {error}"))?;
    let Some(mut child) = child.take() else {
        return Ok(());
    };
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let taskkill = env::var_os("SystemRoot")
            .map(PathBuf::from)
            .map(|root| root.join("System32").join("taskkill.exe"))
            .unwrap_or_else(|| PathBuf::from("taskkill.exe"));
        let status = Command::new(taskkill)
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .creation_flags(0x08000000)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if !matches!(status, Ok(status) if status.success()) && child.try_wait().ok().flatten().is_none() {
            return Err(format!("Could not stop the local service process tree: {status:?}"));
        }
    }
    if child.try_wait().map_err(|error| error.to_string())?.is_none() {
        child.kill().map_err(|error| error.to_string())?;
    }
    child.wait().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn audio_service_base() -> Result<String, String> {
    let port = app_state()?
        .audio_port
        .ok_or_else(|| "The local audio service is unavailable.".to_string())?;
    Ok(format!("http://127.0.0.1:{port}"))
}

fn settings_string(key: &str) -> Result<Option<String>, String> {
    Ok(read_settings()?
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string))
}

pub fn saved_download_folder_name() -> Result<Option<String>, String> {
    let Some(path) = settings_string("downloadDirectory")? else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    Ok(Some(
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .to_string(),
    ))
}

pub async fn choose_download_folder() -> Result<Option<String>, String> {
    let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await else {
        return Ok(None);
    };
    let canonical = fs::canonicalize(folder.path())
        .map_err(|error| format!("The selected download folder is unavailable: {error}"))?;
    if !canonical.is_dir() {
        return Err("Choose an existing folder for downloads.".to_string());
    }
    write_setting(
        "downloadDirectory",
        Value::String(canonical.to_string_lossy().to_string()),
    )?;
    Ok(Some(
        canonical
            .file_name()
            .unwrap_or(canonical.as_os_str())
            .to_string_lossy()
            .to_string(),
    ))
}

pub fn clear_download_folder() -> Result<(), String> {
    write_setting("downloadDirectory", Value::String(String::new()))
}

fn selected_download_directory() -> Result<PathBuf, String> {
    let path = settings_string("downloadDirectory")?
        .ok_or_else(|| "Choose a download folder in Settings first.".to_string())?;
    let canonical = fs::canonicalize(path)
        .map_err(|_| "The selected download folder is unavailable.".to_string())?;
    if !canonical.is_dir() {
        return Err("The selected download folder is unavailable.".to_string());
    }
    Ok(canonical)
}

fn safe_filename(filename: &str) -> Result<String, String> {
    let base = filename
        .trim()
        .chars()
        .map(|character| {
            if character.is_control()
                || matches!(character, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    let base = base.trim_matches([' ', '.']).chars().take(160).collect::<String>();
    if base.is_empty() {
        Err("A valid filename is required.".to_string())
    } else {
        Ok(base)
    }
}

fn audio_extension(content_type: &str) -> &'static str {
    match content_type.split(';').next().unwrap_or("").trim() {
        "audio/flac" => "flac",
        "audio/mpeg" => "mp3",
        "audio/mp4" => "m4a",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/webm" => "webm",
        _ => "audio",
    }
}

fn unique_download_path(directory: &Path, base: &str, extension: &str) -> PathBuf {
    let first = directory.join(format!("{base}.{extension}"));
    if !first.exists() {
        return first;
    }
    for suffix in 1..=9999 {
        let path = directory.join(format!("{base} ({suffix}).{extension}"));
        if !path.exists() {
            return path;
        }
    }
    directory.join(format!("{base}-{}.{}", chrono::Utc::now().timestamp_millis(), extension))
}

pub async fn download(id: &str, filename: &str) -> Result<String, String> {
    if id.len() < 6
        || id.len() > 256
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("This track has an invalid download ID.".to_string());
    }
    let directory = selected_download_directory()?;
    let base = safe_filename(filename)?;
    let url = format!("{}/download/{id}?code=solo", audio_service_base()?);
    let response = reqwest::get(url)
        .await
        .map_err(|error| format!("Audio service request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Download failed with status {}.", response.status()));
    }
    let extension = audio_extension(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or(""),
    );
    let destination = unique_download_path(&directory, &base, extension);
    let temporary = destination.with_extension(format!(
        "{extension}.{}.part",
        std::process::id()
    ));
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Download response failed: {error}"))?;
    fs::write(&temporary, bytes)
        .map_err(|error| format!("Could not save the download: {error}"))?;
    fs::rename(&temporary, &destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("Could not finish saving the download: {error}")
    })?;
    Ok(destination.to_string_lossy().to_string())
}

fn library_roots() -> Result<Vec<PathBuf>, String> {
    let settings = read_settings()?;
    let Some(folders) = settings.get("libraryFolders").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    folders
        .iter()
        .filter_map(Value::as_str)
        .map(|path| {
            fs::canonicalize(path).map_err(|error| {
                format!("Access to the saved folder \"{path}\" is unavailable: {error}")
            })
        })
        .collect()
}

pub fn saved_library_folders() -> Result<Vec<String>, String> {
    Ok(library_roots()?
        .iter()
        .map(|path| {
            path.file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy()
                .to_string()
        })
        .collect())
}

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            AUDIO_EXTENSIONS
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
}

fn encode_path(path: &Path) -> String {
    let encoded = path
        .to_string_lossy()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("local-{encoded}")
}

fn decode_track_path(track_id: &str) -> Result<PathBuf, String> {
    let encoded = track_id
        .strip_prefix("local-")
        .ok_or_else(|| "This is not a local audio track.".to_string())?;
    if encoded.is_empty() || encoded.len() % 2 != 0 || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("The local audio track path is invalid.".to_string());
    }
    let bytes = (0..encoded.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&encoded[index..index + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let path = String::from_utf8(bytes).map_err(|error| error.to_string())?;
    Ok(PathBuf::from(path))
}

fn scan_root(root: &Path) -> Result<Vec<Track>, String> {
    let canonical = fs::canonicalize(root).map_err(|error| error.to_string())?;
    if !canonical.is_dir() {
        return Err(format!("The saved library folder is unavailable: {}", canonical.display()));
    }
    let root_name = canonical
        .file_name()
        .unwrap_or(canonical.as_os_str())
        .to_string_lossy()
        .to_string();
    let mut pending = vec![canonical.clone()];
    let mut tracks = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| {
            format!("Could not read library folder {}: {error}", directory.display())
        })? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let metadata = entry
                .metadata()
                .map_err(|error| format!("Could not inspect {}: {error}", path.display()))?;
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() && is_audio_file(&path) {
                let relative = path.strip_prefix(&canonical).unwrap_or(&path);
                let parent = relative.parent().unwrap_or(Path::new(""));
                let folder_path = if parent.as_os_str().is_empty() {
                    root_name.clone()
                } else {
                    format!(
                        "{}/{}",
                        root_name,
                        parent.to_string_lossy().replace('\\', "/")
                    )
                };
                let title = path
                    .file_stem()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy()
                    .to_string();
                tracks.push(Track {
                    id: encode_path(&path),
                    video_id: None,
                    title,
                    artist: "Unknown artist".to_string(),
                    album: None,
                    thumbnail: None,
                    duration: None,
                    length: None,
                    source: Some("local".to_string()),
                    folder_path: Some(folder_path),
                    audio_id: None,
                });
            }
        }
    }
    tracks.sort_by(|left, right| left.title.to_lowercase().cmp(&right.title.to_lowercase()));
    Ok(tracks)
}

pub async fn scan_library() -> Result<Vec<Track>, String> {
    let roots = library_roots()?;
    tokio::task::spawn_blocking(move || {
        roots.iter().try_fold(Vec::new(), |mut all, root| {
            all.extend(scan_root(root)?);
            Ok::<_, String>(all)
        })
    })
    .await
    .map_err(|error| format!("Could not scan local audio folders: {error}"))?
}

pub async fn choose_folder() -> Result<Option<Vec<Track>>, String> {
    let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await else {
        return Ok(None);
    };
    let path = fs::canonicalize(folder.path())
        .map_err(|error| format!("The selected library folder is unavailable: {error}"))?;
    if !path.is_dir() {
        return Err("Choose an existing folder for the music library.".to_string());
    }
    let mut roots = read_settings()?
        .get("libraryFolders")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let path = path.to_string_lossy().to_string();
    if !roots.iter().any(|root| root.as_str() == Some(&path)) {
        roots.push(Value::String(path.clone()));
        write_setting("libraryFolders", Value::Array(roots))?;
    }
    let tracks = tokio::task::spawn_blocking(move || scan_root(Path::new(&path)))
        .await
        .map_err(|error| format!("Could not scan the selected library folder: {error}"))??;
    Ok(Some(tracks))
}

pub fn remove_saved_library_folder(name: &str) -> Result<(), String> {
    let mut roots = read_settings()?
        .get("libraryFolders")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    roots.retain(|root| {
        root.as_str()
            .and_then(|path| Path::new(path).file_name())
            .and_then(|value| value.to_str())
            != Some(name)
    });
    write_setting("libraryFolders", Value::Array(roots))
}

pub fn local_url(track_id: &str) -> Result<String, String> {
    let path = allowed_local_path(track_id)?;
    if !path.is_file() {
        return Err("The local audio file is no longer available.".to_string());
    }
    let encoded = track_id
        .strip_prefix("local-")
        .ok_or_else(|| "This is not a local audio track.".to_string())?;
    Ok(format!("wavelength-audio://localhost/{encoded}"))
}

pub fn has_local_audio_file(track_id: &str) -> Result<bool, String> {
    Ok(allowed_local_path(track_id)?.is_file())
}

fn allowed_local_path(track_id: &str) -> Result<PathBuf, String> {
    let path = fs::canonicalize(decode_track_path(track_id)?)
        .map_err(|_| "The local audio file is no longer available.".to_string())?;
    if !is_audio_file(&path)
        || !library_roots()?
            .iter()
            .any(|root| path.starts_with(root))
    {
        return Err("The local audio file is not in a saved library folder.".to_string());
    }
    Ok(path)
}

fn audio_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "m4a" => "audio/mp4",
        "ogg" | "opus" => "audio/ogg",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        _ => "application/octet-stream",
    }
}

fn audio_protocol_response(
    request: dioxus::desktop::wry::http::Request<Vec<u8>>,
) -> dioxus::desktop::wry::http::Response<Cow<'static, [u8]>> {
    use dioxus::desktop::wry::http::{Response, StatusCode};
    let not_found = || {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Access-Control-Allow-Origin", "*")
            .body(Cow::Borrowed(&b"Local audio file not found."[..]))
            .expect("static audio error response is valid")
    };
    let encoded = request.uri().path().trim_start_matches('/');
    let Ok(path) = allowed_local_path(&format!("local-{encoded}")) else {
        return not_found();
    };
    let Ok(bytes) = fs::read(&path) else {
        return not_found();
    };
    let total_size = bytes.len() as u64;
    let range = request
        .headers()
        .get("range")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("bytes="))
        .and_then(|value| {
            let (start, end) = value.split_once('-')?;
            let start = start.parse::<u64>().ok()?;
            let end = if end.is_empty() {
                total_size.saturating_sub(1)
            } else {
                end.parse::<u64>().ok()?.min(total_size.saturating_sub(1))
            };
            (start <= end && end < total_size).then_some((start, end))
        });
    let (status, body, content_range) = if let Some((start, end)) = range {
        (
            StatusCode::PARTIAL_CONTENT,
            bytes[start as usize..=end as usize].to_vec(),
            Some(format!("bytes {start}-{end}/{total_size}")),
        )
    } else {
        (StatusCode::OK, bytes, None)
    };
    let mut response = Response::builder()
        .status(status)
        .header("Accept-Ranges", "bytes")
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Type", audio_mime_type(&path))
        .header("Content-Length", body.len().to_string());
    if let Some(content_range) = content_range {
        response = response.header("Content-Range", content_range);
    }
    let body = if request.method() == dioxus::desktop::wry::http::Method::HEAD {
        Vec::new()
    } else {
        body
    };
    response
        .body(Cow::Owned(body))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Cow::Borrowed(&b"Could not serve local audio file."[..]))
                .expect("static audio error response is valid")
        })
}

pub async fn upload_room_track(
    url: &str,
    code: &str,
    track_id: &str,
    host_token: &str,
) -> Result<String, String> {
    let path = decode_track_path(track_id)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !AUDIO_EXTENSIONS.contains(&extension.as_str()) {
        return Err("This local audio format cannot be shared with the room.".to_string());
    }
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|error| format!("Could not read the local audio file: {error}"))?;
    let url = format!(
        "{}/api/parties/{}/audio",
        url.trim_end_matches('/'),
        url::form_urlencoded::byte_serialize(code.as_bytes())
            .collect::<String>()
            .replace('+', "%20")
    );
    let response = reqwest::Client::new()
        .post(url)
        .bearer_auth(host_token)
        .header("x-audio-extension", &extension)
        .body(bytes)
        .send()
        .await
        .map_err(|error| format!("Room audio upload failed: {error}"))?;
    let status = response.status();
    let data: Value = response
        .json()
        .await
        .map_err(|error| format!("Room server returned invalid audio upload data: {error}"))?;
    if !status.is_success() {
        return Err(data
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("Local audio could not be shared.")
            .to_string());
    }
    data.get("audioId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "Room audio upload returned an invalid ID.".to_string())
}

pub fn open_external_track(source: &str, id: &str) -> Result<(), String> {
    let url = match source {
        "spotify" if (22..=22).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric()) => {
            format!("https://open.spotify.com/track/{id}")
        }
        "youtube" if (6..=256).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')) => {
            format!("https://www.youtube.com/watch?v={id}")
        }
        _ => return Err("This track has no supported external page.".to_string()),
    };
    open::that_detached(url).map_err(|error| format!("Could not open the external track: {error}"))
}

pub fn room_invite_code() -> Result<Option<String>, String> {
    Ok(None)
}

fn keyring_entry(key: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(APP_ID, key).map_err(|error| error.to_string())
}

pub fn spotify_refresh_token() -> Result<Option<String>, String> {
    let entry = keyring_entry(SPOTIFY_TOKEN_KEY)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("Could not read the Spotify token from the system keychain: {error}")),
    }
}

pub fn set_spotify_refresh_token(token: Option<&str>) -> Result<(), String> {
    let entry = keyring_entry(SPOTIFY_TOKEN_KEY)?;
    match token {
        Some(token) if !token.is_empty() && token.len() <= 4096 => entry
            .set_password(token)
            .map_err(|error| format!("Could not save the Spotify token to the system keychain: {error}")),
        Some(_) => Err("The Spotify refresh token is invalid.".to_string()),
        None => match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("Could not remove the Spotify token from the system keychain: {error}")),
        },
    }
}

pub fn disconnect_spotify() -> Result<(), String> {
    set_spotify_refresh_token(None)
}

const SPOTIFY_CLIENT_ID: &str = "6e2ca58467e84ea5bf33b328d553c1b9";
const SPOTIFY_CALLBACK_URI: &str = "http://127.0.0.1:43821/callback";

fn spotify_callback(listener: TcpListener, expected_state: String) -> Result<String, String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("Could not prepare Spotify sign-in callback: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(600);
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut request = String::new();
                std::io::BufReader::new(&stream)
                    .read_line(&mut request)
                    .map_err(|error| format!("Could not read Spotify sign-in callback: {error}"))?;
                let callback = request
                    .split_whitespace()
                    .nth(1)
                    .and_then(|path| url::Url::parse(&format!("http://localhost{path}")).ok());
                let Some(callback) = callback else {
                    continue;
                };
                let status = callback
                    .query_pairs()
                    .find(|(key, _)| key == "error")
                    .map(|(_, value)| value.into_owned());
                if let Some(error) = status {
                    let _ = write_callback_page(&mut stream, false);
                    return Err(format!("Spotify authorization failed: {error}"));
                }
                let state = callback
                    .query_pairs()
                    .find(|(key, _)| key == "state")
                    .map(|(_, value)| value.into_owned());
                let code = callback
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned());
                if state.as_deref() != Some(expected_state.as_str()) {
                    let _ = write_callback_page(&mut stream, false);
                    return Err("Spotify sign-in could not be verified.".to_string());
                }
                let Some(code) = code.filter(|code| (1..=2048).contains(&code.len())) else {
                    let _ = write_callback_page(&mut stream, false);
                    return Err("Spotify returned an invalid authorization code.".to_string());
                };
                write_callback_page(&mut stream, true)?;
                return Ok(code);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                return Err(format!("Spotify sign-in callback failed: {error}"));
            }
        }
    }
    Err("Spotify sign-in timed out. Try connecting again.".to_string())
}

fn write_callback_page(stream: &mut TcpStream, succeeded: bool) -> Result<(), String> {
    let (status, message) = if succeeded {
        ("200 OK", "Spotify sign-in is complete. You can return to Wavelength.")
    } else {
        ("400 Bad Request", "Spotify sign-in could not be completed. Return to Wavelength and try again.")
    };
    let body = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Wavelength</title><p>{message}</p>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("Could not finish Spotify sign-in callback: {error}"))
}

#[derive(Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

pub async fn connect_spotify() -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:43821")
        .map_err(|_| "Spotify sign-in callback port 43821 is unavailable.".to_string())?;
    let verifier = URL_SAFE_NO_PAD.encode(rand::random::<[u8; 64]>());
    let state = URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>());
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let mut authorization = url::Url::parse("https://accounts.spotify.com/authorize")
        .map_err(|error| error.to_string())?;
    authorization
        .query_pairs_mut()
        .append_pair("client_id", SPOTIFY_CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", SPOTIFY_CALLBACK_URI)
        .append_pair(
            "scope",
            "user-read-private user-read-email user-library-read user-library-modify playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private",
        )
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", &challenge)
        .append_pair("state", &state);
    open::that_detached(authorization.as_str())
        .map_err(|error| format!("Could not open Spotify sign-in: {error}"))?;

    let expected_state = state;
    let code = tokio::task::spawn_blocking(move || spotify_callback(listener, expected_state))
        .await
        .map_err(|error| format!("Spotify sign-in callback failed: {error}"))??;
    let response = reqwest::Client::new()
        .post("https://accounts.spotify.com/api/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", SPOTIFY_CALLBACK_URI),
            ("client_id", SPOTIFY_CLIENT_ID),
            ("code_verifier", verifier.as_str()),
        ])
        .send()
        .await
        .map_err(|error| format!("Spotify token request failed: {error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("Spotify returned invalid token data: {error}"))?;
    if !status.is_success() {
        return Err(body
            .get("error_description")
            .and_then(Value::as_str)
            .unwrap_or("Spotify sign-in could not be completed.")
            .to_string());
    }
    let token: SpotifyTokenResponse = serde_json::from_value(body)
        .map_err(|error| format!("Spotify returned incomplete authorization data: {error}"))?;
    if token.access_token.is_empty() {
        return Err("Spotify returned an invalid access token.".to_string());
    }
    let refresh_token = token
        .refresh_token
        .ok_or_else(|| "Spotify did not return a refresh token.".to_string())?;
    set_spotify_refresh_token(Some(&refresh_token))?;
    Ok(refresh_token)
}

pub async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    let Some(manifest) = update_manifest().await? else {
        return Ok(None);
    };
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("The installed app version is invalid: {error}"))?;
    let latest = semver::Version::parse(&manifest.version)
        .map_err(|error| format!("The update feed has an invalid version: {error}"))?;
    if latest <= current {
        return Ok(None);
    }
    if !manifest.platforms.contains_key(update_platform()) {
        return Err(format!(
            "Version {} is available, but no update is published for this platform.",
            manifest.version
        ));
    }
    Ok(Some(UpdateInfo {
        version: manifest.version,
        notes: manifest.notes,
    }))
}

#[derive(Deserialize)]
struct UpdateManifest {
    version: String,
    notes: Option<String>,
    platforms: std::collections::HashMap<String, UpdateAsset>,
}

#[derive(Deserialize)]
struct UpdateAsset {
    url: String,
    signature: String,
}

const UPDATE_FEED_URL: &str =
    "https://github.com/zespook/wavelength-rust/releases/latest/download/latest.json";
const UPDATE_PUBLIC_KEY: &str = "untrusted comment: minisign public key BC156A4AB16ED218\nRWQY0m6xSmoVvDWRFYHrZlA6J/qJyZ3qf8iMv6rMnQvqrokliNaiaUSB";
const MAX_UPDATE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

fn update_platform() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "windows-x86_64";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "darwin-aarch64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "darwin-x86_64";
    #[cfg(target_os = "linux")]
    return "linux-x86_64";
    #[allow(unreachable_code)]
    "unsupported"
}

async fn update_manifest() -> Result<Option<UpdateManifest>, String> {
    let response = reqwest::Client::new()
        .get(UPDATE_FEED_URL)
        .header(reqwest::header::USER_AGENT, "Wavelength")
        .send()
        .await
        .map_err(|error| format!("Could not check the update feed: {error}"))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!(
            "The update feed returned status {}.",
            response.status()
        ));
    }
    response
        .json()
        .await
        .map(Some)
        .map_err(|error| format!("The update feed returned invalid data: {error}"))
}

async fn download_verified_update(asset: &UpdateAsset) -> Result<Vec<u8>, String> {
    let url = url::Url::parse(&asset.url)
        .map_err(|_| "The update feed contains an invalid download URL.".to_string())?;
    if url.scheme() != "https" || url.host_str() != Some("github.com") {
        return Err("The update feed contains an unsafe download URL.".to_string());
    }
    let response = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, "Wavelength")
        .send()
        .await
        .map_err(|error| format!("Could not download the update: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "The update download returned status {}.",
            response.status()
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_UPDATE_BYTES)
    {
        return Err("The update package is larger than the supported limit.".to_string());
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Could not read the update package: {error}"))?;
    if bytes.len() as u64 > MAX_UPDATE_BYTES {
        return Err("The update package is larger than the supported limit.".to_string());
    }
    let public_key = minisign_verify::PublicKey::decode(UPDATE_PUBLIC_KEY)
        .map_err(|error| format!("The embedded update verification key is invalid: {error}"))?;
    let signature = minisign_verify::Signature::decode(&asset.signature)
        .map_err(|error| format!("The update signature is invalid: {error}"))?;
    public_key
        .verify(&bytes, &signature, false)
        .map_err(|error| format!("The update signature could not be verified: {error}"))?;
    Ok(bytes.to_vec())
}

pub async fn install_update(expected_version: &str) -> Result<(), String> {
    if expected_version.trim().is_empty() || expected_version.len() > 64 {
        return Err("The requested update version is invalid.".to_string());
    }
    let manifest = update_manifest()
        .await?
        .ok_or_else(|| "No update is currently available.".to_string())?;
    if manifest.version != expected_version {
        return Err("The available update changed. Check for updates again.".to_string());
    }
    let asset = manifest
        .platforms
        .get(update_platform())
        .ok_or_else(|| "No update is published for this platform.".to_string())?;
    let bytes = download_verified_update(asset).await?;
    #[cfg(target_os = "windows")]
    install_windows_update(bytes)?;
    #[cfg(target_os = "macos")]
    install_macos_update(bytes)?;
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return Err("Automatic update installation is not supported on this platform.".to_string());
    Ok(())
}

#[cfg(target_os = "windows")]
fn install_windows_update(bytes: Vec<u8>) -> Result<(), String> {
    let temp_dir = env::temp_dir();
    let installer = temp_dir.join(format!("Wavelength-update-{}.exe", std::process::id()));
    fs::write(&installer, bytes)
        .map_err(|error| format!("Could not save the verified installer: {error}"))?;
    let script_path = temp_dir.join(format!("Wavelength-update-{}.ps1", std::process::id()));
    fs::write(
        &script_path,
        r#"
param([int]$ProcessId, [string]$InstallerPath, [string]$AppPath)
while (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue) { Start-Sleep -Milliseconds 250 }
Start-Process -FilePath $InstallerPath -ArgumentList '/S' -Wait
if (Test-Path $AppPath) { Start-Process -FilePath $AppPath }
Remove-Item -LiteralPath $InstallerPath -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
"#,
    )
    .map_err(|error| format!("Could not prepare the update installer: {error}"))?;
    let app = env::current_exe().map_err(|error| error.to_string())?;
    let powershell = env::var_os("SystemRoot")
        .map(PathBuf::from)
        .map(|root| root.join("System32").join("WindowsPowerShell").join("v1.0").join("powershell.exe"))
        .unwrap_or_else(|| PathBuf::from("powershell.exe"));
    Command::new(powershell)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
        ])
        .arg(&script_path)
        .arg("-ProcessId")
        .arg(std::process::id().to_string())
        .arg("-InstallerPath")
        .arg(&installer)
        .arg("-AppPath")
        .arg(app)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not launch the update installer: {error}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn install_macos_update(bytes: Vec<u8>) -> Result<(), String> {
    let temp_dir = env::temp_dir().join(format!("wavelength-update-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("Could not prepare the update directory: {error}"))?;
    let archive_path = temp_dir.join("update.tar.gz");
    fs::write(&archive_path, bytes)
        .map_err(|error| format!("Could not save the verified update: {error}"))?;
    let archive = fs::File::open(&archive_path)
        .map_err(|error| format!("Could not open the verified update: {error}"))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(archive));
    archive
        .unpack(&temp_dir)
        .map_err(|error| format!("Could not unpack the verified update: {error}"))?;
    let replacement = fs::read_dir(&temp_dir)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .ok_or_else(|| "The update archive does not contain a macOS app bundle.".to_string())?;
    let executable = env::current_exe().map_err(|error| error.to_string())?;
    let installed_app = executable
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| "The installed app bundle path is unavailable.".to_string())?;
    let script_path = temp_dir.join("install-update.sh");
    let script = format!(
        "#!/bin/sh\nset -eu\nwhile kill -0 {} 2>/dev/null; do sleep 0.25; done\nreplacement={}\ntarget={}\nbackup=\"${{target}}.old\"\nrm -rf -- \"$backup\"\nif [ -e \"$target\" ]; then mv -- \"$target\" \"$backup\"; fi\nif ! ditto \"$replacement\" \"$target\"; then if [ -e \"$backup\" ]; then mv -- \"$backup\" \"$target\"; fi; exit 1; fi\nopen \"$target\"\nrm -rf -- \"$backup\"\nrm -rf -- {}\n",
        std::process::id(),
        shell_quote(&replacement.to_string_lossy()),
        shell_quote(&installed_app.to_string_lossy()),
        shell_quote(&temp_dir.to_string_lossy())
    );
    fs::write(&script_path, script)
        .map_err(|error| format!("Could not prepare the macOS update installer: {error}"))?;
    let script = format!("do shell script {} with administrator privileges", applescript_quote(&format!("/bin/sh {}", shell_quote(&script_path.to_string_lossy()))));
    Command::new("osascript")
        .args(["-e", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not launch the macOS update installer: {error}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
