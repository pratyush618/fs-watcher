from pathlib import Path

import pyfs_watcher


def test_disk_usage_basic(tmp_path: Path):
    (tmp_path / "big_dir").mkdir()
    (tmp_path / "big_dir" / "large.bin").write_bytes(b"\x00" * 10000)
    (tmp_path / "small.txt").write_text("hello")

    result = pyfs_watcher.disk_usage(str(tmp_path))

    assert result.total_files == 2
    assert result.total_size == 10005
    assert isinstance(result.total_dirs, int)
    assert len(result.children) > 0


def test_disk_usage_children_sorted(tmp_path: Path):
    (tmp_path / "big").mkdir()
    (tmp_path / "big" / "file.bin").write_bytes(b"\x00" * 5000)
    (tmp_path / "small").mkdir()
    (tmp_path / "small" / "file.txt").write_text("hi")

    result = pyfs_watcher.disk_usage(str(tmp_path))

    assert result.children[0].size >= result.children[1].size


def test_disk_usage_skip_hidden(tmp_path: Path):
    (tmp_path / "visible.txt").write_text("visible")
    (tmp_path / ".hidden").write_text("hidden")

    result = pyfs_watcher.disk_usage(str(tmp_path), skip_hidden=True)
    assert result.total_files == 1

    result_all = pyfs_watcher.disk_usage(str(tmp_path), skip_hidden=False)
    assert result_all.total_files == 2


def test_disk_usage_glob(tmp_path: Path):
    (tmp_path / "a.txt").write_text("text")
    (tmp_path / "b.py").write_text("python")

    result = pyfs_watcher.disk_usage(str(tmp_path), glob_pattern="*.txt")
    assert result.total_files == 1


def test_disk_usage_nonexistent():
    import pytest

    with pytest.raises(pyfs_watcher.DiskUsageError):
        pyfs_watcher.disk_usage("/nonexistent/path/xyz")


def test_disk_usage_repr(tmp_path: Path):
    (tmp_path / "file.txt").write_text("test")
    result = pyfs_watcher.disk_usage(str(tmp_path))
    assert "DiskUsage" in repr(result)
    if result.children:
        assert "DiskUsageEntry" in repr(result.children[0])
