import unittest

from playlist import format_public_track, format_track, parse_public_album_payload, parse_public_playlist_payload, parse_public_track_payload


class PublicSpotifyPayloadTests(unittest.TestCase):
    def test_parse_public_playlist_payload_uses_public_entity_shape(self):
        payload = {
            "entities": {
                "items": {
                    "spotify:playlist:abc123": {
                        "id": "abc123",
                        "name": "silk sheets & robes",
                        "ownerV2": {"profile": {"name": "Koan"}},
                        "images": [{"url": "https://cdn.example/playlist.png"}],
                        "content": {"items": [{
                            "itemV2": {"data": {
                                "__typename": "Track",
                                "id": "track123",
                                "name": "You Already Know",
                                "duration": {"totalMilliseconds": 280125},
                                "artists": {"items": [{"profile": {"name": "Evita"}}]},
                                "albumOfTrack": {"coverArt": {"sources": [{"url": "https://cdn.example/cover.png"}]}}
                            }}
                        }]}
                    }
                }
            }
        }

        results, details = parse_public_playlist_payload(payload, "abc123")

        self.assertEqual(details["name"], "silk sheets & robes")
        self.assertEqual(details["owner"], "Koan")
        self.assertEqual(results[0]["videoId"], "track123")
        self.assertEqual(results[0]["artist"], "Evita")
        self.assertEqual(results[0]["thumbnail"], "https://cdn.example/cover.png")
        self.assertEqual(results[0]["source"], "spotify")

    def test_parse_public_track_payload_reads_track_entity(self):
        payload = {
            "entities": {
                "items": {
                    "spotify:track:track123": {
                        "id": "track123",
                        "name": "Numb",
                        "duration": {"totalMilliseconds": 123400},
                        "artists": {"items": [{"profile": {"name": "Linkin Park"}}]},
                        "albumOfTrack": {"name": "Hybrid Theory", "coverArt": {"sources": [{"url": "https://cdn.example/track-cover.png"}]}}
                    }
                }
            }
        }

        result = parse_public_track_payload(payload, "track123")

        self.assertEqual(result[0]["videoId"], "track123")
        self.assertEqual(result[0]["title"], "Numb")
        self.assertEqual(result[0]["artist"], "Linkin Park")
        self.assertEqual(result[0]["album"], "Hybrid Theory")
        self.assertEqual(result[0]["thumbnail"], "https://cdn.example/track-cover.png")
        self.assertEqual(result[0]["source"], "spotify")

    def test_format_track_includes_album_name(self):
        track = {
            "id": "track123",
            "name": "Numb",
            "artists": [{"name": "Linkin Park"}],
            "album": {
                "name": "Meteora",
                "images": [
                    {"url": "", "width": 640, "height": 640},
                    {"url": "https://cdn.example/large.png", "width": 640, "height": 640},
                    {"url": "https://cdn.example/small.png", "width": 64, "height": 64},
                ],
            },
            "duration_ms": 185000,
            "external_urls": {"spotify": "https://open.spotify.com/track/track123"},
        }

        formatted = format_track(track)

        self.assertEqual(formatted["album"], "Meteora")
        self.assertEqual(formatted["thumbnail"], "https://cdn.example/large.png")

    def test_format_track_includes_spotify_artist_and_album_urls(self):
        track = {
            "id": "track123",
            "name": "Numb",
            "artists": [{"id": "artist123", "name": "Linkin Park"}],
            "album": {"id": "album123", "name": "Meteora", "images": []},
            "duration_ms": 185000,
        }

        formatted = format_track(track)

        self.assertEqual(formatted["spotify_url"], "https://open.spotify.com/track/track123")
        self.assertEqual(formatted["spotify_artist_url"], "https://open.spotify.com/artist/artist123")
        self.assertEqual(formatted["spotify_album_url"], "https://open.spotify.com/album/album123")

    def test_parse_public_album_supplies_parent_album_metadata_to_tracks(self):
        payload = {
            "entities": {
                "items": {
                    "spotify:album:album123": {
                        "id": "album123",
                        "name": "Meteora",
                        "uri": "spotify:album:album123",
                        "tracksV2": {"items": [{"itemV2": {"data": {
                            "id": "track123",
                            "name": "Numb",
                            "duration": {"totalMilliseconds": 185000},
                            "artists": {"items": [{"id": "artist123", "uri": "spotify:artist:artist123", "profile": {"name": "Linkin Park"}}]},
                        }}}]},
                    }
                }
            }
        }

        results, _details = parse_public_album_payload(payload, "album123")

        self.assertEqual(results[0]["album"], "Meteora")
        self.assertEqual(results[0]["duration"], 185)
        self.assertEqual(results[0]["spotify_artist_url"], "https://open.spotify.com/artist/artist123")
        self.assertEqual(results[0]["spotify_album_url"], "https://open.spotify.com/album/album123")

    def test_format_public_track_handles_missing_artist_data(self):
        self.assertEqual(format_public_track({"id": "x", "name": "Song", "duration": {"totalMilliseconds": 5000}, "albumOfTrack": {"coverArt": {"sources": []}}})["artist"], "Unknown artist")

    def test_parse_public_playlist_payload_accepts_uri_based_track_ids(self):
        payload = {
            "entities": {
                "items": {
                    "spotify:playlist:abc123": {
                        "id": "abc123",
                        "name": "Test Playlist",
                        "ownerV2": {"profile": {"name": "Tester"}},
                        "images": [{"url": "https://cdn.example/playlist.png"}],
                        "content": {"items": [{
                            "itemV2": {"data": {
                                "uri": "spotify:track:track123",
                                "name": "Shallow",
                                "duration": {"totalMilliseconds": 240000},
                                "artists": {"items": [{"profile": {"name": "Lady Gaga"}}]},
                                "albumOfTrack": {"coverArt": {"sources": [{"url": "https://cdn.example/cover.png"}]}}
                            }}
                        }]}
                    }
                }
            }
        }

        results, details = parse_public_playlist_payload(payload, "abc123")
        self.assertEqual(results[0]["videoId"], "track123")
        self.assertEqual(details["owner"], "Tester")


if __name__ == "__main__":
    unittest.main()
