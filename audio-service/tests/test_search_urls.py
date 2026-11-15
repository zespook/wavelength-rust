import unittest
from pathlib import Path
from threading import get_ident
from unittest.mock import patch

import main
from main import _format_musical_scale, _spotify_track_id, _youtube_video_id, track_metadata
from playlist import spotify_album_id, spotify_playlist_id


class SearchUrlTests(unittest.IsolatedAsyncioTestCase):
    async def test_spotify_search_runs_off_event_loop_thread(self):
        event_loop_thread = get_ident()
        search_threads = []

        class FakeSpotify:
            def search(self, **_kwargs):
                search_threads.append(get_ident())
                return {
                    "tracks": {"items": [], "total": 0},
                    "albums": {"items": [], "total": 0},
                    "playlists": {"items": [], "total": 0},
                }

        with patch("main._spotify_client", return_value=FakeSpotify()):
            result = await main.search(q="test", source="spotify")

        self.assertEqual(result["results"], [])
        self.assertEqual(len(search_threads), 1)
        self.assertNotEqual(search_threads[0], event_loop_thread)

    async def test_track_metadata_includes_release_date_bpm_and_scale(self):
        class FakeSpotify:
            def track(self, track_id):
                self.requested_id = track_id
                return {"album": {"release_date": "2015-11-20"}}

            def audio_features(self, track_ids):
                self.requested_ids = track_ids
                return [{"tempo": 79.4, "key": 5, "mode": 0}]

        client = FakeSpotify()
        with patch("main._spotify_client", return_value=client):
            result = await track_metadata(
                title="Hello",
                artist="Adele",
                spotify_id="track123",
            )

        self.assertEqual(result, {"bpm": 79, "scale": "F minor", "releaseDate": "2015-11-20"})
        self.assertEqual(client.requested_id, "track123")
        self.assertEqual(client.requested_ids, ["track123"])

    def test_scale_requires_valid_spotify_key_and_mode(self):
        self.assertEqual(_format_musical_scale(0, 1), "C major")
        self.assertEqual(_format_musical_scale(11, 0), "B minor")
        self.assertEqual(_format_musical_scale(-1, 0), "")

    def test_youtube_watch_and_short_links(self):
        self.assertEqual(_youtube_video_id("https://www.youtube.com/watch?v=abcDEF_1234&t=20"), "abcDEF_1234")
        self.assertEqual(_youtube_video_id("https://youtu.be/abcDEF_1234?si=share"), "abcDEF_1234")

    def test_youtube_shorts_embed_and_live_links(self):
        self.assertEqual(_youtube_video_id("https://youtube.com/shorts/abcDEF_1234"), "abcDEF_1234")
        self.assertEqual(_youtube_video_id("https://www.youtube-nocookie.com/embed/abcDEF_1234"), "abcDEF_1234")
        self.assertEqual(_youtube_video_id("https://youtube.com/live/abcDEF_1234"), "abcDEF_1234")

    def test_rejects_non_video_urls(self):
        self.assertIsNone(_youtube_video_id("https://example.com/watch?v=abcDEF_1234"))
        self.assertIsNone(_youtube_video_id("https://youtube.com/watch?v=short"))

    def test_spotify_track_album_and_playlist_links(self):
        self.assertEqual(_spotify_track_id("https://open.spotify.com/intl-us/track/abc123?si=share"), "abc123")
        self.assertEqual(spotify_album_id("https://open.spotify.com/intl-de/album/def456"), "def456")
        self.assertEqual(spotify_playlist_id("https://open.spotify.com/intl-gb/playlist/ghi789"), "ghi789")

    async def test_youtube_video_link_is_extracted_without_searching_for_url_text(self):
        calls = []
        options_seen = []

        class FakeYoutubeDL:
            def __init__(self, options):
                self.options = options
                options_seen.append(options)

            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return None

            def extract_info(self, target, download=False):
                calls.append((target, download))
                return {
                    "id": "abcDEF_1234",
                    "title": "Video title",
                    "channel": "Channel",
                    "duration": 120,
                    "thumbnail": "https://img.example/medium.jpg",
                    "thumbnails": [
                        {"url": "https://img.example/small.jpg", "width": 120, "height": 90},
                        {"url": "https://img.example/large.jpg", "width": 1280, "height": 720},
                    ],
                }

        with patch.object(main, "YoutubeDL", FakeYoutubeDL):
            result = await main.search(q="https://youtu.be/abcDEF_1234", source="youtube")

        self.assertEqual(calls, [("https://www.youtube.com/watch?v=abcDEF_1234", False)])
        self.assertNotIn("extractor_args", options_seen[0])
        self.assertEqual(result["results"][0]["videoId"], "abcDEF_1234")
        self.assertEqual(result["results"][0]["length"], "2:00")
        self.assertEqual(result["results"][0]["source"], "youtube")
        self.assertEqual(result["results"][0]["thumbnail"], "https://img.example/large.jpg")

    async def test_youtube_search_uses_video_thumbnail_fallback(self):
        class FakeYoutubeDL:
            def __init__(self, _options):
                pass

            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return None

            def extract_info(self, _target, download=False):
                return {
                    "entries": [{
                        "id": "abcDEF_1234",
                        "title": "Video title",
                        "thumbnails": [{"url": "invalid-url"}],
                    }]
                }

        with patch.object(main, "YoutubeDL", FakeYoutubeDL):
            result = await main.search(q="video", source="youtube")

        self.assertEqual(
            result["results"][0]["thumbnail"],
            "https://i.ytimg.com/vi/abcDEF_1234/hqdefault.jpg",
        )

    def test_youtube_audio_download_uses_default_player_clients(self):
        options_seen = []
        requested = []

        class FakeYoutubeDL:
            def __init__(self, options):
                options_seen.append(options)

            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return None

            def download(self, urls):
                requested.extend(urls)

        with (
            patch.object(main, "YoutubeDL", FakeYoutubeDL),
            patch.object(main, "CACHE_DIR", Path("test-cache")),
        ):
            main._download_youtube_audio("abcDEF_1234", "yt-abcDEF_1234")

        self.assertNotIn("extractor_args", options_seen[0])
        self.assertEqual(
            requested,
            ["https://www.youtube.com/watch?v=abcDEF_1234"],
        )


if __name__ == "__main__":
    unittest.main()