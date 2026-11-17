use crate::{Track, UpdateInfo};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::Element,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const APP_ID: &str = "com.wavelength.desktop";
const APPLICATION_ICON_RESOURCE_ID: u16 = 32512;
const AUDIO_PORT: u16 = 8787;
const SPOTIFY_TOKEN_KEY: &str = "spotify-refresh-token";
const ROOM_SERVER_PASSCODE_KEY: &str = "room-server-passcode";
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "ogg", "wav", "aac", "opus"];
const SERVICE_REPOSITORY: &str = "zespook/wavelength-rust";
const MAX_SERVICE_PACKAGE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_SERVICE_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const SERVICE_PACKAGE_SHA256: Option<&str> = option_env!("WAVELENGTH_SERVICES_SHA256");
const SERVICE_PACKAGE_OVERRIDE: Option<&str> = option_env!("WAVELENGTH_SERVICES_ARCHIVE");
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
    let logical = window.inner_size().to_logical::<f64>(window.scale_factor());
    if !(750.0..=8192.0).contains(&logical.width) || !(600.0..=8192.0).contains(&logical.height) {
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

#[cfg(target_os = "windows")]
fn load_window_icon() -> Result<Option<dioxus::desktop::tao::window::Icon>, String> {
    use dioxus::desktop::tao::platform::windows::IconExtWindows;

    dioxus::desktop::tao::window::Icon::from_resource(APPLICATION_ICON_RESOURCE_ID, None)
        .map(Some)
        .map_err(|error| format!("Could not load the Wavelength app icon: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn load_window_icon() -> Result<Option<dioxus::desktop::tao::window::Icon>, String> {
    Ok(None)
}

pub fn run(app: fn() -> Element) {
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.first().and_then(|argument| argument.to_str()) == Some("--install-services") {
        let (system_install, archive_override) =
            match parse_service_install_arguments(&arguments[1..]) {
                Ok(arguments) => arguments,
                Err(error) => {
                    eprintln!("Could not install Wavelength services: {error}");
                    std::process::exit(2);
                }
            };
        if let Err(error) =
            install_services_for_installer(system_install, archive_override.as_deref())
        {
            eprintln!("Could not install Wavelength services: {error}");
            std::process::exit(1);
        }
        return;
    }

    let result = initialize();
    if let Err(error) = result {
        show_startup_error(&error);
        std::process::exit(1);
    }
    let state = match app_state() {
        Ok(state) => state,
        Err(error) => {
            show_startup_error(&error);
            std::process::exit(1);
        }
    };
    let settings = match read_settings() {
        Ok(settings) => settings,
        Err(error) => {
            show_startup_error(&error);
            std::process::exit(1);
        }
    };
    let window_icon = match load_window_icon() {
        Ok(icon) => icon,
        Err(error) => {
            show_startup_error(&error);
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
            show_startup_error(&error);
            std::process::exit(1);
        }
    };
    let service_base = serde_json::to_string(&service_base)
        .expect("serializing the local service URL cannot fail");
    let browser_source = include_str!("browser.js")
        .replace("export function ", "function ")
        .replace("export async function ", "async function ");
    let browser_script = format!("(() => {{\n{browser_source}\n}})();");
    let config = Config::new()
        .with_resource_directory(&state.resource_dir)
        .with_data_directory(state.data_dir.join("webview"))
        .with_disable_context_menu(false)
        .with_custom_head(format!(
            "<script>globalThis.__WAVELENGTH_NATIVE__=true;globalThis.__WAVELENGTH_AUDIO_SERVICE__={service_base};</script><script>{browser_script}</script>",
        ))
        .with_asynchronous_custom_protocol("wavelength-audio", |_id, request, responder| {
            responder.respond(audio_protocol_response(request));
        })
        .with_window(
            WindowBuilder::new()
                .with_title("Wavelength")
                .with_window_icon(window_icon)
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
            if matches!(
                event,
                dioxus::desktop::tao::event::Event::WindowEvent {
                    event: dioxus::desktop::WindowEvent::CloseRequested
                        | dioxus::desktop::WindowEvent::Resized(_),
                    ..
                }
            ) {
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

fn show_startup_error(error: &str) {
    eprintln!("Could not start Wavelength: {error}");
    rfd::MessageDialog::new()
        .set_title("Wavelength could not start")
        .set_description(error)
        .set_level(rfd::MessageLevel::Error)
        .show();
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
    let executable = service_executable(&services_dir);
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
    let path =
        env::join_paths(std::iter::once(runtime_dir).chain(env::split_paths(&inherited_path)))
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
    if cfg!(debug_assertions) {
        return Ok(development_dir);
    }

    #[cfg(target_os = "linux")]
    if service_directory_is_current(Path::new("/usr/lib/wavelength/services")) {
        return Ok(PathBuf::from("/usr/lib/wavelength/services"));
    }

    let installed_dir = state.resource_dir.join("services");
    if service_directory_is_current(&installed_dir) {
        return Ok(installed_dir);
    }

    let extraction_dir = state.data_dir.join("bundled-services");
    if service_directory_is_current(&extraction_dir) {
        return Ok(extraction_dir);
    }

    ensure_service_installation(&state.data_dir, None, None, true)
}

fn ensure_service_installation(
    data_dir: &Path,
    destination_override: Option<&Path>,
    archive_override: Option<&Path>,
    show_progress: bool,
) -> Result<PathBuf, String> {
    let destination = destination_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| data_dir.join("bundled-services"));
    if service_directory_is_current(&destination) {
        return Ok(destination);
    }

    if show_progress {
        rfd::MessageDialog::new()
            .set_title("Preparing Wavelength")
            .set_description(
                "Wavelength is preparing its local audio services. An internet connection is only needed if the matching package is not available on this device.",
            )
            .set_level(rfd::MessageLevel::Info)
            .show();
    }

    let download_dir = destination.parent().unwrap_or(data_dir);
    fs::create_dir_all(download_dir).map_err(|error| {
        format!("Could not prepare the service installation directory: {error}")
    })?;
    let staging_dir = destination.with_file_name(format!(
        "{}-{}.staging",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("wavelength-services"),
        std::process::id()
    ));
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)
            .map_err(|error| format!("Could not clear the incomplete service download: {error}"))?;
    }
    fs::create_dir_all(&staging_dir)
        .map_err(|error| format!("Could not prepare the service download: {error}"))?;

    let (archive_path, is_temporary_archive) =
        match download_service_package(download_dir, archive_override) {
            Ok(package) => package,
            Err(error) => return cleanup_failed_service_install(&staging_dir, None, error),
        };
    let install_result = (|| {
        extract_service_package(&archive_path, &staging_dir)?;
        if !service_executable(&staging_dir).is_file() {
            return Err("The downloaded package is missing the local audio service.".to_string());
        }
        if !service_runtime_executable(&staging_dir).is_file() {
            return Err("The downloaded package is missing its Node.js runtime.".to_string());
        }
        let package_hash = service_package_sha256()?;
        fs::write(
            staging_dir.join(".wavelength-service-version"),
            format!("{}\n{package_hash}\n", env!("CARGO_PKG_VERSION")),
        )
        .map_err(|error| format!("Could not mark the installed service version: {error}"))?;
        replace_service_directory(&staging_dir, &destination)
    })();
    if let Err(error) = install_result {
        return cleanup_failed_service_install(
            &staging_dir,
            is_temporary_archive.then_some(archive_path.as_path()),
            error,
        );
    }
    if is_temporary_archive {
        fs::remove_file(&archive_path).map_err(|error| {
            format!(
                "Services were installed, but the downloaded archive could not be removed: {error}"
            )
        })?;
    }
    Ok(destination)
}

fn service_directory_is_current(directory: &Path) -> bool {
    if !service_executable(directory).is_file() || !service_runtime_executable(directory).is_file()
    {
        return false;
    }
    let Ok(marker) = fs::read_to_string(directory.join(".wavelength-service-version")) else {
        return false;
    };
    let mut lines = marker.lines();
    lines
        .next()
        .is_some_and(|version| version == env!("CARGO_PKG_VERSION"))
        && service_package_sha256().is_ok_and(|expected| {
            lines
                .next()
                .is_some_and(|installed| installed.eq_ignore_ascii_case(&expected))
        })
}

fn install_services_for_installer(
    system_install: bool,
    archive_override: Option<&Path>,
) -> Result<(), String> {
    if system_install {
        #[cfg(target_os = "linux")]
        {
            ensure_service_installation(
                Path::new("/usr/lib/wavelength"),
                Some(Path::new("/usr/lib/wavelength/services")),
                archive_override,
                false,
            )?;
            return Ok(());
        }
        #[cfg(not(target_os = "linux"))]
        {
            return Err("System-wide service installation is only available on Linux.".to_string());
        }
    }

    let data_dir = project_dirs()?.data_dir().to_path_buf();
    ensure_service_installation(&data_dir, None, archive_override, false)?;
    Ok(())
}

fn parse_service_install_arguments(
    arguments: &[std::ffi::OsString],
) -> Result<(bool, Option<PathBuf>), String> {
    let mut system_install = false;
    let mut archive_override = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].to_str() {
            Some("--system") if !system_install => {
                system_install = true;
                index += 1;
            }
            Some("--archive") if archive_override.is_none() => {
                let path = arguments
                    .get(index + 1)
                    .filter(|path| !path.is_empty())
                    .ok_or_else(|| "expected a path after --archive.".to_string())?;
                archive_override = Some(PathBuf::from(path));
                index += 2;
            }
            _ => return Err("unexpected installer arguments.".to_string()),
        }
    }
    Ok((system_install, archive_override))
}

fn service_executable(services_dir: &Path) -> PathBuf {
    services_dir
        .join("api")
        .join(if cfg!(target_os = "windows") {
            "WavelengthLocalService.exe"
        } else {
            "WavelengthLocalService"
        })
}

fn service_runtime_executable(services_dir: &Path) -> PathBuf {
    services_dir
        .join("runtime")
        .join(if cfg!(target_os = "windows") {
            "node.exe"
        } else {
            "node"
        })
}

fn service_package_sha256() -> Result<&'static str, String> {
    let hash = SERVICE_PACKAGE_SHA256.ok_or_else(|| {
        "This app build has no verified service package configured. Rebuild it with the release packaging workflow."
            .to_string()
    })?;
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("This app build has an invalid service package checksum.".to_string());
    }
    Ok(hash)
}

fn service_package_platform() -> Result<&'static str, String> {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Ok("windows-x86_64");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return Ok("linux-x86_64");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return Ok("darwin-x86_64");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Ok("darwin-aarch64");
    #[allow(unreachable_code)]
    Err("Compiled service packages are not available for this platform.".to_string())
}

fn service_package_asset_name() -> Result<String, String> {
    Ok(format!(
        "Wavelength_{}_services_{}.zip",
        env!("CARGO_PKG_VERSION"),
        service_package_platform()?
    ))
}

fn download_service_package(
    data_dir: &Path,
    archive_override: Option<&Path>,
) -> Result<(PathBuf, bool), String> {
    let expected_hash = service_package_sha256()?;
    if let Some(path) = archive_override.or_else(|| SERVICE_PACKAGE_OVERRIDE.map(Path::new)) {
        match verify_service_package_hash(path, &expected_hash) {
            Ok(()) => return Ok((path.to_path_buf(), false)),
            Err(error) => {
                eprintln!(
                    "The local compiled services package at '{}' could not be used: {error}. Downloading the matching release package instead.",
                    path.display()
                );
                let local_error = error;
                return download_service_package_from_release(data_dir, &expected_hash)
                    .map(|path| (path, true))
                    .map_err(|error| format!("{error} (local package rejected: {local_error})"));
            }
        }
    }

    download_service_package_from_release(data_dir, &expected_hash).map(|path| (path, true))
}

fn download_service_package_from_release(
    data_dir: &Path,
    expected_hash: &str,
) -> Result<PathBuf, String> {
    fs::create_dir_all(data_dir)
        .map_err(|error| format!("Could not prepare the service download directory: {error}"))?;
    let archive_path = data_dir.join(format!(
        "wavelength-services-{}.zip.part",
        std::process::id()
    ));
    if archive_path.exists() {
        fs::remove_file(&archive_path)
            .map_err(|error| format!("Could not clear a previous service download: {error}"))?;
    }

    let result = (|| {
        let asset_name = service_package_asset_name()?;
        let url = format!(
            "https://github.com/{SERVICE_REPOSITORY}/releases/download/v{}/{asset_name}",
            env!("CARGO_PKG_VERSION")
        );
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30 * 60))
            .user_agent(format!("Wavelength/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| format!("Could not prepare the service download: {error}"))?;
        let mut response = client
            .get(url)
            .send()
            .map_err(|error| format!("Could not download the compiled services: {error}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "The compiled service download failed with status {}.",
                response.status()
            ));
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_SERVICE_PACKAGE_BYTES)
        {
            return Err(
                "The compiled service package exceeds the allowed download size.".to_string(),
            );
        }

        let mut file = File::create(&archive_path)
            .map_err(|error| format!("Could not save the compiled service package: {error}"))?;
        let mut hasher = Sha256::new();
        let mut total_bytes = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = response.read(&mut buffer).map_err(|error| {
                format!("The compiled service download was interrupted: {error}")
            })?;
            if count == 0 {
                break;
            }
            total_bytes = total_bytes.saturating_add(count as u64);
            if total_bytes > MAX_SERVICE_PACKAGE_BYTES {
                return Err(
                    "The compiled service package exceeds the allowed download size.".to_string(),
                );
            }
            file.write_all(&buffer[..count])
                .map_err(|error| format!("Could not save the compiled service package: {error}"))?;
            hasher.update(&buffer[..count]);
        }
        if total_bytes == 0 {
            return Err("The compiled service download was empty.".to_string());
        }
        file.sync_all().map_err(|error| {
            format!("Could not finish saving the compiled service package: {error}")
        })?;
        let actual_hash = format!("{:x}", hasher.finalize());
        if !actual_hash.eq_ignore_ascii_case(expected_hash) {
            return Err(
                "The compiled service package failed its SHA-256 integrity check.".to_string(),
            );
        }
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(cleanup_error) = fs::remove_file(&archive_path) {
            if cleanup_error.kind() != std::io::ErrorKind::NotFound {
                return Err(format!(
                    "{error}; could not remove the incomplete service download: {cleanup_error}"
                ));
            }
        }
        return Err(error);
    }
    Ok(archive_path)
}

fn verify_service_package_hash(path: &Path, expected_hash: &str) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("The configured service package is unavailable: {error}"))?;
    if metadata.len() > MAX_SERVICE_PACKAGE_BYTES {
        return Err("The compiled service package exceeds the allowed download size.".to_string());
    }
    let mut file = File::open(path)
        .map_err(|error| format!("Could not open the compiled service package: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not verify the compiled service package: {error}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let actual_hash = format!("{:x}", hasher.finalize());
    if !actual_hash.eq_ignore_ascii_case(expected_hash) {
        return Err("The compiled service package failed its SHA-256 integrity check.".to_string());
    }
    Ok(())
}

fn extract_service_package(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let archive = File::open(archive_path)
        .map_err(|error| format!("Could not open the compiled service package: {error}"))?;
    let mut archive = zip::ZipArchive::new(archive).map_err(|error| {
        format!("The compiled service package is not a valid ZIP archive: {error}")
    })?;
    let mut total_unpacked = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("Could not read the compiled service package: {error}"))?;
        let relative_path = entry
            .enclosed_name()
            .ok_or_else(|| "The compiled service package contains an unsafe path.".to_string())?
            .to_path_buf();
        if relative_path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err("The compiled service package contains an unsafe path.".to_string());
        }
        let output_path = destination.join(&relative_path);
        if entry.is_dir() {
            fs::create_dir_all(&output_path).map_err(|error| {
                format!("Could not extract the compiled service package: {error}")
            })?;
            continue;
        }
        let mode = entry.unix_mode();
        if mode.is_some_and(|mode| mode & 0o170000 == 0o120000) {
            return Err(
                "The compiled service package contains an unsupported symbolic link.".to_string(),
            );
        }
        let expected_size = entry.size();
        total_unpacked = total_unpacked
            .checked_add(expected_size)
            .ok_or_else(|| "The compiled service package is too large to extract.".to_string())?;
        if total_unpacked > MAX_SERVICE_UNPACKED_BYTES {
            return Err(
                "The compiled service package expands beyond the allowed size.".to_string(),
            );
        }
        let parent = output_path
            .parent()
            .ok_or_else(|| "The compiled service package contains an invalid path.".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not extract the compiled service package: {error}"))?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output_path)
            .map_err(|error| format!("Could not extract a service package file: {error}"))?;
        let remaining_size = MAX_SERVICE_UNPACKED_BYTES - (total_unpacked - expected_size);
        let copied = io::copy(&mut (&mut entry).take(remaining_size + 1), &mut output)
            .map_err(|error| format!("Could not extract a service package file: {error}"))?;
        if copied > remaining_size {
            return Err(
                "The compiled service package expands beyond the allowed size.".to_string(),
            );
        }
        if copied != expected_size {
            return Err("A file in the compiled service package was incomplete.".to_string());
        }
        #[cfg(unix)]
        if let Some(mode) = mode {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&output_path, fs::Permissions::from_mode(mode & 0o777))
                .map_err(|error| format!("Could not set service file permissions: {error}"))?;
        }
    }
    Ok(())
}

fn cleanup_failed_service_install(
    staging_dir: &Path,
    archive_path: Option<&Path>,
    error: String,
) -> Result<PathBuf, String> {
    let mut cleanup_errors = Vec::new();
    if staging_dir.exists() {
        if let Err(cleanup_error) = fs::remove_dir_all(staging_dir) {
            cleanup_errors.push(format!(
                "could not remove incomplete service files: {cleanup_error}"
            ));
        }
    }
    if let Some(archive_path) = archive_path {
        if let Err(cleanup_error) = fs::remove_file(archive_path) {
            if cleanup_error.kind() != std::io::ErrorKind::NotFound {
                cleanup_errors.push(format!(
                    "could not remove the downloaded archive: {cleanup_error}"
                ));
            }
        }
    }
    if cleanup_errors.is_empty() {
        Err(error)
    } else {
        Err(format!("{error}; {}", cleanup_errors.join("; ")))
    }
}

fn replace_service_directory(staging: &Path, destination: &Path) -> Result<(), String> {
    let backup = destination.with_file_name("bundled-services.previous");
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(|error| {
            format!("Could not clear the previous service installation: {error}")
        })?;
    }
    let had_existing = destination.exists();
    if had_existing {
        fs::rename(destination, &backup).map_err(|error| {
            format!("Could not replace the previous service installation: {error}")
        })?;
    }
    if let Err(error) = fs::rename(staging, destination) {
        if had_existing {
            fs::rename(&backup, destination).map_err(|restore_error| {
                format!(
                    "Could not install the service build ({error}) or restore the previous installation ({restore_error})."
                )
            })?;
        }
        return Err(format!("Could not install the service build: {error}"));
    }
    if had_existing {
        fs::remove_dir_all(&backup).map_err(|error| {
            format!("Could not remove the previous service installation: {error}")
        })?;
    }
    Ok(())
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
        if !matches!(status, Ok(status) if status.success())
            && child.try_wait().ok().flatten().is_none()
        {
            return Err(format!(
                "Could not stop the local service process tree: {status:?}"
            ));
        }
    }
    if child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_none()
    {
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
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
            {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    let base = base
        .trim_matches([' ', '.'])
        .chars()
        .take(160)
        .collect::<String>();
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
    directory.join(format!(
        "{base}-{}.{}",
        chrono::Utc::now().timestamp_millis(),
        extension
    ))
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
        return Err(format!(
            "Download failed with status {}.",
            response.status()
        ));
    }
    let extension = audio_extension(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or(""),
    );
    let destination = unique_download_path(&directory, &base, extension);
    let temporary = destination.with_extension(format!("{extension}.{}.part", std::process::id()));
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
    if encoded.is_empty()
        || encoded.len() % 2 != 0
        || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
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
        return Err(format!(
            "The saved library folder is unavailable: {}",
            canonical.display()
        ));
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
            format!(
                "Could not read library folder {}: {error}",
                directory.display()
            )
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
    Ok(local_audio_protocol_url(encoded))
}

fn local_audio_protocol_url(encoded: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        // Wry's WebView2 workaround only rewrites custom schemes for top-level navigation.
        format!("http://wavelength-audio.localhost/{encoded}")
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("wavelength-audio://localhost/{encoded}")
    }
}

pub fn has_local_audio_file(track_id: &str) -> Result<bool, String> {
    Ok(allowed_local_path(track_id)?.is_file())
}

fn allowed_local_path(track_id: &str) -> Result<PathBuf, String> {
    let path = fs::canonicalize(decode_track_path(track_id)?)
        .map_err(|_| "The local audio file is no longer available.".to_string())?;
    if !is_audio_file(&path) || !library_roots()?.iter().any(|root| path.starts_with(root)) {
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
    response.body(Cow::Owned(body)).unwrap_or_else(|_| {
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
        "{}/api/v2/rooms/{}/audio",
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
        "spotify"
            if (22..=22).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric()) =>
        {
            format!("https://open.spotify.com/track/{id}")
        }
        "youtube"
            if (6..=256).contains(&id.len())
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')) =>
        {
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

pub fn room_server_passcode() -> Result<Option<String>, String> {
    let entry = keyring_entry(ROOM_SERVER_PASSCODE_KEY)?;
    match entry.get_password() {
        Ok(passcode) if !passcode.is_empty() => Ok(Some(passcode)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => match load_local::<String>("wavelength-room-passcode")? {
            Some(passcode) if !passcode.is_empty() => {
                set_room_server_passcode(Some(&passcode))?;
                remove_local("wavelength-room-passcode")?;
                Ok(Some(passcode))
            }
            Some(_) => {
                remove_local("wavelength-room-passcode")?;
                Ok(None)
            }
            None => Ok(None),
        },
        Err(error) => Err(format!(
            "Could not read the room server passcode from the system keychain: {error}"
        )),
    }
}

pub fn set_room_server_passcode(passcode: Option<&str>) -> Result<(), String> {
    let entry = keyring_entry(ROOM_SERVER_PASSCODE_KEY)?;
    match passcode.filter(|passcode| !passcode.is_empty()) {
        Some(passcode) if passcode.len() <= 128 => entry.set_password(passcode).map_err(|error| {
            format!("Could not save the room server passcode to the system keychain: {error}")
        }),
        Some(_) => Err("The room server passcode is invalid.".to_string()),
        None => match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!(
                "Could not remove the room server passcode from the system keychain: {error}"
            )),
        },
    }
}

pub fn spotify_refresh_token() -> Result<Option<String>, String> {
    let entry = keyring_entry(SPOTIFY_TOKEN_KEY)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!(
            "Could not read the Spotify token from the system keychain: {error}"
        )),
    }
}

pub fn set_spotify_refresh_token(token: Option<&str>) -> Result<(), String> {
    let entry = keyring_entry(SPOTIFY_TOKEN_KEY)?;
    match token {
        Some(token) if !token.is_empty() && token.len() <= 4096 => {
            entry.set_password(token).map_err(|error| {
                format!("Could not save the Spotify token to the system keychain: {error}")
            })
        }
        Some(_) => Err("The Spotify refresh token is invalid.".to_string()),
        None => match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!(
                "Could not remove the Spotify token from the system keychain: {error}"
            )),
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
        (
            "200 OK",
            "Spotify sign-in is complete. You can return to Wavelength.",
        )
    } else {
        (
            "400 Bad Request",
            "Spotify sign-in could not be completed. Return to Wavelength and try again.",
        )
    };
    let body =
        format!("<!doctype html><meta charset=\"utf-8\"><title>Wavelength</title><p>{message}</p>");
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
    #[cfg(target_os = "linux")]
    install_linux_update(bytes)?;
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
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
        .map(|root| {
            root.join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe")
        })
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
        "#!/bin/sh\nset -eu\nwhile kill -0 {} 2>/dev/null; do sleep 0.25; done\nreplacement={}\ntarget={}\nbackup=\"${{target}}.old\"\nrm -rf -- \"$backup\"\nif [ -e \"$target\" ]; then mv -- \"$target\" \"$backup\"; fi\nif ! ditto \"$replacement\" \"$target\"; then if [ -e \"$backup\" ]; then mv -- \"$backup\" \"$target\"; fi; exit 1; fi\nconsole_user=$(stat -f %Su /dev/console)\nif [ \"$console_user\" != root ]; then launchctl asuser \"$(id -u \"$console_user\")\" /usr/bin/open \"$target\"; fi\nrm -rf -- \"$backup\"\nrm -rf -- {}\n",
        std::process::id(),
        shell_quote(&replacement.to_string_lossy()),
        shell_quote(&installed_app.to_string_lossy()),
        shell_quote(&temp_dir.to_string_lossy())
    );
    fs::write(&script_path, script)
        .map_err(|error| format!("Could not prepare the macOS update installer: {error}"))?;
    let script = format!(
        "do shell script {} with administrator privileges",
        applescript_quote(&format!(
            "/bin/sh {}",
            shell_quote(&script_path.to_string_lossy())
        ))
    );
    Command::new("osascript")
        .args(["-e", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not launch the macOS update installer: {error}"))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_linux_update(bytes: Vec<u8>) -> Result<(), String> {
    let appimage = env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .ok_or_else(|| "The running AppImage path is unavailable.".to_string())?;
    let appimage = fs::canonicalize(appimage)
        .map_err(|error| format!("Could not locate the installed AppImage: {error}"))?;
    let parent = appimage
        .parent()
        .ok_or_else(|| "The installed AppImage directory is unavailable.".to_string())?;
    let staging_dir = parent.join(format!(".wavelength-update-{}", std::process::id()));
    fs::create_dir_all(&staging_dir)
        .map_err(|error| format!("Could not prepare the update directory: {error}"))?;
    let replacement = staging_dir.join("Wavelength.AppImage");
    fs::write(&replacement, bytes)
        .map_err(|error| format!("Could not save the verified AppImage update: {error}"))?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&replacement, fs::Permissions::from_mode(0o755))
        .map_err(|error| format!("Could not make the updated AppImage executable: {error}"))?;
    let script_path = staging_dir.join("install-update.sh");
    let script = format!(
        "#!/bin/sh\nset -eu\nwhile kill -0 {} 2>/dev/null; do sleep 0.25; done\nreplacement={}\ntarget={}\nstaging={}\nbackup=\"$staging/previous.AppImage\"\nif ! mv -- \"$target\" \"$backup\"; then exit 1; fi\nif ! mv -- \"$replacement\" \"$target\"; then mv -- \"$backup\" \"$target\"; exit 1; fi\nif ! chmod +x \"$target\"; then rm -f -- \"$target\"; mv -- \"$backup\" \"$target\"; exit 1; fi\n\"$target\" >/dev/null 2>&1 &\nrm -rf -- \"$staging\"\n",
        std::process::id(),
        shell_quote(&replacement.to_string_lossy()),
        shell_quote(&appimage.to_string_lossy()),
        shell_quote(&staging_dir.to_string_lossy())
    );
    fs::write(&script_path, script)
        .map_err(|error| format!("Could not prepare the AppImage update installer: {error}"))?;
    Command::new("sh")
        .arg(&script_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not launch the AppImage update installer: {error}"))?;
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::{
        extract_service_package, local_audio_protocol_url, parse_service_install_arguments,
        verify_service_package_hash,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        io::Write,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zip::write::SimpleFileOptions;

    fn test_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("wavelength-{name}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn local_audio_uses_the_platform_protocol_url() {
        #[cfg(target_os = "windows")]
        assert_eq!(
            local_audio_protocol_url("0123abcd"),
            "http://wavelength-audio.localhost/0123abcd"
        );

        #[cfg(not(target_os = "windows"))]
        assert_eq!(
            local_audio_protocol_url("0123abcd"),
            "wavelength-audio://localhost/0123abcd"
        );
    }

    #[test]
    fn service_archive_extraction_rejects_parent_paths() {
        let directory = test_directory("unsafe-service-archive");
        fs::create_dir_all(&directory).unwrap();
        let archive_path = directory.join("services.zip");
        let file = fs::File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file("../outside", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"unsafe").unwrap();
        archive.finish().unwrap();

        let result = extract_service_package(&archive_path, &directory.join("services"));
        assert!(result.unwrap_err().contains("unsafe path"));
        assert!(!directory.join("outside").exists());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn service_archive_extraction_preserves_executable_files() {
        let directory = test_directory("service-archive");
        fs::create_dir_all(&directory).unwrap();
        let archive_path = directory.join("services.zip");
        let file = fs::File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        let executable_options = SimpleFileOptions::default().unix_permissions(0o755);
        archive
            .start_file("api/WavelengthLocalService", executable_options)
            .unwrap();
        archive.write_all(b"service").unwrap();
        archive
            .start_file("runtime/node", executable_options)
            .unwrap();
        archive.write_all(b"node").unwrap();
        archive.finish().unwrap();

        let destination = directory.join("services");
        extract_service_package(&archive_path, &destination).unwrap();
        assert_eq!(
            fs::read(destination.join("api/WavelengthLocalService")).unwrap(),
            b"service"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(destination.join("runtime/node"))
                .unwrap()
                .permissions()
                .mode();
            assert_ne!(mode & 0o111, 0);
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn service_archive_hash_must_match_the_embedded_checksum() {
        let directory = test_directory("service-archive-hash");
        fs::create_dir_all(&directory).unwrap();
        let archive_path = directory.join("services.zip");
        fs::write(&archive_path, b"compiled services").unwrap();
        let expected = format!("{:x}", Sha256::digest(b"compiled services"));

        assert!(verify_service_package_hash(&archive_path, &expected).is_ok());
        assert!(
            verify_service_package_hash(&archive_path, &"0".repeat(64))
                .unwrap_err()
                .contains("SHA-256")
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn installer_arguments_accept_a_local_service_archive() {
        let arguments = [
            std::ffi::OsString::from("--archive"),
            std::ffi::OsString::from(r"C:\packages\services.zip"),
            std::ffi::OsString::from("--system"),
        ];

        let (system_install, archive) = parse_service_install_arguments(&arguments).unwrap();
        assert!(system_install);
        assert_eq!(
            archive,
            Some(PathBuf::from(r"C:\packages\services.zip"))
        );
    }

    #[test]
    fn installer_arguments_reject_a_missing_archive_path() {
        let arguments = [std::ffi::OsString::from("--archive")];

        assert!(
            parse_service_install_arguments(&arguments)
                .unwrap_err()
                .contains("path after --archive")
        );
    }
}
