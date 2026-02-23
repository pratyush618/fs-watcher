from pathlib import Path

import pyfs_watcher
import pytest


def test_copy_single_file(tmp_path):
    src = tmp_path / "source.txt"
    src.write_text("hello")
    dst_dir = tmp_path / "dest"
    dst_dir.mkdir()

    result = pyfs_watcher.copy_files([str(src)], str(dst_dir))
    assert len(result) == 1
    assert Path(result[0]).read_text() == "hello"


def test_copy_multiple_files(tmp_path):
    src1 = tmp_path / "a.txt"
    src2 = tmp_path / "b.txt"
    src1.write_text("aaa")
    src2.write_text("bbb")
    dst = tmp_path / "dest"
    dst.mkdir()

    result = pyfs_watcher.copy_files([str(src1), str(src2)], str(dst))
    assert len(result) == 2
    assert (dst / "a.txt").read_text() == "aaa"
    assert (dst / "b.txt").read_text() == "bbb"


def test_copy_directory(copy_source, tmp_path):
    dst = tmp_path / "dest"
    dst.mkdir()

    result = pyfs_watcher.copy_files([str(copy_source)], str(dst))
    assert len(result) > 0
    # Check nested file was copied
    assert (dst / "src" / "subdir" / "nested.txt").read_text() == "nested content"


def test_copy_overwrite_false(tmp_path):
    src = tmp_path / "src.txt"
    src.write_text("new")
    dst = tmp_path / "dst.txt"
    dst.write_text("existing")

    with pytest.raises(pyfs_watcher.CopyError, match="overwrite"):
        pyfs_watcher.copy_files([str(src)], str(dst))


def test_copy_overwrite_true(tmp_path):
    src = tmp_path / "src.txt"
    src.write_text("new content")
    dst = tmp_path / "dst.txt"
    dst.write_text("old content")

    pyfs_watcher.copy_files([str(src)], str(dst), overwrite=True)
    assert dst.read_text() == "new content"


def test_copy_nonexistent_source(tmp_path):
    with pytest.raises(pyfs_watcher.CopyError):
        pyfs_watcher.copy_files(["/nonexistent/file.txt"], str(tmp_path))


def test_copy_with_progress(tmp_path):
    src = tmp_path / "big.bin"
    src.write_bytes(b"x" * 100_000)
    dst = tmp_path / "dest"
    dst.mkdir()

    progress_updates = []
    pyfs_watcher.copy_files(
        [str(src)],
        str(dst),
        progress_callback=lambda p: progress_updates.append(p),
    )
    # Should get at least a final callback
    assert len(progress_updates) >= 1
    last = progress_updates[-1]
    assert last.total_files == 1
    assert last.files_completed == 1


def test_move_file(tmp_path):
    src = tmp_path / "source.txt"
    src.write_text("moveme")
    dst = tmp_path / "dest"
    dst.mkdir()

    result = pyfs_watcher.move_files([str(src)], str(dst))
    assert len(result) == 1
    assert not src.exists()
    assert Path(result[0]).read_text() == "moveme"


def test_move_nonexistent_source(tmp_path):
    with pytest.raises(pyfs_watcher.CopyError):
        pyfs_watcher.move_files(["/nonexistent/file.txt"], str(tmp_path))
