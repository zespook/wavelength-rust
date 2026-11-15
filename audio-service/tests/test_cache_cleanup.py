import os
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

import main


class AudioCacheCleanupTests(unittest.TestCase):
    def test_cleanup_removes_only_expired_room_audio_and_temp_directories(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            cache_dir = Path(temp_dir)
            now = time.time()
            old_timestamp = time.time() - (7 * 24 * 60 * 60 + 60)
            fresh_timestamp = time.time() - (7 * 24 * 60 * 60 - 60)
            expired_audio = cache_dir / "room-expired.flac"
            fresh_audio = cache_dir / "room-fresh.flac"
            permanent_audio = cache_dir / "regular-permanent.wav"
            active_audio = cache_dir / "room-active.wav"
            status_file = cache_dir / "download-status.json"
            temp_root = cache_dir / "spotiflac_temp"
            expired_temp = temp_root / "expired"
            active_temp = temp_root / "active"

            for path in (expired_audio, fresh_audio, permanent_audio, active_audio, status_file):
                path.write_bytes(b"data")
            for directory in (expired_temp, active_temp):
                directory.mkdir(parents=True)
                (directory / "temporary.part").write_bytes(b"data")
            for path in (expired_audio, active_audio):
                path.touch()
                os.utime(path, (old_timestamp, old_timestamp))
            fresh_audio.touch()
            os.utime(fresh_audio, (fresh_timestamp, fresh_timestamp))
            os.utime(permanent_audio, (old_timestamp, old_timestamp))
            for directory in (expired_temp, active_temp):
                os.utime(directory, (old_timestamp, old_timestamp))

            retention = {
                "room-expired": {"scope": "room", "expiresAt": now - 1},
                "room-fresh": {"scope": "room", "expiresAt": now + 60},
                "room-active": {"scope": "room", "expiresAt": now - 1},
            }
            statuses = {"room-expired": "ready", "room-fresh": "ready", "room-active": "ready"}
            with (
                patch.object(main, "CACHE_DIR", cache_dir),
                patch.object(main, "STATUS_FILE", status_file),
                patch.object(main, "CACHE_MAX_AGE_SECONDS", 7 * 24 * 60 * 60),
                patch.object(main, "_downloading", {"active"}),
                patch.object(main, "_streaming_tracks", {"room-active"}),
                patch.object(main, "_download_retention", retention),
                patch.object(main, "_download_status", statuses),
                patch.object(main, "_download_progress", {}),
                patch.object(main, "_download_errors", {}),
            ):
                main._cleanup_cache()

            self.assertFalse(expired_audio.exists())
            self.assertTrue(fresh_audio.exists())
            self.assertTrue(permanent_audio.exists())
            self.assertTrue(active_audio.exists())
            self.assertTrue(status_file.exists())
            self.assertFalse(expired_temp.exists())
            self.assertTrue(active_temp.exists())
            self.assertNotIn("room-expired", retention)
            self.assertIn("room-active", retention)
            self.assertNotIn("room-expired", statuses)

    def test_room_scope_does_not_downgrade_an_existing_permanent_download(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            with (
                patch.object(main, "CACHE_DIR", Path(temp_dir)),
                patch.object(main, "_download_retention", {}),
            ):
                main._mark_cached_download("regular-track", "room")
                self.assertNotIn("regular-track", main._download_retention)

    def test_regular_use_promotes_room_download_to_permanent(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            with (
                patch.object(main, "CACHE_DIR", Path(temp_dir)),
                patch.object(main, "_download_retention", {}),
            ):
                main._mark_download_complete("room-track", "room")
                self.assertEqual(main._download_retention["room-track"]["scope"], "room")
                self.assertGreater(main._download_retention["room-track"]["expiresAt"], time.time())

                main._mark_cached_download("room-track", "permanent")
                self.assertNotIn("room-track", main._download_retention)

    def test_cleanup_interval_is_at_most_one_hour(self):
        self.assertGreaterEqual(main.CACHE_CLEANUP_INTERVAL_SECONDS, 60)
        self.assertLessEqual(main.CACHE_CLEANUP_INTERVAL_SECONDS, 60 * 60)


if __name__ == "__main__":
    unittest.main()