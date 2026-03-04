from pathlib import Path

import pyfs_watcher


def _create_sync_dirs(tmp_path: Path):
    src = tmp_path / "source"
    tgt = tmp_path / "target"
    src.mkdir()
    tgt.mkdir()

    # New file (only in source)
    (src / "new.txt").write_text("new content")

    # Unchanged file
    (src / "same.txt").write_text("same content")
    (tgt / "same.txt").write_text("same content")

    # Modified file
    (src / "modified.txt").write_text("updated version")
    (tgt / "modified.txt").write_text("old version")

    # Extra file (only in target)
    (tgt / "extra.txt").write_text("extra file")

    return src, tgt


def test_sync_basic(tmp_path: Path):
    src, tgt = _create_sync_dirs(tmp_path)
    result = pyfs_watcher.sync(str(src), str(tgt))

    assert "new.txt" in result.copied
    assert "modified.txt" in result.copied
    assert "same.txt" in result.skipped
    assert len(result.errors) == 0

    # Verify files exist in target
    assert (tgt / "new.txt").read_text() == "new content"
    assert (tgt / "modified.txt").read_text() == "updated version"


def test_sync_dry_run(tmp_path: Path):
    src, tgt = _create_sync_dirs(tmp_path)
    result = pyfs_watcher.sync(str(src), str(tgt), dry_run=True)

    assert len(result.copied) > 0
    # Files should NOT have been copied in dry run
    assert not (tgt / "new.txt").exists()


def test_sync_delete_extra(tmp_path: Path):
    src, tgt = _create_sync_dirs(tmp_path)
    result = pyfs_watcher.sync(str(src), str(tgt), delete_extra=True)

    assert "extra.txt" in result.deleted
    assert not (tgt / "extra.txt").exists()


def test_sync_no_delete_by_default(tmp_path: Path):
    src, tgt = _create_sync_dirs(tmp_path)
    result = pyfs_watcher.sync(str(src), str(tgt))

    assert len(result.deleted) == 0
    assert (tgt / "extra.txt").exists()


def test_sync_creates_target(tmp_path: Path):
    src = tmp_path / "source"
    tgt = tmp_path / "target_new"
    src.mkdir()
    (src / "file.txt").write_text("content")

    result = pyfs_watcher.sync(str(src), str(tgt))

    assert len(result.copied) == 1
    assert (tgt / "file.txt").read_text() == "content"


def test_sync_subdirectories(tmp_path: Path):
    src = tmp_path / "source"
    tgt = tmp_path / "target"
    src.mkdir()
    tgt.mkdir()

    sub = src / "subdir"
    sub.mkdir()
    (sub / "nested.txt").write_text("nested")

    result = pyfs_watcher.sync(str(src), str(tgt))

    assert any("nested.txt" in p for p in result.copied)
    assert (tgt / "subdir" / "nested.txt").read_text() == "nested"


def test_sync_nonexistent_source(tmp_path: Path):
    import pytest

    with pytest.raises(pyfs_watcher.SyncError):
        pyfs_watcher.sync("/nonexistent/path", str(tmp_path))


def test_sync_repr(tmp_path: Path):
    src, tgt = _create_sync_dirs(tmp_path)
    result = pyfs_watcher.sync(str(src), str(tgt))
    assert "SyncResult" in repr(result)


def test_sync_with_progress(tmp_path: Path):
    src, tgt = _create_sync_dirs(tmp_path)
    stages = []

    def on_progress(p):
        stages.append(p.stage)

    pyfs_watcher.sync(str(src), str(tgt), progress_callback=on_progress)
    assert len(stages) > 0
