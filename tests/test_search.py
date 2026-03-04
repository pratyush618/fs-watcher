from pathlib import Path

import pyfs_watcher


def _create_search_tree(tmp_path: Path) -> Path:
    (tmp_path / "hello.txt").write_text("Hello World\nfoo bar\nHello Again\n")
    (tmp_path / "code.py").write_text("def hello():\n    print('hello')\n")
    (tmp_path / "empty.txt").write_text("")
    (tmp_path / "binary.bin").write_bytes(b"\x00\x01\x02\x00\x03")
    sub = tmp_path / "sub"
    sub.mkdir()
    (sub / "nested.txt").write_text("Hello from nested\n")
    return tmp_path


def test_search_basic(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = pyfs_watcher.search(str(tmp_path), "Hello")

    assert len(results) > 0
    total_matches = sum(r.match_count for r in results)
    assert total_matches >= 3  # hello.txt(2) + nested.txt(1)


def test_search_case_insensitive(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = pyfs_watcher.search(str(tmp_path), "hello", ignore_case=True)

    total_matches = sum(r.match_count for r in results)
    # Should match "Hello" and "hello" in all files
    assert total_matches >= 4


def test_search_glob_filter(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = pyfs_watcher.search(str(tmp_path), "Hello", glob_pattern="*.txt")

    for r in results:
        assert r.path.endswith(".txt")


def test_search_max_count(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = pyfs_watcher.search(str(tmp_path), "Hello", max_count=1)

    for r in results:
        assert r.match_count <= 1


def test_search_context_lines(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = pyfs_watcher.search(str(tmp_path), "foo", context_lines=1)

    for r in results:
        for m in r.matches:
            assert len(m.context_before) <= 1
            assert len(m.context_after) <= 1


def test_search_skips_binary(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = pyfs_watcher.search(str(tmp_path), ".*")

    paths = [r.path for r in results]
    assert not any("binary.bin" in p for p in paths)


def test_search_result_properties(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = pyfs_watcher.search(str(tmp_path), r"Hello\s+\w+")

    for r in results:
        assert isinstance(r.path, str)
        assert len(r) == r.match_count
        for m in r.matches:
            assert m.line_number > 0
            assert m.match_start < m.match_end
            assert isinstance(m.line_text, str)


def test_search_iter(tmp_path: Path):
    _create_search_tree(tmp_path)
    results = list(pyfs_watcher.search_iter(str(tmp_path), "Hello"))
    assert len(results) > 0


def test_search_nonexistent():
    import pytest

    with pytest.raises(pyfs_watcher.SearchError):
        pyfs_watcher.search("/nonexistent/path/xyz", "pattern")


def test_search_invalid_regex():
    import tempfile

    import pytest

    with tempfile.TemporaryDirectory() as tmp, pytest.raises(pyfs_watcher.SearchError):
        pyfs_watcher.search(tmp, "[invalid")


def test_search_max_filesize(tmp_path: Path):
    (tmp_path / "small.txt").write_text("hello match")
    (tmp_path / "large.txt").write_text("hello match " * 1000)

    results = pyfs_watcher.search(str(tmp_path), "hello", max_filesize=100)
    paths = [r.path for r in results]
    assert any("small.txt" in p for p in paths)
    assert not any("large.txt" in p for p in paths)
