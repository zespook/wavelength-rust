#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
	env,
	fs,
	io::{self, BufRead, Write},
	net::{SocketAddr, TcpListener, TcpStream},
	path::PathBuf,
	process::{Child, Command, Stdio},
	sync::Mutex,
	thread,
	time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use url::Url;

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
	app.path().app_config_dir().map(|directory| directory.join("settings.json")).map_err(|error| error.to_string())
}

fn read_server_url(app: &AppHandle) -> Result<Option<String>, String> {
	let path = settings_path(app)?;
	let contents = match fs::read_to_string(path) {
		Ok(contents) => contents,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
		Err(error) => return Err(error.to_string()),
	};
	let settings: serde_json::Value = serde_json::from_str(&contents).map_err(|_| "Saved server settings could not be read.".to_string())?;
	Ok(settings.get("serverUrl").and_then(serde_json::Value::as_str).map(str::to_string))
}

fn read_settings(app: &AppHandle) -> Result<serde_json::Value, String> {
	let path = settings_path(app)?;
	match fs::read_to_string(path) {
		Ok(contents) => serde_json::from_str(&contents).map_err(|_| "Saved settings could not be read.".to_string()),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
		Err(error) => Err(error.to_string()),
	}
}

fn write_setting(app: &AppHandle, key: &str, value: serde_json::Value) -> Result<(), String> {
	let path = settings_path(app)?;
	let directory = path.parent().ok_or_else(|| "Settings location is unavailable.".to_string())?;
	fs::create_dir_all(directory).map_err(|error| error.to_string())?;
	let mut settings = read_settings(app)?;
	let object = settings.as_object_mut().ok_or_else(|| "Saved settings are invalid.".to_string())?;
	object.insert(key.to_string(), value);
	fs::write(path, settings.to_string()).map_err(|error| error.to_string())
}

fn normalize_server_url(value: &str) -> Result<Url, String> {
	let url = Url::parse(value.trim()).map_err(|_| "Enter a valid server address.".to_string())?;
	if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
		return Err("Use an HTTP or HTTPS server address without credentials.".into());
	}
	if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
		return Err("Enter the server address without a path, query, or fragment.".into());
	}
	let local_http = url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1" | "[::1]"));
	if url.scheme() == "http" && !local_http {
		return Err("Use HTTPS for servers outside this device.".into());
	}
	Ok(url)
}

#[tauri::command]
fn get_server_url(app: AppHandle, window: WebviewWindow) -> Result<Option<String>, String> {
	if window.label() != "player" { return Err("This command is only available to the player window.".into()); }
	read_server_url(&app)
}

#[tauri::command]
fn set_server_url(app: AppHandle, window: WebviewWindow, server_url: String) -> Result<Option<String>, String> {
	if window.label() != "player" { return Err("This command is only available to the player window.".into()); }
	let server_url = if server_url.trim().is_empty() { None } else { Some(normalize_server_url(&server_url)?) };
	let value = server_url.as_ref().map(Url::as_str).unwrap_or("");
	write_setting(&app, "serverUrl", serde_json::json!(value))?;
	Ok(server_url.map(|url| url.origin().ascii_serialization()))
}

#[tauri::command]
fn get_room_server_passcode(window: WebviewWindow) -> Result<Option<String>, String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	let entry = keyring::Entry::new("com.wavelength.desktop", "room-server-passcode")
		.map_err(|error| error.to_string())?;
	match entry.get_password() {
		Ok(passcode) => Ok(Some(passcode)),
		Err(keyring::Error::NoEntry) => Ok(None),
		Err(error) => Err(error.to_string()),
	}
}

#[tauri::command]
fn set_room_server_passcode(window: WebviewWindow, passcode: String) -> Result<(), String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	let entry = keyring::Entry::new("com.wavelength.desktop", "room-server-passcode")
		.map_err(|error| error.to_string())?;
	if passcode.is_empty() {
		match entry.delete_credential() {
			Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
			Err(error) => Err(error.to_string()),
		}
	} else {
		entry
			.set_password(&passcode)
			.map_err(|error| error.to_string())
	}
}

#[tauri::command]
fn get_download_directory(app: AppHandle, window: WebviewWindow) -> Result<Option<String>, String> {
	if window.label() != "player" { return Err("This command is only available to the player window.".into()); }
	let settings = read_settings(&app)?;
	Ok(settings.get("downloadDirectory").and_then(serde_json::Value::as_str).map(str::to_string))
}

#[tauri::command]
fn set_download_directory(app: AppHandle, window: WebviewWindow, directory: String) -> Result<String, String> {
	if window.label() != "player" { return Err("This command is only available to the player window.".into()); }
	let canonical = fs::canonicalize(directory.trim()).map_err(|_| "The selected download folder is unavailable.".to_string())?;
	if !canonical.is_dir() { return Err("Choose an existing folder for downloads.".into()); }
	let path = canonical.to_string_lossy().to_string();
	write_setting(&app, "downloadDirectory", serde_json::json!(path))?;
	Ok(path)
}

#[tauri::command]
async fn choose_download_directory(app: AppHandle, window: WebviewWindow) -> Result<Option<String>, String> {
	if window.label() != "player" { return Err("This command is only available to the player window.".into()); }
	let dialog_app = app.clone();
	let selection = tauri::async_runtime::spawn_blocking(move || {
		dialog_app.dialog().file().blocking_pick_folder()
	})
	.await
	.map_err(|error| error.to_string())?;
	let Some(selection) = selection else {
		return Ok(None);
	};
	let directory = selection.into_path().map_err(|error| error.to_string())?;
	let path = directory.to_string_lossy().to_string();
	set_download_directory(app, window, path).map(Some)
}

#[tauri::command]
async fn download_audio(
	app: AppHandle,
	window: WebviewWindow,
	track_id: String,
	filename: String,
) -> Result<String, String> {
	if window.label() != "player" { return Err("This command is only available to the player window.".into()); }
	if track_id.len() < 6 || track_id.len() > 256 || !track_id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')) {
		return Err("This track has an invalid download ID.".into());
	}
	let settings = read_settings(&app)?;
	let directory = settings.get("downloadDirectory").and_then(serde_json::Value::as_str)
		.ok_or_else(|| "Choose a download folder in Settings first.".to_string())?;
	let directory = fs::canonicalize(directory).map_err(|_| "The selected download folder is unavailable.".to_string())?;
	if !directory.is_dir() { return Err("The selected download folder is unavailable.".into()); }
	let base_name = filename.trim().chars()
		.map(|character| if character.is_control() || matches!(character, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') { '_' } else { character })
		.collect::<String>();
	let base_name = base_name.trim_matches([' ', '.']).chars().take(160).collect::<String>();
	if base_name.is_empty() { return Err("A valid filename is required.".into()); }
	let port = local_service_port(&app)?;
	let url = format!("http://127.0.0.1:{port}/download/{track_id}?code=solo");
	let response = reqwest::get(url).await.map_err(|error| format!("Audio service request failed: {error}"))?;
	if !response.status().is_success() {
		return Err(format!("Download failed with status {}.", response.status()));
	}
	let extension = match response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("").split(';').next().unwrap_or("") {
		"audio/flac" => "flac",
		"audio/mpeg" => "mp3",
		"audio/mp4" => "m4a",
		"audio/wav" => "wav",
		"audio/ogg" => "ogg",
		"audio/webm" => "webm",
		_ => "audio",
	};
	let destination = directory.join(format!("{base_name}.{extension}"));
	let bytes = response.bytes().await.map_err(|error| format!("Download response failed: {error}"))?;
	let temporary = destination.with_extension(format!("{extension}.part"));
	fs::write(&temporary, bytes).map_err(|error| format!("Could not save the download: {error}"))?;
	fs::rename(&temporary, &destination).map_err(|error| {
		let _ = fs::remove_file(&temporary);
		format!("Could not finish saving the download: {error}")
	})?;
	Ok(destination.to_string_lossy().to_string())
}

struct LocalServices {
	port: Option<u16>,
	children: Mutex<Vec<Child>>,
}

fn stop_local_service(child: &mut Child) -> Result<(), String> {
	#[cfg(windows)]
	let tree_kill_error = {
		use std::os::windows::process::CommandExt;

		let pid = child.id().to_string();
		let taskkill = env::var_os("SystemRoot")
			.map(PathBuf::from)
			.map(|root| root.join("System32").join("taskkill.exe"))
			.unwrap_or_else(|| PathBuf::from("taskkill.exe"));
		let result = Command::new(taskkill)
			.args(["/PID", &pid, "/T", "/F"])
			.creation_flags(0x08000000)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.status();
		if matches!(&result, Ok(status) if status.success()) {
			None
		} else {
			match child.try_wait() {
				Ok(Some(_)) => None,
				Ok(None) => Some(format!("Could not stop the local service process tree: {result:?}")),
				Err(error) => Some(format!("Could not check the local service process: {error}; taskkill result: {result:?}")),
			}
		}
	};
	#[cfg(not(windows))]
	let tree_kill_error: Option<String> = None;

	if let Err(error) = child.kill() {
		match child.try_wait() {
			Ok(Some(_)) => {}
			Ok(None) => return Err(format!("Could not stop the local service process: {error}")),
			Err(wait_error) => {
				return Err(format!(
					"Could not stop the local service process: {error}; could not confirm exit: {wait_error}"
				));
			}
		}
	}
	child.wait().map_err(|error| format!("Could not wait for the local service process to exit: {error}"))?;
	tree_kill_error.map_or(Ok(()), Err)
}

#[tauri::command]
fn stop_local_services(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	let services = app
		.try_state::<LocalServices>()
		.ok_or_else(|| "Local service state is unavailable.".to_string())?;
	let mut children = services
		.children
		.lock()
		.map_err(|error| format!("Could not lock local services for shutdown: {error}"))?;
	for child in children.iter_mut() {
		stop_local_service(child)?;
	}
	children.clear();
	Ok(())
}

fn local_service_port(app: &AppHandle) -> Result<u16, String> {
	if let Some(port) = app
		.try_state::<LocalServices>()
		.and_then(|services| services.port)
	{
		return Ok(port);
	}
	if cfg!(debug_assertions) {
		return Ok(8787);
	}
	Err("The bundled local audio service is unavailable.".into())
}

#[tauri::command]
fn get_local_service_base(
	app: AppHandle,
	window: WebviewWindow,
) -> Result<String, String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	Ok(format!("http://127.0.0.1:{}", local_service_port(&app)?))
}

#[tauri::command]
fn start_spotify_oauth(
	app: AppHandle,
	window: WebviewWindow,
	server_url: String,
) -> Result<(), String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	let server_url = normalize_server_url(&server_url)?;
	let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
	listener
		.set_nonblocking(true)
		.map_err(|error| error.to_string())?;
	let callback_origin = format!(
		"http://127.0.0.1:{}",
		listener.local_addr().map_err(|error| error.to_string())?.port()
	);
	let mut login_url = server_url
		.join("/api/spotify/login")
		.map_err(|error| error.to_string())?;
	login_url
		.query_pairs_mut()
		.append_pair("return_origin", &callback_origin);

	let callback_app = app.clone();
	thread::spawn(move || {
		let deadline = Instant::now() + Duration::from_secs(600);
		while Instant::now() < deadline {
			match listener.accept() {
				Ok((mut stream, _)) => {
					let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
					let mut request = String::new();
					let _ = io::BufReader::new(&stream).read_line(&mut request);
					let callback = request
						.split_whitespace()
						.nth(1)
						.and_then(|path| Url::parse(&format!("http://localhost{path}")).ok());
					let status = callback
						.as_ref()
						.and_then(|url| url.query_pairs().find(|(key, _)| key == "spotify").map(|(_, value)| value.into_owned()));
					let ticket = callback.as_ref().and_then(|url| {
						url.query_pairs()
							.find(|(key, _)| key == "ticket")
							.map(|(_, value)| value.into_owned())
					});
					if status.as_deref() == Some("connected")
						&& ticket.as_ref().is_some_and(|value| {
							value.len() == 40
								&& value.bytes().all(|byte| byte.is_ascii_alphanumeric())
						})
					{
						let _ = callback_app.emit(
							"spotify-oauth-result",
							serde_json::json!({"status": "connected", "ticket": ticket}),
						);
					} else if status.as_deref() == Some("error") {
						let _ = callback_app.emit(
							"spotify-oauth-result",
							serde_json::json!({"status": "error", "ticket": null}),
						);
					} else {
						continue;
					}

					let body = "<!doctype html><meta charset=\"utf-8\"><title>Wavelength</title><p>Spotify sign-in is complete. You can return to Wavelength.</p>";
					let response = format!(
						"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
						body.len(),
						body
					);
					let _ = stream.write_all(response.as_bytes());
					return;
				}
				Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
					thread::sleep(Duration::from_millis(100));
				}
				Err(_) => return,
			}
		}
	});

	app.opener()
		.open_url(login_url.as_str(), None::<&str>)
		.map_err(|error| error.to_string())
}

const SPOTIFY_CALLBACK_URI: &str = "http://127.0.0.1:43821/callback";

fn is_spotify_pkce_value(value: &str) -> bool {
	(43..=128).contains(&value.len())
		&& value
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[tauri::command]
fn start_spotify_pkce_oauth(
	app: AppHandle,
	window: WebviewWindow,
	client_id: String,
	code_challenge: String,
	state: String,
) -> Result<(), String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	let client_id = client_id.trim();
	if !(16..=64).contains(&client_id.len())
		|| !client_id.bytes().all(|byte| byte.is_ascii_alphanumeric())
	{
		return Err("Enter a valid Spotify Client ID in Settings.".into());
	}
	if !is_spotify_pkce_value(&code_challenge) || !is_spotify_pkce_value(&state) {
		return Err("Spotify sign-in could not be started securely.".into());
	}

	let listener = TcpListener::bind("127.0.0.1:43821")
		.map_err(|_| "Spotify sign-in callback port 43821 is unavailable.".to_string())?;
	listener
		.set_nonblocking(true)
		.map_err(|error| error.to_string())?;

	let mut login_url = Url::parse("https://accounts.spotify.com/authorize")
		.map_err(|error| error.to_string())?;
	login_url
		.query_pairs_mut()
		.append_pair("client_id", client_id)
		.append_pair("response_type", "code")
		.append_pair("redirect_uri", SPOTIFY_CALLBACK_URI)
		.append_pair(
			"scope",
			"user-read-private user-read-email user-library-read user-library-modify playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private",
		)
		.append_pair("code_challenge_method", "S256")
		.append_pair("code_challenge", &code_challenge)
		.append_pair("state", &state);

	app.opener()
		.open_url(login_url.as_str(), None::<&str>)
		.map_err(|error| error.to_string())?;

	let callback_app = app.clone();
	let expected_state = state;
	thread::spawn(move || {
		let deadline = Instant::now() + Duration::from_secs(600);
		while Instant::now() < deadline {
			match listener.accept() {
				Ok((mut stream, _)) => {
					let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
					let mut request = String::new();
					let _ = io::BufReader::new(&stream).read_line(&mut request);
					let callback = request
						.split_whitespace()
						.nth(1)
						.and_then(|path| Url::parse(&format!("http://localhost{path}")).ok());
					if callback
						.as_ref()
						.map_or(true, |url| url.path() != "/callback")
					{
						continue;
					}
					let callback_state = callback.as_ref().and_then(|url| {
						url.query_pairs()
							.find(|(key, _)| key == "state")
							.map(|(_, value)| value.into_owned())
					});
					if callback_state.as_deref() != Some(expected_state.as_str()) {
						let body = "Spotify sign-in state did not match. Return to Wavelength and try again.";
						let response = format!(
							"HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
							body.len(),
							body
						);
						let _ = stream.write_all(response.as_bytes());
						continue;
					}

					let code = callback.as_ref().and_then(|url| {
						url.query_pairs()
							.find(|(key, _)| key == "code")
							.map(|(_, value)| value.into_owned())
					});
					let error = callback.as_ref().and_then(|url| {
						url.query_pairs()
							.find(|(key, _)| key == "error")
							.map(|(_, value)| value.into_owned())
					});
					let body_text;
					if let Some(code) = code.filter(|code| !code.is_empty() && code.len() <= 4096) {
						let _ = callback_app.emit(
							"spotify-oauth-result",
							serde_json::json!({
								"status": "authorized",
								"code": code,
								"redirectUri": SPOTIFY_CALLBACK_URI,
							}),
						);
						body_text = "Spotify sign-in is complete. You can return to Wavelength.";
					} else if error.is_some() {
						let _ = callback_app.emit(
							"spotify-oauth-result",
							serde_json::json!({"status": "error"}),
						);
						body_text = "Spotify sign-in was not completed. You can return to Wavelength.";
					} else {
						continue;
					}

					let body = format!(
						"<!doctype html><meta charset=\"utf-8\"><title>Wavelength</title><p>{body_text}</p>"
					);
					let response = format!(
						"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
						body.len(),
						body
					);
					let _ = stream.write_all(response.as_bytes());
					return;
				}
				Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
					thread::sleep(Duration::from_millis(100));
				}
				Err(_) => return,
			}
		}
	});

	Ok(())
}

#[tauri::command]
fn get_spotify_refresh_token(window: WebviewWindow) -> Result<Option<String>, String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	let entry = keyring::Entry::new("com.wavelength.desktop", "spotify-refresh-token")
		.map_err(|error| error.to_string())?;
	match entry.get_password() {
		Ok(token) => Ok(Some(token)),
		Err(keyring::Error::NoEntry) => Ok(None),
		Err(error) => Err(error.to_string()),
	}
}

#[tauri::command]
fn set_spotify_refresh_token(
	window: WebviewWindow,
	refresh_token: Option<String>,
) -> Result<(), String> {
	if window.label() != "player" {
		return Err("This command is only available to the player window.".into());
	}
	let entry = keyring::Entry::new("com.wavelength.desktop", "spotify-refresh-token")
		.map_err(|error| error.to_string())?;
	match refresh_token {
		Some(token) if !token.is_empty() && token.len() <= 4096 => {
			entry.set_password(&token).map_err(|error| error.to_string())
		}
		Some(_) => Err("The Spotify refresh token is invalid.".into()),
		None => match entry.delete_credential() {
			Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
			Err(error) => Err(error.to_string()),
		},
	}
}

fn start_bundled_service(
	app: &tauri::App,
	service_path: &str,
	port: u16,
	environment: &[(&str, String)],
) -> Result<Option<Child>, Box<dyn std::error::Error>> {
	let address = SocketAddr::from(([127, 0, 0, 1], port));
	if TcpStream::connect_timeout(&address, Duration::from_millis(150)).is_ok() {
		return Ok(None);
	}

	let executable = app.path().resource_dir()?.join(service_path);
	if !executable.exists() {
		if cfg!(debug_assertions) {
			return Ok(None);
		}
		return Err(io::Error::new(
			io::ErrorKind::NotFound,
			format!("Bundled local service is missing: {}", executable.display()),
		)
		.into());
	}

	let mut command = Command::new(&executable);
	command
		.current_dir(executable.parent().unwrap_or(&executable))
		.envs(environment.iter().map(|(key, value)| (*key, value.as_str())))
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null());
	#[cfg(windows)]
	{
		use std::os::windows::process::CommandExt;
		command.creation_flags(0x08000000);
	}
	let mut child = command.spawn()?;
	let started = Instant::now();
	loop {
		if TcpStream::connect_timeout(&address, Duration::from_millis(150)).is_ok() {
			return Ok(Some(child));
		}
		if let Some(status) = child.try_wait()? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				format!("Local service exited before opening port {port}: {status}"),
			)
			.into());
		}
		if started.elapsed() > Duration::from_secs(45) {
			let _ = child.kill();
			let _ = child.wait();
			return Err(io::Error::new(
				io::ErrorKind::TimedOut,
				format!("Local service did not open port {port} in time"),
			)
			.into());
		}
		thread::sleep(Duration::from_millis(250));
	}
}

fn configure_webview_hardware_acceleration() {
	#[cfg(target_os = "windows")]
	{
		const WEBVIEW_ARGS: &str = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";
		let existing = env::var(WEBVIEW_ARGS).unwrap_or_default();
		let args = existing.split_whitespace().collect::<Vec<_>>();
		if args.iter().any(|argument| *argument == "--disable-gpu")
			|| args.iter().any(|argument| *argument == "--enable-gpu")
		{
			return;
		}
		let configured = if existing.trim().is_empty() {
			"--enable-gpu".to_string()
		} else {
			format!("{} --enable-gpu", existing.trim())
		};
		env::set_var(WEBVIEW_ARGS, configured);
	}
}

fn main() {
	configure_webview_hardware_acceleration();
	tauri::Builder::default()
		.plugin(
			tauri::plugin::Builder::<tauri::Wry, ()>::new("webview-navigation-guard")
				.on_navigation(|_, url| {
					let packaged_app = (url.scheme() == "tauri"
						&& url.host_str() == Some("localhost"))
						|| (matches!(url.scheme(), "http" | "https")
							&& url.host_str() == Some("tauri.localhost"));
					let dev_server = cfg!(debug_assertions)
						&& url.scheme() == "http"
						&& matches!(url.host_str(), Some("localhost" | "127.0.0.1"))
						&& url.port() == Some(5173);
					packaged_app || dev_server
				})
				.build(),
		)
		.setup(|app| {
			let resource_dir = app.path().resource_dir()?;
			let runtime_dir = resource_dir.join("services").join("runtime");
			let inherited_path = env::var_os("PATH").unwrap_or_default();
			let service_path = env::join_paths(
				std::iter::once(runtime_dir).chain(env::split_paths(&inherited_path)),
			)?
			.to_string_lossy()
			.to_string();

			let app_data_dir = app.path().app_data_dir()?;
			let audio_data_dir = app_data_dir.join("audio-service");
			let qobuz_download_dir = app_data_dir.join("qobuz-service").join("downloads");
			fs::create_dir_all(&audio_data_dir)?;
			fs::create_dir_all(&qobuz_download_dir)?;

			let mut local_service = None;
			for _ in 0..8 {
				let port_listener = TcpListener::bind("127.0.0.1:0")?;
				let port = port_listener.local_addr()?.port();
				drop(port_listener);
				let service_environment = [
					("PORT", port.to_string()),
					("DOWNLOAD_ROOT", qobuz_download_dir.to_string_lossy().to_string()),
					("WAVELENGTH_DATA_DIR", audio_data_dir.to_string_lossy().to_string()),
					("QOBUZ_API_BASE", "https://www.qobuz.com/api.json/0.2".into()),
					("QOBUZ_API_URL", format!("http://127.0.0.1:{port}/qobuz")),
					("LOG_LEVEL", "WARNING".into()),
					("PATH", service_path.clone()),
				];
				let service_executable = if cfg!(target_os = "windows") {
					"services/api/WavelengthLocalService.exe"
				} else {
					"services/api/WavelengthLocalService"
				};
				if let Some(child) = start_bundled_service(
					app,
					service_executable,
					port,
					&service_environment,
				)? {
					local_service = Some((port, child));
					break;
				}
			}
			if local_service.is_none() && !cfg!(debug_assertions) {
				return Err(io::Error::new(
					io::ErrorKind::AddrNotAvailable,
					"Could not start the bundled local audio service",
				)
				.into());
			}
			let port = local_service.as_ref().map(|(port, _)| *port);
			let children = local_service.map(|(_, child)| vec![child]).unwrap_or_default();
			app.manage(LocalServices {
				port,
				children: Mutex::new(children),
			});
			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_process::init())
		.plugin(tauri_plugin_updater::Builder::new().build())
		.invoke_handler(tauri::generate_handler![
			get_server_url,
			set_server_url,
			get_room_server_passcode,
			set_room_server_passcode,
			get_local_service_base,
			stop_local_services,
			get_download_directory,
			set_download_directory,
			choose_download_directory,
			download_audio,
			start_spotify_oauth,
			start_spotify_pkce_oauth,
			get_spotify_refresh_token,
			set_spotify_refresh_token
		])
		.build(tauri::generate_context!())
		.expect("error while building Wavelength")
		.run(|app, event| {
			if matches!(event, RunEvent::Exit) {
				if let Some(services) = app.try_state::<LocalServices>() {
					match services.children.lock() {
						Ok(mut children) => {
							for child in children.iter_mut() {
								if let Err(error) = stop_local_service(child) {
									eprintln!("Could not stop local service during app exit: {error}");
								}
							}
						}
						Err(error) => {
							eprintln!("Could not lock local services for shutdown: {error}");
						}
					}
				}
			}
		});
}

#[cfg(test)]
mod tests {
	use super::{is_spotify_pkce_value, normalize_server_url};

	#[test]
	fn server_url_allows_https_and_loopback_http_only() {
		assert!(normalize_server_url("https://music.example.com").is_ok());
		assert!(normalize_server_url("http://localhost:3030").is_ok());
		assert!(normalize_server_url("http://127.0.0.1:3030").is_ok());
		assert!(normalize_server_url("http://[::1]:3030").is_ok());
	}

	#[test]
	fn server_url_rejects_insecure_or_ambiguous_addresses() {
		assert!(normalize_server_url("http://music.example.com").is_err());
		assert!(normalize_server_url("https://user:pass@music.example.com").is_err());
		assert!(normalize_server_url("https://music.example.com/player").is_err());
		assert!(normalize_server_url("file:///tmp/app").is_err());
	}

	#[test]
	fn spotify_pkce_values_require_base64url_characters_and_length() {
		assert!(is_spotify_pkce_value(&"A".repeat(43)));
		assert!(is_spotify_pkce_value(&"A".repeat(128)));
		assert!(!is_spotify_pkce_value(&"A".repeat(42)));
		assert!(!is_spotify_pkce_value(&"A".repeat(129)));
		assert!(!is_spotify_pkce_value(&format!("{}+", "A".repeat(43))));
	}
}