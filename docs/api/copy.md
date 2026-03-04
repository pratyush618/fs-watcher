# Copy / Move API

## copy_files()

```python
def copy_files(
    sources: Sequence[str | PathLike[str]],
    destination: str | PathLike[str],
    *,
    overwrite: bool = False,
    preserve_metadata: bool = True,
    progress_callback: Callable[[CopyProgress], None] | None = None,
    callback_interval_ms: int = 100,
) -> list[str]
```

Copy files and directories to a destination.

Performs chunked I/O with optional progress reporting. Directories are copied recursively.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `sources` | `Sequence[str \| PathLike[str]]` | *required* | Paths of files or directories to copy |
| `destination` | `str \| PathLike[str]` | *required* | Target directory to copy into |
| `overwrite` | `bool` | `False` | Whether to overwrite existing files |
| `preserve_metadata` | `bool` | `True` | Preserve file timestamps and permissions |
| `progress_callback` | `Callable[[CopyProgress], None] \| None` | `None` | Called with progress snapshots at regular intervals |
| `callback_interval_ms` | `int` | `100` | Minimum milliseconds between progress callbacks |

### Returns

A `list[str]` of destination paths for the copied files.

### Raises

- `CopyError` — If a copy operation fails.
- `FileNotFoundError` — If a source path does not exist.

### Example

```python
def on_progress(p):
    pct = p.bytes_copied / p.total_bytes * 100
    print(f"{pct:.0f}% - {p.current_file}")

pyfs_watcher.copy_files(
    ["data/file1.bin", "data/file2.bin"],
    "/backup",
    progress_callback=on_progress,
)
```

---

## move_files()

```python
def move_files(
    sources: Sequence[str | PathLike[str]],
    destination: str | PathLike[str],
    *,
    overwrite: bool = False,
    progress_callback: Callable[[CopyProgress], None] | None = None,
    callback_interval_ms: int = 100,
) -> list[str]
```

Move files and directories to a destination.

Attempts a fast rename first. Falls back to copy-then-delete when the source and destination are on different filesystems.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `sources` | `Sequence[str \| PathLike[str]]` | *required* | Paths of files or directories to move |
| `destination` | `str \| PathLike[str]` | *required* | Target directory to move into |
| `overwrite` | `bool` | `False` | Whether to overwrite existing files |
| `progress_callback` | `Callable[[CopyProgress], None] \| None` | `None` | Called with progress snapshots (only during fallback copy) |
| `callback_interval_ms` | `int` | `100` | Minimum milliseconds between progress callbacks |

### Returns

A `list[str]` of destination paths for the moved files.

### Raises

- `CopyError` — If a move operation fails.
- `FileNotFoundError` — If a source path does not exist.

### Example

```python
pyfs_watcher.move_files(["old/data.csv", "old/report.pdf"], "/archive")
```

---

## CopyProgress

```python
class CopyProgress
```

Snapshot of progress during a copy or move operation. Passed to the `progress_callback` at regular intervals.

### Properties

| Property | Type | Description |
|---|---|---|
| `src` | `str` | Source base path |
| `dst` | `str` | Destination base path |
| `bytes_copied` | `int` | Total bytes copied so far across all files |
| `total_bytes` | `int` | Total bytes to copy across all files |
| `files_completed` | `int` | Number of files fully copied so far |
| `total_files` | `int` | Total number of files to copy |
| `current_file` | `str` | Path of the file currently being copied |

### Example

```python
def on_progress(p):
    pct = p.bytes_copied / p.total_bytes * 100 if p.total_bytes else 0
    print(
        f"[{pct:5.1f}%] {p.files_completed}/{p.total_files} | "
        f"{p.current_file}"
    )

pyfs_watcher.copy_files(sources, "/dest", progress_callback=on_progress)
```
