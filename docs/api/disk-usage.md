# Disk Usage API

## disk_usage()

```python
def disk_usage(
    path: str | PathLike[str],
    *,
    max_depth: int | None = None,
    skip_hidden: bool = False,
    follow_symlinks: bool = False,
    glob_pattern: str | None = None,
    max_workers: int | None = None,
) -> DiskUsage
```

Calculate disk usage for a directory using parallel traversal.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Directory to analyze |
| `max_depth` | `int \| None` | `None` | Maximum recursion depth |
| `skip_hidden` | `bool` | `False` | Skip hidden files |
| `follow_symlinks` | `bool` | `False` | Follow symbolic links |
| `glob_pattern` | `str \| None` | `None` | Only count files matching this glob |
| `max_workers` | `int \| None` | `None` | Max parallel threads |

### Returns

A [`DiskUsage`](#diskusage) object.

### Raises

- `DiskUsageError` — If the path does not exist or is not a directory.

### Example

```python
usage = pyfs_watcher.disk_usage("/data")
print(f"Total: {usage.total_size:,} bytes in {usage.total_files} files")
for child in usage.children[:5]:
    print(f"  {child.path}: {child.size:,} bytes")
```

---

## DiskUsage

```python
class DiskUsage
```

Result of disk usage calculation.

### Properties

| Property | Type | Description |
|---|---|---|
| `total_size` | `int` | Total bytes across all files |
| `total_files` | `int` | Total number of files |
| `total_dirs` | `int` | Total number of directories |
| `children` | `list[DiskUsageEntry]` | Per-child breakdown (sorted by size desc) |

### Protocols

- `__repr__() -> str`

---

## DiskUsageEntry

```python
class DiskUsageEntry
```

A single entry in the disk usage breakdown.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Name of the top-level child |
| `size` | `int` | Total bytes in this subtree |
| `file_count` | `int` | Number of files |
| `dir_count` | `int` | Number of subdirectories |
| `is_dir` | `bool` | Whether this entry is a directory |

### Protocols

- `__repr__() -> str`
