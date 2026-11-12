use axum::{body::Body, extract::{Path, Query, State}, http::{header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE}, HeaderMap, HeaderValue, Method, Request, StatusCode}, response::{IntoResponse, Redirect, Response}, routing::{any, get, post, put}, Json, Router};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mongodb::{bson::doc, Client, Collection, Database};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use std::{collections::HashMap, env, net::SocketAddr, path::PathBuf, sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::info;

const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

#[derive(Clone)] struct AppState { database: Database, parties: Collection<Party>, party_auth: Collection<PartyAuth>, http: reqwest::Client, audio_url: String, music_root: PathBuf, spotify_states: Arc<Mutex<HashMap<String, SpotifyOAuthState>>>, spotify_sessions: Arc<Mutex<HashMap<String, SpotifySession>>> }
#[derive(Clone, Serialize, Deserialize, Default)] #[serde(rename_all="camelCase")] struct Track { #[serde(default)] id: String, #[serde(default)] video_id: String, #[serde(default)] title: String, #[serde(default)] artist: String, #[serde(default)] thumbnail: String, #[serde(default)] duration: f64, #[serde(default)] added_by: String, #[serde(default="pending")] status: String }
fn pending() -> String { "pending".into() }
#[derive(Clone, Serialize, Deserialize, Default)] #[serde(rename_all="camelCase")] struct Permissions { #[serde(default)] can_add: bool, #[serde(default)] can_manage_queue: bool, #[serde(default)] can_control_playback: bool }
#[derive(Clone, Serialize, Deserialize, Default)] #[serde(rename_all="camelCase")] struct PlaylistTrackReference { #[serde(default)] id: String, #[serde(default)] platform: String }
#[derive(Clone, Serialize, Deserialize, Default)] #[serde(rename_all="camelCase")] struct SharedPlaylist { #[serde(default)] id: String, #[serde(default)] name: String, #[serde(default)] tracks: Vec<PlaylistTrackReference> }
#[derive(Clone, Serialize, Deserialize, Default)] #[serde(rename_all="camelCase")] struct Party { code: String, #[serde(default)] members: Vec<String>, host: String, #[serde(default)] queue: Vec<Track>, #[serde(default)] playlists: Vec<SharedPlaylist>, #[serde(default)] history: Vec<Track>, #[serde(default)] current_track_id: String, #[serde(default)] is_playing: bool, #[serde(default)] start_epoch: Option<i64>, #[serde(default)] paused_at: f64, #[serde(default="off")] repeat_mode: String, #[serde(default)] shuffle: bool, #[serde(default)] permissions: Permissions, #[serde(default)] updated_at: i64 }
fn off() -> String { "off".into() }
#[derive(Deserialize)] struct CreateInput { host: Option<String>, host_nickname: Option<String> }
#[derive(Deserialize)] struct JoinInput { code: String, username: Option<String>, nickname: Option<String> }
#[derive(Serialize, Deserialize)] struct PartyAuth { code: String, token_hash: String }
#[derive(Deserialize)] #[serde(rename_all="camelCase")] struct UpdateInput { queue: Option<Vec<Track>>, playlists: Option<Vec<SharedPlaylist>>, current_track_id: Option<String>, is_playing: Option<bool>, start_epoch: Option<i64>, paused_at: Option<f64>, repeat_mode: Option<String>, shuffle: Option<bool>, permissions: Option<Permissions> }
#[derive(Deserialize)] struct SearchQuery { q: Option<String>, source: Option<String>, limit: Option<u32>, offset: Option<u32> }
#[derive(Deserialize)] struct LyricsQuery { title: String, artist: String }
#[derive(Deserialize)] struct AudioQuery { video_ids: Option<String> }
#[derive(Deserialize)] struct LogsQuery { limit: Option<u32> }
#[derive(Deserialize)] #[serde(rename_all="camelCase")] struct SpotifyPlaylistInput { playlist_id: String, name: Option<String>, access_token: String, track_ids: Vec<String> }
#[derive(Clone)] struct SpotifyOAuthState { expires_at: i64, return_origin: String }
#[derive(Clone)] struct SpotifySession { access_token: String, refresh_token: Option<String>, expires_at: i64 }
#[derive(Deserialize)] struct SpotifyLoginQuery { return_origin: Option<String> }
#[derive(Deserialize)] struct SpotifyCallbackQuery { code: Option<String>, state: Option<String>, error: Option<String> }
#[derive(Deserialize)] struct SpotifyTokenResponse { access_token: String, expires_in: u64, #[serde(default)] refresh_token: Option<String> }
#[derive(Serialize)] struct ErrorBody { error: String }
fn now() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }
fn clean(value: &str) -> String { value.replace(['<','>'], "").split_whitespace().collect::<Vec<_>>().join(" ") }
fn code(value: &str) -> String { clean(value).to_uppercase() }
fn party_token_hash(token: &str) -> String { BASE64.encode(Sha256::digest(token.as_bytes())) }
fn party_token_is_valid(candidate: &str, expected_hash: &str) -> bool { party_token_hash(candidate).as_bytes().ct_eq(expected_hash.as_bytes()).unwrap_u8() == 1 }
fn failure(status: StatusCode, message: impl Into<String>) -> Response { (status, Json(ErrorBody { error: message.into() })).into_response() }
fn json(value: impl Serialize) -> Response { Json(value).into_response() }

fn random_token(length: usize) -> String {
	let mut rng = rand::thread_rng();
	(0..length).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect()
}

fn frontend_origin(candidate: Option<&str>) -> String {
	let configured = env::var("FRONTEND_URL").ok().filter(|value| !value.trim().is_empty())
		.or_else(|| env::var("SPOTIFY_REDIRECT_URI").ok())
		.unwrap_or_else(|| "http://localhost:5173".into());
	let configured_origin = reqwest::Url::parse(&configured)
		.map(|url| url.origin().ascii_serialization())
		.unwrap_or_else(|_| "http://localhost:5173".into());
	if let Some(value) = candidate {
		if let Ok(url) = reqwest::Url::parse(value) {
			let origin = url.origin().ascii_serialization();
			let local = url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
			if origin == configured_origin || local { return origin; }
		}
	}
	configured_origin
}

fn spotify_redirect(origin: &str, status: &str) -> Response {
	let mut url = reqwest::Url::parse(origin).unwrap_or_else(|_| reqwest::Url::parse("http://localhost:5173").unwrap());
	url.set_path("/settings");
	url.set_query(None);
	url.query_pairs_mut().append_pair("spotify", status);
	Redirect::temporary(url.as_str()).into_response()
}

fn spotify_cookie(headers: &HeaderMap) -> Option<String> {
	headers.get_all(COOKIE).iter().filter_map(|header| header.to_str().ok()).flat_map(|value| value.split(';'))
		.find_map(|part| part.trim().strip_prefix("wavelength_spotify=").map(str::to_string))
}

async fn spotify_login(State(state): State<AppState>, Query(query): Query<SpotifyLoginQuery>) -> Response {
	let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap_or_default();
	let redirect_uri = env::var("SPOTIFY_REDIRECT_URI").unwrap_or_default();
	if client_id.trim().is_empty() || redirect_uri.trim().is_empty() {
		return failure(StatusCode::INTERNAL_SERVER_ERROR, "Spotify sign-in is not configured");
	}
	let return_origin = frontend_origin(query.return_origin.as_deref());
	let oauth_state = random_token(40);
	let mut states = state.spotify_states.lock().await;
	states.retain(|_, value| value.expires_at > now());
	states.insert(oauth_state.clone(), SpotifyOAuthState { expires_at: now() + 600_000, return_origin });
	drop(states);
	let url = match reqwest::Url::parse_with_params("https://accounts.spotify.com/authorize", &[
		("client_id", client_id.as_str()),
		("response_type", "code"),
		("redirect_uri", redirect_uri.as_str()),
		("scope", "user-read-private user-read-email"),
		("state", oauth_state.as_str()),
	]) {
		Ok(url) => url,
		Err(_) => return failure(StatusCode::INTERNAL_SERVER_ERROR, "Spotify sign-in URL could not be created"),
	};
	Redirect::temporary(url.as_str()).into_response()
}

async fn spotify_callback(State(state): State<AppState>, Query(query): Query<SpotifyCallbackQuery>) -> Response {
	let Some(state_key) = query.state.as_deref() else {
		return spotify_redirect(&frontend_origin(None), "error");
	};
	let oauth_state = state.spotify_states.lock().await.remove(state_key);
	let Some(oauth_state) = oauth_state.filter(|value| value.expires_at > now()) else {
		return spotify_redirect(&frontend_origin(None), "error");
	};
	if query.error.is_some() { return spotify_redirect(&oauth_state.return_origin, "error"); }
	let Some(code) = query.code.as_deref() else {
		return spotify_redirect(&oauth_state.return_origin, "error");
	};
	let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap_or_default();
	let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default();
	let redirect_uri = env::var("SPOTIFY_REDIRECT_URI").unwrap_or_default();
	if client_id.is_empty() || client_secret.is_empty() || redirect_uri.is_empty() {
		return spotify_redirect(&oauth_state.return_origin, "error");
	}
	let credentials = BASE64.encode(format!("{client_id}:{client_secret}"));
	let body = url::form_urlencoded::Serializer::new(String::new())
		.append_pair("grant_type", "authorization_code")
		.append_pair("code", code)
		.append_pair("redirect_uri", &redirect_uri)
		.finish();
	let token = match state.http.post("https://accounts.spotify.com/api/token")
		.header("Authorization", format!("Basic {credentials}"))
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body)
		.send().await {
			Ok(response) if response.status().is_success() => match response.json::<SpotifyTokenResponse>().await { Ok(token) => token, Err(_) => return spotify_redirect(&oauth_state.return_origin, "error") },
			_ => return spotify_redirect(&oauth_state.return_origin, "error"),
		};
	let session_id = random_token(48);
	let session_max_age = token.expires_in.max(60);
	state.spotify_sessions.lock().await.insert(session_id.clone(), SpotifySession {
		access_token: token.access_token,
		refresh_token: token.refresh_token,
		expires_at: now() + (session_max_age as i64 * 1000),
	});
	let secure = if oauth_state.return_origin.starts_with("https://") { "; Secure" } else { "" };
	let cookie = format!("wavelength_spotify={session_id}; Path=/; Max-Age=2592000; HttpOnly; SameSite=Lax{secure}");
	let mut response = spotify_redirect(&oauth_state.return_origin, "connected");
	if let Ok(value) = HeaderValue::from_str(&cookie) { response.headers_mut().insert(SET_COOKIE, value); }
	response
}

async fn spotify_me(State(state): State<AppState>, headers: HeaderMap) -> Response {
	let Some(session_id) = spotify_cookie(&headers) else {
		return failure(StatusCode::UNAUTHORIZED, "Spotify is not connected");
	};
	let Some(mut session) = state.spotify_sessions.lock().await.get(&session_id).cloned() else {
		return failure(StatusCode::UNAUTHORIZED, "Spotify is not connected");
	};
	if session.expires_at <= now() + 30_000 {
		let refreshed = match refresh_spotify_session(&state, &session).await {
			Some(session) => session,
			None => {
				state.spotify_sessions.lock().await.remove(&session_id);
				return failure(StatusCode::UNAUTHORIZED, "Spotify session expired");
			}
		};
		state.spotify_sessions.lock().await.insert(session_id.clone(), refreshed.clone());
		session = refreshed;
	}
	match state.http.get("https://api.spotify.com/v1/me").bearer_auth(session.access_token).send().await {
		Ok(response) if response.status().is_success() => forward(response).await,
		Ok(response) if response.status() == StatusCode::UNAUTHORIZED => {
			state.spotify_sessions.lock().await.remove(&session_id);
			failure(StatusCode::UNAUTHORIZED, "Spotify session expired")
		},
		_ => failure(StatusCode::BAD_GATEWAY, "Spotify profile is temporarily unavailable"),
	}
}

async fn refresh_spotify_session(state: &AppState, session: &SpotifySession) -> Option<SpotifySession> {
	let refresh_token = session.refresh_token.as_deref()?;
	let client_id = env::var("SPOTIFY_CLIENT_ID").ok()?;
	let client_secret = env::var("SPOTIFY_CLIENT_SECRET").ok()?;
	let credentials = BASE64.encode(format!("{client_id}:{client_secret}"));
	let body = url::form_urlencoded::Serializer::new(String::new())
		.append_pair("grant_type", "refresh_token")
		.append_pair("refresh_token", refresh_token)
		.finish();
	let response = state.http.post("https://accounts.spotify.com/api/token")
		.header("Authorization", format!("Basic {credentials}"))
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body).send().await.ok()?;
	if !response.status().is_success() { return None; }
	let token = response.json::<SpotifyTokenResponse>().await.ok()?;
	Some(SpotifySession {
		access_token: token.access_token,
		refresh_token: token.refresh_token.or_else(|| session.refresh_token.clone()),
		expires_at: now() + (token.expires_in.max(60) as i64 * 1000),
	})
}

async fn spotify_logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
	if let Some(session_id) = spotify_cookie(&headers) { state.spotify_sessions.lock().await.remove(&session_id); }
	let mut response = json(serde_json::json!({ "disconnected": true }));
	response.headers_mut().insert(SET_COOKIE, HeaderValue::from_static("wavelength_spotify=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"));
	response
}

async fn health(State(state): State<AppState>) -> Response { let db=state.database.run_command(doc!{"ping":1}).await.is_ok(); json(serde_json::json!({"status":"ok","dbState":if db {1}else{0},"timestamp":now()})) }
async fn library(State(state): State<AppState>) -> Response {
	let mut tracks = Vec::new();
	let mut pending = vec![state.music_root.clone()];
	while let Some(directory) = pending.pop() {
		let mut entries = match tokio::fs::read_dir(&directory).await { Ok(entries) => entries, Err(_) => continue };
		while let Ok(Some(entry)) = entries.next_entry().await {
			let path = entry.path();
			if path.is_dir() { pending.push(path); continue; }
			let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_lowercase();
			if !["mp3", "flac", "m4a", "ogg", "wav", "aac"].contains(&extension.as_str()) { continue; }
			let title = path.file_stem().and_then(|value| value.to_str()).unwrap_or("Untitled").to_string();
			tracks.push(serde_json::json!({ "id": path.to_string_lossy(), "title": title, "artist": "Local file", "album": "Local library", "path": path.to_string_lossy(), "status": "ready" }));
		}
	}
	json(serde_json::json!({ "root": state.music_root.to_string_lossy(), "tracks": tracks }))
}
async fn create(State(state): State<AppState>, Json(input): Json<CreateInput>) -> Response {
	let host = clean(input.host.as_deref().or(input.host_nickname.as_deref()).unwrap_or(""));
	if host.is_empty() || host.len() > 50 { return failure(StatusCode::BAD_REQUEST, "Host name must be between 1 and 50 characters"); }
	if let Some(existing) = state.parties.find_one(doc! { "host": &host }).await.ok().flatten() {
		match state.party_auth.find_one(doc! { "code": &existing.code }).await {
			Ok(Some(_)) => return failure(StatusCode::CONFLICT, "Close your active room before creating another one"),
			Ok(None) => {
				if state.parties.delete_one(doc! { "code": &existing.code }).await.is_err() {
					return failure(StatusCode::INTERNAL_SERVER_ERROR, "Unable to replace the expired room");
				}
			},
			Err(_) => return failure(StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
		}
	}
	for _ in 0..10 {
		let party_code: String = { let mut rng = rand::thread_rng(); (0..6).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect() };
		if state.parties.find_one(doc! { "code": &party_code }).await.ok().flatten().is_some() { continue; }
		let party = Party { code: party_code.clone(), host: host.clone(), members: vec![host.clone()], permissions: Permissions::default(), updated_at: now(), ..Default::default() };
		if state.parties.insert_one(&party).await.is_err() { continue; }
		let host_token = random_token(64);
		let auth = PartyAuth { code: party_code.clone(), token_hash: party_token_hash(&host_token) };
		if state.party_auth.insert_one(auth).await.is_err() {
			let _ = state.parties.delete_one(doc! { "code": &party_code }).await;
			return failure(StatusCode::INTERNAL_SERVER_ERROR, "Party authorization could not be stored");
		}
		return json(serde_json::json!({ "code": party.code, "party": party, "hostToken": host_token }));
	}
	failure(StatusCode::CONFLICT, "Unable to generate a unique party code")
}
async fn find(state:&AppState, raw:&str)->Result<Party,Response>{let party_code=code(raw);if party_code.len()<4||party_code.len()>6||!party_code.chars().all(|c|c.is_ascii_alphanumeric()){return Err(failure(StatusCode::BAD_REQUEST,"Party code must be between 4 and 6 characters"))}state.parties.find_one(doc!{"code":party_code}).await.map_err(|_|failure(StatusCode::INTERNAL_SERVER_ERROR,"Database error occurred"))?.ok_or_else(||failure(StatusCode::NOT_FOUND,"Party not found"))}
async fn join(State(state):State<AppState>,Json(input):Json<JoinInput>)->Response{let name=clean(input.username.as_deref().or(input.nickname.as_deref()).unwrap_or(""));if name.is_empty()||name.len()>50{return failure(StatusCode::BAD_REQUEST,"Username must be between 1 and 50 characters")}if state.parties.find_one(doc!{"host":&name}).await.ok().flatten().is_some(){return failure(StatusCode::CONFLICT,"Close your active room before joining another one")}let mut party=match find(&state,&input.code).await{Ok(v)=>v,Err(e)=>return e};if !party.members.contains(&name){party.members.push(name);party.updated_at=now();}match state.parties.replace_one(doc!{"code":&party.code},&party).await{Ok(_)=>json(serde_json::json!({"code":party.code,"party":party})),Err(_)=>failure(StatusCode::INTERNAL_SERVER_ERROR,"Database error occurred")}}
async fn read(State(state): State<AppState>, Path(raw_code): Path<String>, headers: HeaderMap) -> Response {
	match find(&state, &raw_code).await {
		Ok(party) => {
			let host_authorized = party_host_authorized(&state, &party.code, &headers).await;
			let mut response = serde_json::to_value(party).unwrap_or_default();
			response["hostAuthorized"] = serde_json::json!(host_authorized);
			json(response)
		},
		Err(error) => error,
	}
}
async fn party_host_authorized(state: &AppState, party_code: &str, headers: &HeaderMap) -> bool {
	let Some(token) = headers.get(AUTHORIZATION).and_then(|value| value.to_str().ok()).and_then(|value| value.strip_prefix("Bearer ")) else { return false; };
	if token.len() != 64 || !token.is_ascii() { return false; }
	match state.party_auth.find_one(doc! { "code": code(party_code) }).await {
		Ok(Some(auth)) => party_token_is_valid(&token, &auth.token_hash),
		_ => false,
	}
}
async fn close_party(State(state): State<AppState>, Path(raw_code): Path<String>, headers: HeaderMap) -> Response {
	let party = match find(&state, &raw_code).await { Ok(party) => party, Err(error) => return error };
	if !party_host_authorized(&state, &party.code, &headers).await { return failure(StatusCode::UNAUTHORIZED, "A valid party host token is required"); }
	match state.parties.delete_one(doc! { "code": &party.code }).await {
		Ok(result) if result.deleted_count == 1 => {
			let _ = state.party_auth.delete_one(doc! { "code": &party.code }).await;
			json(serde_json::json!({ "closed": true }))
		},
		Ok(_) => failure(StatusCode::CONFLICT, "Room could not be closed"),
		Err(_) => failure(StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
	}
}
async fn update(State(state): State<AppState>, Path(raw_code): Path<String>, headers: HeaderMap, Json(input): Json<UpdateInput>) -> Response {
	let mut party = match find(&state, &raw_code).await { Ok(party) => party, Err(error) => return error };
	if !party_host_authorized(&state, &party.code, &headers).await { return failure(StatusCode::UNAUTHORIZED, "A valid party host token is required"); }
	if let Some(value) = input.queue { party.queue = value; }
	if let Some(value) = input.playlists { party.playlists = value; }
	if let Some(value) = input.current_track_id { party.current_track_id = value; }
	if let Some(value) = input.is_playing { party.is_playing = value; }
	if let Some(value) = input.start_epoch { party.start_epoch = Some(value); }
	if let Some(value) = input.paused_at { party.paused_at = value; }
	if let Some(value) = input.repeat_mode { party.repeat_mode = value; }
	if let Some(value) = input.shuffle { party.shuffle = value; }
	if let Some(value) = input.permissions { party.permissions = value; }
	party.updated_at = now();
	match state.parties.replace_one(doc! { "code": &party.code }, &party).await {
		Ok(_) => json(party),
		Err(_) => failure(StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
	}
}
async fn search(State(state):State<AppState>,Query(q):Query<SearchQuery>)->Response{let response=state.http.get(format!("{}/search",state.audio_url)).query(&[("q",q.q.unwrap_or_default()),("source",q.source.unwrap_or_else(||"spotify".into())),("limit",q.limit.unwrap_or(10).min(20).to_string()),("offset",q.offset.unwrap_or(0).to_string())]).send().await;match response{Ok(r)=>forward(r).await,Err(_)=>failure(StatusCode::BAD_GATEWAY,"Search service is temporarily unavailable")}}
async fn audio_status(State(state):State<AppState>,Query(q):Query<AudioQuery>)->Response{let response=state.http.get(format!("{}/status",state.audio_url)).query(&[("video_ids",q.video_ids.unwrap_or_default())]).send().await;match response{Ok(r)=>forward(r).await,Err(_)=>failure(StatusCode::BAD_GATEWAY,"Audio status is temporarily unavailable")}}
async fn audio_logs(State(state):State<AppState>,Query(q):Query<LogsQuery>)->Response{let response=state.http.get(format!("{}/logs",state.audio_url)).query(&[("limit",q.limit.unwrap_or(200).min(500).to_string())]).send().await;match response{Ok(r)=>forward(r).await,Err(_)=>failure(StatusCode::BAD_GATEWAY,"Audio logs are temporarily unavailable")}}
async fn lyrics(State(state):State<AppState>,Query(q):Query<LyricsQuery>)->Response{match state.http.get("https://lrclib.net/api/get").query(&[("track_name",q.title),("artist_name",q.artist)]).header("User-Agent","Wavelength/2.0").send().await{Ok(r)=>forward(r).await,Err(_)=>failure(StatusCode::BAD_GATEWAY,"Lyrics are temporarily unavailable")}}
async fn stream(State(state):State<AppState>,Path(track):Path<String>,Query(q):Query<HashMap<String,String>>,request:Request<Body>)->Response{let mut r=state.http.get(format!("{}/stream/{}",state.audio_url,track));if let Some(value)=q.get("code"){r=r.query(&[("code",value)])}if let Some(value)=request.headers().get("range"){r=r.header("Range",value)}match r.send().await{Ok(v)=>forward(v).await,Err(_)=>failure(StatusCode::BAD_GATEWAY,"Audio stream is temporarily unavailable")}}
async fn download(State(state):State<AppState>,Path(track):Path<String>,Query(q):Query<HashMap<String,String>>)->Response{let mut r=state.http.get(format!("{}/download/{}",state.audio_url,track));if let Some(value)=q.get("code"){r=r.query(&[("code",value)])}match r.send().await{Ok(v)=>forward(v).await,Err(_)=>failure(StatusCode::BAD_GATEWAY,"Audio download is temporarily unavailable")}}
async fn audio_action(State(state):State<AppState>,Path(action):Path<String>,Query(q):Query<AudioQuery>)->Response{match state.http.post(format!("{}/{}",state.audio_url,action)).query(&[("video_ids",q.video_ids.unwrap_or_default())]).send().await{Ok(v)=>forward(v).await,Err(_)=>failure(StatusCode::BAD_GATEWAY,"Audio service is temporarily unavailable")}}
async fn update_spotify_playlist(State(state): State<AppState>, Json(input): Json<SpotifyPlaylistInput>) -> Response { if input.access_token.trim().is_empty() || input.playlist_id.trim().is_empty() { return failure(StatusCode::BAD_REQUEST, "Spotify playlist ID and access token are required"); } if let Some(name) = input.name.as_deref().filter(|value| !value.trim().is_empty()) { let result = state.http.put(format!("https://api.spotify.com/v1/playlists/{}", input.playlist_id)).bearer_auth(&input.access_token).json(&serde_json::json!({ "name": name })).send().await; if result.map(|response| !response.status().is_success()).unwrap_or(true) { return failure(StatusCode::BAD_GATEWAY, "Spotify playlist could not be renamed"); } } let uris: Vec<String> = input.track_ids.iter().filter(|id| id.starts_with("spotify:")).cloned().collect(); match state.http.put(format!("https://api.spotify.com/v1/playlists/{}/tracks", input.playlist_id)).bearer_auth(&input.access_token).json(&serde_json::json!({ "uris": uris })).send().await { Ok(response) if response.status().is_success() => json(serde_json::json!({ "updated": true })), Ok(response) if response.status() == StatusCode::UNAUTHORIZED => failure(StatusCode::UNAUTHORIZED, "Spotify access token expired"), _ => failure(StatusCode::BAD_GATEWAY, "Spotify playlist could not be updated") } }
async fn forward(response:reqwest::Response)->Response{let status=StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);let content_type=response.headers().get("content-type").cloned();let content_disposition=response.headers().get("content-disposition").cloned();let content_length=response.headers().get("content-length").cloned();let content_range=response.headers().get("content-range").cloned();let accept_ranges=response.headers().get("accept-ranges").cloned();let body=Body::from_stream(response.bytes_stream());let mut result=Response::new(body);*result.status_mut()=status;for (name,value) in [("content-type",content_type),("content-disposition",content_disposition),("content-length",content_length),("content-range",content_range),("accept-ranges",accept_ranges)]{if let Some(value)=value{result.headers_mut().insert(name,value);}}result}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt().with_env_filter("wavelength_backend=info").init();
	let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://mongodb:27017/wavelength".into());
	let client = Client::with_uri_str(uri).await.expect("connect to MongoDB");
	let database = client.database("wavelength");
	let state = AppState {
		database: database.clone(),
		parties: database.collection("parties"),
		party_auth: database.collection("party_auth"),
		http: reqwest::Client::new(),
		audio_url: env::var("AUDIO_SERVICE_URL").unwrap_or_else(|_| "http://audio-service:8787".into()),
		music_root: PathBuf::from(env::var("MUSIC_ROOT").unwrap_or_else(|_| "/music".into())),
		spotify_states: Arc::new(Mutex::new(HashMap::new())),
		spotify_sessions: Arc::new(Mutex::new(HashMap::new())),
	};
	let mut allowed_origins = vec![
		HeaderValue::from_static("http://tauri.localhost"),
		HeaderValue::from_static("tauri://localhost"),
		HeaderValue::from_static("http://localhost:5173"),
		HeaderValue::from_static("http://127.0.0.1:5173"),
	];
	if let Ok(frontend_url) = env::var("FRONTEND_URL") {
		if let Ok(url) = reqwest::Url::parse(&frontend_url) {
			if let Ok(origin) = HeaderValue::from_str(&url.origin().ascii_serialization()) { allowed_origins.push(origin); }
		}
	}
	allowed_origins.sort();
	allowed_origins.dedup();
	let api = Router::new()
		.route("/parties", post(create))
		.route("/parties/join", post(join))
		.route("/parties/:code", get(read).put(update).delete(close_party));
	let app = Router::new()
		.route("/health", get(health))
		.layer(CorsLayer::new().allow_origin(allowed_origins).allow_headers([AUTHORIZATION, CONTENT_TYPE]).allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]))
		.with_state(state);
	let port = env::var("PORT").unwrap_or_else(|_| "5000".into()).parse().unwrap_or(5000);
	let address = SocketAddr::from(([0, 0, 0, 0], port));
	info!(%address, "Rust backend listening");
	let listener = tokio::net::TcpListener::bind(address).await.expect("bind backend port");
	axum::serve(listener, app).await.expect("serve backend");
}

#[cfg(test)]
mod tests {
	use super::{party_token_hash, party_token_is_valid};
	use axum::http::{header::AUTHORIZATION, HeaderMap, HeaderValue};

	#[test]
	fn party_host_bearer_token_is_checked() {
		let mut headers = HeaderMap::new();
		headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer secret-token"));
		let candidate = headers.get(AUTHORIZATION).unwrap().to_str().unwrap().strip_prefix("Bearer ").unwrap();
		let expected = party_token_hash("secret-token");
		assert!(party_token_is_valid(candidate, &expected));
		assert!(!party_token_is_valid("other-token", &expected));
	}
}
