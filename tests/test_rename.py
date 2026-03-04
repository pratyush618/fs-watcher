from pathlib import Path

import pyfs_watcher
import pytest


def test_bulk_rename_dry_run(tmp_path: Path):
    (tmp_path / "photo_001.jpg").write_text("img1")
    (tmp_path / "photo_002.jpg").write_text("img2")
    (tmp_path / "document.pdf").write_text("doc")

    result = pyfs_watcher.bulk_rename(str(tmp_path), r"photo_(\d+)", r"img_\1")

    assert result.dry_run is True
    assert len(result.renamed) == 2
    assert result.skipped == 1

    # Files should NOT have been renamed (dry run)
    assert (tmp_path / "photo_001.jpg").exists()
    assert (tmp_path / "photo_002.jpg").exists()


def test_bulk_rename_actual(tmp_path: Path):
    (tmp_path / "old_a.txt").write_text("a")
    (tmp_path / "old_b.txt").write_text("b")

    result = pyfs_watcher.bulk_rename(str(tmp_path), r"old_", "new_", dry_run=False)

    assert result.dry_run is False
    assert len(result.renamed) == 2
    assert (tmp_path / "new_a.txt").exists()
    assert (tmp_path / "new_b.txt").exists()
    assert not (tmp_path / "old_a.txt").exists()


def test_bulk_rename_undo(tmp_path: Path):
    (tmp_path / "before.txt").write_text("content")

    result = pyfs_watcher.bulk_rename(str(tmp_path), "before", "after", dry_run=False)

    assert (tmp_path / "after.txt").exists()
    assert not (tmp_path / "before.txt").exists()

    errors = result.undo()
    assert len(errors) == 0
    assert (tmp_path / "before.txt").exists()
    assert not (tmp_path / "after.txt").exists()


def test_bulk_rename_undo_dry_run_raises(tmp_path: Path):
    (tmp_path / "file.txt").write_text("content")

    result = pyfs_watcher.bulk_rename(str(tmp_path), "file", "renamed")

    with pytest.raises(pyfs_watcher.RenameError):
        result.undo()


def test_bulk_rename_recursive(tmp_path: Path):
    sub = tmp_path / "sub"
    sub.mkdir()
    (tmp_path / "old_top.txt").write_text("top")
    (sub / "old_nested.txt").write_text("nested")

    result = pyfs_watcher.bulk_rename(str(tmp_path), r"old_", "new_", recursive=True, dry_run=False)

    assert len(result.renamed) == 2
    assert (tmp_path / "new_top.txt").exists()
    assert (sub / "new_nested.txt").exists()


def test_bulk_rename_non_recursive(tmp_path: Path):
    sub = tmp_path / "sub"
    sub.mkdir()
    (tmp_path / "old_top.txt").write_text("top")
    (sub / "old_nested.txt").write_text("nested")

    result = pyfs_watcher.bulk_rename(
        str(tmp_path), r"old_", "new_", recursive=False, dry_run=False
    )

    assert len(result.renamed) == 1
    assert (tmp_path / "new_top.txt").exists()
    assert (sub / "old_nested.txt").exists()  # not renamed


def test_bulk_rename_glob_filter(tmp_path: Path):
    (tmp_path / "old_a.txt").write_text("a")
    (tmp_path / "old_b.py").write_text("b")

    result = pyfs_watcher.bulk_rename(
        str(tmp_path), r"old_", "new_", glob_pattern="*.txt", dry_run=False
    )

    assert len(result.renamed) == 1
    assert (tmp_path / "new_a.txt").exists()
    assert (tmp_path / "old_b.py").exists()  # not renamed


def test_bulk_rename_include_dirs(tmp_path: Path):
    old_dir = tmp_path / "old_dir"
    old_dir.mkdir()
    (old_dir / "file.txt").write_text("content")

    result = pyfs_watcher.bulk_rename(
        str(tmp_path), r"old_", "new_", include_dirs=True, dry_run=False
    )

    assert len(result.renamed) == 1
    assert (tmp_path / "new_dir").exists()
    assert (tmp_path / "new_dir" / "file.txt").exists()


def test_bulk_rename_no_match(tmp_path: Path):
    (tmp_path / "file.txt").write_text("content")

    result = pyfs_watcher.bulk_rename(str(tmp_path), r"nonexistent", "replacement")

    assert len(result.renamed) == 0
    assert result.skipped == 1


def test_bulk_rename_invalid_regex(tmp_path: Path):
    with pytest.raises(pyfs_watcher.RenameError):
        pyfs_watcher.bulk_rename(str(tmp_path), "[invalid", "replacement")


def test_bulk_rename_repr(tmp_path: Path):
    (tmp_path / "file.txt").write_text("content")
    result = pyfs_watcher.bulk_rename(str(tmp_path), "file", "renamed")
    assert "RenameResult" in repr(result)

    for entry in result.renamed:
        assert "RenameEntry" in repr(entry)
