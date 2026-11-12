import os
import asyncio
import logging
import shutil
import json
import re
import time
from collections import deque
from pathlib import Path
from threading import Lock
from urllib.parse import parse_qs, urlparse
from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, StreamingResponse
from starlette.background import BackgroundTask
from dotenv import load_dotenv
from pydantic import BaseModel
import spotipy
from spotipy.exceptions import SpotifyException
from spotipy.oauth2 import SpotifyClientCredentials
from spotipy.cache_handler import CacheFileHandler
from SpotiFLAC import SpotiFLAC
import httpx
from yt_dlp import YoutubeDL
from playlist import format_track, load_album, load_playlist, load_public_album, load_public_playlist, load_public_track, spotify_album_id, spotify_playlist_id

load_dotenv()

if not os.getenv("SPOTIFLAC_REGISTRIES"):
    os.environ["SPOTIFLAC_REGISTRIES"] = os.getenv(
        "MORNYE_REGISTRIES",
        "https://raw.githubusercontent.com/mornyeapp/Mornye-Extension/main/registry.json",
    )

app = FastAPI()
app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_methods=["*"], allow_headers=["*"])

_data_dir = os.getenv("WAVELENGTH_DATA_DIR")
CACHE_DIR = Path(_data_dir) / "cache" if _data_dir else Path(__file__).parent / "cache"
CACHE_DIR.mkdir(exist_ok=True)
STATUS_FILE = CACHE_DIR / "download-status.json"
LOG_FILE = CACHE_DIR / "download-logs.json"
_log_file_mtime_ns = 0
SPOTIFY_TOKEN_CACHE = CACHE_DIR / "spotipy-token-cache"

logging.basicConfig(level=os.getenv("LOG_LEVEL", "WARNING").upper())
logger = logging.getLogger("wavelength-audio")

_downloading = set()
_download_errors = {}
_download_status = {}
_download_progress = {}
_download_retention = {}
_download_scopes = {}
_download_log_events = deque(maxlen=500)
_download_log_lock = Lock()
_download_tasks = {}
_cancelled_downloads = set()
_spotiflac_lock = asyncio.Lock()
_last_download_started = 0.0
_streaming_tracks = set()
QOBUZ_API_URL = os.getenv("QOBUZ_API_URL", "http://qobuz-api:8000")
QOBUZ_API_KEY = os.getenv("QOBUZ_API_KEY", "").strip()
_qobuz_credentials = {}
CACHE_MAX_AGE_SECONDS = int(os.getenv("CACHE_MAX_AGE_SECONDS", str(7 * 24 * 60 * 60)))
ROOM_DOWNLOAD_TTL_SECONDS = 14 * 24 * 60 * 60
CACHE_CLEANUP_INTERVAL_SECONDS = min(
    60 * 60,
    max(60, int(os.getenv("CACHE_CLEANUP_INTERVAL_SECONDS", str(60 * 60)))),
)
DOWNLOAD_DELAY_SECONDS = max(0.0, float(os.getenv("DOWNLOAD_DELAY_SECONDS", "2")))
_cache_cleanup_task = None


def _record_download_event(level, message, track_id=None, source=None, progress=None):
    global _log_file_mtime_ns
    event = {
        "timestamp": int(time.time() * 1000),
        "level": level,
        "message": message,
        "trackId": track_id,
        "source": source,
        "progress": progress,
    }
    with _download_log_lock:
        _download_log_events.append(event)
        temp_file = LOG_FILE.with_suffix(".tmp")
        try:
            temp_file.write_text(json.dumps(list(_download_log_events)), encoding="utf-8")
            temp_file.replace(LOG_FILE)
            _log_file_mtime_ns = LOG_FILE.stat().st_mtime_ns
        except OSError:
            logger.exception("Unable to persist audio event log")
    getattr(logger, level if level in {"debug", "info", "warning", "error"} else "info")(message)

try:
    _download_log_events.extend(json.loads(LOG_FILE.read_text(encoding="utf-8"))[-500:])
except (FileNotFoundError, json.JSONDecodeError, OSError):
    pass
try:
    _log_file_mtime_ns = LOG_FILE.stat().st_mtime_ns
except OSError:
    pass

try:
    _download_status.update(json.loads(STATUS_FILE.read_text(encoding="utf-8")))
except (FileNotFoundError, json.JSONDecodeError, OSError):
    pass

try:
    retention_file = CACHE_DIR / "download-retention.json"
    retention_data = json.loads(retention_file.read_text(encoding="utf-8"))
    if isinstance(retention_data, dict):
        _download_retention.update(retention_data)
except (FileNotFoundError, json.JSONDecodeError, OSError):
    pass

_record_download_event("info", "Audio service started", source="server")

def _save_download_status():
    temp_file = STATUS_FILE.with_suffix(".tmp")
    temp_file.write_text(json.dumps(_download_status), encoding="utf-8")
    temp_file.replace(STATUS_FILE)


def _save_download_retention():
    retention_file = CACHE_DIR / "download-retention.json"
    temp_file = retention_file.with_suffix(".tmp")
    temp_file.write_text(json.dumps(_download_retention), encoding="utf-8")
    temp_file.replace(retention_file)


def _normalize_download_scope(scope):
    return "room" if str(scope or "").lower() == "room" else "permanent"


def _mark_download_permanent(track_id):
    if _download_retention.pop(track_id, None) is not None:
        _save_download_retention()


def _merge_download_scope(track_id, scope):
    scope = _normalize_download_scope(scope)
    if _download_scopes.get(track_id) == "permanent" or scope == "permanent":
        _download_scopes[track_id] = "permanent"
    else:
        _download_scopes[track_id] = "room"
    return _download_scopes[track_id]


def _mark_room_download_requested(track_id):
    entry = _download_retention.get(track_id)
    if not isinstance(entry, dict) or entry.get("scope") != "room":
        _download_retention[track_id] = {"scope": "room", "expiresAt": None}
        _save_download_retention()


def _mark_cached_download(track_id, scope):
    if scope != "room":
        _mark_download_permanent(track_id)
        return

    entry = _download_retention.get(track_id)
    if isinstance(entry, dict) and entry.get("scope") == "room":
        entry["expiresAt"] = time.time() + ROOM_DOWNLOAD_TTL_SECONDS
        _save_download_retention()


def _mark_download_complete(track_id, scope):
    if scope != "room":
        _mark_download_permanent(track_id)
        return

    _download_retention[track_id] = {
        "scope": "room",
        "expiresAt": time.time() + ROOM_DOWNLOAD_TTL_SECONDS,
    }
    _save_download_retention()


def _discard_pending_room_download(track_id):
    entry = _download_retention.get(track_id)
    if isinstance(entry, dict) and entry.get("scope") == "room" and entry.get("expiresAt") is None:
        _download_retention.pop(track_id, None)
        _save_download_retention()


def _download_scope_for(track_id):
    entry = _download_retention.get(track_id)
    return "room" if isinstance(entry, dict) and entry.get("scope") == "room" else "permanent"


def _cleanup_cache():
    now = time.time()
    removable_extensions = {".flac", ".m4a", ".mp3", ".wav", ".webm", ".ogg", ".opus", ".mp4"}
    removed_tracks = []
    for track_id, entry in list(_download_retention.items()):
        if not isinstance(entry, dict) or entry.get("scope") != "room":
            continue
        expires_at = entry.get("expiresAt")
        if expires_at is None or float(expires_at) > now:
            continue
        if track_id in _downloading or track_id in _streaming_tracks:
            continue
        for path in CACHE_DIR.iterdir():
            if path.is_file() and path.stem == track_id and path.suffix.lower() in removable_extensions:
                path.unlink(missing_ok=True)
        removed_tracks.append(track_id)

    for track_id in removed_tracks:
        _download_retention.pop(track_id, None)
        _download_status.pop(track_id, None)
        _download_progress.pop(track_id, None)
        _download_errors.pop(track_id, None)

    if removed_tracks:
        _save_download_retention()
        _save_download_status()

    cutoff = now - CACHE_MAX_AGE_SECONDS
    temp_root = CACHE_DIR / "spotiflac_temp"
    if temp_root.exists():
        for path in temp_root.iterdir():
            if path.is_dir() and path.stat().st_mtime < cutoff and path.name not in _downloading:
                shutil.rmtree(path, ignore_errors=True)


async def _cache_cleanup_loop():
    while True:
        await asyncio.sleep(CACHE_CLEANUP_INTERVAL_SECONDS)
        try:
            await asyncio.to_thread(_cleanup_cache)
        except Exception:
            logger.exception("[Cache] cleanup failed")


@app.on_event("startup")
async def start_cache_cleanup():
    global _cache_cleanup_task
    await asyncio.to_thread(_cleanup_cache)
    _cache_cleanup_task = asyncio.create_task(_cache_cleanup_loop(), name="audio-cache-cleanup")


@app.on_event("shutdown")
async def stop_cache_cleanup():
    if _cache_cleanup_task:
        _cache_cleanup_task.cancel()

# ----------------------------------------------------------------------
# Search providers
# ----------------------------------------------------------------------
def _spotify_track_id(value: str):
    match = re.search(r'(?:open\.spotify\.com/(?:intl-[a-z]{2}(?:-[a-z]{2})?/)?track/|spotify:track:)([A-Za-z0-9]+)', value)
    return match.group(1) if match else None


def _youtube_video_id(value: str):
    candidate = value.strip()
    if not re.match(r'^https?://', candidate, re.IGNORECASE):
        candidate = f'https://{candidate}'
    parsed = urlparse(candidate)
    host = (parsed.hostname or '').lower()
    video_id = None
    if host == 'youtu.be' or host.endswith('.youtu.be'):
        video_id = parsed.path.strip('/').split('/', 1)[0]
    elif host == 'youtube.com' or host.endswith('.youtube.com') or host == 'youtube-nocookie.com' or host.endswith('.youtube-nocookie.com'):
        if parsed.path.rstrip('/') == '/watch':
            video_id = parse_qs(parsed.query).get('v', [None])[0]
        else:
            match = re.match(r'^/(?:shorts|embed|live)/([A-Za-z0-9_-]+)', parsed.path)
            video_id = match.group(1) if match else None
    return video_id if video_id and re.fullmatch(r'[A-Za-z0-9_-]{6,}', video_id) else None


def format_search_collection(collection, result_type):
    collection = collection or {}
    artists = collection.get("artists") or []
    artist = ", ".join(item.get("name", "") for item in artists if isinstance(item, dict) and item.get("name"))
    if result_type == "playlist":
        artist = (collection.get("owner") or {}).get("display_name", "Spotify")
    images = collection.get("images") or []
    thumbnail = next((image.get("url") for image in images if isinstance(image, dict) and image.get("url")), None)
    return {
        "type": result_type,
        "videoId": collection.get("id"),
        "spotify_uri": collection.get("uri"),
        "title": collection.get("name"),
        "artist": artist or "Spotify",
        "thumbnail": thumbnail,
        "trackCount": collection.get("tracks", {}).get("total", 0),
    }


def _spotify_client(client_id=None, client_secret=None, access_token=None):
    if access_token:
        return spotipy.Spotify(auth=access_token)
    client_id = client_id or os.getenv("SPOTIFY_CLIENT_ID")
    client_secret = client_secret or os.getenv("SPOTIFY_CLIENT_SECRET")
    if not client_id or not client_secret:
        raise HTTPException(400, "Spotify credentials required")
    credentials = SpotifyClientCredentials(
        client_id=client_id,
        client_secret=client_secret,
        cache_handler=CacheFileHandler(cache_path=str(SPOTIFY_TOKEN_CACHE)),
    )
    return spotipy.Spotify(client_credentials_manager=credentials)


def _normalize_track_text(value):
    return re.sub(r"[^a-z0-9]+", " ", str(value or "").casefold()).strip()


def _format_musical_scale(key, mode):
    key_names = ("C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B")
    try:
        key = int(key)
        mode = int(mode)
    except (TypeError, ValueError):
        return ""
    if not 0 <= key < len(key_names) or mode not in (0, 1):
        return ""
    return f"{key_names[key]} {'major' if mode == 1 else 'minor'}"


@app.get("/track-metadata")
async def track_metadata(
    title: str,
    artist: str = "",
    spotify_id: str = "",
    client_id: str = None,
    client_secret: str = None,
    access_token: str = None,
):
    metadata = {"bpm": None, "scale": "", "releaseDate": ""}
    try:
        spotify = _spotify_client(client_id, client_secret, access_token)
        track_id = _spotify_track_id(spotify_id) or spotify_id
        if not track_id:
            search_title = re.sub(
                rf"^\s*{re.escape(artist)}\s*[-–—:]\s*", "", title, flags=re.IGNORECASE
            ) if artist else title
            search_title = re.sub(
                r"\s*[([](?:official|lyrics|music video|audio|visualizer|video)[^\])]*[\])]?\s*$",
                "",
                search_title,
                flags=re.IGNORECASE,
            ).strip()
            results = await asyncio.to_thread(
                spotify.search,
                q=f"{search_title} {artist}".strip(),
                limit=10,
                type="track",
            )
            expected_title = _normalize_track_text(search_title)
            expected_artist = _normalize_track_text(artist)
            for candidate in results.get("tracks", {}).get("items", []):
                candidate_title = _normalize_track_text(candidate.get("name"))
                candidate_artist = _normalize_track_text(
                    " ".join(item.get("name", "") for item in candidate.get("artists", []))
                )
                title_matches = (
                    candidate_title == expected_title
                    or (len(candidate_title) > 2 and candidate_title in expected_title)
                    or (len(expected_title) > 2 and expected_title in candidate_title)
                )
                artist_matches = (
                    not expected_artist
                    or expected_artist in candidate_artist
                    or candidate_artist in expected_artist
                )
                if title_matches and artist_matches:
                    track_id = candidate.get("id", "")
                    break
        if not track_id:
            return metadata

        track = await asyncio.to_thread(spotify.track, track_id)
        metadata["releaseDate"] = (track.get("album") or {}).get("release_date") or ""
        try:
            features = await asyncio.to_thread(spotify.audio_features, [track_id])
            feature = features[0] if features else None
            if feature:
                tempo = feature.get("tempo")
                metadata["bpm"] = round(float(tempo)) if tempo is not None else None
                metadata["scale"] = _format_musical_scale(
                    feature.get("key"), feature.get("mode")
                )
        except SpotifyException as error:
            logger.info("[Spotify] audio features unavailable: %s", error)
    except Exception as error:
        logger.info("[Spotify] track metadata unavailable: %s", error)
    return metadata


@app.get("/search")
async def search(
    q: str,
    limit: int = 15,
    offset: int = 0,
    source: str = "spotify",
    client_id: str = None,
    client_secret: str = None,
    access_token: str = None
):
    try:
        if source == "youtube":
            video_id = _youtube_video_id(q)
            options = {
                "quiet": True,
                "skip_download": True,
                "ignoreerrors": True,
                "extract_flat": "in_playlist" if not video_id else False,
            }
            target = f"https://www.youtube.com/watch?v={video_id}" if video_id else f"ytsearch{min(limit, 20)}:{q}"

            def extract_youtube_search():
                with YoutubeDL(options) as ydl:
                    return ydl.extract_info(target, download=False)

            result = await asyncio.to_thread(extract_youtube_search)
            formatted = []
            items = [result] if video_id and result else (result or {}).get("entries", [])
            for item in items:
                if not item or not item.get("id"):
                    continue
                thumbnails = item.get("thumbnails") or []
                thumbnail = item.get("thumbnail") or next(
                    (image.get("url") for image in reversed(thumbnails) if image.get("url")),
                    f"https://i.ytimg.com/vi/{item['id']}/hqdefault.jpg",
                )
                duration = item.get("duration")
                duration_string = item.get("duration_string")
                if not duration_string and isinstance(duration, (int, float)):
                    total_seconds = int(duration)
                    minutes, seconds = divmod(total_seconds, 60)
                    hours, minutes = divmod(minutes, 60)
                    duration_string = f"{hours}:{minutes:02d}:{seconds:02d}" if hours else f"{minutes}:{seconds:02d}"
                formatted.append({
                    "type": "track",
                    "source": "youtube",
                    "videoId": item["id"],
                    "title": item.get("title") or "Untitled video",
                    "artist": item.get("channel") or item.get("uploader") or "YouTube",
                    "album": "YouTube",
                    "thumbnail": thumbnail,
                    "length": duration_string or "—",
                    "url": item.get("webpage_url") or f"https://www.youtube.com/watch?v={item['id']}",
                })
            return {"query": q, "source": "youtube", "results": formatted, "total": len(formatted)}

        if source != "spotify":
            raise HTTPException(400, "Unsupported search source")

        playlist_id = spotify_playlist_id(q)
        album_id = spotify_album_id(q)
        track_id = _spotify_track_id(q)
        public_url = bool(playlist_id or album_id or track_id)

        if public_url:
            try:
                if playlist_id:
                    formatted, playlist_info = await asyncio.to_thread(load_public_playlist, playlist_id)
                    return {"query": q, "results": formatted, "playlist": playlist_info}
                if album_id:
                    formatted, album_info = await asyncio.to_thread(load_public_album, album_id)
                    return {"query": q, "results": formatted, "playlist": album_info}
                if track_id:
                    formatted = await asyncio.to_thread(load_public_track, track_id)
                    return {"query": q, "results": formatted}
            except Exception as public_error:
                logger.info("[Spotify] public metadata unavailable; using API fallback: %s", public_error)

        sp = await asyncio.to_thread(_spotify_client, client_id, client_secret, access_token)

        if playlist_id:
            formatted, playlist_info = await asyncio.to_thread(load_playlist, sp, playlist_id)
            return {"query": q, "results": formatted, "playlist": playlist_info}
        if album_id:
            formatted, album_info = await asyncio.to_thread(load_album, sp, album_id)
            return {"query": q, "results": formatted, "playlist": album_info}
        if track_id:
            formatted = [format_track(await asyncio.to_thread(sp.track, track_id))]
        else:
            results = await asyncio.to_thread(
                sp.search,
                q=q,
                limit=min(limit, 10),
                offset=max(offset, 0),
                type="track,album,playlist",
            )
            formatted = (
                [format_track(track) for track in results["tracks"]["items"]]
                + [format_search_collection(album, "album") for album in results["albums"]["items"]]
                + [format_search_collection(playlist, "playlist") for playlist in results["playlists"]["items"]]
            )
            total = sum(results.get(result_type, {}).get("total", 0) for result_type in ("tracks", "albums", "playlists"))
        return {"query": q, "source": "spotify", "results": formatted, "total": total if not (playlist_id or album_id or track_id) else len(formatted)}
    except HTTPException:
        raise
    except SpotifyException as e:
        status = e.http_status if e.http_status in (401, 403, 429) else 502
        raise HTTPException(status, f"Spotify search failed: {e}")
    except Exception as e:
        raise HTTPException(502, f"Spotify search failed: {str(e)}")

# ----------------------------------------------------------------------
# Download using SpotiFLAC with local Qobuz API
# ----------------------------------------------------------------------
class QobuzCredentials(BaseModel):
    app_id: str
    app_secret: str
    user_token: str


@app.get("/qobuz/settings")
async def get_qobuz_settings():
    return {"configured": bool(_qobuz_credentials)}


@app.put("/qobuz/settings")
async def set_qobuz_settings(credentials: QobuzCredentials):
    app_id = credentials.app_id.strip()
    app_secret = credentials.app_secret.strip()
    user_token = credentials.user_token.strip()
    if not app_id or not app_secret or not user_token:
        raise HTTPException(400, "Qobuz App ID, App secret, and User token are required.")
    headers = {"X-API-Key": QOBUZ_API_KEY} if QOBUZ_API_KEY else {}
    try:
        async with httpx.AsyncClient(timeout=10) as client:
            credentials_response = await client.post(
                f"{QOBUZ_API_URL.rstrip('/')}/set-credentials",
                json={"app_id": app_id, "secret": app_secret},
                headers=headers,
            )
            credentials_response.raise_for_status()
            token_response = await client.post(
                f"{QOBUZ_API_URL.rstrip('/')}/set-token",
                json={"token": user_token},
                headers=headers,
            )
            token_response.raise_for_status()
    except (httpx.HTTPError, httpx.RequestError) as error:
        logger.warning("Qobuz credentials could not be applied: %s", type(error).__name__)
        raise HTTPException(503, "Qobuz service could not accept these settings.") from error
    _qobuz_credentials.update(
        {"app_id": app_id, "app_secret": app_secret, "user_token": user_token}
    )
    return {"configured": True}


async def _download_audio(track_id: str, scope: str = "permanent") -> Path:
    scope = _normalize_download_scope(scope)
    if track_id in _cancelled_downloads:
        raise asyncio.CancelledError()
    existing = list(CACHE_DIR.glob(f"{track_id}.*"))
    if existing:
        _mark_cached_download(track_id, scope)
        _download_status[track_id] = "ready"
        _download_progress[track_id] = 100
        _save_download_status()
        return existing[0]

    if track_id in _downloading:
        scope = _merge_download_scope(track_id, scope)
        while track_id in _downloading:
            await asyncio.sleep(0.5)
        existing = list(CACHE_DIR.glob(f"{track_id}.*"))
        if existing:
            _mark_cached_download(track_id, scope)
            return existing[0]
        raise FileNotFoundError(f"No file for {track_id} after waiting.")

    scope = _merge_download_scope(track_id, scope)
    if scope == "room":
        _mark_room_download_requested(track_id)
    else:
        _mark_download_permanent(track_id)
    _downloading.add(track_id)
    _download_status[track_id] = "downloading"
    source = "youtube" if track_id.startswith("yt-") else "spotify"
    _download_progress[track_id] = 0 if source == "youtube" else None
    _save_download_status()
    _download_errors.pop(track_id, None)
    _record_download_event("info", "Download started", track_id, source, 0)
    # Per-track temp dir: concurrent downloads must not share a directory,
    # otherwise one finishing would delete the others' in-progress files.
    temp_dir = CACHE_DIR / "spotiflac_temp" / track_id
    temp_dir.mkdir(parents=True, exist_ok=True)
    try:
        if track_id.startswith("yt-"):
            video_id = track_id[3:]
            if not re.fullmatch(r"[A-Za-z0-9_-]{6,}", video_id):
                raise ValueError("Invalid YouTube video ID")
            last_reported = -5

            def progress_hook(data):
                nonlocal last_reported
                if data.get("status") == "finished":
                    _download_progress[track_id] = 100
                    _record_download_event("progress", "YouTube audio extracted", track_id, source, 100)
                    return
                if data.get("status") != "downloading":
                    return
                total = data.get("total_bytes") or data.get("total_bytes_estimate")
                if not total:
                    return
                progress = min(99, max(0, round(data.get("downloaded_bytes", 0) * 100 / total)))
                _download_progress[track_id] = progress
                if progress >= last_reported + 5:
                    last_reported = progress
                    _record_download_event("progress", f"Downloading YouTube audio ({progress}%)", track_id, source, progress)

            logger.info("[YouTube] starting track=%s", track_id)
            await asyncio.to_thread(_download_youtube_audio, video_id, track_id, progress_hook)
            audio_files = list(CACHE_DIR.glob(f"{track_id}.*"))
            if not audio_files:
                raise FileNotFoundError("No YouTube audio created")
            audio_files.sort(key=lambda f: f.stat().st_ctime, reverse=True)
            _download_status[track_id] = "ready"
            _download_progress[track_id] = 100
            _mark_download_complete(track_id, _download_scopes.get(track_id, scope))
            _save_download_status()
            _record_download_event("info", "Download ready", track_id, source, 100)
            return audio_files[0]

        spotify_url = f"https://open.spotify.com/track/{track_id}"
        logger.info("[SpotiFLAC] starting track=%s", track_id)
        _record_download_event("info", "Resolving Spotify track with audio providers", track_id, source, None)
        kwargs = {
            "url": spotify_url,
            "output_dir": str(temp_dir),
            "services": ["qobuz"] if _qobuz_credentials else ["qobuz", "tidal", "amazon", "deezer"],
            "qobuz_local_api_url": QOBUZ_API_URL,  # local Qobuz API
            "track_max_retries": 0,
            "timeout_s": 45,
        }
        if _qobuz_credentials:
            kwargs["qobuz_token"] = _qobuz_credentials["user_token"]
            kwargs["allow_fallback"] = False
            kwargs["use_extensions_fallback"] = False
        global _last_download_started
        async with _spotiflac_lock:
            elapsed = time.monotonic() - _last_download_started
            if track_id in _cancelled_downloads:
                raise asyncio.CancelledError()
            if elapsed < DOWNLOAD_DELAY_SECONDS:
                await asyncio.sleep(DOWNLOAD_DELAY_SECONDS - elapsed)
            _last_download_started = time.monotonic()
            await asyncio.to_thread(SpotiFLAC, **kwargs)
        if track_id in _cancelled_downloads:
            raise asyncio.CancelledError()
        audio_files = list(temp_dir.glob("**/*.flac")) + list(temp_dir.glob("**/*.m4a")) + list(temp_dir.glob("**/*.mp3"))
        logger.info("[SpotiFLAC] received %d candidate file(s) track=%s", len(audio_files), track_id)
        if not audio_files:
            raise FileNotFoundError("No audio created")
        audio_files.sort(key=lambda f: f.stat().st_ctime, reverse=True)
        latest = audio_files[0]
        ext = latest.suffix
        target = CACHE_DIR / f"{track_id}{ext}"
        shutil.move(str(latest), str(target))
        _download_status[track_id] = "ready"
        _download_progress[track_id] = 100
        _mark_download_complete(track_id, _download_scopes.get(track_id, scope))
        _save_download_status()
        logger.info("[SpotiFLAC] download ready track=%s", track_id)
        _record_download_event("info", "Download ready", track_id, source, 100)
        return target
    except asyncio.CancelledError:
        _discard_pending_room_download(track_id)
        raise
    except Exception as exc:
        _discard_pending_room_download(track_id)
        _download_status[track_id] = "error"
        _download_errors[track_id] = str(exc)
        _save_download_status()
        logger.exception("[Audio] download failed track=%s error=%s", track_id, exc)
        _record_download_event("error", f"Download failed: {exc}", track_id, source, _download_progress.get(track_id))
        raise
    finally:
        _downloading.discard(track_id)
        _download_scopes.pop(track_id, None)
        shutil.rmtree(temp_dir, ignore_errors=True)

def _read_file_range(path: Path, start: int, end: int, chunk_size: int = 1024 * 1024):
    remaining = end - start + 1
    with path.open("rb") as audio_file:
        audio_file.seek(start)
        while remaining > 0:
            chunk = audio_file.read(min(chunk_size, remaining))
            if not chunk:
                break
            remaining -= len(chunk)
            yield chunk


def _download_youtube_audio(video_id: str, track_id: str, progress_hook=None):
    options = {
        "quiet": True,
        "no_warnings": True,
        "noplaylist": True,
        "format": "bestaudio/best",
        "extractor_args": {"youtube": {"player_client": ["android"]}},
        "outtmpl": str(CACHE_DIR / f"{track_id}.%(ext)s"),
    }
    if progress_hook:
        options["progress_hooks"] = [progress_hook]
    with YoutubeDL(options) as ydl:
        ydl.download([f"https://www.youtube.com/watch?v={video_id}"])


@app.get("/stream/{track_id}")
async def get_stream(track_id: str, code: str, request: Request, scope: str = Query("permanent")):
    try:
        cache_path = await _download_audio(track_id, scope)
        ext = cache_path.suffix.lower()
        media_type = {
            ".flac": "audio/flac",
            ".mp3": "audio/mpeg",
            ".m4a": "audio/mp4",
            ".mp4": "audio/mp4",
            ".wav": "audio/wav",
            ".webm": "audio/webm",
            ".ogg": "audio/ogg",
            ".opus": "audio/ogg"
        }.get(ext, "application/octet-stream")
        file_size = cache_path.stat().st_size
        _streaming_tracks.add(track_id)
        range_header = request.headers.get("range")
        headers = {
            "Accept-Ranges": "bytes",
            "Content-Disposition": f'inline; filename="{cache_path.name}"'
        }

        if not range_header:
            headers["Content-Length"] = str(file_size)
            return StreamingResponse(
                _read_file_range(cache_path, 0, file_size - 1),
                media_type=media_type,
                headers=headers,
                background=BackgroundTask(_streaming_tracks.discard, track_id)
            )

        if not range_header.startswith("bytes="):
            headers["Content-Range"] = f"bytes */{file_size}"
            _streaming_tracks.discard(track_id)
            return StreamingResponse(iter(()), status_code=416, headers=headers)

        requested_range = range_header[6:].split(",", 1)[0].strip()
        start_text, end_text = requested_range.split("-", 1)
        if start_text:
            start = int(start_text)
            end = int(end_text) if end_text else file_size - 1
        else:
            suffix_length = int(end_text)
            start = max(file_size - suffix_length, 0)
            end = file_size - 1

        if start >= file_size or start < 0 or end < start:
            headers["Content-Range"] = f"bytes */{file_size}"
            _streaming_tracks.discard(track_id)
            return StreamingResponse(iter(()), status_code=416, headers=headers)

        end = min(end, file_size - 1)
        content_length = end - start + 1
        headers.update({
            "Content-Length": str(content_length),
            "Content-Range": f"bytes {start}-{end}/{file_size}"
        })
        return StreamingResponse(
            _read_file_range(cache_path, start, end),
            status_code=206,
            media_type=media_type,
            headers=headers,
            background=BackgroundTask(_streaming_tracks.discard, track_id)
        )
    except (ValueError, IndexError):
        _streaming_tracks.discard(track_id)
        raise HTTPException(416, "Invalid byte range")
    except Exception as e:
        _streaming_tracks.discard(track_id)
        raise HTTPException(502, f"Stream failed: {str(e)}")


@app.get("/download/{track_id}")
async def download_file(track_id: str, code: str, scope: str = Query("permanent")):
    try:
        cache_path = await _download_audio(track_id, scope)
        media_type = {
            ".flac": "audio/flac",
            ".mp3": "audio/mpeg",
            ".m4a": "audio/mp4",
            ".mp4": "audio/mp4",
            ".wav": "audio/wav",
            ".webm": "audio/webm",
            ".ogg": "audio/ogg",
            ".opus": "audio/ogg"
        }.get(cache_path.suffix.lower(), "application/octet-stream")
        return FileResponse(
            cache_path,
            media_type=media_type,
            filename=cache_path.name,
            headers={"Content-Disposition": f'attachment; filename="{cache_path.name}"'}
        )
    except Exception as e:
        raise HTTPException(502, f"Download failed: {str(e)}")

@app.post("/preload")
async def preload(video_ids: str = Query(...), scope: str = Query("permanent")):
    scope = _normalize_download_scope(scope)
    ids = [tid.strip() for tid in video_ids.split(",") if tid.strip()]
    logger.info("[SpotiFLAC] preload requested tracks=%s", ids)
    for tid in ids:
        _cancelled_downloads.discard(tid)
        if any(CACHE_DIR.glob(f"{tid}.*")):
            _mark_cached_download(tid, scope)
            _download_status[tid] = "ready"
            _download_progress[tid] = 100
            _record_download_event("info", "Audio already cached", tid, "youtube" if tid.startswith("yt-") else "spotify", 100)
        else:
            _schedule_download(tid, scope)
    _save_download_status()
    return {"status": "queued", "count": len(ids)}

@app.post("/cancel")
async def cancel_downloads(video_ids: str = Query(...)):
    ids = [tid.strip() for tid in video_ids.split(",") if tid.strip()]
    for track_id in ids:
        _cancelled_downloads.add(track_id)
        task = _download_tasks.get(track_id)
        if task is not None and not task.done():
            task.cancel()
        _download_status[track_id] = "cancelled"
        _download_errors.pop(track_id, None)
        _record_download_event("warning", "Download cancelled", track_id, "youtube" if track_id.startswith("yt-") else "spotify")
    _save_download_status()
    return {"status": "cancelled", "count": len(ids)}

def _schedule_download(track_id, scope="permanent"):
    scope = _normalize_download_scope(scope)
    task = _download_tasks.get(track_id)
    if task is not None and not task.done():
        scope = _merge_download_scope(track_id, scope)
        if scope == "permanent":
            _mark_download_permanent(track_id)
        return
    scope = _merge_download_scope(track_id, scope)
    if scope == "room":
        _mark_room_download_requested(track_id)
    else:
        _mark_download_permanent(track_id)
    _download_status[track_id] = "queued"
    _download_progress.setdefault(track_id, 0 if track_id.startswith("yt-") else None)
    _record_download_event("info", "Queued for download", track_id, "youtube" if track_id.startswith("yt-") else "spotify")
    task = asyncio.create_task(_download_audio(track_id, scope), name=f"download-{track_id}")
    _download_tasks[track_id] = task
    task.add_done_callback(lambda completed, queued_id=track_id: _finish_download_task(queued_id, completed))

def _finish_download_task(track_id, task):
    _download_scopes.pop(track_id, None)
    if task.cancelled():
        logger.warning("[SpotiFLAC] download cancelled track=%s", track_id)
    elif task.exception():
        logger.error("[SpotiFLAC] background task failed track=%s error=%s", track_id, task.exception())
    _download_tasks.pop(track_id, None)

@app.get("/status")
async def download_status(video_ids: str = Query(...)):
    ids = [tid.strip() for tid in video_ids.split(",") if tid.strip()]
    tracks = {}
    for track_id in ids:
        cached = any(CACHE_DIR.glob(f"{track_id}.*"))
        progress = _download_progress.get(track_id)
        if cached:
            tracks[track_id] = {"status": "ready", "progress": 100}
        elif _download_status.get(track_id) in {"queued", "downloading"} and track_id not in _download_tasks:
            _schedule_download(track_id, _download_scope_for(track_id))
            tracks[track_id] = {"status": "queued", "progress": progress}
        elif track_id in _downloading:
            tracks[track_id] = {"status": "downloading", "progress": progress}
        elif track_id in _download_errors:
            tracks[track_id] = {"status": "error", "progress": progress, "error": _download_errors[track_id]}
        else:
            tracks[track_id] = {"status": _download_status.get(track_id, "pending"), "progress": progress}
    return {"tracks": tracks}


@app.get("/logs")
async def download_logs(limit: int = Query(100, ge=1, le=500)):
    global _log_file_mtime_ns
    with _download_log_lock:
        try:
            mtime_ns = LOG_FILE.stat().st_mtime_ns
            if mtime_ns != _log_file_mtime_ns:
                persisted = json.loads(LOG_FILE.read_text(encoding="utf-8"))
                if isinstance(persisted, list):
                    _download_log_events.clear()
                    _download_log_events.extend(persisted[-500:])
                _log_file_mtime_ns = mtime_ns
        except (FileNotFoundError, json.JSONDecodeError, OSError):
            pass
    return {"logs": list(_download_log_events)[-limit:][::-1]}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=int(os.getenv("PORT", "8787")))