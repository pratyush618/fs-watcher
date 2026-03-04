# Diff API

## diff_dirs()

```python
def diff_dirs(
    source: str | PathLike[str],
    target: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    compare_content: bool = True,
    skip_hidden: bool = False,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    detect_moves: bool = False,
    max_workers: int | None = None,
    progress_callback: Callable[[str], None] | None = None,
) -> DirDiff
```

Compare two directories and return their differences.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `source` | `str \| PathLike[str]` | *required* | Source directory |
| `target` | `str \| PathLike[str]` | *required* | Target directory |
| `algorithm` | `Literal["sha256", "blake3"]` | `"blake3"` | Hash algorithm for content comparison |
| `compare_content` | `bool` | `True` | Hash same-size files to detect modifications |
| `skip_hidden` | `bool` | `False` | Skip hidden files |
| `glob_pattern` | `str \| None` | `None` | Only compare files matching this glob |
| `max_depth` | `int \| None` | `None` | Maximum recursion depth |
| `detect_moves` | `bool` | `False` | Detect moved/renamed files |
| `max_workers` | `int \| None` | `None` | Max parallel threads |
| `progress_callback` | `Callable[[str], None] \| None` | `None` | `(stage)` callback |

### Progress Callback

The callback receives a single stage string: `"walking_source"`, `"walking_target"`, `"comparing"`.

### Returns

A [`DirDiff`](#dirdiff) object.

### Raises

- `DirDiffError` — If source or target is not a directory.

### Example

```python
diff = pyfs_watcher.diff_dirs("/original", "/copy", detect_moves=True)
print(f"Changes: {diff.total_changes}")
```

---

## DirDiff

```python
class DirDiff
```

Result of comparing two directories.

### Properties

| Property | Type | Description |
|---|---|---|
| `added` | `list[DiffEntry]` | Files only in target |
| `removed` | `list[DiffEntry]` | Files only in source |
| `modified` | `list[DiffEntry]` | Files with different content |
| `unchanged` | `list[DiffEntry]` | Identical files |
| `moved` | `list[MovedEntry]` | Files detected as moved |
| `total_changes` | `int` | `added + removed + modified + moved` |

---

## DiffEntry

```python
class DiffEntry
```

A file that differs between source and target.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Relative path |
| `source_size` | `int \| None` | Size in source (None if not present) |
| `target_size` | `int \| None` | Size in target (None if not present) |
| `source_hash` | `str \| None` | Hash in source (if computed) |
| `target_hash` | `str \| None` | Hash in target (if computed) |

---

## MovedEntry

```python
class MovedEntry
```

A file detected as moved (same content, different path).

### Properties

| Property | Type | Description |
|---|---|---|
| `source_path` | `str` | Relative path in source |
| `target_path` | `str` | Relative path in target |
| `hash_hex` | `str` | Hash digest (confirming identical content) |
| `file_size` | `int` | Size in bytes |
