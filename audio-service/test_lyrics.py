import unittest
from types import SimpleNamespace
from unittest.mock import patch

import main


class LyricsRouteTests(unittest.IsolatedAsyncioTestCase):
    async def test_lrclib_lookup_uses_spotify_album_and_duration(self):
        requests = []
        lyrics = {
            "syncedLyrics": "[00:01.00]First line",
            "plainLyrics": "First line",
        }

        class FakeClient:
            async def __aenter__(self):
                return self

            async def __aexit__(self, *_):
                return None

            async def get(self, url, params, headers):
                requests.append((url, params, headers))
                return SimpleNamespace(
                    status_code=200,
                    json=lambda: lyrics,
                    raise_for_status=lambda: None,
                )

        with patch.object(main.httpx, "AsyncClient", return_value=FakeClient()):
            result = await main.lyrics(
                "  Song title  ",
                "  Artist name  ",
                "  Album name  ",
                213.2,
                "spotify",
            )

        self.assertEqual(
            requests,
            [
                (
                    "https://lrclib.net/api/get",
                    {
                        "track_name": "Song title",
                        "artist_name": "Artist name",
                        "album_name": "Album name",
                        "duration": 213,
                    },
                    {
                        "User-Agent": (
                            "Wavelength/2.0 "
                            "(https://github.com/zespook/wavelength-rust)"
                        )
                    },
                )
            ],
        )
        self.assertEqual(result, lyrics)

    async def test_duration_mismatch_searches_for_the_matching_edit(self):
        requests = []
        mismatched_lyrics = {
            "trackName": "Song title",
            "artistName": "Artist name",
            "duration": 170,
            "syncedLyrics": "[00:01.00]Lyrics from the shorter edit",
        }
        matching_lyrics = {
            "trackName": "Song title",
            "artistName": "Artist name",
            "duration": 200,
            "syncedLyrics": "[00:01.00]Lyrics from the matching edit",
        }

        class FakeClient:
            async def __aenter__(self):
                return self

            async def __aexit__(self, *_):
                return None

            async def get(self, url, params, headers):
                requests.append(url)
                if url.endswith("/api/get"):
                    return SimpleNamespace(
                        status_code=200,
                        json=lambda: mismatched_lyrics,
                        raise_for_status=lambda: None,
                    )
                return SimpleNamespace(
                    status_code=200,
                    json=lambda: [mismatched_lyrics, matching_lyrics],
                    raise_for_status=lambda: None,
                )

        self.assertIsNone(
            main._find_lyrics_search_match(
                [mismatched_lyrics], "Song title", "Artist name", "spotify", 200
            )
        )
        with patch.object(main.httpx, "AsyncClient", return_value=FakeClient()):
            result = await main.lyrics(
                "Song title", "Artist name", None, 200, "spotify"
            )

        self.assertEqual(requests, ["https://lrclib.net/api/get", "https://lrclib.net/api/search"])
        self.assertEqual(
            result["syncedLyrics"], "[00:01.00]Lyrics from the matching edit"
        )

    async def test_youtube_title_and_channel_are_normalized_before_search(self):
        requests = []
        lyrics = {
            "trackName": "Song title",
            "artistName": "Artist name",
            "duration": 182,
            "syncedLyrics": "[00:01.00]First line",
            "plainLyrics": "First line",
        }

        class FakeClient:
            async def __aenter__(self):
                return self

            async def __aexit__(self, *_):
                return None

            async def get(self, url, params, headers):
                requests.append((url, params, headers))
                if url.endswith("/api/get"):
                    return SimpleNamespace(status_code=404)
                return SimpleNamespace(
                    status_code=200,
                    json=lambda: [lyrics],
                    raise_for_status=lambda: None,
                )

        with patch.object(main.httpx, "AsyncClient", return_value=FakeClient()):
            result = await main.lyrics(
                "Artist name - Song title (Official Video)",
                "Artist name - Topic",
                "YouTube",
                182,
                "youtube",
            )

        self.assertEqual(
            requests[0][1],
            {
                "track_name": "Song title",
                "artist_name": "Artist name",
                "duration": 182,
            },
        )
        self.assertEqual(
            requests[1][0:2],
            (
                "https://lrclib.net/api/search",
                {"q": "Artist name - Song title"},
            ),
        )
        self.assertEqual(result["syncedLyrics"], lyrics["syncedLyrics"])

    async def test_search_does_not_return_a_different_spotify_artist(self):
        class FakeClient:
            async def __aenter__(self):
                return self

            async def __aexit__(self, *_):
                return None

            async def get(self, url, *_args, **_kwargs):
                if url.endswith("/api/get"):
                    return SimpleNamespace(status_code=404)
                return SimpleNamespace(
                    status_code=200,
                    json=lambda: [
                        {
                            "trackName": "Song title",
                            "artistName": "Another artist",
                            "duration": 213,
                        }
                    ],
                    raise_for_status=lambda: None,
                )

        with patch.object(main.httpx, "AsyncClient", return_value=FakeClient()):
            with self.assertRaises(main.HTTPException) as raised:
                await main.lyrics(
                    "Song title", "Artist name", None, 213, "spotify"
                )

        self.assertEqual(raised.exception.status_code, 404)


if __name__ == "__main__":
    unittest.main()
