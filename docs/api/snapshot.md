# Snapshot API

## snapshot()

```python
def snapshot(
    path: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    skip_hidden: bool = False,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    follow_symlinks: bool = False,
    max_workers: int | None = None,
    progress_callback: Callable[[str, int, int], None] | None = None,
) -> Snapshot
```

Create a snapshot of a directory, capturing file hashes and metadata.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Directory to snapshot |
| `algorithm` | `Literal["sha256", "blake3"]` | `"blake3"` | Hash algorithm |
| `skip_hidden` | `bool` | `False` | Skip hidden files |
| `glob_pattern` | `str \| None` | `None` | Only include matching files |
| `max_depth` | `int \| None` | `None` | Maximum recursion depth |
| `follow_symlinks` | `bool` | `False` | Follow symbolic links |
| `max_workers` | `int \| None` | `None` | Max parallel threads |
| `progress_callback` | `Callable[[str, int, int], None] \| None` | `None` | `(stage, processed, total)` callback |

### Returns

A [`Snapshot`](#snapshot-class) object.

### Raises

- `SnapshotError` — If the path is not a directory.

### Example

```python
snap = pyfs_watcher.snapshot("/data", algorithm="blake3")
snap.save("snapshot.json")
```

---

## verify()

```python
def verify(
    snapshot_or_path: Snapshot | str | PathLike[str],
    *,
    max_workers: int | None = None,
    progress_callback: Callable[[str, int, int], None] | None = None,
) -> VerifyResult
```

Verify a snapshot against the current filesystem state.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `snapshot_or_path` | `Snapshot \| str \| PathLike[str]` | *required* | Snapshot object or path to snapshot JSON |
| `max_workers` | `int \| None` | `None` | Max parallel threads |
| `progress_callback` | `Callable[[str, int, int], None] \| None` | `None` | `(stage, processed, total)` callback |

### Returns

A [`VerifyResult`](#verifyresult) object.

### Raises

- `SnapshotError` — If the snapshot root no longer exists or the JSON is invalid.

### Example

```python
result = pyfs_watcher.verify("snapshot.json")
if not result.ok:
    print(f"Changes detected!")
```

---

## Snapshot {#snapshot-class}

```python
class Snapshot
```

A filesystem snapshot capturing hashes and metadata.

### Properties

| Property | Type | Description |
|---|---|---|
| `root_path` | `str` | Directory that was snapshotted |
| `algorithm` | `str` | Hash algorithm used |
| `created_at` | `str` | ISO 8601 timestamp |
| `total_files` | `int` | Number of files in snapshot |
| `total_size` | `int` | Total bytes across all files |
| `entries` | `list[SnapshotEntry]` | All snapshot entries |

### Methods

- `save(path)` — Save snapshot to a JSON file
- `Snapshot.load(path)` — Load snapshot from a JSON file (static method)

### Protocols

- `__len__() -> int` — Returns `total_files`
- `__repr__() -> str`

---

## SnapshotEntry

```python
class SnapshotEntry
```

A single entry in a filesystem snapshot.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Relative path from snapshot root |
| `hash_hex` | `str` | Hex-encoded hash digest |
| `file_size` | `int` | Size in bytes |
| `mtime` | `float` | Modification time (Unix timestamp) |
| `permissions` | `int` | File permissions (Unix mode bits, 0 on Windows) |

---

## VerifyResult

```python
class VerifyResult
```

Result of verifying a snapshot against the filesystem.

### Properties

| Property | Type | Description |
|---|---|---|
| `ok` | `bool` | `True` if no changes detected |
| `added` | `list[VerifyChange]` | Files on disk but not in snapshot |
| `removed` | `list[VerifyChange]` | Files in snapshot but not on disk |
| `modified` | `list[VerifyChange]` | Files with different hash or size |
| `errors` | `list[str]` | Files that could not be verified |

---

## VerifyChange

```python
class VerifyChange
```

A change detected during verification.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Relative path |
| `change_type` | `str` | `"added"`, `"removed"`, or `"modified"` |
| `expected_hash` | `str \| None` | Hash from snapshot |
| `actual_hash` | `str \| None` | Hash from current filesystem |
| `expected_size` | `int \| None` | Size from snapshot |
| `actual_size` | `int \| None` | Size from current filesystem |
