from pathlib import Path

import pytest


@pytest.fixture
def sample_tree(tmp_path: Path) -> Path:
    """Create a realistic directory tree for testing."""
    for depth in range(4):
        d = tmp_path / "/".join(f"level{i}" for i in range(depth + 1))
        d.mkdir(parents=True, exist_ok=True)
        for j in range(5):
            (d / f"file_{j}.txt").write_text(f"content at depth {depth}, file {j}")
    return tmp_path


@pytest.fixture
def duplicate_tree(tmp_path: Path) -> Path:
    """Create a tree with known duplicates."""
    content_a = b"x" * 10000
    content_b = b"y" * 10000
    for i in range(3):
        (tmp_path / f"dup_a_{i}.bin").write_bytes(content_a)
    for i in range(2):
        (tmp_path / f"dup_b_{i}.bin").write_bytes(content_b)
    (tmp_path / "unique.bin").write_bytes(b"z" * 5000)
    return tmp_path


@pytest.fixture
def copy_source(tmp_path: Path) -> Path:
    """Create a source tree for copy/move tests."""
    src = tmp_path / "src"
    src.mkdir()
    (src / "file1.txt").write_text("hello")
    (src / "file2.txt").write_text("world")
    sub = src / "subdir"
    sub.mkdir()
    (sub / "nested.txt").write_text("nested content")
    return src
