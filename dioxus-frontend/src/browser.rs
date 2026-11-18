use crate::{
    AccessResult, DownloadStatus, DownloadStatusResponse, EqualizerSettings, LyricsResponse,
    RoomParty, RoomPermissions, RoomResult, RoomSocketEvent, SpotifyAlbum, SpotifyPlaylist,
    SpotifyRestore, Track, UpdateInfo,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::{AUDIO_SERVICE, format_error};

#[cfg(target_arch = "wasm32")]
#[path = "browser_web.rs"]
mod implementation;

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[path = "browser_desktop.rs"]
mod implementation;

pub(crate) use implementation::*;

#[derive(Clone)]
pub(crate) struct RoomClient {
    url: String,
    access_token: String,
}

impl RoomClient {
    pub(crate) fn new(url: &str, access_token: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            access_token: access_token.to_string(),
        }
    }

    pub(crate) async fn authenticate(&self, passcode: &str) -> Result<AccessResult, String> {
        room_request(
            &self.url,
            "/session",
            "POST",
            &serde_json::json!({ "passcode": passcode }),
            "",
            "",
        )
        .await
    }

    pub(crate) async fn create(&self, host: &str) -> Result<RoomResult, String> {
        room_request(
            &self.url,
            "/rooms",
            "POST",
            &serde_json::json!({ "host": host }),
            &self.access_token,
            "",
        )
        .await
    }

    pub(crate) async fn join(&self, code: &str, username: &str) -> Result<RoomResult, String> {
        room_request(
            &self.url,
            "/rooms/join",
            "POST",
            &serde_json::json!({ "code": code, "username": username }),
            &self.access_token,
            "",
        )
        .await
    }

    pub(crate) async fn read(&self, code: &str, host_token: &str) -> Result<RoomParty, String> {
        room_request(
            &self.url,
            &format!("/rooms/{code}"),
            "GET",
            &serde_json::json!({}),
            &self.access_token,
            host_token,
        )
        .await
    }

    pub(crate) async fn update(
        &self,
        code: &str,
        body: &serde_json::Value,
        host_token: &str,
    ) -> Result<RoomParty, String> {
        room_request(
            &self.url,
            &format!("/rooms/{code}"),
            "PUT",
            body,
            &self.access_token,
            host_token,
        )
        .await
    }

    pub(crate) async fn leave(&self, code: &str, username: &str) -> Result<(), String> {
        room_request::<serde_json::Value>(
            &self.url,
            &format!("/rooms/{code}/leave"),
            "POST",
            &serde_json::json!({ "username": username }),
            &self.access_token,
            "",
        )
        .await
        .map(|_| ())
    }

    pub(crate) async fn end(&self, code: &str, host_token: &str) -> Result<(), String> {
        room_request::<serde_json::Value>(
            &self.url,
            &format!("/rooms/{code}"),
            "DELETE",
            &serde_json::json!({}),
            &self.access_token,
            host_token,
        )
        .await
        .map(|_| ())
    }
}
