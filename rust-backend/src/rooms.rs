use axum::{
    body::Body,
    extract::{rejection::JsonRejection, ws::{Message, WebSocket, WebSocketUpgrade}, ConnectInfo, DefaultBodyLimit, Path, Query, State},
    http::{header::{ACCEPT_RANGES, AUTHORIZATION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, COOKIE, RANGE, RETRY_AFTER, SET_COOKIE}, HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use mongodb::{bson::{doc, Document}, options::ReturnDocument, Client, Database};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::{HashMap, HashSet, VecDeque}, env, net::{IpAddr, SocketAddr}, path::PathBuf, sync::Arc, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use subtle::ConstantTimeEq;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::{broadcast, Mutex};
use tokio_util::io::ReaderStream;
use tower_http::cors::CorsLayer;
use tracing::info;
mod room_service;
use room_service::RoomService;

const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const ROOM_CODE_LENGTH: usize = 4;
const LEGACY_ROOM_CODE_LENGTH: usize = 12;
const ROOM_SERVER_PASSCODE_BCRYPT_COST: u32 = 12;
const MAX_RATE_LIMIT_CLIENTS: usize = 10_000;
const MAX_ROOM_UPDATE_BYTES: usize = 8 * 1024 * 1024;
const MAX_ROOM_QUEUE_TRACKS: usize = 1_000;
const ROOM_ACCESS_TTL: Duration = Duration::from_secs(12 * 60 * 60);
const ROOM_IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const ROOM_CLEANUP_INTERVAL: Duration = Duration::from_secs(10);
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const SHARED_AUDIO_ACCESS_TTL: Duration = Duration::from_secs(30 * 60);
const ROOM_ACCESS_REQUIRED_MESSAGE: &str = "Enter the server passcode in Settings to use this server";

#[derive(Clone)]
struct AppState {
    database: Database,
    rooms: RoomService,
    http: reqwest::Client,
    spotify_states: Arc<Mutex<HashMap<String, SpotifyOAuthState>>>,
    spotify_sessions: Arc<Mutex<HashMap<String, SpotifySession>>>,
    spotify_handoffs: Arc<Mutex<HashMap<String, SpotifyHandoff>>>,
    room_passcode_hash: String,
    trusted_proxies: HashSet<IpAddr>,
    passcode_rate_limiter: IpRateLimiter,
    room_join_rate_limiter: IpRateLimiter,
    room_read_rate_limiter: IpRateLimiter,
    room_socket_ticket_rate_limiter: IpRateLimiter,
    shared_audio_stream_rate_limiter: IpRateLimiter,
    room_access_sessions: Arc<Mutex<HashMap<String, i64>>>,
    shared_audio_access: Arc<Mutex<HashMap<String, SharedAudioGrant>>>,
}

#[derive(Clone)]
struct SpotifyOAuthState { expires_at: i64, return_origin: String }

#[derive(Clone)]
struct SpotifySession { access_token: String, refresh_token: Option<String>, expires_at: i64 }

#[derive(Clone)]
struct SpotifyHandoff { expires_at: i64, session_id: String }

#[derive(Deserialize)]
struct SpotifyLoginQuery { return_origin: Option<String> }

#[derive(Deserialize)]
struct SpotifyCallbackQuery { code: Option<String>, state: Option<String>, error: Option<String> }

#[derive(Deserialize)]
struct SpotifyDesktopExchange { ticket: String }

#[derive(Deserialize)]
struct RoomPasscodeInput { passcode: String }

#[derive(Deserialize)]
struct SharedAudioQuery { access: String }

#[derive(Clone)]
struct SharedAudioGrant { code: String, audio_id: String, expires_at: i64 }

#[derive(Clone)]
struct IpRateLimiter {
    attempts: Arc<Mutex<HashMap<IpAddr, VecDeque<Instant>>>>,
    max_attempts: usize,
    window: Duration,
}

impl IpRateLimiter {
    fn new(max_attempts: usize) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window: RATE_LIMIT_WINDOW,
        }
    }

    async fn check(&self, address: IpAddr) -> Result<(), u64> {
        let now = Instant::now();
        let mut clients = self.attempts.lock().await;
        clients.retain(|_, attempts| {
            attempts.back().is_some_and(|last| now.saturating_duration_since(*last) < self.window)
        });
        if !clients.contains_key(&address) && clients.len() >= MAX_RATE_LIMIT_CLIENTS {
            return Err(self.window.as_secs().max(1));
        }

        let attempts = clients.entry(address).or_default();
        while attempts.front().is_some_and(|first| now.saturating_duration_since(*first) >= self.window) {
            attempts.pop_front();
        }
        if attempts.len() >= self.max_attempts {
            let elapsed = now.saturating_duration_since(*attempts.front().expect("limited bucket is non-empty"));
            return Err(self.window.saturating_sub(elapsed).as_secs().max(1));
        }

        attempts.push_back(now);
        Ok(())
    }
}

#[derive(Deserialize)]
struct SpotifyTokenResponse { access_token: String, expires_in: u64, refresh_token: Option<String> }

#[derive(Deserialize)]
struct SpotifyPlaylistInput { name: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpotifyTracksInput { track_ids: Vec<String> }

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Track {
    #[serde(default)] id: String,
    #[serde(default)] source: String,
    #[serde(default)] video_id: String,
    #[serde(default)] audio_id: String,
    #[serde(default)] title: String,
    #[serde(default)] artist: String,
    #[serde(default)] thumbnail: String,
    #[serde(default)] duration: f64,
    #[serde(default)] added_by: String,
    #[serde(default = "pending")] status: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PlaylistTrackReference {
    #[serde(default)] id: String,
    #[serde(default)] platform: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SharedPlaylist {
    #[serde(default)] id: String,
    #[serde(default)] name: String,
    #[serde(default)] tracks: Vec<PlaylistTrackReference>,
}

fn pending() -> String { "pending".into() }

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Permissions {
    #[serde(default)] can_add: bool,
    #[serde(default)] can_manage_queue: bool,
    #[serde(default)] can_control_playback: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Party {
    code: String,
    #[serde(default)] members: Vec<String>,
    host: String,
    #[serde(default)] queue: Vec<Track>,
    #[serde(default)] playlists: Vec<SharedPlaylist>,
    #[serde(default)] history: Vec<Track>,
    #[serde(default)] current_track_id: String,
    #[serde(default)] is_playing: bool,
    #[serde(default)] start_epoch: Option<i64>,
    #[serde(default)] paused_at: f64,
    #[serde(default = "off")] repeat_mode: String,
    #[serde(default)] shuffle: bool,
    #[serde(default)] permissions: Permissions,
    #[serde(default)] sync_revision: i64,
    #[serde(default)] updated_at: i64,
    #[serde(default)] empty_since: Option<i64>,
}

fn off() -> String { "off".into() }

#[derive(Serialize, Deserialize)]
struct PartyAuth {
    code: String,
    token_hash: String,
}

#[derive(Deserialize)]
struct CreateInput {
    host: Option<String>,
    host_nickname: Option<String>,
}

#[derive(Deserialize)]
struct JoinInput {
    code: String,
    username: Option<String>,
    nickname: Option<String>,
}

#[derive(Deserialize)]
struct RoomSocketAuth {
    ticket: String,
}

#[derive(Deserialize)]
struct RoomSocketTicketInput {
    username: String,
}

#[derive(Deserialize)]
struct RoomSocketMessage {
    event: String,
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackProgress {
    current_track_id: String,
    current_time: f64,
    duration: f64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackControl {
    current_track_id: String,
    current_time: f64,
    is_playing: bool,
    sent_at_epoch_ms: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoomCommand {
    action: String,
    #[serde(default)] track: Option<Track>,
    #[serde(default)] track_id: String,
    #[serde(default)] target_index: Option<usize>,
    #[serde(default)] current_time: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RoomCommandEnvelope {
    username: String,
    command: RoomCommand,
}

fn room_track_is_shareable(track: &Track) -> bool {
    !track.id.is_empty()
        && track.id.len() <= 256
        && track.audio_id.len() <= 256
        && (track.source.eq_ignore_ascii_case("spotify")
            || track.source.eq_ignore_ascii_case("youtube")
            || !track.video_id.is_empty()
            || !track.audio_id.is_empty())
}

fn room_command_allowed(party: &Party, command: &RoomCommand) -> bool {
    match command.action.as_str() {
        "add-track" => party.permissions.can_add
            && command.track.as_ref().is_some_and(room_track_is_shareable),
        "remove-track" | "clear-queue" | "reorder-queue" => {
            party.permissions.can_manage_queue
                && (command.action != "remove-track"
                    || (!command.track_id.is_empty() && command.track_id.len() <= 256))
                && (command.action != "reorder-queue"
                    || (!command.track_id.is_empty()
                        && command.track_id.len() <= 256
                        && command.target_index.is_some()))
        }
        "play-track" => party.permissions.can_control_playback
            && command.track.as_ref().is_some_and(room_track_is_shareable),
        "toggle-playback" | "previous-track" | "next-track" | "toggle-shuffle"
        | "toggle-repeat" => party.permissions.can_control_playback,
        "seek" => {
            party.permissions.can_control_playback
                && command.current_time.is_some_and(|time| {
                    time.is_finite() && (0.0..=86_400.0).contains(&time)
                })
        }
        _ => false,
    }
}

fn is_server_queue_command(command: &RoomCommand) -> bool {
    matches!(
        command.action.as_str(),
        "add-track" | "remove-track" | "clear-queue" | "reorder-queue"
    )
}

fn apply_server_queue_command(
    party: &mut Party,
    command: &RoomCommand,
    username: &str,
) -> bool {
    match command.action.as_str() {
        "add-track" => {
            let Some(mut track) = command.track.clone() else {
                return false;
            };
            if party.queue.len() >= MAX_ROOM_QUEUE_TRACKS
                || party.queue.iter().any(|queued| queued.id == track.id)
                || party.current_track_id == track.id
            {
                return false;
            }
            track.added_by = clean(username);
            if track.status.is_empty() {
                track.status = pending();
            }
            party.queue.push(track);
            true
        }
        "remove-track" => {
            let previous_len = party.queue.len();
            party.queue.retain(|track| track.id != command.track_id);
            party.queue.len() != previous_len
        }
        "clear-queue" => {
            if party.queue.is_empty() {
                return false;
            }
            party.queue.clear();
            true
        }
        "reorder-queue" => {
            let Some(source) = party.queue.iter().position(|track| track.id == command.track_id)
            else {
                return false;
            };
            let Some(target) = command.target_index.filter(|index| *index < party.queue.len())
            else {
                return false;
            };
            if source == target {
                return false;
            }
            let track = party.queue.remove(source);
            party.queue.insert(target.min(party.queue.len()), track);
            true
        }
        _ => false,
    }
}

#[derive(Deserialize)]
struct LeaveInput {
    username: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    queue: Option<Vec<Track>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    playlists: Option<Vec<SharedPlaylist>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_track_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_playing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_epoch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    paused_at: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shuffle: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_revision: Option<i64>,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_millis() as i64
}

fn clean(value: &str) -> String {
    value.replace(['<', '>'], "").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_code(value: &str) -> String { clean(value).to_uppercase() }
fn valid_room_code(value: &str) -> bool {
    let normalized = normalize_code(value);
    ((4..=6).contains(&normalized.len()) || normalized.len() == LEGACY_ROOM_CODE_LENGTH)
        && normalized.chars().all(|character| character.is_ascii_alphanumeric())
}
fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    (0..ROOM_CODE_LENGTH).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect()
}
fn room_empty_cutoff(current_time: i64) -> i64 {
    current_time.saturating_sub(ROOM_IDLE_TIMEOUT.as_millis() as i64)
}

fn room_empty_timeout_expired(empty_since: Option<i64>, current_time: i64) -> bool {
    empty_since.is_some_and(|empty_since| empty_since <= room_empty_cutoff(current_time))
}
fn token_hash(token: &str) -> String { BASE64.encode(Sha256::digest(token.as_bytes())) }

fn passcode_bcrypt_input(passcode: &str) -> String {
    let mut input = b"wavelength-room-server-passcode-v1:".to_vec();
    input.extend_from_slice(passcode.as_bytes());
    BASE64.encode(Sha256::digest(input))
}

fn hash_room_server_passcode(passcode: &str, cost: u32) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(passcode_bcrypt_input(passcode), cost)
}

fn verify_room_server_passcode(passcode: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(passcode_bcrypt_input(passcode), hash)
}

fn parse_proxy_ip(value: &str) -> Option<IpAddr> {
    let value = value.trim().trim_matches('"');
    if let Ok(address) = value.parse() {
        return Some(address);
    }
    if let Some(value) = value.strip_prefix('[') {
        let (address, suffix) = value.split_once(']')?;
        if !suffix.is_empty()
            && (!suffix.starts_with(':') || suffix[1..].parse::<u16>().is_err())
        {
            return None;
        }
        return address.parse().ok();
    }
    value.parse::<SocketAddr>().ok().map(|address| address.ip())
}

fn x_forwarded_for_chain(headers: &HeaderMap) -> Vec<Option<IpAddr>> {
    let mut chain = Vec::new();
    for value in headers.get_all("x-forwarded-for") {
        if let Ok(value) = value.to_str() {
            chain.extend(value.split(',').map(parse_proxy_ip));
        }
    }
    chain
}

fn forwarded_for_chain(headers: &HeaderMap) -> Vec<Option<IpAddr>> {
    let mut chain = Vec::new();
    for value in headers.get_all("forwarded") {
        let Ok(value) = value.to_str() else {
            continue;
        };
        for element in value.split(',') {
            let address = element.split(';').find_map(|parameter| {
                let (name, value) = parameter.trim().split_once('=')?;
                name.eq_ignore_ascii_case("for").then_some(value)
            });
            chain.push(address.and_then(parse_proxy_ip));
        }
    }
    chain
}

fn client_ip_from_chain(
    peer: IpAddr,
    chain: &[Option<IpAddr>],
    trusted_proxies: &HashSet<IpAddr>,
) -> Option<IpAddr> {
    if !trusted_proxies.contains(&peer) {
        return None;
    }
    let mut address = peer;
    let mut found_forwarded_address = false;
    for candidate in chain.iter().rev() {
        if !trusted_proxies.contains(&address) {
            break;
        }
        let Some(candidate) = candidate else {
            break;
        };
        address = *candidate;
        found_forwarded_address = true;
    }
    found_forwarded_address.then_some(address)
}

fn client_ip(peer: SocketAddr, headers: &HeaderMap, trusted_proxies: &HashSet<IpAddr>) -> IpAddr {
    if !trusted_proxies.contains(&peer.ip()) {
        return peer.ip();
    }
    for header in [
        "cf-connecting-ip",
        "true-client-ip",
        "fastly-client-ip",
        "x-real-ip",
        "x-client-ip",
    ] {
        if let Some(address) = headers
            .get(header)
            .and_then(|value| value.to_str().ok())
            .and_then(parse_proxy_ip)
        {
            return address;
        }
    }
    client_ip_from_chain(peer.ip(), &x_forwarded_for_chain(headers), trusted_proxies)
        .or_else(|| client_ip_from_chain(peer.ip(), &forwarded_for_chain(headers), trusted_proxies))
        .unwrap_or(peer.ip())
}

fn token_matches(candidate: &str, expected_hash: &str) -> bool {
    token_hash(candidate).as_bytes().ct_eq(expected_hash.as_bytes()).unwrap_u8() == 1
}

fn public_api_request(method: &axum::http::Method, path: &str) -> bool {
    let path = path.strip_prefix("/api/v2").or_else(|| path.strip_prefix("/api")).unwrap_or(path);
    (method == axum::http::Method::POST && path == "/session")
        || (method == axum::http::Method::GET
            && (matches!(path, "/spotify/login" | "/spotify/callback")
                || (path.starts_with("/rooms/") && path.ends_with("/socket"))))
}

fn is_shared_audio_stream_path(method: &axum::http::Method, path: &str) -> bool {
    let path = path.strip_prefix("/api/v2").or_else(|| path.strip_prefix("/api")).unwrap_or(path);
    let segments = path.split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
    method == axum::http::Method::GET
        && segments.len() == 4
        && segments[0] == "rooms"
        && segments[2] == "audio"
        && parse_audio_id(segments[3]).is_some()
}

fn is_shared_audio_upload_path(method: &axum::http::Method, path: &str) -> bool {
    let path = path.strip_prefix("/api/v2").unwrap_or(path);
    let segments = path.split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
    method == axum::http::Method::POST
        && segments.len() == 3
        && segments[0] == "rooms"
        && segments[2] == "audio"
}

fn shared_audio_grant_matches(
    grant: &SharedAudioGrant,
    room_code: &str,
    audio_id: &str,
    current_time: i64,
) -> bool {
    grant.expires_at > current_time
        && grant.code == room_code
        && grant.audio_id == audio_id
}

async fn room_access_session_expiry(state: &AppState, token: &str) -> Option<i64> {
    if token.is_empty() || token.len() > 128 {
        return None;
    }
    let hash = token_hash(token);
    let current_time = epoch_millis();
    let mut sessions = state.room_access_sessions.lock().await;
    sessions.retain(|_, expires_at| *expires_at > current_time);
    sessions.get(&hash).copied().filter(|expires_at| *expires_at > current_time)
}

async fn valid_room_access_token(state: &AppState, token: &str) -> bool {
    room_access_session_expiry(state, token).await.is_some()
}

async fn require_room_access(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if request.method() == axum::http::Method::OPTIONS
        || public_api_request(request.method(), request.uri().path())
    {
        return next.run(request).await;
    }
    if is_shared_audio_stream_path(request.method(), request.uri().path())
        || is_shared_audio_upload_path(request.method(), request.uri().path())
    {
        return next.run(request).await;
    }
    let token = request.headers().get("x-room-access-token")
        .and_then(|value| value.to_str().ok()).unwrap_or_default();
    if !valid_room_access_token(&state, token).await {
        return failure(StatusCode::UNAUTHORIZED, ROOM_ACCESS_REQUIRED_MESSAGE);
    }
    next.run(request).await
}

async fn create_room_access_session(
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<RoomPasscodeInput>,
) -> Response {
    let address = client_ip(address, &headers, &state.trusted_proxies);
    if let Err(retry_after) = state.passcode_rate_limiter.check(address).await {
        return too_many_requests(retry_after);
    }
    if input.passcode.len() > 128 {
        return failure(StatusCode::UNAUTHORIZED, "The server passcode is incorrect");
    }
    let passcode = input.passcode;
    let expected_hash = state.room_passcode_hash.clone();
    let verification = tokio::task::spawn_blocking(move || {
        verify_room_server_passcode(&passcode, &expected_hash)
    }).await;
    let verified = match verification {
        Ok(Ok(verified)) => verified,
        Ok(Err(error)) => {
            tracing::error!(%error, "Room server passcode hash could not be verified");
            return failure(StatusCode::INTERNAL_SERVER_ERROR, "Passcode verification is temporarily unavailable");
        }
        Err(error) => {
            tracing::error!(%error, "Room server passcode verification task failed");
            return failure(StatusCode::INTERNAL_SERVER_ERROR, "Passcode verification is temporarily unavailable");
        }
    };
    if !verified {
        return failure(StatusCode::UNAUTHORIZED, "The server passcode is incorrect");
    }
    let token = random_token(48);
    let expires_at = epoch_millis() + ROOM_ACCESS_TTL.as_millis() as i64;
    let mut sessions = state.room_access_sessions.lock().await;
    sessions.retain(|_, expiry| *expiry > epoch_millis());
    sessions.insert(token_hash(&token), expires_at);
    json(serde_json::json!({ "token": token, "expiresAt": expires_at }))
}

async fn create_shared_audio_access(
    State(state): State<AppState>,
    Path((room_code, audio_id)): Path<(String, String)>,
) -> Response {
    let party = match find_party(&state, &room_code).await { Ok(party) => party, Err(error) => return error };
    if parse_audio_id(&audio_id).is_none() {
        return failure(StatusCode::BAD_REQUEST, "Invalid audio ID");
    }
    let token = random_token(48);
    let expires_at = epoch_millis() + SHARED_AUDIO_ACCESS_TTL.as_millis() as i64;
    let mut grants = state.shared_audio_access.lock().await;
    grants.retain(|_, grant| grant.expires_at > epoch_millis());
    grants.insert(token_hash(&token), SharedAudioGrant {
        code: party.code.clone(),
        audio_id: audio_id.clone(),
        expires_at,
    });
    json(serde_json::json!({
        "url": format!("/api/v2/rooms/{}/audio/{}?access={token}", party.code, audio_id),
        "expiresAt": expires_at
    }))
}

async fn valid_shared_audio_access(
    state: &AppState,
    token: &str,
    room_code: &str,
    audio_id: &str,
) -> bool {
    if token.is_empty() || token.len() > 128 {
        return false;
    }
    let hash = token_hash(token);
    let current_time = epoch_millis();
    let mut grants = state.shared_audio_access.lock().await;
    grants.retain(|_, grant| grant.expires_at > current_time);
    grants.get(&hash).is_some_and(|grant| {
        shared_audio_grant_matches(grant, room_code, audio_id, current_time)
    })
}

fn random_token(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect()
}

fn failure(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(ErrorBody { error: message.into() })).into_response()
}

fn too_many_requests(retry_after: u64) -> Response {
    let mut response = failure(StatusCode::TOO_MANY_REQUESTS, "Too many attempts. Please wait before trying again");
    response.headers_mut().insert(
        RETRY_AFTER,
        HeaderValue::from_str(&retry_after.to_string()).expect("numeric retry-after is a valid header"),
    );
    response
}

fn json(value: impl Serialize) -> Response { Json(value).into_response() }

fn epoch_millis() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64
}

fn frontend_origin(candidate: Option<&str>) -> String {
    let configured = env::var("FRONTEND_URL").ok().filter(|value| !value.trim().is_empty())
        .or_else(|| env::var("SPOTIFY_REDIRECT_URI").ok())
        .unwrap_or_else(|| "http://localhost:5173".into());
    let configured_origin = reqwest::Url::parse(&configured).map(|url| url.origin().ascii_serialization())
        .unwrap_or_else(|_| "http://localhost:5173".into());
    if let Some(value) = candidate {
        if let Ok(url) = reqwest::Url::parse(value) {
            let origin = url.origin().ascii_serialization();
            let local = url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
            let dioxus = (url.scheme() == "http" && url.host_str() == Some("dioxus.index.html"))
                || (url.scheme() == "dioxus" && url.host_str() == Some("index.html"));
            if dioxus && url.scheme() == "dioxus" {
                return "dioxus://index.html".into();
            }
            if origin == configured_origin || local || dioxus { return origin; }
        }
    }
    configured_origin
}

fn spotify_redirect(origin: &str, status: &str) -> Response {
    let mut url = reqwest::Url::parse(origin).unwrap_or_else(|_| reqwest::Url::parse("http://localhost:5173").unwrap());
    url.set_path("/settings");
    url.set_query(None);
    url.query_pairs_mut().append_pair("spotify", status);
    axum::response::Redirect::temporary(url.as_str()).into_response()
}

fn spotify_desktop_redirect(origin: &str, status: &str, ticket: &str) -> Response {
    let mut url = reqwest::Url::parse(origin).unwrap_or_else(|_| reqwest::Url::parse("http://localhost:5173").unwrap());
    url.set_path("/settings");
    url.set_query(None);
    url.query_pairs_mut().append_pair("spotify", status).append_pair("ticket", ticket);
    axum::response::Redirect::temporary(url.as_str()).into_response()
}

fn is_desktop_return_origin(origin: &str) -> bool {
    reqwest::Url::parse(origin)
        .map(|url| url.scheme() == "http" && url.host_str() == Some("127.0.0.1"))
        .unwrap_or(false)
}

fn spotify_cookie(headers: &HeaderMap) -> Option<String> {
    headers.get_all(COOKIE).iter().filter_map(|header| header.to_str().ok()).flat_map(|value| value.split(';'))
        .find_map(|part| part.trim().strip_prefix("wavelength_spotify=").map(str::to_string))
}

fn allowed_origins(frontend_url: Option<&str>) -> Vec<HeaderValue> {
    let mut origins = vec![
        HeaderValue::from_static("http://dioxus.index.html"),
        HeaderValue::from_static("dioxus://index.html"),
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("http://127.0.0.1:5173"),
        HeaderValue::from_static("http://localhost:8080"),
        HeaderValue::from_static("http://127.0.0.1:8080"),
    ];
    if let Some(frontend_url) = frontend_url {
        if let Ok(url) = reqwest::Url::parse(frontend_url) {
            if let Ok(origin) = HeaderValue::from_str(&url.origin().ascii_serialization()) {
                origins.push(origin);
            }
        }
    }
    origins.sort();
    origins.dedup();
    origins
}

async fn spotify_login(State(state): State<AppState>, Query(query): Query<SpotifyLoginQuery>) -> Response {
    let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap_or_default();
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default();
    let redirect_uri = env::var("SPOTIFY_REDIRECT_URI").unwrap_or_default();
    if client_id.trim().is_empty() || client_secret.trim().is_empty() || redirect_uri.trim().is_empty() {
        return failure(StatusCode::INTERNAL_SERVER_ERROR, "Set SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET, and SPOTIFY_REDIRECT_URI on the room server, then restart it");
    }
    let return_origin = frontend_origin(query.return_origin.as_deref());
    let oauth_state = random_token(40);
    let mut states = state.spotify_states.lock().await;
    states.retain(|_, value| value.expires_at > epoch_millis());
    states.insert(oauth_state.clone(), SpotifyOAuthState { expires_at: epoch_millis() + 600_000, return_origin: return_origin.clone() });
    drop(states);
    let url = match reqwest::Url::parse_with_params("https://accounts.spotify.com/authorize", &[
        ("client_id", client_id.as_str()),
        ("response_type", "code"),
        ("redirect_uri", redirect_uri.as_str()),
        ("scope", "user-read-private user-read-email user-library-read user-library-modify playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private"),
        ("state", oauth_state.as_str()),
    ]) {
        Ok(url) => url,
        Err(_) => return failure(StatusCode::INTERNAL_SERVER_ERROR, "Spotify sign-in URL could not be created"),
    };
    let mut response = axum::response::Redirect::temporary(url.as_str()).into_response();
    let secure = if return_origin.starts_with("https://") { "; Secure" } else { "" };
    let cookie = format!("wavelength_spotify_oauth={oauth_state}; Path=/api/spotify; Max-Age=600; HttpOnly; SameSite=Lax{secure}");
    if let Ok(value) = HeaderValue::from_str(&cookie) { response.headers_mut().insert(SET_COOKIE, value); }
    response
}

async fn spotify_callback(State(state): State<AppState>, headers: HeaderMap, Query(query): Query<SpotifyCallbackQuery>) -> Response {
    let Some(state_key) = query.state.as_deref() else { return spotify_redirect(&frontend_origin(None), "error"); };
    let state_cookie = headers.get_all(COOKIE).iter().filter_map(|header| header.to_str().ok()).flat_map(|value| value.split(';'))
        .find_map(|part| part.trim().strip_prefix("wavelength_spotify_oauth=").map(str::to_string));
    if state_cookie.as_deref() != Some(state_key) { return spotify_redirect(&frontend_origin(None), "error"); }
    let oauth_state = state.spotify_states.lock().await.remove(state_key);
    let Some(oauth_state) = oauth_state.filter(|value| value.expires_at > epoch_millis()) else {
        return spotify_redirect(&frontend_origin(None), "error");
    };
    if query.error.is_some() { return spotify_redirect(&oauth_state.return_origin, "error"); }
    let Some(code) = query.code.as_deref() else { return spotify_redirect(&oauth_state.return_origin, "error"); };
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
        .header(AUTHORIZATION, format!("Basic {credentials}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body).send().await {
            Ok(response) if response.status().is_success() => match response.json::<SpotifyTokenResponse>().await {
                Ok(token) => token,
                Err(_) => return spotify_redirect(&oauth_state.return_origin, "error"),
            },
            _ => return spotify_redirect(&oauth_state.return_origin, "error"),
        };
    let session_id = random_token(48);
    state.spotify_sessions.lock().await.insert(session_id.clone(), SpotifySession {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires_at: epoch_millis() + (token.expires_in.max(60) as i64 * 1000),
    });
    if is_desktop_return_origin(&oauth_state.return_origin) {
        let ticket = random_token(40);
        let mut handoffs = state.spotify_handoffs.lock().await;
        handoffs.retain(|_, value| value.expires_at > epoch_millis());
        handoffs.insert(ticket.clone(), SpotifyHandoff {
            expires_at: epoch_millis() + 5 * 60 * 1000,
            session_id,
        });
        drop(handoffs);
        let mut response = spotify_desktop_redirect(&oauth_state.return_origin, "connected", &ticket);
        response.headers_mut().append(SET_COOKIE, HeaderValue::from_static("wavelength_spotify_oauth=; Path=/api/spotify; Max-Age=0; HttpOnly; SameSite=Lax"));
        return response;
    }
    let cross_origin = oauth_state.return_origin == "dioxus://index.html";
    let secure = if oauth_state.return_origin.starts_with("https://") || cross_origin { "; Secure" } else { "" };
    let same_site = if cross_origin { "None" } else { "Lax" };
    let cookie = format!("wavelength_spotify={session_id}; Path=/; Max-Age=2592000; HttpOnly; SameSite={same_site}{secure}");
    let mut response = spotify_redirect(&oauth_state.return_origin, "connected");
    if let Ok(value) = HeaderValue::from_str(&cookie) { response.headers_mut().append(SET_COOKIE, value); }
    response.headers_mut().append(SET_COOKIE, HeaderValue::from_static("wavelength_spotify_oauth=; Path=/api/spotify; Max-Age=0; HttpOnly; SameSite=Lax"));
    response
}

async fn spotify_desktop_exchange(
    State(state): State<AppState>,
    Json(input): Json<SpotifyDesktopExchange>,
) -> Response {
    if input.ticket.len() != 40
        || !input.ticket.bytes().all(|byte| byte.is_ascii_alphanumeric())
    {
        return failure(StatusCode::BAD_REQUEST, "Spotify sign-in ticket is invalid");
    }
    let handoff = state.spotify_handoffs.lock().await.remove(&input.ticket);
    let Some(handoff) = handoff.filter(|value| value.expires_at > epoch_millis()) else {
        return failure(StatusCode::UNAUTHORIZED, "Spotify sign-in ticket expired");
    };
    if !state.spotify_sessions.lock().await.contains_key(&handoff.session_id) {
        return failure(StatusCode::UNAUTHORIZED, "Spotify sign-in session expired");
    }
    let cookie = format!(
        "wavelength_spotify={}; Path=/; Max-Age=2592000; HttpOnly; SameSite=None; Secure",
        handoff.session_id
    );
    let mut response = json(serde_json::json!({ "connected": true }));
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(SET_COOKIE, value);
    }
    response
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
        .header(AUTHORIZATION, format!("Basic {credentials}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body).send().await.ok()?;
    if !response.status().is_success() { return None; }
    let token = response.json::<SpotifyTokenResponse>().await.ok()?;
    Some(SpotifySession {
        access_token: token.access_token,
        refresh_token: token.refresh_token.or_else(|| session.refresh_token.clone()),
        expires_at: epoch_millis() + (token.expires_in.max(60) as i64 * 1000),
    })
}

async fn spotify_session(state: &AppState, headers: &HeaderMap) -> Result<(String, SpotifySession), Response> {
    let Some(session_id) = spotify_cookie(headers) else { return Err(failure(StatusCode::UNAUTHORIZED, "Spotify is not connected")); };
    let Some(mut session) = state.spotify_sessions.lock().await.get(&session_id).cloned() else {
        return Err(failure(StatusCode::UNAUTHORIZED, "Spotify is not connected"));
    };
    if session.expires_at <= epoch_millis() + 30_000 {
        let Some(refreshed) = refresh_spotify_session(state, &session).await else {
            state.spotify_sessions.lock().await.remove(&session_id);
            return Err(failure(StatusCode::UNAUTHORIZED, "Spotify session expired"));
        };
        state.spotify_sessions.lock().await.insert(session_id.clone(), refreshed.clone());
        session = refreshed;
    }
    Ok((session_id, session))
}

async fn spotify_me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let (_, session) = match spotify_session(&state, &headers).await { Ok(session) => session, Err(error) => return error };
    match state.http.get("https://api.spotify.com/v1/me").bearer_auth(session.access_token).send().await {
        Ok(response) if response.status().is_success() => match response.json::<serde_json::Value>().await { Ok(profile) => json(profile), Err(_) => failure(StatusCode::BAD_GATEWAY, "Spotify profile is temporarily unavailable") },
        Ok(response) if response.status() == StatusCode::UNAUTHORIZED => failure(StatusCode::UNAUTHORIZED, "Spotify session expired"),
        _ => failure(StatusCode::BAD_GATEWAY, "Spotify profile is temporarily unavailable"),
    }
}

fn spotify_search_item(item: &serde_json::Value, item_type: &str) -> serde_json::Value {
    let id = item.get("id").and_then(serde_json::Value::as_str).unwrap_or("");
    let artists = item
        .get("artists")
        .and_then(serde_json::Value::as_array)
        .map(|artists| {
            artists
                .iter()
                .filter_map(|artist| artist.get("name").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let artist = if item_type == "playlist" {
        item.get("owner")
            .and_then(|owner| owner.get("display_name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Spotify")
            .to_string()
    } else {
        artists
    };
    let album = item
        .get("album")
        .and_then(|album| album.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let images = if item_type == "track" {
        item.get("album").and_then(|album| album.get("images"))
    } else {
        item.get("images")
    };
    let thumbnail = images
        .and_then(serde_json::Value::as_array)
        .and_then(|images| images.first())
        .and_then(|image| image.get("url"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let spotify_url = item
        .get("external_urls")
        .and_then(|urls| urls.get("spotify"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let duration = item
        .get("duration_ms")
        .and_then(serde_json::Value::as_u64)
        .map(|milliseconds| milliseconds / 1000)
        .unwrap_or(0);
    let track_count = if item_type == "album" {
        item.get("total_tracks").and_then(serde_json::Value::as_u64)
    } else {
        item.get("tracks")
            .and_then(|tracks| tracks.get("total"))
            .and_then(serde_json::Value::as_u64)
    }
    .unwrap_or(0);

    serde_json::json!({
        "id": id,
        "videoId": id,
        "type": item_type,
        "mediaType": item_type,
        "source": "spotify",
        "title": item.get("name").and_then(serde_json::Value::as_str).unwrap_or("Unknown"),
        "artist": artist,
        "album": album,
        "thumbnail": thumbnail,
        "duration": duration,
        "trackCount": track_count,
        "spotify_uri": item.get("uri").and_then(serde_json::Value::as_str).unwrap_or(""),
        "spotify_url": spotify_url,
    })
}

async fn spotify_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let (_, session) = match spotify_session(&state, &headers).await {
        Ok(session) => session,
        Err(error) => return error,
    };
    let q = query.get("q").map(String::as_str).unwrap_or("").trim();
    if q.is_empty() {
        return failure(StatusCode::BAD_REQUEST, "A Spotify search query is required");
    }
    let limit = query
        .get("limit")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(10)
        .clamp(1, 50);
    let offset = query
        .get("offset")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0)
        .min(1000);
    let response = match state
        .http
        .get("https://api.spotify.com/v1/search")
        .query(&[("q", q)])
        .query(&[("type", "track,album,playlist")])
        .query(&[("limit", limit)])
        .query(&[("offset", offset)])
        .bearer_auth(session.access_token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return failure(StatusCode::BAD_GATEWAY, "Spotify search is temporarily unavailable"),
    };
    if !response.status().is_success() {
        let status = StatusCode::from_u16(response.status().as_u16())
            .unwrap_or(StatusCode::BAD_GATEWAY);
        let message = if status == StatusCode::UNAUTHORIZED {
            "Spotify session expired"
        } else if status == StatusCode::FORBIDDEN {
            "Spotify search is unavailable for this account"
        } else {
            "Spotify search is temporarily unavailable"
        };
        return failure(status, message);
    }
    let payload = match response.json::<serde_json::Value>().await {
        Ok(payload) => payload,
        Err(_) => return failure(StatusCode::BAD_GATEWAY, "Spotify search response was invalid"),
    };
    let mut results = Vec::new();
    let mut total = 0;
    for (item_type, group) in [
        ("track", "tracks"),
        ("album", "albums"),
        ("playlist", "playlists"),
    ] {
        let collection = &payload[group];
        total += collection
            .get("total")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if let Some(items) = collection.get("items").and_then(serde_json::Value::as_array) {
            results.extend(
                items
                    .iter()
                    .filter(|item| item.get("id").and_then(serde_json::Value::as_str).is_some())
                    .map(|item| spotify_search_item(item, item_type)),
            );
        }
    }
    Json(serde_json::json!({ "query": q, "source": "spotify", "results": results, "total": total }))
        .into_response()
}

async fn spotify_items(state: &AppState, token: &str, endpoint: String) -> Result<Vec<serde_json::Value>, StatusCode> {
    let mut next = Some(endpoint);
    let mut items = Vec::new();
    while let Some(url) = next {
        let response = state.http.get(url).bearer_auth(token).send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        if !response.status().is_success() { return Err(StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)); }
        let page = response.json::<serde_json::Value>().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        if let Some(page_items) = page.get("items").and_then(serde_json::Value::as_array) { items.extend(page_items.iter().cloned()); }
        next = page.get("next").and_then(serde_json::Value::as_str).map(str::to_string);
    }
    Ok(items)
}

async fn spotify_library(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let (_, session) = match spotify_session(&state, &headers).await { Ok(session) => session, Err(error) => return error };
    let token = session.access_token;
    let playlists = match spotify_items(&state, &token, "https://api.spotify.com/v1/me/playlists?limit=50".into()).await {
        Ok(items) => items,
        Err(status) => return failure(status, "Spotify playlists are temporarily unavailable"),
    };
    let saved_tracks = match spotify_items(&state, &token, "https://api.spotify.com/v1/me/tracks?limit=50".into()).await {
        Ok(items) => items.into_iter().filter_map(|item| item.get("track").cloned()).collect::<Vec<_>>(),
        Err(status) => return failure(status, "Spotify saved tracks are temporarily unavailable"),
    };
    let mut collections = Vec::new();
    for playlist in playlists {
        let Some(id) = playlist.get("id").and_then(serde_json::Value::as_str) else { continue; };
        let tracks = match spotify_items(&state, &token, format!("https://api.spotify.com/v1/playlists/{id}/tracks?limit=50")).await {
            Ok(items) => items.into_iter().filter_map(|item| item.get("track").cloned()).collect::<Vec<_>>(),
            Err(_) => Vec::new(),
        };
        collections.push(serde_json::json!({ "id": id, "name": playlist.get("name"), "tracks": tracks, "spotifyId": id, "kind": "Spotify playlist" }));
    }
    json(serde_json::json!({ "playlists": collections, "savedTracks": saved_tracks }))
}

fn spotify_track_uris(track_ids: Vec<String>) -> Vec<String> {
    track_ids.into_iter().filter_map(|id| {
        let id = id.strip_prefix("spotify:track:").unwrap_or(&id);
        if !id.is_empty() && id.chars().all(|character| character.is_ascii_alphanumeric()) {
            Some(format!("spotify:track:{id}"))
        } else { None }
    }).collect()
}

async fn spotify_create_playlist(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<SpotifyPlaylistInput>) -> Response {
    let (_, session) = match spotify_session(&state, &headers).await { Ok(session) => session, Err(error) => return error };
    let name = clean(&input.name);
    if name.is_empty() || name.len() > 100 { return failure(StatusCode::BAD_REQUEST, "Playlist name must be between 1 and 100 characters"); }
    let profile = match state.http.get("https://api.spotify.com/v1/me").bearer_auth(&session.access_token).send().await {
        Ok(response) if response.status().is_success() => match response.json::<serde_json::Value>().await { Ok(value) => value, Err(_) => return failure(StatusCode::BAD_GATEWAY, "Spotify profile is unavailable") },
        _ => return failure(StatusCode::BAD_GATEWAY, "Spotify profile is unavailable"),
    };
    let Some(user_id) = profile.get("id").and_then(serde_json::Value::as_str) else { return failure(StatusCode::BAD_GATEWAY, "Spotify profile is unavailable"); };
    match state.http.post(format!("https://api.spotify.com/v1/users/{user_id}/playlists")).bearer_auth(session.access_token)
        .json(&serde_json::json!({ "name": name, "public": false })).send().await {
        Ok(response) if response.status().is_success() => match response.json::<serde_json::Value>().await {
            Ok(playlist) => json(serde_json::json!({ "id": playlist.get("id"), "name": playlist.get("name"), "tracks": [], "spotifyId": playlist.get("id"), "kind": "Spotify playlist" })),
            Err(_) => failure(StatusCode::BAD_GATEWAY, "Spotify playlist could not be read"),
        },
        _ => failure(StatusCode::BAD_GATEWAY, "Spotify playlist could not be created"),
    }
}

async fn spotify_add_playlist_tracks(State(state): State<AppState>, Path(playlist_id): Path<String>, headers: HeaderMap, Json(input): Json<SpotifyTracksInput>) -> Response {
    if playlist_id.is_empty() || !playlist_id.chars().all(|character| character.is_ascii_alphanumeric()) { return failure(StatusCode::BAD_REQUEST, "Invalid Spotify playlist ID"); }
    let (_, session) = match spotify_session(&state, &headers).await { Ok(session) => session, Err(error) => return error };
    let uris = spotify_track_uris(input.track_ids);
    if uris.is_empty() { return failure(StatusCode::BAD_REQUEST, "No Spotify tracks were provided"); }
    match state.http.post(format!("https://api.spotify.com/v1/playlists/{playlist_id}/tracks")).bearer_auth(session.access_token)
        .json(&serde_json::json!({ "uris": uris })).send().await {
        Ok(response) if response.status().is_success() => json(serde_json::json!({ "updated": true })),
        _ => failure(StatusCode::BAD_GATEWAY, "Spotify playlist could not be updated"),
    }
}

async fn spotify_update_saved_tracks(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<SpotifyTracksInput>) -> Response {
    let (_, session) = match spotify_session(&state, &headers).await { Ok(session) => session, Err(error) => return error };
    let ids = spotify_track_uris(input.track_ids).into_iter().filter_map(|uri| uri.strip_prefix("spotify:track:").map(str::to_string)).collect::<Vec<_>>();
    if ids.is_empty() { return failure(StatusCode::BAD_REQUEST, "No Spotify tracks were provided"); }
    match state.http.put("https://api.spotify.com/v1/me/tracks").bearer_auth(session.access_token).json(&serde_json::json!({ "ids": ids })).send().await {
        Ok(response) if response.status().is_success() => json(serde_json::json!({ "updated": true })),
        _ => failure(StatusCode::BAD_GATEWAY, "Spotify liked songs could not be updated"),
    }
}

async fn spotify_remove_saved_tracks(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<SpotifyTracksInput>) -> Response {
    let (_, session) = match spotify_session(&state, &headers).await { Ok(session) => session, Err(error) => return error };
    let ids = spotify_track_uris(input.track_ids).into_iter().filter_map(|uri| uri.strip_prefix("spotify:track:").map(str::to_string)).collect::<Vec<_>>();
    if ids.is_empty() { return failure(StatusCode::BAD_REQUEST, "No Spotify tracks were provided"); }
    match state.http.delete("https://api.spotify.com/v1/me/tracks").bearer_auth(session.access_token).json(&serde_json::json!({ "ids": ids })).send().await {
        Ok(response) if response.status().is_success() => json(serde_json::json!({ "updated": true })),
        _ => failure(StatusCode::BAD_GATEWAY, "Spotify liked songs could not be updated"),
    }
}

async fn spotify_logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(session_id) = spotify_cookie(&headers) { state.spotify_sessions.lock().await.remove(&session_id); }
    let mut response = json(serde_json::json!({ "disconnected": true }));
    response.headers_mut().insert(SET_COOKIE, HeaderValue::from_static("wavelength_spotify=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"));
    response
}

async fn health(State(state): State<AppState>) -> Response {
    let healthy = state.database.run_command(doc! { "ping": 1 }).await.is_ok();
    let status = if healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(serde_json::json!({ "status": if healthy { "ok" } else { "unavailable" } }))).into_response()
}

async fn find_party(state: &AppState, raw_code: &str) -> Result<Party, Response> {
    let party_code = normalize_code(raw_code);
    if !valid_room_code(&party_code) {
        return Err(failure(StatusCode::BAD_REQUEST, "Room code must be 4 to 6 or 12 letters or numbers"));
    }
    let party = state.rooms.parties().find_one(doc! { "code": &party_code }).await
        .map_err(|error| {
            tracing::error!(room_code = %party_code, %error, "Room lookup failed");
            failure(StatusCode::SERVICE_UNAVAILABLE, "Room data is temporarily unavailable")
        })?
        .ok_or_else(|| failure(StatusCode::NOT_FOUND, "Room not found"))?;
    if room_empty_timeout_expired(party.empty_since, now())
        && !state.rooms.has_live_sockets(&party.code).await
    {
        return Err(failure(StatusCode::NOT_FOUND, "Room not found"));
    }
    Ok(party)
}

fn room_update_document(input: &UpdateInput, updated_at: i64) -> Result<Document, mongodb::bson::ser::Error> {
    let mut fields = mongodb::bson::to_document(input)?;
    fields.insert("updatedAt", updated_at);
    Ok(doc! { "$set": fields })
}

fn room_update_filter(room_code: &str, sync_revision: Option<i64>) -> Document {
    let code = normalize_code(room_code);
    match sync_revision {
        Some(revision) => doc! {
            "code": code,
            "$or": [
                { "syncRevision": { "$exists": false } },
                { "syncRevision": { "$lt": revision } },
            ],
        },
        None => doc! { "code": code },
    }
}

async fn persist_server_queue_command(
    service: &RoomService,
    code: &str,
    username: &str,
    command: &RoomCommand,
) -> Result<Option<Party>, String> {
    for _ in 0..3 {
        let party = service
            .parties()
            .find_one(doc! { "code": normalize_code(code) })
            .await
            .map_err(|error| error.to_string())?;
        let Some(mut party) = party else {
            return Ok(None);
        };
        if !apply_server_queue_command(&mut party, command, username) {
            return Ok(None);
        }

        let revision = party.sync_revision.saturating_add(1).max(now());
        let input = UpdateInput {
            queue: Some(party.queue),
            playlists: None,
            current_track_id: None,
            is_playing: None,
            start_epoch: None,
            paused_at: None,
            repeat_mode: None,
            shuffle: None,
            permissions: None,
            sync_revision: Some(revision),
        };
        let update = room_update_document(&input, now()).map_err(|error| error.to_string())?;
        let updated = service
            .parties()
            .find_one_and_update(
                room_update_filter(code, Some(revision)),
                update,
            )
            .return_document(ReturnDocument::After)
            .await
            .map_err(|error| error.to_string())?;
        if updated.is_some() {
            return Ok(updated);
        }
    }
    Ok(None)
}

async fn persist_host_room_state(
    service: &RoomService,
    code: &str,
    mut input: UpdateInput,
) -> Result<Option<Party>, String> {
    for _ in 0..3 {
        let current = service
            .parties()
            .find_one(doc! { "code": normalize_code(code) })
            .await
            .map_err(|error| error.to_string())?;
        let Some(current) = current else {
            return Ok(None);
        };
        let revision = now().max(current.sync_revision.saturating_add(1));
        input.sync_revision = Some(revision);
        let update = room_update_document(&input, now()).map_err(|error| error.to_string())?;
        let update_size = mongodb::bson::to_vec(&update)
            .map_err(|error| error.to_string())?
            .len();
        if update_size > MAX_ROOM_UPDATE_BYTES {
            return Err("Host room state exceeds the data limit".to_string());
        }
        let updated = service
            .parties()
            .find_one_and_update(
                room_update_filter(code, Some(revision)),
                update,
            )
            .return_document(ReturnDocument::After)
            .await
            .map_err(|error| error.to_string())?;
        if updated.is_some() {
            return Ok(updated);
        }
    }
    Ok(None)
}

async fn host_authorized(state: &AppState, room_code: &str, headers: &HeaderMap) -> Result<bool, Response> {
    let Some(token) = headers.get(AUTHORIZATION).and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ")) else { return Ok(false); };
    if token.len() != 64 || !token.is_ascii() { return Ok(false); }
    match state.rooms.party_auth().find_one(doc! { "code": normalize_code(room_code) }).await {
        Ok(Some(auth)) => Ok(token_matches(token, &auth.token_hash)),
        Ok(None) => Ok(false),
        Err(error) => {
            tracing::error!(room_code = %normalize_code(room_code), %error, "Room host authorization lookup failed");
            Err(failure(StatusCode::SERVICE_UNAVAILABLE, "Room authorization is temporarily unavailable"))
        }
    }
}

async fn create_room(State(state): State<AppState>, Json(input): Json<CreateInput>) -> Response {
    let host = clean(input.host.as_deref().or(input.host_nickname.as_deref()).unwrap_or(""));
    if host.is_empty() || host.len() > 50 {
        return failure(StatusCode::BAD_REQUEST, "Host name must be between 1 and 50 characters");
    }

    for _ in 0..10 {
        let room_code = generate_room_code();
        if state.rooms.parties().find_one(doc! { "code": &room_code }).await.ok().flatten().is_some() { continue; }
        let party = Party {
            code: room_code.clone(),
            host: host.clone(),
            members: vec![host.clone()],
            updated_at: now(),
            empty_since: Some(now()),
            ..Default::default()
        };
        if state.rooms.parties().insert_one(&party).await.is_err() { continue; }
        let host_token = random_token(64);
        let auth = PartyAuth { code: room_code.clone(), token_hash: token_hash(&host_token) };
        if state.rooms.party_auth().insert_one(auth).await.is_err() {
            let _ = state.rooms.parties().delete_one(doc! { "code": &room_code }).await;
            return failure(StatusCode::INTERNAL_SERVER_ERROR, "Room authorization could not be stored");
        }
        return json(serde_json::json!({ "code": room_code, "party": party, "hostToken": host_token }));
    }
    failure(StatusCode::CONFLICT, "Unable to generate a unique room code")
}

async fn join_room(
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<JoinInput>,
) -> Response {
    let address = client_ip(address, &headers, &state.trusted_proxies);
    if let Err(retry_after) = state.room_join_rate_limiter.check(address).await {
        return too_many_requests(retry_after);
    }
    let name = clean(input.username.as_deref().or(input.nickname.as_deref()).unwrap_or(""));
    if name.is_empty() || name.len() > 50 {
        return failure(StatusCode::BAD_REQUEST, "Name must be between 1 and 50 characters");
    }
    let party = match find_party(&state, &input.code).await { Ok(party) => party, Err(error) => return error };
    if party.host.eq_ignore_ascii_case(&name) {
        return failure(StatusCode::CONFLICT, "The room host cannot join their own room as a guest");
    }
    match state.rooms.parties().update_one(
        doc! { "code": &party.code },
        doc! { "$addToSet": { "members": &name }, "$set": { "updatedAt": now() } },
    ).await {
        Ok(result) if result.matched_count == 1 => {}
        Ok(_) => return failure(StatusCode::NOT_FOUND, "Room not found"),
        Err(error) => {
            tracing::error!(room_code = %party.code, %error, "Room member could not be added");
            return failure(StatusCode::SERVICE_UNAVAILABLE, "Room membership is temporarily unavailable");
        }
    }
    let party = match find_party(&state, &party.code).await { Ok(party) => party, Err(error) => return error };
    state.rooms.publish(&party.code, "room-state", &party).await;
    json(serde_json::json!({ "code": party.code, "party": party }))
}

async fn leave_room(State(state): State<AppState>, Path(room_code): Path<String>, Json(input): Json<LeaveInput>) -> Response {
    let name = clean(input.username.as_deref().unwrap_or(""));
    if name.is_empty() || name.len() > 50 {
        return failure(StatusCode::BAD_REQUEST, "A valid listener name is required");
    }
    let party = match find_party(&state, &room_code).await { Ok(party) => party, Err(error) => return error };
    if party.host == name {
        return failure(StatusCode::BAD_REQUEST, "The host must end the session instead of leaving it");
    }
    match state.rooms.parties().update_one(
        doc! { "code": &party.code },
        doc! { "$pull": { "members": &name }, "$set": { "updatedAt": now() } },
    ).await {
        Ok(result) if result.matched_count == 1 => {
            if let Ok(Some(updated)) = state.rooms.parties().find_one(doc! { "code": &party.code }).await {
                state.rooms.publish(&party.code, "room-state", &updated).await;
            }
            json(serde_json::json!({ "left": true }))
        }
        Ok(_) => failure(StatusCode::NOT_FOUND, "Room not found"),
        Err(error) => {
            tracing::error!(room_code = %party.code, %error, "Room member could not be removed");
            failure(StatusCode::SERVICE_UNAVAILABLE, "Room membership is temporarily unavailable")
        }
    }
}

async fn read_room(
    State(state): State<AppState>,
    Path(room_code): Path<String>,
    headers: HeaderMap,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Response {
    let address = client_ip(address, &headers, &state.trusted_proxies);
    if let Err(retry_after) = state.room_read_rate_limiter.check(address).await {
        return too_many_requests(retry_after);
    }
    match find_party(&state, &room_code).await {
        Ok(party) => {
            let authorized = match host_authorized(&state, &party.code, &headers).await {
                Ok(authorized) => authorized,
                Err(error) => return error,
            };
            let mut response = serde_json::to_value(party).unwrap_or_default();
            response["hostAuthorized"] = serde_json::json!(authorized);
            json(response)
        }
        Err(error) => error,
    }
}

async fn update_room(
    State(state): State<AppState>,
    Path(room_code): Path<String>,
    headers: HeaderMap,
    body: Result<Json<UpdateInput>, JsonRejection>,
) -> Response {
    let Json(input) = match body {
        Ok(body) => body,
        Err(error) => {
            let status = error.status();
            let message = if status == StatusCode::PAYLOAD_TOO_LARGE {
                "Room update exceeds the 8 MiB request limit"
            } else {
                "Room update must contain valid JSON"
            };
            return failure(status, message);
        }
    };
    if input.sync_revision.is_some_and(|revision| revision < 0) {
        return failure(StatusCode::BAD_REQUEST, "Room state revision must be non-negative");
    }
    let party = match find_party(&state, &room_code).await { Ok(party) => party, Err(error) => return error };
    match host_authorized(&state, &party.code, &headers).await {
        Ok(true) => {}
        Ok(false) => return failure(StatusCode::UNAUTHORIZED, "A valid room host token is required"),
        Err(error) => return error,
    }
    let update = match room_update_document(&input, now()) {
        Ok(update) => update,
        Err(error) => {
            tracing::error!(room_code = %party.code, %error, "Room update could not be serialized");
            return failure(StatusCode::BAD_REQUEST, "Room update contains unsupported data");
        }
    };
    let update_size = match mongodb::bson::to_vec(&update) {
        Ok(bytes) => bytes.len(),
        Err(error) => {
            tracing::error!(room_code = %party.code, %error, "Room update size could not be checked");
            return failure(StatusCode::BAD_REQUEST, "Room update contains unsupported data");
        }
    };
    if update_size > MAX_ROOM_UPDATE_BYTES {
        return failure(StatusCode::PAYLOAD_TOO_LARGE, "Room update exceeds the 8 MiB data limit");
    }
    match state.rooms.parties().find_one_and_update(room_update_filter(&party.code, input.sync_revision), update)
        .return_document(ReturnDocument::After).await {
        Ok(Some(updated)) => {
            state.rooms.publish(&party.code, "room-state", &updated).await;
            json(updated)
        }
        Ok(None) => match find_party(&state, &party.code).await {
            Ok(current) => json(current),
            Err(error) => error,
        },
        Err(error) => {
            tracing::error!(room_code = %party.code, %error, "Room state update failed");
            failure(StatusCode::SERVICE_UNAVAILABLE, "Room state could not be saved; retry sync")
        }
    }
}

async fn close_room(State(state): State<AppState>, Path(room_code): Path<String>, headers: HeaderMap) -> Response {
    let party = match find_party(&state, &room_code).await { Ok(party) => party, Err(error) => return error };
    match host_authorized(&state, &party.code, &headers).await {
        Ok(true) => {}
        Ok(false) => return failure(StatusCode::UNAUTHORIZED, "A valid room host token is required"),
        Err(error) => return error,
    }
    match state.rooms.parties().delete_one(doc! { "code": &party.code }).await {
        Ok(result) if result.deleted_count == 1 => {
            let _ = state.rooms.party_auth().delete_one(doc! { "code": &party.code }).await;
            state.rooms.publish(&party.code, "room-closed", &party.code).await;
            json(serde_json::json!({ "closed": true }))
        }
        Ok(_) => failure(StatusCode::CONFLICT, "Room could not be closed"),
        Err(_) => failure(StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
    }
}

async fn create_room_socket_ticket(
    State(state): State<AppState>,
    Path(room_code): Path<String>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<RoomSocketTicketInput>,
) -> Response {
    let address = client_ip(address, &headers, &state.trusted_proxies);
    if let Err(retry_after) = state.room_socket_ticket_rate_limiter.check(address).await {
        return too_many_requests(retry_after);
    }
    let username = clean(&input.username);
    if username.is_empty() || username.len() > 50 {
        return failure(StatusCode::BAD_REQUEST, "A valid room name is required");
    }
    let access_token = headers
        .get("x-room-access-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let session_expires_at = match room_access_session_expiry(&state, access_token).await {
        Some(expires_at) => expires_at,
        None => return failure(StatusCode::UNAUTHORIZED, ROOM_ACCESS_REQUIRED_MESSAGE),
    };
    let party = match find_party(&state, &room_code).await {
        Ok(party) => party,
        Err(error) => return error,
    };
    if !party.members.iter().any(|member| member == &username) {
        return failure(StatusCode::FORBIDDEN, "Join the room before opening its live connection");
    }
    if party.host == username {
        match host_authorized(&state, &party.code, &headers).await {
            Ok(true) => {}
            Ok(false) => return failure(StatusCode::UNAUTHORIZED, "A valid room host token is required"),
            Err(error) => return error,
        }
    }
    let is_host = party.host == username;
    let ticket = state.rooms.create_socket_ticket(
        &party.code,
        &username,
        session_expires_at,
        is_host,
    ).await;
    json(serde_json::json!({ "ticket": ticket }))
}

async fn room_socket(
    State(state): State<AppState>,
    Path(room_code): Path<String>,
    Query(auth): Query<RoomSocketAuth>,
    ws: WebSocketUpgrade,
) -> Response {
    let code = normalize_code(&room_code);
    if !valid_room_code(&code) {
        return failure(StatusCode::UNAUTHORIZED, "A valid room session is required");
    }
    let Some(ticket) = state.rooms.consume_socket_ticket(&auth.ticket).await else {
        return failure(StatusCode::UNAUTHORIZED, "The room connection ticket is invalid or expired");
    };
    if ticket.code != code || ticket.session_expires_at <= epoch_millis() {
        return failure(StatusCode::UNAUTHORIZED, "The room connection ticket is invalid or expired");
    }
    let username = ticket.username;
    let is_host = ticket.is_host;

    let party = match find_party(&state, &code).await {
        Ok(party) => party,
        Err(error) if error.status() == StatusCode::NOT_FOUND => {
            let closed = serde_json::json!({
                "event": "room-closed",
                "data": code,
                "sender": null,
            }).to_string();
            return ws.on_upgrade(move |mut socket| async move {
                let _ = socket.send(Message::Text(closed.into())).await;
                let _ = socket.send(Message::Close(None)).await;
            });
        }
        Err(error) => return error,
    };
    if !party.members.iter().any(|member| member == &username) {
        return failure(StatusCode::FORBIDDEN, "Join the room before opening its live connection");
    }

    let service = state.rooms.clone();
    ws.on_upgrade(move |socket| handle_room_socket(
        socket,
        service,
        code,
        username,
        ticket.session_expires_at,
        is_host,
    ))
}

async fn handle_room_socket(
    mut socket: WebSocket,
    service: RoomService,
    code: String,
    username: String,
    expires_at: i64,
    is_host: bool,
) {
    let socket_id = random_token(16);
    let mut events = service.subscribe(&code).await;
    match service.connected(&code, &username).await {
        Ok(true) => {}
        Ok(false) => {
            let closed = serde_json::json!({
                "event": "room-closed",
                "data": code,
                "sender": null,
            }).to_string();
            let _ = socket.send(Message::Text(closed.into())).await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
        Err(error) => {
            tracing::error!(room_code = %code, %error, "Room presence could not be registered");
            let failure = serde_json::json!({
                "event": "error",
                "data": "Room presence could not be registered; retry the connection.",
                "sender": null,
            }).to_string();
            let _ = socket.send(Message::Text(failure.into())).await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    }
    match service.parties().find_one(doc! { "code": &code }).await {
        Ok(Some(party)) => service.publish(&code, "room-state", &party).await,
        Ok(None) => {
            service.disconnected(&code, &username).await;
            let closed = serde_json::json!({
                "event": "room-closed",
                "data": code,
                "sender": null,
            }).to_string();
            let _ = socket.send(Message::Text(closed.into())).await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
        Err(error) => {
            tracing::error!(room_code = %code, %error, "Room snapshot could not be loaded for a live connection");
            service.disconnected(&code, &username).await;
            let failure = serde_json::json!({
                "event": "error",
                "data": "Room state could not be loaded; retry the connection.",
                "sender": null,
            }).to_string();
            let _ = socket.send(Message::Text(failure.into())).await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    }

    let (mut outgoing, mut incoming) = socket.split();
    let connected = serde_json::json!({
        "event": "connected",
        "data": { "socketId": socket_id },
        "sender": null,
    });
    if outgoing.send(Message::Text(connected.to_string().into())).await.is_err() {
        service.disconnected(&code, &username).await;
        return;
    }
    let mut last_progress_persist = Instant::now();
    let session_deadline = tokio::time::Instant::now()
        + Duration::from_millis(expires_at.saturating_sub(epoch_millis()).max(0) as u64);
    loop {
        tokio::select! {
            message = incoming.next() => {
                match message {
                    Some(Ok(Message::Text(text))) => {
                        let Ok(message) = serde_json::from_str::<RoomSocketMessage>(&text) else {
                            tracing::warn!(room_code = %code, username = %username, "Ignoring malformed room socket message");
                            continue;
                        };
                        match message.event.as_str() {
                            "room-command" if !is_host => {
                                let party = match service.parties().find_one(doc! { "code": &code }).await {
                                    Ok(Some(party)) => party,
                                    Ok(None) => {
                                        let _ = outgoing.send(Message::Close(None)).await;
                                        break;
                                    }
                                    Err(error) => {
                                        tracing::error!(room_code = %code, %error, "Room command authorization lookup failed");
                                        let failure = serde_json::json!({
                                            "event": "error",
                                            "data": "Room command could not be authorized; try again.",
                                            "sender": null,
                                        });
                                        let _ = outgoing.send(Message::Text(failure.to_string().into())).await;
                                        continue;
                                    }
                                };
                                let Ok(command) = serde_json::from_value::<RoomCommand>(message.data) else { continue };
                                if party.members.iter().any(|member| member == &username)
                                    && room_command_allowed(&party, &command)
                                {
                                    if is_server_queue_command(&command) {
                                        match persist_server_queue_command(&service, &code, &username, &command).await {
                                            Ok(Some(updated)) => {
                                                service.publish(&code, "room-state", &updated).await;
                                            }
                                            Ok(None) => {}
                                            Err(error) => {
                                                tracing::error!(room_code = %code, %error, "Room queue command could not be saved");
                                                let failure = serde_json::json!({
                                                    "event": "error",
                                                    "data": "The room queue could not be saved; try again.",
                                                    "sender": null,
                                                });
                                                let _ = outgoing.send(Message::Text(failure.to_string().into())).await;
                                            }
                                        }
                                    } else {
                                        service.publish_from(
                                            &code,
                                            "room-command",
                                            RoomCommandEnvelope { username: username.clone(), command },
                                            Some(&socket_id),
                                        ).await;
                                    }
                                }
                            }
                            "host-room-state" if is_host => {
                                let Ok(input) = serde_json::from_value::<UpdateInput>(message.data) else {
                                    continue;
                                };
                                if input.sync_revision.is_none()
                                    || input.sync_revision.is_some_and(|revision| revision < 0)
                                    || input.queue.as_ref().is_some_and(|queue| {
                                        queue.len() > MAX_ROOM_QUEUE_TRACKS
                                            || queue.iter().any(|track| !room_track_is_shareable(track))
                                    })
                                    || input.repeat_mode.as_ref().is_some_and(|mode| {
                                        !matches!(mode.as_str(), "off" | "all" | "one")
                                    })
                                {
                                    continue;
                                }
                                match persist_host_room_state(&service, &code, input).await {
                                    Ok(Some(updated)) => {
                                        service.publish_from(
                                            &code,
                                            "room-state",
                                            &updated,
                                            Some(&socket_id),
                                        ).await;
                                    }
                                    Ok(None) => {}
                                    Err(error) => {
                                        tracing::error!(room_code = %code, %error, "Host room state could not be saved");
                                        let failure = serde_json::json!({
                                            "event": "error",
                                            "data": "Room state could not be saved; reconnect to retry.",
                                            "sender": null,
                                        });
                                        let _ = outgoing.send(Message::Text(failure.to_string().into())).await;
                                    }
                                }
                            }
                            "playback-control" if is_host => {
                                let Ok(control) = serde_json::from_value::<PlaybackControl>(message.data) else { continue };
                                if !control.current_track_id.is_empty()
                                    && control.current_track_id.len() <= 256
                                    && control.current_time.is_finite()
                                    && (0.0..=86_400.0).contains(&control.current_time)
                                    && control.sent_at_epoch_ms > 0
                                {
                                    let updated_at = now();
                                    let input = UpdateInput {
                                        queue: None,
                                        playlists: None,
                                        current_track_id: Some(control.current_track_id.clone()),
                                        is_playing: Some(control.is_playing),
                                        start_epoch: Some(updated_at.saturating_sub((control.current_time * 1000.0) as i64)),
                                        paused_at: Some(control.current_time),
                                        repeat_mode: None,
                                        shuffle: None,
                                        permissions: None,
                                        sync_revision: None,
                                    };
                                    let update = match room_update_document(&input, updated_at) {
                                        Ok(update) => update,
                                        Err(error) => {
                                            tracing::error!(room_code = %code, %error, "Playback state could not be serialized");
                                            continue;
                                        }
                                    };
                                    match service
                                        .parties()
                                        .find_one_and_update(
                                            doc! { "code": &code },
                                            update,
                                        )
                                        .return_document(ReturnDocument::After)
                                        .await
                                    {
                                        Ok(Some(_)) => {}
                                        Ok(None) => continue,
                                        Err(error) => {
                                            tracing::error!(room_code = %code, %error, "Playback state could not be saved");
                                            let failure = serde_json::json!({
                                                "event": "error",
                                                "data": "Playback state could not be saved; try again.",
                                                "sender": null,
                                            });
                                            let _ = outgoing.send(Message::Text(failure.to_string().into())).await;
                                            continue;
                                        }
                                    }
                                    service.publish_from(&code, "playback-control", control, Some(&socket_id)).await;
                                }
                            }
                            "playback-progress" if is_host => {
                                let Ok(progress) = serde_json::from_value::<PlaybackProgress>(message.data) else { continue };
                                if !progress.current_track_id.is_empty()
                                    && progress.current_track_id.len() <= 256
                                    && progress.current_time.is_finite()
                                    && (0.0..=86_400.0).contains(&progress.current_time)
                                    && progress.duration.is_finite()
                                    && (0.0..=86_400.0).contains(&progress.duration)
                                {
                                    if last_progress_persist.elapsed() >= Duration::from_secs(5) {
                                        let updated_at = now();
                                        let input = UpdateInput {
                                            queue: None,
                                            playlists: None,
                                            current_track_id: Some(progress.current_track_id.clone()),
                                            is_playing: Some(true),
                                            start_epoch: Some(updated_at.saturating_sub((progress.current_time * 1000.0) as i64)),
                                            paused_at: Some(progress.current_time),
                                            repeat_mode: None,
                                            shuffle: None,
                                            permissions: None,
                                            sync_revision: None,
                                        };
                                        if let Ok(update) = room_update_document(&input, updated_at) {
                                            if let Err(error) = service
                                                .parties()
                                                .update_one(doc! { "code": &code }, update)
                                                .await
                                            {
                                                tracing::error!(room_code = %code, %error, "Playback progress could not be saved");
                                            }
                                        }
                                        last_progress_persist = Instant::now();
                                    }
                                    service.publish_from(&code, "playback-progress", progress, Some(&socket_id)).await;
                                }
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if outgoing.send(Message::Pong(payload)).await.is_err() { break; }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        let is_own_event = serde_json::from_str::<serde_json::Value>(&event)
                            .ok()
                            .and_then(|value| value.get("sender").and_then(serde_json::Value::as_str).map(str::to_owned))
                            .is_some_and(|sender| sender == socket_id);
                        if !is_own_event && outgoing.send(Message::Text(event.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = tokio::time::sleep_until(session_deadline) => {
                let _ = outgoing.send(Message::Close(None)).await;
                break;
            }
        }
    }
    service.disconnected(&code, &username).await;
    if let Ok(Some(party)) = service.parties().find_one(doc! { "code": &code }).await {
        service.publish(&code, "room-state", &party).await;
    }
}

const SHARED_AUDIO_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MAX_SHARED_AUDIO_BYTES: u64 = 1024 * 1024 * 1024;

fn valid_audio_extension(extension: &str) -> bool {
    matches!(extension, "mp3" | "flac" | "m4a" | "ogg" | "wav" | "aac" | "opus")
}

fn audio_content_type(extension: &str) -> &'static str {
    match extension {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "m4a" => "audio/mp4",
        "ogg" | "opus" => "audio/ogg",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        _ => "application/octet-stream",
    }
}

fn parse_audio_id(audio_id: &str) -> Option<(&str, &str)> {
    let (digest, extension) = audio_id.split_once('.')?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) || !valid_audio_extension(extension) {
        return None;
    }
    Some((digest, extension))
}

fn parse_byte_range(value: &str, file_size: u64) -> Option<(u64, u64)> {
    if file_size == 0 || !value.starts_with("bytes=") || value.contains(',') { return None; }
    let (start, end) = value[6..].split_once('-')?;
    let (start, end) = if start.is_empty() {
        let suffix = end.parse::<u64>().ok()?;
        if suffix == 0 { return None; }
        (file_size.saturating_sub(suffix), file_size - 1)
    } else {
        let start = start.parse::<u64>().ok()?;
        let end = if end.is_empty() { file_size - 1 } else { end.parse::<u64>().ok()? };
        (start, end.min(file_size - 1))
    };
    (start < file_size && end >= start).then_some((start, end))
}

async fn upload_shared_audio(State(state): State<AppState>, Path(room_code): Path<String>, request: Request<Body>) -> Response {
    let party = match find_party(&state, &room_code).await { Ok(party) => party, Err(error) => return error };
    match host_authorized(&state, &party.code, request.headers()).await {
        Ok(true) => {}
        Ok(false) => return failure(StatusCode::UNAUTHORIZED, "A valid room host token is required"),
        Err(error) => return error,
    }
    let Some(extension) = request.headers().get("x-audio-extension").and_then(|value| value.to_str().ok())
        .map(str::to_ascii_lowercase).filter(|value| valid_audio_extension(value)) else {
        return failure(StatusCode::BAD_REQUEST, "Unsupported audio file type");
    };
    let directory = state.rooms.audio_root().join("uploads").join(&party.code);
    if tokio::fs::create_dir_all(&directory).await.is_err() {
        return failure(StatusCode::INTERNAL_SERVER_ERROR, "Audio storage is unavailable");
    }
    let temporary_path = directory.join(format!(".upload-{}", random_token(24)));
    let mut file = match tokio::fs::File::create(&temporary_path).await {
        Ok(file) => file,
        Err(_) => return failure(StatusCode::INTERNAL_SERVER_ERROR, "Audio storage is unavailable"),
    };
    let mut body = request.into_body().into_data_stream();
    let mut hasher = Sha256::new();
    let mut total_bytes = 0u64;
    while let Some(chunk) = body.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(_) => {
                drop(file);
                let _ = tokio::fs::remove_file(&temporary_path).await;
                return failure(StatusCode::BAD_REQUEST, "Audio upload was interrupted");
            }
        };
        total_bytes += chunk.len() as u64;
        if total_bytes > MAX_SHARED_AUDIO_BYTES {
            drop(file);
            let _ = tokio::fs::remove_file(&temporary_path).await;
            return failure(StatusCode::PAYLOAD_TOO_LARGE, "Audio files must be smaller than 1 GiB");
        }
        hasher.update(&chunk);
        if file.write_all(&chunk).await.is_err() {
            drop(file);
            let _ = tokio::fs::remove_file(&temporary_path).await;
            return failure(StatusCode::INTERNAL_SERVER_ERROR, "Audio file could not be saved");
        }
    }
    if total_bytes == 0 || file.flush().await.is_err() {
        drop(file);
        let _ = tokio::fs::remove_file(&temporary_path).await;
        return failure(StatusCode::BAD_REQUEST, "Audio file is empty or could not be saved");
    }
    drop(file);
    let audio_id = format!("{:x}.{extension}", hasher.finalize());
    let destination = directory.join(&audio_id);
    if tokio::fs::rename(&temporary_path, destination).await.is_err() {
        let _ = tokio::fs::remove_file(&temporary_path).await;
        return failure(StatusCode::INTERNAL_SERVER_ERROR, "Audio file could not be saved");
    }
    json(serde_json::json!({ "audioId": audio_id, "size": total_bytes }))
}

async fn stream_shared_audio(
    State(state): State<AppState>,
    Path((room_code, audio_id)): Path<(String, String)>,
    Query(query): Query<SharedAudioQuery>,
    request: Request<Body>,
) -> Response {
    let Some(address) = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0)
    else {
        return failure(StatusCode::INTERNAL_SERVER_ERROR, "Client address is unavailable");
    };
    let address = client_ip(address, request.headers(), &state.trusted_proxies);
    if let Err(retry_after) = state.shared_audio_stream_rate_limiter.check(address).await {
        return too_many_requests(retry_after);
    }
    let party = match find_party(&state, &room_code).await { Ok(party) => party, Err(error) => return error };
    if !valid_shared_audio_access(&state, &query.access, &party.code, &audio_id).await {
        return failure(StatusCode::UNAUTHORIZED, "Shared audio access has expired");
    }
    let Some((_, extension)) = parse_audio_id(&audio_id) else {
        return failure(StatusCode::BAD_REQUEST, "Invalid audio ID");
    };
    let path = state.rooms.audio_root().join("uploads").join(party.code).join(&audio_id);
    let metadata = match tokio::fs::metadata(&path).await {
        Ok(metadata) if metadata.is_file() => metadata,
        _ => return failure(StatusCode::NOT_FOUND, "Shared audio has expired or does not exist"),
    };
    if metadata.modified().ok().and_then(|modified| SystemTime::now().duration_since(modified).ok()).unwrap_or_default() >= SHARED_AUDIO_TTL {
        let _ = tokio::fs::remove_file(&path).await;
        return failure(StatusCode::NOT_FOUND, "Shared audio has expired or does not exist");
    }
    let file_size = metadata.len();
    let requested_range = request.headers().get(RANGE).and_then(|value| value.to_str().ok());
    let (start, end, status) = match requested_range {
        Some(value) => match parse_byte_range(value, file_size) {
            Some((start, end)) => (start, end, StatusCode::PARTIAL_CONTENT),
            None => {
                let mut response = failure(StatusCode::RANGE_NOT_SATISFIABLE, "Invalid byte range");
                if let Ok(value) = HeaderValue::from_str(&format!("bytes */{file_size}")) {
                    response.headers_mut().insert(CONTENT_RANGE, value);
                }
                return response;
            }
        },
        None if file_size > 0 => (0, file_size - 1, StatusCode::OK),
        None => return failure(StatusCode::NOT_FOUND, "Shared audio file is empty"),
    };
    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(_) => return failure(StatusCode::NOT_FOUND, "Shared audio has expired or does not exist"),
    };
    if file.seek(SeekFrom::Start(start)).await.is_err() {
        return failure(StatusCode::INTERNAL_SERVER_ERROR, "Audio file could not be read");
    }
    let content_length = end - start + 1;
    let mut response = Response::new(Body::from_stream(ReaderStream::new(file.take(content_length))));
    *response.status_mut() = status;
    response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(audio_content_type(extension)));
    response.headers_mut().insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    if let Ok(value) = HeaderValue::from_str(&content_length.to_string()) { response.headers_mut().insert(CONTENT_LENGTH, value); }
    if status == StatusCode::PARTIAL_CONTENT {
        if let Ok(value) = HeaderValue::from_str(&format!("bytes {start}-{end}/{file_size}")) {
            response.headers_mut().insert(CONTENT_RANGE, value);
        }
    }
    response
}

async fn cleanup_expired_audio(audio_root: PathBuf) {
    let upload_root = audio_root.join("uploads");
    let Ok(mut rooms) = tokio::fs::read_dir(&upload_root).await else { return; };
    while let Ok(Some(room)) = rooms.next_entry().await {
        if !matches!(room.file_type().await, Ok(kind) if kind.is_dir()) { continue; }
        let Ok(mut files) = tokio::fs::read_dir(room.path()).await else { continue; };
        let mut has_files = false;
        while let Ok(Some(entry)) = files.next_entry().await {
            if !matches!(entry.file_type().await, Ok(kind) if kind.is_file()) { continue; }
            let expired = entry.metadata().await.ok().and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .is_some_and(|age| age >= SHARED_AUDIO_TTL);
            if expired { let _ = tokio::fs::remove_file(entry.path()).await; }
            else { has_files = true; }
        }
        if !has_files { let _ = tokio::fs::remove_dir(room.path()).await; }
    }
}

async fn cleanup_idle_rooms(state: &AppState) {
    let cutoff = room_empty_cutoff(now());
    let mut rooms = match state.rooms.parties().find(doc! { "emptySince": { "$lte": cutoff } }).await {
        Ok(rooms) => rooms,
        Err(error) => {
            tracing::error!(%error, "Idle room cleanup query failed");
            return;
        }
    };

    while let Some(result) = rooms.next().await {
        let party = match result {
            Ok(party) => party,
            Err(error) => {
                tracing::error!(%error, "Idle room cleanup could not read a room");
                continue;
            }
        };
        if state.rooms.has_live_sockets(&party.code).await {
            continue;
        }
        match state.rooms.parties().delete_one(doc! {
            "code": &party.code,
            "emptySince": { "$lte": cutoff },
        }).await {
            Ok(result) if result.deleted_count == 1 => {
                if let Err(error) = state.rooms.party_auth().delete_one(doc! { "code": &party.code }).await {
                    tracing::error!(room_code = %party.code, %error, "Idle room authorization cleanup failed");
                }
                state.rooms.publish(&party.code, "room-closed", &party.code).await;
                info!(room_code = %party.code, "Closed room after five minutes without activity");
            }
            Ok(_) => {}
            Err(error) => {
                tracing::error!(room_code = %party.code, %error, "Idle room cleanup failed");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("wavelength_backend=info").init();
    let passcode = env::var("ROOM_SERVER_PASSCODE")
        .expect("ROOM_SERVER_PASSCODE must be configured before starting the room server");
    if !(16..=128).contains(&passcode.len()) {
        panic!("ROOM_SERVER_PASSCODE must contain between 16 and 128 bytes");
    }
    let room_passcode_hash = hash_room_server_passcode(&passcode, ROOM_SERVER_PASSCODE_BCRYPT_COST)
        .expect("hash room server passcode with bcrypt");
    let trusted_proxies = env::var("ROOM_SERVER_TRUSTED_PROXIES")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::parse::<IpAddr>)
        .collect::<Result<HashSet<_>, _>>()
        .expect("ROOM_SERVER_TRUSTED_PROXIES must contain comma-separated IP addresses");
    let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://mongodb:27017/wavelength".into());
    let client = Client::with_uri_str(uri).await.expect("connect to MongoDB");
    let database = client.database("wavelength");
    let state = AppState {
        database: database.clone(),
        rooms: RoomService::new(
            database.collection("parties"),
            database.collection("party_auth"),
            PathBuf::from(env::var("MUSIC_ROOT").unwrap_or_else(|_| "/music".into())),
        ),
        http: reqwest::Client::new(),
        spotify_states: Arc::new(Mutex::new(HashMap::new())),
        spotify_sessions: Arc::new(Mutex::new(HashMap::new())),
        spotify_handoffs: Arc::new(Mutex::new(HashMap::new())),
        room_passcode_hash,
        trusted_proxies,
        passcode_rate_limiter: IpRateLimiter::new(8),
        room_join_rate_limiter: IpRateLimiter::new(10),
        room_read_rate_limiter: IpRateLimiter::new(60),
        room_socket_ticket_rate_limiter: IpRateLimiter::new(60),
        shared_audio_stream_rate_limiter: IpRateLimiter::new(60),
        room_access_sessions: Arc::new(Mutex::new(HashMap::new())),
        shared_audio_access: Arc::new(Mutex::new(HashMap::new())),
    };
    state.rooms.parties().update_many(
        doc! {},
        doc! { "$set": { "emptySince": now() }, "$setOnInsert": { "updatedAt": now() } },
    ).await.expect("initialize empty-room timers");
    state.rooms.parties().update_many(
        doc! { "updatedAt": { "$exists": false } },
        doc! { "$set": { "updatedAt": now() } },
    ).await.expect("initialize room timestamps");
    let origins = allowed_origins(env::var("FRONTEND_URL").ok().as_deref());
    cleanup_expired_audio(state.rooms.audio_root().to_path_buf()).await;
    let cleanup_root = state.rooms.audio_root().to_path_buf();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        loop {
            interval.tick().await;
            cleanup_expired_audio(cleanup_root.clone()).await;
        }
    });
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(ROOM_CLEANUP_INTERVAL);
        loop {
            interval.tick().await;
            cleanup_idle_rooms(&cleanup_state).await;
        }
    });
    let v2_api = Router::new()
        .route("/session", post(create_room_access_session))
        .route("/rooms", post(create_room))
        .route("/rooms/join", post(join_room))
        .route("/rooms/:code/leave", post(leave_room))
        .route("/rooms/:code/socket-ticket", post(create_room_socket_ticket))
        .route("/rooms/:code/socket", get(room_socket))
        .route("/rooms/:code", get(read_room).put(update_room).delete(close_room)
            .layer(DefaultBodyLimit::max(MAX_ROOM_UPDATE_BYTES)))
        .route("/rooms/:code/audio", post(upload_shared_audio))
        .route("/rooms/:code/audio/:audio_id", get(stream_shared_audio))
        .route("/rooms/:code/audio/:audio_id/access", get(create_shared_audio_access))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_room_access));
    let spotify_api = Router::new()
        .route("/spotify/login", get(spotify_login))
        .route("/spotify/callback", get(spotify_callback))
        .route("/spotify/desktop-exchange", post(spotify_desktop_exchange))
        .route("/spotify/me", get(spotify_me))
        .route("/spotify/search", get(spotify_search))
        .route("/spotify/library", get(spotify_library))
        .route("/spotify/logout", post(spotify_logout))
        .route("/spotify/playlists", post(spotify_create_playlist))
        .route("/spotify/playlists/:playlist_id/tracks", post(spotify_add_playlist_tracks))
        .route("/spotify/saved-tracks", put(spotify_update_saved_tracks).delete(spotify_remove_saved_tracks))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_room_access));
    let app = Router::new()
        .route("/health", get(health))
        .nest("/api/v2", v2_api)
        .nest("/api", spotify_api)
        .layer(CorsLayer::new()
            .allow_origin(origins)
            .allow_headers([
                AUTHORIZATION,
                CONTENT_TYPE,
                HeaderName::from_static("x-audio-extension"),
                HeaderName::from_static("x-room-access-token"),
            ])
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE])
            .expose_headers([RETRY_AFTER])
            .allow_credentials(true))
        .with_state(state);
    let port = env::var("PORT").unwrap_or_else(|_| "5000".into()).parse().unwrap_or(5000);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%address, "Wavelength room server listening");
    let listener = tokio::net::TcpListener::bind(address).await.expect("bind room server port");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("serve room API");
}

#[cfg(test)]
mod tests {
    use super::{
        allowed_origins, client_ip, frontend_origin, hash_room_server_passcode,
        generate_room_code, is_shared_audio_stream_path, is_shared_audio_upload_path,
        normalize_code, parse_audio_id, parse_byte_range,
        apply_server_queue_command, public_api_request, room_command_allowed, room_empty_cutoff, room_empty_timeout_expired,
        room_update_document, room_update_filter,
        shared_audio_grant_matches, spotify_track_uris, token_hash, token_matches,
        too_many_requests, valid_room_code, verify_room_server_passcode, IpRateLimiter, Party,
        RoomSocketMessage,
        Permissions, RoomCommand, SharedAudioGrant, UpdateInput, LEGACY_ROOM_CODE_LENGTH,
        ROOM_IDLE_TIMEOUT, SHARED_AUDIO_TTL,
    };
    use axum::http::{HeaderMap, HeaderValue, Method};
    use std::{collections::HashSet, net::{IpAddr, SocketAddr}};

    #[test]
    fn dioxus_desktop_webview_origins_are_allowed() {
        assert_eq!(
            frontend_origin(Some("http://dioxus.index.html")),
            "http://dioxus.index.html"
        );
        assert_eq!(
            frontend_origin(Some("dioxus://index.html")),
            "dioxus://index.html"
        );
        assert_ne!(
            frontend_origin(Some("http://dioxus.evil")),
            "http://dioxus.evil"
        );
    }

    #[test]
    fn dioxus_web_dev_origins_are_allowed_for_cors() {
        let origins = allowed_origins(None);
        for origin in ["http://localhost:8080", "http://127.0.0.1:8080"] {
            assert!(
                origins.contains(&HeaderValue::from_static(origin)),
                "missing allowed origin {origin}"
            );
        }
    }

    #[test]
    fn configured_frontend_origin_is_allowed_for_cors() {
        let origins = allowed_origins(Some("https://music.example.com/app"));
        assert!(origins.contains(&HeaderValue::from_static("https://music.example.com")));
    }

    #[test]
    fn shared_audio_ids_and_ranges_are_validated() {
        let audio_id = format!("{}.flac", "a".repeat(64));
        assert!(parse_audio_id(&audio_id).is_some());
        assert!(parse_audio_id(&format!("../{audio_id}")).is_none());
        assert_eq!(parse_byte_range("bytes=10-19", 100), Some((10, 19)));
        assert_eq!(parse_byte_range("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse_byte_range("bytes=100-", 100), None);
        assert_eq!(SHARED_AUDIO_TTL.as_secs(), 7 * 24 * 60 * 60);
    }

    #[test]
    fn room_codes_are_normalized_and_bounded() {
        assert_eq!(normalize_code(" ab12 "), "AB12");
        assert!(valid_room_code("AB12"));
        assert!(valid_room_code("ABC123"));
        assert!(valid_room_code(&"A".repeat(LEGACY_ROOM_CODE_LENGTH)));
        assert!(!valid_room_code("ABC"));
        assert!(!valid_room_code("ABCDEFG"));
        assert!(!valid_room_code(&"A".repeat(LEGACY_ROOM_CODE_LENGTH + 1)));
    }

    #[test]
    fn new_room_codes_are_four_alphanumeric_characters() {
        let code = generate_room_code();
        assert_eq!(code.len(), 4);
        assert!(code.chars().all(|character| character.is_ascii_alphanumeric()));
    }

    #[test]
    fn empty_rooms_expire_five_minutes_after_the_last_socket_disconnect() {
        let current_time = 1_000_000;
        let idle_timeout = ROOM_IDLE_TIMEOUT.as_millis() as i64;

        assert_eq!(idle_timeout, 5 * 60 * 1000);
        assert_eq!(room_empty_cutoff(current_time), current_time - idle_timeout);
        assert!(!room_empty_timeout_expired(
            Some(current_time - idle_timeout + 1),
            current_time
        ));
        assert!(room_empty_timeout_expired(
            Some(current_time - idle_timeout),
            current_time
        ));
        assert!(room_empty_timeout_expired(
            Some(current_time - idle_timeout - 1),
            current_time
        ));
        assert!(!room_empty_timeout_expired(None, current_time));
        assert!(!room_empty_timeout_expired(Some(current_time + 1), current_time));
    }

    #[test]
    fn server_passcode_bcrypt_handles_long_inputs() {
        let passcode = "p".repeat(128);
        let hash = hash_room_server_passcode(&passcode, 4).unwrap();
        assert!(verify_room_server_passcode(&passcode, &hash).unwrap());
        assert!(!verify_room_server_passcode("different-passcode", &hash).unwrap());
    }

    #[tokio::test]
    async fn request_rate_limiter_is_per_ip_and_enforces_attempt_limit() {
        let limiter = IpRateLimiter::new(2);
        let client: IpAddr = "192.0.2.1".parse().unwrap();
        let other_client: IpAddr = "192.0.2.2".parse().unwrap();

        assert!(limiter.check(client).await.is_ok());
        assert!(limiter.check(client).await.is_ok());
        assert!(limiter.check(client).await.is_err());
        assert!(limiter.check(other_client).await.is_ok());
    }

    #[test]
    fn rate_limit_response_includes_retry_after() {
        let response = too_many_requests(30);
        assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers().get("retry-after").unwrap(), "30");
    }

    #[test]
    fn client_ip_headers_are_used_only_for_trusted_proxies() {
        let proxy_ip: IpAddr = "192.0.2.10".parse().unwrap();
        let client_address: IpAddr = "198.51.100.42".parse().unwrap();
        let proxy = SocketAddr::new(proxy_ip, 8080);
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.42"));
        let trusted_proxies = HashSet::from([proxy_ip]);

        assert_eq!(client_ip(proxy, &headers, &trusted_proxies), client_address);
        assert_eq!(
            client_ip(
                SocketAddr::new("203.0.113.5".parse().unwrap(), 8080),
                &headers,
                &trusted_proxies,
            ),
            "203.0.113.5".parse::<IpAddr>().unwrap(),
        );
    }

    #[test]
    fn cloudflare_client_ip_is_used_only_for_trusted_proxies() {
        let proxy_ip: IpAddr = "192.0.2.10".parse().unwrap();
        let client_address: IpAddr = "198.51.100.42".parse().unwrap();
        let proxy = SocketAddr::new(proxy_ip, 8080);
        let mut headers = HeaderMap::new();
        headers.insert(
            "cf-connecting-ip",
            HeaderValue::from_static("198.51.100.42"),
        );
        let trusted_proxies = HashSet::from([proxy_ip]);

        assert_eq!(client_ip(proxy, &headers, &trusted_proxies), client_address);
        assert_eq!(
            client_ip(
                SocketAddr::new("203.0.113.5".parse().unwrap(), 8080),
                &headers,
                &trusted_proxies,
            ),
            "203.0.113.5".parse::<IpAddr>().unwrap(),
        );
    }

    #[test]
    fn x_forwarded_for_chain_skips_trusted_proxies_and_ignores_spoofed_prefixes() {
        let proxy_ip: IpAddr = "192.0.2.10".parse().unwrap();
        let upstream_proxy_ip: IpAddr = "192.0.2.11".parse().unwrap();
        let client_address: IpAddr = "198.51.100.42".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.9, 198.51.100.42, 192.0.2.11"),
        );
        let trusted_proxies = HashSet::from([proxy_ip, upstream_proxy_ip]);

        assert_eq!(
            client_ip(
                SocketAddr::new(proxy_ip, 8080),
                &headers,
                &trusted_proxies,
            ),
            client_address,
        );
    }

    #[test]
    fn forwarded_header_supports_quoted_ipv6_addresses_with_ports() {
        let proxy_ip: IpAddr = "192.0.2.10".parse().unwrap();
        let client_address: IpAddr = "2001:db8::1".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            "forwarded",
            HeaderValue::from_static("for=\"[2001:db8::1]:4711\";proto=https"),
        );
        let trusted_proxies = HashSet::from([proxy_ip]);

        assert_eq!(
            client_ip(
                SocketAddr::new(proxy_ip, 8080),
                &headers,
                &trusted_proxies,
            ),
            client_address,
        );
    }

    #[test]
    fn host_tokens_are_verified_by_hash() {
        let token = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzAB";
        let expected = token_hash(token);
        assert!(token_matches(token, &expected));
        assert!(!token_matches("different-token", &expected));
    }

    #[test]
    fn room_api_auth_exempts_only_v2_session_socket_and_oauth_entry_points() {
        assert!(public_api_request(&Method::POST, "/api/v2/session"));
        assert!(public_api_request(&Method::GET, "/api/v2/rooms/AB12/socket"));
        assert!(public_api_request(&Method::GET, "/api/spotify/login"));
        assert!(public_api_request(&Method::GET, "/api/spotify/callback"));
        assert!(!public_api_request(&Method::GET, "/api/v2/session"));
        assert!(!public_api_request(&Method::POST, "/api/auth/session"));
        assert!(!public_api_request(&Method::GET, "/api/v2/rooms"));
        assert!(!public_api_request(&Method::POST, "/api/v2/rooms/AB12/socket-ticket"));
        assert!(!public_api_request(&Method::GET, "/api/spotify/me"));

        let audio_id = format!("{}.flac", "a".repeat(64));
        assert!(is_shared_audio_stream_path(
            &Method::GET,
            &format!("/api/v2/rooms/AB12/audio/{audio_id}")
        ));
        assert!(!is_shared_audio_stream_path(
            &Method::GET,
            &format!("/api/v2/rooms/AB12/audio/{audio_id}/access")
        ));
        assert!(!is_shared_audio_stream_path(
            &Method::POST,
            &format!("/api/v2/rooms/AB12/audio/{audio_id}")
        ));
        assert!(is_shared_audio_upload_path(
            &Method::POST,
            "/api/v2/rooms/AB12/audio"
        ));
        assert!(!is_shared_audio_upload_path(
            &Method::POST,
            "/api/v2/rooms/AB12/audio/not-an-upload"
        ));
    }

    #[test]
    fn direct_websocket_protocol_uses_json_event_envelopes() {
        let message: RoomSocketMessage = serde_json::from_str(
            r#"{"event":"room-command","data":{"action":"clear-queue"}}"#,
        ).unwrap();
        assert_eq!(message.event, "room-command");
        assert_eq!(message.data["action"], "clear-queue");
        let outgoing = serde_json::json!({
            "event": "room-state",
            "data": { "code": "AB12" },
            "sender": null,
        });
        assert_eq!(outgoing["event"], "room-state");
        assert_eq!(outgoing["data"]["code"], "AB12");
    }

    #[test]
    fn shared_audio_grants_are_bound_to_room_audio_and_expiry() {
        let grant = SharedAudioGrant {
            code: "AB12".into(),
            audio_id: "track.flac".into(),
            expires_at: 20,
        };
        assert!(shared_audio_grant_matches(&grant, "AB12", "track.flac", 19));
        assert!(!shared_audio_grant_matches(&grant, "CD34", "track.flac", 19));
        assert!(!shared_audio_grant_matches(&grant, "AB12", "other.flac", 19));
        assert!(!shared_audio_grant_matches(&grant, "AB12", "track.flac", 20));
    }

    #[test]
    fn spotify_track_uris_reject_non_track_ids() {
        assert_eq!(spotify_track_uris(vec!["spotify:track:abc123".into(), "bad/id".into()]), vec!["spotify:track:abc123"]);
    }

    #[test]
    fn desktop_spotify_handoffs_only_use_loopback_origins() {
        assert!(super::is_desktop_return_origin("http://127.0.0.1:49152"));
        assert!(!super::is_desktop_return_origin("https://127.0.0.1:49152"));
        assert!(!super::is_desktop_return_origin("http://example.test"));
    }

    #[test]
    fn spotify_search_results_are_normalized_for_the_client() {
        let track = serde_json::json!({
            "id": "track123",
            "uri": "spotify:track:track123",
            "name": "Song",
            "artists": [{"name": "Artist"}],
            "album": {
                "name": "Record",
                "images": [{"url": "https://example.test/cover.jpg"}]
            },
            "duration_ms": 183000,
            "external_urls": {"spotify": "https://open.spotify.com/track/track123"}
        });
        let formatted = super::spotify_search_item(&track, "track");
        assert_eq!(formatted["id"], "track123");
        assert_eq!(formatted["artist"], "Artist");
        assert_eq!(formatted["album"], "Record");
        assert_eq!(formatted["duration"], 183);
        assert_eq!(formatted["thumbnail"], "https://example.test/cover.jpg");

        let playlist = serde_json::json!({
            "id": "playlist123",
            "name": "Mix",
            "owner": {"display_name": "Listener"},
            "tracks": {"total": 17}
        });
        let formatted = super::spotify_search_item(&playlist, "playlist");
        assert_eq!(formatted["type"], "playlist");
        assert_eq!(formatted["artist"], "Listener");
        assert_eq!(formatted["trackCount"], 17);
    }

    #[test]
    fn room_update_start_epoch_requires_integer_milliseconds() {
        let fractional = serde_json::json!({ "startEpoch": 1700000000123.5 });
        assert!(serde_json::from_value::<UpdateInput>(fractional).is_err());

        let rounded = serde_json::json!({ "startEpoch": 1700000000124_i64 });
        assert!(serde_json::from_value::<UpdateInput>(rounded).is_ok());
    }

    #[test]
    fn room_updates_only_set_fields_that_are_present() {
        let input = UpdateInput {
            queue: Some(Vec::new()),
            playlists: None,
            current_track_id: None,
            is_playing: None,
            start_epoch: None,
            paused_at: None,
            repeat_mode: None,
            shuffle: None,
            permissions: None,
            sync_revision: None,
        };
        let update = room_update_document(&input, 123).unwrap();
        let fields = update.get_document("$set").unwrap();
        assert!(fields.contains_key("queue"));
        assert!(!fields.contains_key("playlists"));
        assert_eq!(fields.get_i64("updatedAt").unwrap(), 123);
    }

    #[test]
    fn room_revision_filter_rejects_stale_snapshots() {
        let filter = room_update_filter("ab12", Some(8));
        assert_eq!(filter.get_str("code").unwrap(), "AB12");
        let alternatives = filter.get_array("$or").unwrap();
        assert_eq!(alternatives.len(), 2);
        assert_eq!(
            alternatives[1]
                .as_document()
                .unwrap()
                .get_document("syncRevision")
                .unwrap()
                .get_i64("$lt")
                .unwrap(),
            8
        );
        assert!(!room_update_filter("AB12", None).contains_key("$or"));
    }

    #[test]
    fn room_commands_follow_guest_permissions() {
        let mut party = Party {
            permissions: Permissions { can_add: true, ..Default::default() },
            ..Default::default()
        };
        let add_track = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "add-track",
            "track": { "id": "spotify-id", "source": "spotify" }
        })).unwrap();
        let clear_queue = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "clear-queue"
        })).unwrap();
        let toggle_playback = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "toggle-playback"
        })).unwrap();

        assert!(room_command_allowed(&party, &add_track));
        assert!(!room_command_allowed(&party, &clear_queue));
        assert!(!room_command_allowed(&party, &toggle_playback));

        party.permissions.can_manage_queue = true;
        party.permissions.can_control_playback = true;
        assert!(room_command_allowed(&party, &clear_queue));
        assert!(room_command_allowed(&party, &toggle_playback));
    }

    #[test]
    fn server_queue_commands_mutate_canonical_queue() {
        let mut party = Party {
            permissions: Permissions {
                can_add: true,
                can_manage_queue: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let add_track = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "add-track",
            "track": { "id": "spotify:track:one", "source": "spotify" }
        }))
        .unwrap();
        let add_second_track = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "add-track",
            "track": { "id": "spotify:track:two", "source": "spotify" }
        }))
        .unwrap();
        let reorder = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "reorder-queue",
            "trackId": "spotify:track:one",
            "targetIndex": 1
        }))
        .unwrap();
        let remove = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "remove-track",
            "trackId": "spotify:track:one"
        }))
        .unwrap();
        let clear = serde_json::from_value::<RoomCommand>(serde_json::json!({
            "action": "clear-queue"
        }))
        .unwrap();

        assert!(apply_server_queue_command(&mut party, &add_track, "Guest"));
        assert_eq!(party.queue[0].added_by, "Guest");
        assert!(!apply_server_queue_command(&mut party, &add_track, "Guest"));
        assert!(apply_server_queue_command(
            &mut party,
            &add_second_track,
            "Guest"
        ));
        assert!(apply_server_queue_command(&mut party, &reorder, "Guest"));
        assert_eq!(party.queue[1].id, "spotify:track:one");
        assert!(apply_server_queue_command(&mut party, &remove, "Guest"));
        assert_eq!(party.queue[0].id, "spotify:track:two");
        assert!(apply_server_queue_command(&mut party, &clear, "Guest"));
        assert!(party.queue.is_empty());
    }
}
