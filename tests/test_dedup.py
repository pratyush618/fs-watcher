import pyfs_watcher


def test_find_duplicates(duplicate_tree):
    groups = pyfs_watcher.find_duplicates([str(duplicate_tree)])
    assert len(groups) == 2  # Two groups: a's and b's

    # Sort by number of copies for consistent assertions
    groups_sorted = sorted(groups, key=lambda g: len(g.paths), reverse=True)

    # Group A has 3 copies
    assert len(groups_sorted[0].paths) == 3
    assert groups_sorted[0].file_size == 10000
    assert groups_sorted[0].wasted_bytes == 20000  # 10000 * 2

    # Group B has 2 copies
    assert len(groups_sorted[1].paths) == 2
    assert groups_sorted[1].file_size == 10000
    assert groups_sorted[1].wasted_bytes == 10000


def test_find_duplicates_no_dups(tmp_path):
    for i in range(5):
        (tmp_path / f"unique_{i}.bin").write_bytes(bytes([i]) * 1000)

    groups = pyfs_watcher.find_duplicates([str(tmp_path)])
    assert len(groups) == 0


def test_find_duplicates_min_size(duplicate_tree):
    # All files are 10000 or 5000 bytes. Setting min_size=10001 should exclude everything.
    groups = pyfs_watcher.find_duplicates([str(duplicate_tree)], min_size=10001)
    assert len(groups) == 0


def test_find_duplicates_with_progress(duplicate_tree):
    stages = []
    groups = pyfs_watcher.find_duplicates(
        [str(duplicate_tree)],
        progress_callback=lambda stage, done, total: stages.append(stage),
    )
    assert "size_grouping" in stages
    assert "partial_hash" in stages
    assert "full_hash" in stages
    assert len(groups) > 0


def test_find_duplicates_sha256(duplicate_tree):
    groups = pyfs_watcher.find_duplicates([str(duplicate_tree)], algorithm="sha256")
    assert len(groups) == 2


def test_duplicate_group_repr(duplicate_tree):
    groups = pyfs_watcher.find_duplicates([str(duplicate_tree)])
    for g in groups:
        r = repr(g)
        assert "DuplicateGroup" in r
        assert len(g) >= 2


def test_find_duplicates_sorted_by_waste(duplicate_tree):
    groups = pyfs_watcher.find_duplicates([str(duplicate_tree)])
    wastes = [g.wasted_bytes for g in groups]
    assert wastes == sorted(wastes, reverse=True)
