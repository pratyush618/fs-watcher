# Disk Usage

Calculate directory sizes using parallel traversal. Returns total size, file/directory counts, and a per-child size breakdown sorted by largest first.

## Basic Usage

```python
import pyfs_watcher

usage = pyfs_watcher.disk_usage("/data")
print(f"Total: {usage.total_size:,} bytes")
print(f"Files: {usage.total_files}")
print(f"Dirs:  {usage.total_dirs}")
```

---

## Per-Child Breakdown

The `children` list shows size breakdown by top-level entries, sorted by size descending:

```python
usage = pyfs_watcher.disk_usage("/home/user")
for child in usage.children:
    mb = child.size / 1_048_576
    print(f"  {child.path:30s} {mb:8.1f} MB  ({child.file_count} files)")
```

---

## Filtering

```python
# Skip hidden files and directories
usage = pyfs_watcher.disk_usage("/data", skip_hidden=True)

# Only count certain file types
usage = pyfs_watcher.disk_usage("/data", glob_pattern="*.log")

# Limit traversal depth
usage = pyfs_watcher.disk_usage("/data", max_depth=2)
```

---

## DiskUsage Properties

| Property | Type | Description |
|---|---|---|
| `total_size` | `int` | Total bytes across all files |
| `total_files` | `int` | Total number of files |
| `total_dirs` | `int` | Total number of directories |
| `children` | `list[DiskUsageEntry]` | Per-child breakdown (sorted by size desc) |

## DiskUsageEntry Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Name of the top-level child |
| `size` | `int` | Total bytes in this subtree |
| `file_count` | `int` | Number of files in this subtree |
| `dir_count` | `int` | Number of subdirectories |
| `is_dir` | `bool` | Whether this entry is a directory |

---

## Recipes

### Find the largest subdirectories

```python
usage = pyfs_watcher.disk_usage("/data")
for child in usage.children[:10]:
    pct = child.size / usage.total_size * 100
    print(f"  {child.path}: {child.size / 1_048_576:.1f} MB ({pct:.1f}%)")
```

### Human-readable sizes

```python
def human_size(n):
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if n < 1024:
            return f"{n:.1f} {unit}"
        n /= 1024

usage = pyfs_watcher.disk_usage("/data")
print(f"Total: {human_size(usage.total_size)}")
```

---

## Error Handling

```python
try:
    usage = pyfs_watcher.disk_usage("/data")
except pyfs_watcher.DiskUsageError as e:
    print(f"Disk usage failed: {e}")
```
