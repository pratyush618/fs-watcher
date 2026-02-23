import hashlib

import pyfs_watcher
import pytest


def test_hash_file_blake3(tmp_path):
    f = tmp_path / "test.txt"
    f.write_text("hello world")
    result = pyfs_watcher.hash_file(str(f), algorithm="blake3")
    assert result.algorithm == "blake3"
    assert result.file_size == 11
    assert result.hash_hex == "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
    assert result.path == str(f)


def test_hash_file_sha256(tmp_path):
    f = tmp_path / "test.txt"
    f.write_text("hello world")
    result = pyfs_watcher.hash_file(str(f), algorithm="sha256")
    assert result.algorithm == "sha256"
    # Verify against Python's hashlib
    expected = hashlib.sha256(b"hello world").hexdigest()
    assert result.hash_hex == expected


def test_hash_file_empty(tmp_path):
    f = tmp_path / "empty.txt"
    f.write_bytes(b"")
    result = pyfs_watcher.hash_file(str(f))
    assert result.file_size == 0
    assert len(result.hash_hex) > 0


def test_hash_file_nonexistent():
    with pytest.raises(FileNotFoundError):
        pyfs_watcher.hash_file("/nonexistent/file.txt")


def test_hash_file_invalid_algorithm(tmp_path):
    f = tmp_path / "test.txt"
    f.write_text("data")
    with pytest.raises(pyfs_watcher.HashError):
        pyfs_watcher.hash_file(str(f), algorithm="md5")


def test_hash_files_parallel(tmp_path):
    paths = []
    for i in range(10):
        f = tmp_path / f"file_{i}.txt"
        f.write_text(f"content {i}")
        paths.append(str(f))

    results = pyfs_watcher.hash_files(paths, algorithm="blake3")
    assert len(results) == 10
    # All hashes should be unique (different content)
    hashes = [r.hash_hex for r in results]
    assert len(set(hashes)) == 10


def test_hash_files_with_callback(tmp_path):
    paths = []
    for i in range(5):
        f = tmp_path / f"file_{i}.txt"
        f.write_text(f"content {i}")
        paths.append(str(f))

    callback_results = []
    results = pyfs_watcher.hash_files(
        paths,
        callback=lambda r: callback_results.append(r.hash_hex),
    )
    assert len(results) == 5
    assert len(callback_results) == 5


def test_hash_result_repr(tmp_path):
    f = tmp_path / "test.txt"
    f.write_text("data")
    result = pyfs_watcher.hash_file(str(f))
    r = repr(result)
    assert "HashResult" in r


def test_hash_result_equality(tmp_path):
    f = tmp_path / "test.txt"
    f.write_text("same content")
    r1 = pyfs_watcher.hash_file(str(f))
    r2 = pyfs_watcher.hash_file(str(f))
    assert r1 == r2
