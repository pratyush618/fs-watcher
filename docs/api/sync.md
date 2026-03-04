# Sync API

## sync()

```python
def sync(
    source: str | PathLike[str],
    target: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    delete_extra: bool = False,
    skip_hidden: bool = False,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    dry_run: bool = False,
    preserve_metadata: bool = True,
    max_workers: int | None = None,
    progress_callback: Callable[[SyncProgress], None] | None = None,
) -> SyncResult
```

Synchronize source directory to target directory. Only copies files that are new or modified.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `source` | `str \| PathLike[str]` | *required* | Source directory |
| `target` | `str \| PathLike[str]` | *required* | Target directory (created if missing) |
| `algorithm` | `Literal["sha256", "blake3"]` | `"blake3"` | Hash algorithm for change detection |
| `delete_extra` | `bool` | `False` | Delete files in target not present in source |
| `skip_hidden` | `bool` | `False` | Skip hidden files |
| `glob_pattern` | `str \| None` | `None` | Only sync files matching this glob |
| `max_depth` | `int \| None` | `None` | Maximum recursion depth |
| `dry_run` | `bool` | `False` | Preview changes without writing |
| `preserve_metadata` | `bool` | `True` | Preserve file permissions |
| `max_workers` | `int \| None` | `None` | Max parallel threads |
| `progress_callback` | `Callable[[SyncProgress], None] \| None` | `None` | Progress callback |

### Returns

A [`SyncResult`](#syncresult) object.

### Raises

- `SyncError` — If the source is not a directory.

### Example

```python
result = pyfs_watcher.sync("/source", "/backup", delete_extra=True)
print(f"Copied: {len(result.copied)}, Deleted: {len(result.deleted)}")
```

---

## SyncResult

```python
class SyncResult
```

Result of a sync operation.

### Properties

| Property | Type | Description |
|---|---|---|
| `copied` | `list[str]` | Relative paths of copied files |
| `deleted` | `list[str]` | Relative paths of deleted files |
| `skipped` | `list[str]` | Relative paths of unchanged files |
| `total_bytes_transferred` | `int` | Total bytes copied |
| `errors` | `list[SyncFileError]` | Per-file errors |

---

## SyncProgress

```python
class SyncProgress
```

Progress snapshot during sync.

### Properties

| Property | Type | Description |
|---|---|---|
| `current_file` | `str` | File being processed |
| `files_completed` | `int` | Files completed so far |
| `total_files` | `int` | Total files to process |
| `bytes_transferred` | `int` | Bytes transferred so far |
| `stage` | `str` | Current stage (`"walking"`, `"comparing"`, `"syncing"`) |

---

## SyncFileError

```python
class SyncFileError
```

A per-file error during sync.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Relative path of the failed file |
| `message` | `str` | Error message |
