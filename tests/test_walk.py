import os

import pyfs_watcher


def test_walk_collect_returns_entries(sample_tree):
    entries = pyfs_watcher.walk_collect(str(sample_tree))
    assert len(entries) > 0
    for e in entries:
        assert isinstance(e.path, str)
        assert isinstance(e.depth, int)
        assert isinstance(e.file_size, int)


def test_walk_collect_files_only(sample_tree):
    entries = pyfs_watcher.walk_collect(str(sample_tree), file_type="file")
    assert all(e.is_file for e in entries)
    assert all(not e.is_dir for e in entries)
    assert len(entries) == 20  # 4 depths * 5 files


def test_walk_collect_dirs_only(sample_tree):
    entries = pyfs_watcher.walk_collect(str(sample_tree), file_type="dir")
    assert all(e.is_dir for e in entries)
    assert all(not e.is_file for e in entries)


def test_walk_max_depth(sample_tree):
    entries = pyfs_watcher.walk_collect(str(sample_tree), max_depth=1)
    assert all(e.depth <= 1 for e in entries)


def test_walk_glob_pattern(sample_tree):
    entries = pyfs_watcher.walk_collect(str(sample_tree), glob_pattern="*.txt")
    assert all(e.path.endswith(".txt") for e in entries)


def test_walk_sorted(sample_tree):
    entries = pyfs_watcher.walk_collect(str(sample_tree), sort=True, file_type="file")
    paths = [e.path for e in entries]
    assert paths == sorted(paths)


def test_walk_nonexistent_raises():
    with __import__("pytest").raises(pyfs_watcher.WalkError):
        pyfs_watcher.walk_collect("/nonexistent/path/xyz")


def test_walk_iterator(sample_tree):
    """Test the streaming walk() iterator."""
    count = 0
    for entry in pyfs_watcher.walk(str(sample_tree), file_type="file"):
        assert entry.is_file
        count += 1
    assert count == 20


def test_walk_matches_os_walk(sample_tree):
    """Verify walk_collect finds the same files as os.walk."""
    os_files = set()
    for root, _dirs, files in os.walk(str(sample_tree)):
        for f in files:
            os_files.add(os.path.join(root, f))

    fw_entries = pyfs_watcher.walk_collect(str(sample_tree), file_type="file")
    fw_files = set(e.path for e in fw_entries)

    assert fw_files == os_files


def test_walk_entry_repr(sample_tree):
    entries = pyfs_watcher.walk_collect(str(sample_tree), file_type="file")
    assert len(entries) > 0
    r = repr(entries[0])
    assert "WalkEntry" in r
