import json
from pathlib import Path

import pyfs_watcher
from pyfs_watcher.types import Snapshot


def _create_snapshot_tree(tmp_path: Path) -> Path:
    (tmp_path / "a.txt").write_text("hello")
    (tmp_path / "b.txt").write_text("world")
    sub = tmp_path / "sub"
    sub.mkdir()
    (sub / "c.txt").write_text("nested")
    return tmp_path


def test_snapshot_basic(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))

    assert snap.total_files == 3
    assert snap.total_size == 5 + 5 + 6  # hello + world + nested
    assert len(snap.entries) == 3
    assert snap.algorithm == "blake3"
    assert len(snap) == 3
    assert snap.created_at  # ISO 8601 string


def test_snapshot_save_load(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))

    snap_path = str(tmp_path / "snapshot.json")
    snap.save(snap_path)

    # Verify JSON is valid
    with open(snap_path) as f:
        data = json.load(f)
    assert data["total_files"] == 3
    assert data["algorithm"] == "blake3"

    # Load and verify
    loaded = Snapshot.load(snap_path)
    assert loaded.total_files == snap.total_files
    assert loaded.algorithm == snap.algorithm
    assert len(loaded.entries) == len(snap.entries)


def test_verify_unchanged(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))

    result = pyfs_watcher.verify(snap)

    assert result.ok
    assert len(result.added) == 0
    assert len(result.removed) == 0
    assert len(result.modified) == 0


def test_verify_modified_file(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))

    # Modify a file
    (root / "a.txt").write_text("changed!")

    result = pyfs_watcher.verify(snap)

    assert not result.ok
    assert len(result.modified) == 1
    modified_paths = [c.path for c in result.modified]
    assert "a.txt" in modified_paths


def test_verify_added_file(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))

    # Add a new file
    (root / "new.txt").write_text("new file")

    result = pyfs_watcher.verify(snap)

    assert not result.ok
    assert len(result.added) == 1


def test_verify_removed_file(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))

    # Remove a file
    (root / "a.txt").unlink()

    result = pyfs_watcher.verify(snap)

    assert not result.ok
    assert len(result.removed) == 1


def test_verify_from_file(tmp_path: Path):
    root = tmp_path / "data"
    root.mkdir()
    (root / "a.txt").write_text("hello")
    (root / "b.txt").write_text("world")

    snap = pyfs_watcher.snapshot(str(root))

    # Save snapshot outside the data dir so it doesn't appear as "added"
    snap_path = str(tmp_path / "snapshot.json")
    snap.save(snap_path)

    result = pyfs_watcher.verify(snap_path)
    assert result.ok


def test_snapshot_sha256(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root), algorithm="sha256")

    assert snap.algorithm == "sha256"
    assert snap.total_files == 3


def test_snapshot_skip_hidden(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    (root / ".hidden").write_text("secret")

    snap = pyfs_watcher.snapshot(str(root), skip_hidden=True)
    assert snap.total_files == 3  # .hidden excluded

    snap_all = pyfs_watcher.snapshot(str(root), skip_hidden=False)
    assert snap_all.total_files == 4


def test_snapshot_entry_properties(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))

    for entry in snap.entries:
        assert isinstance(entry.path, str)
        assert isinstance(entry.hash_hex, str)
        assert len(entry.hash_hex) > 0
        assert entry.file_size > 0
        assert entry.mtime > 0
        assert isinstance(entry.permissions, int)


def test_snapshot_repr(tmp_path: Path):
    root = _create_snapshot_tree(tmp_path)
    snap = pyfs_watcher.snapshot(str(root))
    assert "Snapshot" in repr(snap)
