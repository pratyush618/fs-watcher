# Sync

Incrementally synchronize a source directory to a target directory. Only copies files that are new or modified, skipping unchanged files.

## Basic Usage

```python
import pyfs_watcher

result = pyfs_watcher.sync("/source", "/backup")
print(f"Copied:  {len(result.copied)}")
print(f"Skipped: {len(result.skipped)}")
print(f"Errors:  {len(result.errors)}")
```

---

## How It Works

1. **Walk** both directories and build file maps
2. **Diff** — compare by size, then hash same-size files
3. **Execute** — copy new/modified files, optionally delete extras

Only files that differ are copied, making repeated syncs very fast.

---

## Dry Run

Preview what would be synced without writing anything:

```python
result = pyfs_watcher.sync("/source", "/backup", dry_run=True)
print("Would copy:")
for path in result.copied:
    print(f"  {path}")
print("Would delete:")
for path in result.deleted:
    print(f"  {path}")
```

---

## Delete Extra Files

Remove files from the target that don't exist in the source:

```python
result = pyfs_watcher.sync("/source", "/backup", delete_extra=True)
for path in result.deleted:
    print(f"Deleted: {path}")
```

By default, extra files are left untouched.

---

## Progress Callback

```python
def on_progress(p):
    print(f"[{p.stage}] {p.files_completed}/{p.total_files} - {p.current_file}")

result = pyfs_watcher.sync("/source", "/backup", progress_callback=on_progress)
```

Stages: `"walking"`, `"comparing"`, `"syncing"`.

---

## Filtering

```python
# Skip hidden files
result = pyfs_watcher.sync("/source", "/backup", skip_hidden=True)

# Only sync Python files
result = pyfs_watcher.sync("/source", "/backup", glob_pattern="*.py")

# Limit depth
result = pyfs_watcher.sync("/source", "/backup", max_depth=3)
```

---

## Error Tolerance

Sync is error-tolerant — if one file fails to copy, the remaining files are still processed. Errors are collected in `result.errors`:

```python
result = pyfs_watcher.sync("/source", "/backup")
for err in result.errors:
    print(f"Failed: {err.path} — {err.message}")
```

---

## SyncResult Properties

| Property | Type | Description |
|---|---|---|
| `copied` | `list[str]` | Relative paths of copied files |
| `deleted` | `list[str]` | Relative paths of deleted files |
| `skipped` | `list[str]` | Relative paths of unchanged files |
| `total_bytes_transferred` | `int` | Total bytes copied |
| `errors` | `list[SyncFileError]` | Per-file errors |

---

## Error Handling

```python
try:
    result = pyfs_watcher.sync("/source", "/backup")
except pyfs_watcher.SyncError as e:
    print(f"Sync failed: {e}")
```
