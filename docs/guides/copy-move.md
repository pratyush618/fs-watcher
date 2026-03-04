# Copy / Move

Bulk copy and move files with real-time progress tracking and smart cross-device handling.

## Copying Files

```python
import pyfs_watcher

copied = pyfs_watcher.copy_files(
    ["data/file1.bin", "data/file2.bin"],
    "/backup",
)
print(f"Copied to: {copied}")
```

The function returns a list of destination paths for the copied files.

---

## Progress Tracking

Monitor copy progress with a callback:

```python
def on_progress(p):
    pct = p.bytes_copied / p.total_bytes * 100 if p.total_bytes else 0
    print(
        f"[{pct:5.1f}%] {p.files_completed}/{p.total_files} files | "
        f"Current: {p.current_file}"
    )

pyfs_watcher.copy_files(
    sources,
    "/backup",
    progress_callback=on_progress,
)
```

### CopyProgress properties

| Property | Type | Description |
|---|---|---|
| `src` | `str` | Source base path |
| `dst` | `str` | Destination base path |
| `bytes_copied` | `int` | Total bytes copied so far |
| `total_bytes` | `int` | Total bytes to copy |
| `files_completed` | `int` | Files fully copied |
| `total_files` | `int` | Total file count |
| `current_file` | `str` | File currently being copied |

### Callback interval

By default, the callback fires at most every 100ms. Adjust this with `callback_interval_ms`:

```python
pyfs_watcher.copy_files(
    sources, "/backup",
    progress_callback=on_progress,
    callback_interval_ms=50,  # More frequent updates
)
```

---

## Overwriting Existing Files

By default, existing files at the destination are **not** overwritten:

```python
# This will raise CopyError if /backup/file1.bin already exists
pyfs_watcher.copy_files(["file1.bin"], "/backup")

# Allow overwriting
pyfs_watcher.copy_files(["file1.bin"], "/backup", overwrite=True)
```

---

## Metadata Preservation

File timestamps and permissions are preserved by default:

```python
# Disable metadata preservation
pyfs_watcher.copy_files(sources, "/backup", preserve_metadata=False)
```

---

## Directory Copying

Directories are copied recursively, preserving the internal structure:

```python
pyfs_watcher.copy_files(["project/src"], "/backup")
# Result: /backup/src/... with all contents
```

---

## Moving Files

```python
pyfs_watcher.move_files(["old/data.csv", "old/report.pdf"], "/archive")
```

### Smart cross-device handling

`move_files()` attempts a fast `rename()` first. If the source and destination are on different filesystems (EXDEV error), it automatically falls back to copy-then-delete.

The progress callback is only invoked during the fallback copy — renames are instantaneous.

```python
# Move across filesystems with progress tracking
pyfs_watcher.move_files(
    ["data.bin"],
    "/mnt/external/backup",
    progress_callback=lambda p: print(f"{p.bytes_copied / p.total_bytes:.0%}"),
)
```

---

## Error Handling

```python
try:
    pyfs_watcher.copy_files(["/nonexistent"], "/backup")
except FileNotFoundError:
    print("Source file does not exist")
except pyfs_watcher.CopyError as e:
    print(f"Copy failed: {e}")
```

Common error scenarios:

- Source file does not exist → `FileNotFoundError`
- Destination file exists and `overwrite=False` → `CopyError`
- Permission denied → `PermissionError`
- Disk full → `CopyError`

---

## Performance Tips

- Use `copy_files()` with multiple sources in a single call rather than calling it repeatedly for individual files.
- The `callback_interval_ms` parameter prevents excessive callback overhead — the default 100ms is suitable for most UIs.
- For same-filesystem operations, `move_files()` is nearly instantaneous (just a rename).
