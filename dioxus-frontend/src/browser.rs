use crate::{
    DownloadStatus, DownloadStatusResponse, EqualizerSettings, LyricsResponse, RoomPermissions,
    RoomSocketEvent, SpotifyPlaylist, SpotifyRestore, Track, UpdateInfo,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::{format_error, AUDIO_SERVICE};

#[cfg(target_arch = "wasm32")]
#[path = "browser_web.rs"]
mod implementation;

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[path = "browser_desktop.rs"]
mod implementation;

pub(crate) use implementation::*;
