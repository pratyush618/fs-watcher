import threading
import time

import pyfs_watcher
import pytest


def test_watcher_context_manager(tmp_path):
    """Test that FileWatcher works as a context manager."""
    with pyfs_watcher.FileWatcher(str(tmp_path)) as w:
        # Create a file in a background thread
        def create_file():
            time.sleep(0.3)
            (tmp_path / "test.txt").write_text("hello")

        t = threading.Thread(target=create_file)
        t.start()

        w.poll_events(timeout_ms=2000)
        t.join()

    # We might or might not catch the event depending on timing
    # Just verify it doesn't crash


def test_watcher_detects_creation(tmp_path):
    """Test that watcher detects file creation."""
    watcher = pyfs_watcher.FileWatcher(str(tmp_path), debounce_ms=100)
    watcher.start()

    try:
        # Create a file
        (tmp_path / "created.txt").write_text("new file")

        # Poll for events (with retries for debouncing)
        all_changes = []
        for _ in range(10):
            changes = watcher.poll_events(timeout_ms=300)
            all_changes.extend(changes)
            if all_changes:
                break

        assert len(all_changes) > 0
        assert any(c.change_type in ("created", "modified") for c in all_changes)
    finally:
        watcher.stop()


def test_watcher_detects_modification(tmp_path):
    """Test that watcher detects file modification."""
    f = tmp_path / "existing.txt"
    f.write_text("original")

    watcher = pyfs_watcher.FileWatcher(str(tmp_path), debounce_ms=100)
    watcher.start()

    try:
        time.sleep(0.2)  # Let watcher settle
        f.write_text("modified")

        all_changes = []
        for _ in range(10):
            changes = watcher.poll_events(timeout_ms=300)
            all_changes.extend(changes)
            if all_changes:
                break

        assert len(all_changes) > 0
    finally:
        watcher.stop()


def test_watcher_nonexistent_path():
    with pytest.raises(pyfs_watcher.WatchError):
        pyfs_watcher.FileWatcher("/nonexistent/path/xyz")


def test_watcher_ignore_patterns(tmp_path):
    watcher = pyfs_watcher.FileWatcher(
        str(tmp_path),
        debounce_ms=100,
        ignore_patterns=["*.tmp"],
    )
    watcher.start()

    try:
        (tmp_path / "ignored.tmp").write_text("should be ignored")
        (tmp_path / "visible.txt").write_text("should be seen")

        all_changes = []
        for _ in range(10):
            changes = watcher.poll_events(timeout_ms=300)
            all_changes.extend(changes)
            if any("visible" in c.path for c in all_changes):
                break

        # Ensure no .tmp files in changes
        tmp_changes = [c for c in all_changes if c.path.endswith(".tmp")]
        assert len(tmp_changes) == 0
    finally:
        watcher.stop()


def test_file_change_repr(tmp_path):
    watcher = pyfs_watcher.FileWatcher(str(tmp_path), debounce_ms=100)
    watcher.start()

    try:
        (tmp_path / "test.txt").write_text("data")

        for _ in range(10):
            changes = watcher.poll_events(timeout_ms=300)
            if changes:
                r = repr(changes[0])
                assert "FileChange" in r
                break
    finally:
        watcher.stop()
