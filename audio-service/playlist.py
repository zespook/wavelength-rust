import base64
import json
import re
import urllib.request


def spotify_playlist_id(value: str):
    match = re.search(r'(?:open\.spotify\.com/(?:intl-[a-z]{2}(?:-[a-z]{2})?/)?playlist/|spotify:playlist:)([A-Za-z0-9]+)', value)
    return match.group(1) if match else None


def spotify_album_id(value: str):
    match = re.search(r'(?:open\.spotify\.com/(?:intl-[a-z]{2}(?:-[a-z]{2})?/)?album/|spotify:album:)([A-Za-z0-9]+)', value)
    return match.group(1) if match else None


def spotify_track_id(value: str):
    match = re.search(r'(?:open\.spotify\.com/(?:intl-[a-z]{2}(?:-[a-z]{2})?/)?track/|spotify:track:)([A-Za-z0-9]+)', value)
    return match.group(1) if match else None


def _first_image_url(images):
    if not images:
        return None
    if not isinstance(images, list):
        return None
    candidates = [
        (index, item)
        for index, item in enumerate(images)
        if isinstance(item, dict)
        and isinstance(item.get("url"), str)
        and item["url"].startswith(("https://", "http://"))
    ]
    if not candidates:
        return None
    _, image = max(
        candidates,
        key=lambda candidate: (
            (candidate[1].get("width") or 0) * (candidate[1].get("height") or 0),
            -candidate[0],
        ),
    )
    return image["url"]


def _spotify_entity_id(value, entity):
    if not isinstance(value, str):
        return None
    match = re.search(rf'(?:spotify:{entity}:|open\.spotify\.com/(?:intl-[a-z]{{2}}(?:-[a-z]{{2}})?/)?{entity}/)([A-Za-z0-9]+)', value)
    return match.group(1) if match else None


def _album_track_data(entry):
    if not isinstance(entry, dict):
        return None
    for key in ("itemV2", "item", "track", "data"):
        candidate = entry.get(key)
        if isinstance(candidate, dict):
            nested = candidate.get("data")
            return nested if isinstance(nested, dict) else candidate
    return entry


def _artist_name(track):
    if not isinstance(track, dict):
        return "Unknown artist"

    artist_items = []
    artists_block = track.get("artists")
    if isinstance(artists_block, dict):
        artist_items = artists_block.get("items", []) or []

    if not artist_items:
        first_artist = track.get("firstArtist") or track.get("artist")
        if isinstance(first_artist, dict):
            items = first_artist.get("items", [])
            if items and isinstance(items[0], dict):
                artist_items = items
            else:
                name = first_artist.get("name") or first_artist.get("profile", {}).get("name")
                if name:
                    return name
        if isinstance(first_artist, str):
            return first_artist

    for item in artist_items:
        if isinstance(item, dict):
            profile = item.get("profile") or item.get("item", {}).get("profile")
            if isinstance(profile, dict) and profile.get("name"):
                return profile["name"]
            if item.get("name"):
                return item["name"]
            if item.get("item") and isinstance(item.get("item"), dict):
                nested_name = item["item"].get("name") or item["item"].get("profile", {}).get("name")
                if nested_name:
                    return nested_name
    return "Unknown artist"


def format_public_track(track):
    if not isinstance(track, dict):
        return {"videoId": None, "title": "Unknown track", "artist": "Unknown artist", "thumbnail": None, "duration": 0, "source": "spotify"}

    track_id = track.get("id") or spotify_track_id(track.get("uri") or "")
    images = []
    album = track.get("albumOfTrack") or track.get("album") or {}
    album_name = album.get("name", "") if isinstance(album, dict) else ""
    cover = album.get("coverArt", {}) if isinstance(album, dict) else {}
    if isinstance(cover, dict):
        images = cover.get("sources", []) or []
    thumbnail = _first_image_url(images)
    duration_ms = 0
    duration_block = track.get("duration")
    if isinstance(duration_block, dict):
        duration_ms = int(duration_block.get("totalMilliseconds", 0) or 0)
    if not duration_ms:
        duration_ms = int(track.get("duration_ms", 0) or 0)

    artist_items = track.get("artists", {}).get("items", []) if isinstance(track.get("artists"), dict) else []
    first_artist = artist_items[0] if artist_items else {}
    first_artist_id = (first_artist.get("id") or _spotify_entity_id(first_artist.get("uri"), "artist")) if isinstance(first_artist, dict) else None
    album_id = (album.get("id") or _spotify_entity_id(album.get("uri"), "album")) if isinstance(album, dict) else None

    return {
        "videoId": track_id,
        "title": track.get("name") or track.get("title") or "Unknown track",
        "artist": _artist_name(track),
        "album": album_name,
        "thumbnail": thumbnail,
        "duration": duration_ms // 1000,
        "spotify_url": f"https://open.spotify.com/track/{track_id}" if track_id else None,
        "spotify_artist_url": f"https://open.spotify.com/artist/{first_artist_id}" if first_artist_id else None,
        "spotify_album_url": f"https://open.spotify.com/album/{album_id}" if album_id else None,
        "source": "spotify",
    }


def format_track(track):
    artists = track.get("artists") or []
    album = track.get("album") or {}
    artist_name = ", ".join(artist.get("name", "") for artist in artists if isinstance(artist, dict) and artist.get("name"))
    first_artist_id = artists[0].get("id") if artists and isinstance(artists[0], dict) else None
    album_id = album.get("id")
    album_images = album.get("images") or []
    track_id = track.get("id")
    return {
        "type": "track",
        "videoId": track_id,
        "title": track.get("name") or "Unknown track",
        "artist": artist_name or "Unknown artist",
        "album": album.get("name", ""),
        "thumbnail": _first_image_url(album_images),
        "duration": int(track.get("duration_ms") or 0) // 1000,
        "spotify_url": (track.get("external_urls") or {}).get("spotify") or (f"https://open.spotify.com/track/{track_id}" if track_id else None),
        "spotify_artist_url": f"https://open.spotify.com/artist/{first_artist_id}" if first_artist_id else None,
        "spotify_album_url": f"https://open.spotify.com/album/{album_id}" if album_id else None,
        "source": "spotify"
    }


def _decode_initial_state(html):
    match = re.search(r'<script id="initialState" type="text/plain">(.*?)</script>', html, re.S)
    if not match:
        return None
    encoded = match.group(1).strip()
    if not encoded:
        return None
    raw = base64.b64decode(encoded + "=" * (-len(encoded) % 4))
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return None


def parse_public_playlist_payload(payload, playlist_id: str):
    if not isinstance(payload, dict):
        raise ValueError("Spotify public payload is not an object")

    entity_map = payload.get("entities", {}).get("items", {})
    playlist = entity_map.get(f"spotify:playlist:{playlist_id}")
    if playlist is None:
        for candidate in entity_map.values():
            if isinstance(candidate, dict) and candidate.get("id") == playlist_id:
                playlist = candidate
                break
    if playlist is None:
        raise ValueError(f"Playlist {playlist_id} was not found in the public Spotify payload")

    items = []
    for entry in playlist.get("content", {}).get("items", []):
        item = entry.get("itemV2") or entry.get("item") or {}
        data = item.get("data") if isinstance(item, dict) else None
        if not isinstance(data, dict):
            continue
        if not data.get("id"):
            uri = data.get("uri") or ""
            data = {**data, "id": spotify_track_id(uri)}
        if data.get("id"):
            items.append(data)

    formatted = [format_public_track(track) for track in items]
    details = {
        "id": playlist.get("id") or playlist_id,
        "name": playlist.get("name") or "Spotify Playlist",
        "owner": (playlist.get("ownerV2", {}).get("profile", {}).get("name") or "Spotify"),
        "thumbnail": _first_image_url(playlist.get("images")),
        "trackCount": len(formatted),
    }
    return formatted, details


def parse_public_track_payload(payload, track_id: str):
    if not isinstance(payload, dict):
        raise ValueError("Spotify public payload is not an object")

    entity_map = payload.get("entities", {}).get("items", {})
    track = entity_map.get(f"spotify:track:{track_id}")
    if track is None:
        for candidate in entity_map.values():
            if isinstance(candidate, dict) and candidate.get("id") == track_id:
                track = candidate
                break
    if track is None:
        raise ValueError(f"Track {track_id} was not found in the public Spotify payload")
    return [format_public_track(track)]


def fetch_public_spotify_page(url: str):
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=30) as response:
        return response.read().decode("utf-8", errors="replace")


def load_public_playlist(playlist_id: str):
    html = fetch_public_spotify_page(f"https://open.spotify.com/playlist/{playlist_id}")
    payload = _decode_initial_state(html)
    if payload is None:
        raise ValueError("Unable to decode the public Spotify playlist metadata")
    return parse_public_playlist_payload(payload, playlist_id)


def parse_public_album_payload(payload, album_id: str):
    if not isinstance(payload, dict):
        raise ValueError("Spotify public payload is not an object")

    entity_map = payload.get("entities", {}).get("items", {})
    album = entity_map.get(f"spotify:album:{album_id}")
    if album is None:
        for candidate in entity_map.values():
            if isinstance(candidate, dict) and candidate.get("id") == album_id:
                album = candidate
                break
    if album is None:
        raise ValueError(f"Album {album_id} was not found in the public Spotify payload")

    items = []
    tracks_block = album.get("tracksV2") or album.get("tracks") or album.get("trackList") or {}
    track_items = tracks_block.get("items", []) if isinstance(tracks_block, dict) else []
    if not track_items:
        track_items = album.get("content", {}).get("items", [])
    for entry in track_items:
        data = _album_track_data(entry)
        if not isinstance(data, dict):
            continue
        if not data.get("id"):
            data = {**data, "id": spotify_track_id(data.get("uri") or "")}
        if data.get("id"):
            items.append(data)

    album_metadata = {
        "id": album.get("id") or album_id,
        "uri": album.get("uri"),
        "name": album.get("name"),
        "coverArt": album.get("coverArt") or {"sources": album.get("images", [])},
    }
    formatted = [format_public_track({**track, "albumOfTrack": track.get("albumOfTrack") or album_metadata}) for track in items]
    details = {
        "id": album.get("id") or album_id,
        "name": album.get("name") or "Spotify Album",
        "owner": _artist_name(album),
        "thumbnail": _first_image_url(album.get("images")) or _first_image_url(album.get("coverArt", {}).get("sources", [])) or _first_image_url(album.get("visuals", {}).get("main", {}).get("sources", [])),
        "trackCount": len(formatted),
        "type": "album",
    }
    return formatted, details


def load_public_album(album_id: str):
    html = fetch_public_spotify_page(f"https://open.spotify.com/album/{album_id}")
    payload = _decode_initial_state(html)
    if payload is None:
        raise ValueError("Unable to decode the public Spotify album metadata")
    return parse_public_album_payload(payload, album_id)


def load_public_track(track_id: str):
    html = fetch_public_spotify_page(f"https://open.spotify.com/track/{track_id}")
    payload = _decode_initial_state(html)
    if payload is None:
        raise ValueError("Unable to decode the public Spotify track metadata")
    return parse_public_track_payload(payload, track_id)


def load_playlist(spotify, playlist_id: str):
    details = spotify.playlist(playlist_id, fields="id,name,owner.display_name,images,tracks.total")
    items = []
    offset = 0
    while True:
        page = spotify.playlist_items(playlist_id, limit=100, offset=offset, additional_types=["track"])
        items.extend(page["items"])
        if not page.get("next"):
            break
        offset += len(page["items"])

    tracks = [item["track"] for item in items if item.get("track") and item["track"].get("id")]
    formatted = [format_track(track) for track in tracks]
    return formatted, {
        "id": details["id"],
        "name": details["name"],
        "owner": details.get("owner", {}).get("display_name", "Spotify"),
        "thumbnail": _first_image_url(details.get("images")),
        "trackCount": len(formatted)
    }


def load_album(spotify, album_id: str):
    details = spotify.album(album_id)
    items = []
    offset = 0
    while True:
        page = spotify.album_tracks(album_id, limit=50, offset=offset)
        items.extend(page["items"])
        if not page.get("next"):
            break
        offset += len(page["items"])

    formatted = [format_track({**track, "album": track.get("album") or details}) for track in items if track.get("id")]
    return formatted, {
        "id": details["id"],
        "name": details["name"],
        "owner": ", ".join(artist["name"] for artist in details.get("artists", [])),
        "thumbnail": _first_image_url(details.get("images")),
        "trackCount": len(formatted),
        "type": "album",
    }
