import asyncio
import json
import tempfile
import unittest
from collections import deque
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import main


class SpotiFLACRouteTests(unittest.IsolatedAsyncioTestCase):
    async def test_async_client_uses_legacy_single_track_api_when_report_api_is_unavailable(self):
        requested = []

        class FakeClient:
            async def __aenter__(self):
                return self

            async def __aexit__(self, *_):
                return None

            async def download_track(self, url):
                requested.append(url)
                return []

        with patch.object(main, "AsyncSpotiFLAC", return_value=FakeClient()):
            result = await main._run_spotiflac_download(
                url="https://open.spotify.com/track/test",
                output_dir="downloads",
                services=["qobuz"],
                use_extensions_fallback=False,
            )

        self.assertEqual(result, [])
        self.assertEqual(requested, ["https://open.spotify.com/track/test"])

    async def test_qobuz_settings_forward_all_credentials_without_echoing_them(self):
        calls = []

        class FakeClient:
            async def __aenter__(self):
                return self

            async def __aexit__(self, *_):
                return None

            async def post(self, url, json, headers):
                calls.append((url, json, headers))
                return SimpleNamespace(raise_for_status=lambda: None)

        with (
            patch.object(main.httpx, "AsyncClient", return_value=FakeClient()),
            patch.object(main, "QOBUZ_API_URL", "http://qobuz.test"),
            patch.object(main, "QOBUZ_API_KEY", ""),
            patch.object(main, "_qobuz_credentials", {}),
        ):
            response = await main.set_qobuz_settings(
                main.QobuzCredentials(
                    app_id="app-id",
                    app_secret="app-secret",
                    user_token="user-token",
                )
            )

        self.assertEqual(response, {"configured": True})
        self.assertEqual(len(calls), 2)
        self.assertEqual(calls[0][1], {"app_id": "app-id", "secret": "app-secret"})
        self.assertEqual(calls[1][1], {"token": "user-token"})
    async def test_logs_endpoint_reloads_external_file_changes(self):
        event = {"timestamp": 1234, "level": "info", "message": "external update"}
        with tempfile.TemporaryDirectory() as temp_dir:
            log_file = Path(temp_dir) / "logs.json"
            log_file.write_text(json.dumps([event]), encoding="utf-8")
            with (
                patch.object(main, "LOG_FILE", log_file),
                patch.object(main, "_log_file_mtime_ns", 0),
                patch.object(main, "_download_log_events", deque()),
            ):
                result = await main.download_logs(100)

        self.assertEqual(result["logs"], [event])

    async def test_spotify_id_uses_one_bounded_spotiflac_run(self):
        track_id = "test-spotify-track-2026"
        calls = []

        async def fake_spotiflac(**kwargs):
            calls.append(kwargs)
            output_dir = Path(kwargs["output_dir"])
            output_dir.mkdir(parents=True, exist_ok=True)
            (output_dir / "matched.flac").write_bytes(b"audio")
            return SimpleNamespace(failed=[], skipped=[])

        with tempfile.TemporaryDirectory() as temp_dir:
            cache_dir = Path(temp_dir) / "cache"
            cache_dir.mkdir()
            with (
                patch.object(main, "CACHE_DIR", cache_dir),
                patch.object(main, "STATUS_FILE", cache_dir / "status.json"),
                patch.object(main, "LOG_FILE", cache_dir / "logs.json"),
                patch.object(main, "QOBUZ_API_URL", "http://qobuz.test"),
                patch.object(main, "DOWNLOAD_DELAY_SECONDS", 0),
                patch.object(main, "_run_spotiflac_download", side_effect=fake_spotiflac),
                patch.object(main, "_record_download_event"),
            ):
                result = await main._download_audio(track_id)
                result_bytes = result.read_bytes()

        self.assertEqual(result_bytes, b"audio")
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0]["url"], f"https://open.spotify.com/track/{track_id}")
        self.assertEqual(calls[0]["services"], ["qobuz", "tidal", "amazon", "deezer"])
        self.assertEqual(calls[0]["qobuz_local_api_url"], "http://qobuz.test")
        self.assertEqual(calls[0]["track_max_retries"], 0)
        self.assertEqual(calls[0]["timeout_s"], 45)
        self.assertNotIn("loop", calls[0])

    async def test_configured_qobuz_credentials_disable_other_providers(self):
        track_id = "test-spotify-track-qobuz"
        calls = []

        async def fake_spotiflac(**kwargs):
            calls.append(kwargs)
            output_dir = Path(kwargs["output_dir"])
            output_dir.mkdir(parents=True, exist_ok=True)
            (output_dir / "matched.flac").write_bytes(b"audio")
            return SimpleNamespace(failed=[], skipped=[])

        with tempfile.TemporaryDirectory() as temp_dir:
            cache_dir = Path(temp_dir) / "cache"
            cache_dir.mkdir()
            with (
                patch.object(main, "CACHE_DIR", cache_dir),
                patch.object(main, "STATUS_FILE", cache_dir / "status.json"),
                patch.object(main, "LOG_FILE", cache_dir / "logs.json"),
                patch.object(main, "QOBUZ_API_URL", "http://qobuz.test"),
                patch.object(main, "_qobuz_credentials", {
                    "app_id": "test-app",
                    "app_secret": "test-secret",
                    "user_token": "test-token",
                }),
                patch.object(main, "DOWNLOAD_DELAY_SECONDS", 0),
                patch.object(main, "_run_spotiflac_download", side_effect=fake_spotiflac),
                patch.object(main, "_record_download_event"),
            ):
                result = await main._download_audio(track_id)
                result_bytes = result.read_bytes()

        self.assertEqual(result_bytes, b"audio")
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0]["services"], ["qobuz"])
        self.assertEqual(calls[0]["qobuz_token"], "test-token")
        self.assertFalse(calls[0]["allow_fallback"])
        self.assertFalse(calls[0]["use_extensions_fallback"])

    async def test_spotiflac_provider_failure_is_reported_instead_of_generic_no_audio(self):
        async def fake_spotiflac(**_kwargs):
            return SimpleNamespace(
                failed=[
                    SimpleNamespace(provider="qobuz", reason="provider unavailable")
                ],
                skipped=[],
            )

        with tempfile.TemporaryDirectory() as temp_dir:
            cache_dir = Path(temp_dir) / "cache"
            cache_dir.mkdir()
            with (
                patch.object(main, "CACHE_DIR", cache_dir),
                patch.object(main, "STATUS_FILE", cache_dir / "status.json"),
                patch.object(main, "LOG_FILE", cache_dir / "logs.json"),
                patch.object(main, "DOWNLOAD_DELAY_SECONDS", 0),
                patch.object(main, "_run_spotiflac_download", side_effect=fake_spotiflac),
                patch.object(main, "_record_download_event"),
            ):
                with self.assertRaisesRegex(
                    RuntimeError, "qobuz: provider unavailable"
                ):
                    await main._download_audio("test-spotify-track-failure")


if __name__ == "__main__":
    unittest.main()