# Walk API

## walk()

```python
def walk(
    path: str | PathLike[str],
    *,
    max_depth: int | None = None,
    follow_symlinks: bool = False,
    sort: bool = False,
    skip_hidden: bool = False,
    file_type: Literal["file", "dir", "any"] = "any",
    glob_pattern: str | None = None,
) -> WalkIter
```

Recursively walk a directory tree, yielding entries as they are found.

Uses parallel traversal (jwalk/rayon) for high throughput. Entries are streamed through an internal channel so iteration can begin before the full tree is scanned.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Root directory to walk |
| `max_depth` | `int \| None` | `None` | Maximum recursion depth (`None` for unlimited) |
| `follow_symlinks` | `bool` | `False` | Whether to follow symbolic links |
| `sort` | `bool` | `False` | Sort entries by path within each directory |
| `skip_hidden` | `bool` | `False` | Skip entries whose name starts with a dot |
| `file_type` | `Literal["file", "dir", "any"]` | `"any"` | Filter by entry type |
| `glob_pattern` | `str \| None` | `None` | Only yield entries whose filename matches this glob (e.g. `"*.py"`) |

### Returns

A streaming [`WalkIter`](#walkiter) iterator of [`WalkEntry`](#walkentry) objects.

### Raises

- `WalkError` — If the root path cannot be read.

### Example

```python
for entry in pyfs_watcher.walk("/data", file_type="file", glob_pattern="*.py"):
    print(entry.path, entry.file_size)
```

---

## walk_collect()

```python
def walk_collect(
    path: str | PathLike[str],
    *,
    max_depth: int | None = None,
    follow_symlinks: bool = False,
    sort: bool = False,
    skip_hidden: bool = False,
    file_type: Literal["file", "dir", "any"] = "any",
    glob_pattern: str | None = None,
) -> list[WalkEntry]
```

Recursively walk a directory tree and return all entries at once.

Faster than `walk()` when you need the full result set, because it avoids per-item GIL overhead by collecting everything in Rust first.

### Parameters

Same as [`walk()`](#walk).

### Returns

A `list` of all matching [`WalkEntry`](#walkentry) objects.

### Raises

- `WalkError` — If the root path cannot be read.

### Example

```python
entries = pyfs_watcher.walk_collect("/data", max_depth=3, sort=True, skip_hidden=True)
print(f"Found {len(entries)} entries")
```

---

## WalkEntry

```python
class WalkEntry
```

A single entry discovered during directory traversal. Represents a file, directory, or symlink found by `walk()` or `walk_collect()`.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Absolute path of the entry |
| `is_dir` | `bool` | Whether the entry is a directory |
| `is_file` | `bool` | Whether the entry is a regular file |
| `is_symlink` | `bool` | Whether the entry is a symbolic link |
| `depth` | `int` | Depth relative to the walk root (root children = 1) |
| `file_size` | `int` | Size of the file in bytes (0 for directories) |

### Example

```python
entry = next(iter(pyfs_watcher.walk("/data")))
print(entry.path)       # "/data/file.txt"
print(entry.is_file)    # True
print(entry.file_size)  # 1024
print(entry.depth)      # 1
```

---

## WalkIter

```python
class WalkIter
```

Streaming iterator over directory entries. Yields `WalkEntry` objects as they are discovered by the parallel traversal engine. Supports `Ctrl+C` interruption.

### Protocols

- `__iter__() -> Iterator[WalkEntry]`
- `__next__() -> WalkEntry`

### Example

```python
walker = pyfs_watcher.walk("/data")
for entry in walker:
    if entry.file_size > 1_000_000:
        print(f"Large file: {entry.path}")
        break  # Early termination is efficient
```
