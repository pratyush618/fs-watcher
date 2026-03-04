# Snapshot

Capture a directory's file hashes and metadata as a snapshot, then verify later to detect changes. Useful for integrity monitoring, deployment validation, and change detection.

## Basic Usage

```python
import pyfs_watcher

# Take a snapshot
snap = pyfs_watcher.snapshot("/important_data")
print(f"{snap.total_files} files, {snap.total_size} bytes")

# Save to JSON
snap.save("baseline.json")
```

---

## Verify Integrity

Compare the current state of a directory against a snapshot:

```python
# From a Snapshot object
result = pyfs_watcher.verify(snap)

# Or from a saved JSON file
result = pyfs_watcher.verify("baseline.json")

if result.ok:
    print("All files unchanged")
else:
    for c in result.added:
        print(f"  Added:    {c.path}")
    for c in result.removed:
        print(f"  Removed:  {c.path}")
    for c in result.modified:
        print(f"  Modified: {c.path}")
```

---

## Save and Load

Snapshots are saved as human-readable JSON:

```python
# Save
snap = pyfs_watcher.snapshot("/data")
snap.save("/backups/snapshot_2024.json")

# Load
loaded = pyfs_watcher.Snapshot.load("/backups/snapshot_2024.json")
print(loaded.total_files, loaded.created_at)
```

The JSON format includes the root path, algorithm, timestamp, and an array of entries with path, hash, size, mtime, and permissions.

---

## Snapshot Entry Properties

Each entry captures:

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Relative path from snapshot root |
| `hash_hex` | `str` | Hex-encoded hash digest |
| `file_size` | `int` | Size in bytes |
| `mtime` | `float` | Modification time (Unix timestamp) |
| `permissions` | `int` | File permissions (Unix mode bits, 0 on Windows) |

---

## Filtering

```python
# Skip hidden files
snap = pyfs_watcher.snapshot("/data", skip_hidden=True)

# Only snapshot Python files
snap = pyfs_watcher.snapshot("/data", glob_pattern="*.py")

# Limit depth
snap = pyfs_watcher.snapshot("/data", max_depth=3)
```

---

## Progress Callback

```python
def on_progress(stage, processed, total):
    print(f"[{stage}] {processed}/{total}")

snap = pyfs_watcher.snapshot("/data", progress_callback=on_progress)
```

Stages: `"walking"`, `"hashing"`, `"done"`.

---

## Recipes

### Scheduled integrity check

```python
import pyfs_watcher

result = pyfs_watcher.verify("baseline.json")
if not result.ok:
    changes = len(result.added) + len(result.removed) + len(result.modified)
    print(f"WARNING: {changes} file(s) changed since baseline!")
```

### Update baseline after verified changes

```python
# Take a new snapshot after approved changes
snap = pyfs_watcher.snapshot("/data")
snap.save("baseline.json")
```

---

## Error Handling

```python
try:
    snap = pyfs_watcher.snapshot("/data")
except pyfs_watcher.SnapshotError as e:
    print(f"Snapshot failed: {e}")
```
