# Walk

Recursively traverse directory trees with parallel I/O powered by [jwalk](https://crates.io/crates/jwalk) and Rayon.

## Streaming vs Collecting

pyfs-watcher offers two walk functions, each suited to different use cases:

| Function | Returns | Best For |
|---|---|---|
| `walk()` | Streaming `WalkIter` | Large trees, early termination, low memory |
| `walk_collect()` | `list[WalkEntry]` | Full result set, maximum throughput |

`walk_collect()` is faster when you need all results because it avoids per-item GIL overhead by collecting everything in Rust before returning to Python.

---

## Basic Usage

### Streaming iteration

```python
import pyfs_watcher

for entry in pyfs_watcher.walk("/data"):
    print(entry.path, entry.is_file, entry.file_size)
```

The iterator yields `WalkEntry` objects as the parallel traversal engine discovers them. You can break out of the loop early without waiting for the full scan to complete.

### Bulk collection

```python
entries = pyfs_watcher.walk_collect("/data", sort=True)
print(f"Found {len(entries)} entries")
```

---

## Filtering

Both functions accept the same filtering parameters:

### By file type

```python
# Only files
files = pyfs_watcher.walk_collect("/src", file_type="file")

# Only directories
dirs = pyfs_watcher.walk_collect("/src", file_type="dir")
```

### By glob pattern

```python
# Only Python files
for entry in pyfs_watcher.walk("/project", file_type="file", glob_pattern="*.py"):
    print(entry.path)
```

The glob pattern matches against the **filename** only, not the full path.

### By depth

```python
# Only top-level contents (depth 1)
entries = pyfs_watcher.walk_collect("/data", max_depth=1)

# Up to 3 levels deep
entries = pyfs_watcher.walk_collect("/data", max_depth=3)
```

### Skip hidden files

```python
entries = pyfs_watcher.walk_collect("/home/user", skip_hidden=True)
```

Entries whose name starts with a dot (`.git`, `.env`, etc.) are excluded.

### Sorting

```python
entries = pyfs_watcher.walk_collect("/data", sort=True)
```

When `sort=True`, entries within each directory are sorted by path. This makes output deterministic but adds overhead.

### Symlinks

```python
entries = pyfs_watcher.walk_collect("/data", follow_symlinks=True)
```

By default, symlinks are **not** followed to avoid infinite loops.

---

## WalkEntry Properties

Each entry provides:

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Absolute path |
| `is_file` | `bool` | Regular file? |
| `is_dir` | `bool` | Directory? |
| `is_symlink` | `bool` | Symbolic link? |
| `depth` | `int` | Depth relative to root (children = 1) |
| `file_size` | `int` | Size in bytes (0 for directories) |

---

## Comparison with os.walk

```python
# os.walk — single-threaded, yields (dirpath, dirnames, filenames)
import os
for root, dirs, files in os.walk("/data"):
    for f in files:
        print(os.path.join(root, f))

# pyfs_watcher.walk — parallel, yields WalkEntry with metadata
import pyfs_watcher
for entry in pyfs_watcher.walk("/data", file_type="file"):
    print(entry.path, entry.file_size)
```

Key differences:

- **Speed**: pyfs-watcher uses parallel I/O across multiple threads, which is significantly faster on SSDs and network filesystems.
- **Metadata**: `WalkEntry` includes `file_size`, `is_symlink`, and `depth` without extra `os.stat()` calls.
- **Filtering**: Built-in glob, depth, type, and hidden-file filtering happens in Rust before crossing the Python boundary.

---

## Error Handling

```python
try:
    entries = pyfs_watcher.walk_collect("/nonexistent")
except pyfs_watcher.WalkError as e:
    print(f"Walk failed: {e}")
```

Individual unreadable subdirectories are silently skipped rather than raising exceptions. Only root-level errors raise `WalkError`.

---

## Performance Tips

- Use `walk_collect()` when you need all results — it avoids per-entry GIL acquisition.
- Use `file_type` and `glob_pattern` to filter in Rust rather than in Python.
- Avoid `sort=True` unless you need deterministic ordering.
- Set `max_depth` when you only need shallow results.
