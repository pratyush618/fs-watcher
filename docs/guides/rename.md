# Rename

Batch rename files using regex patterns with dry-run preview and undo support. Safe by default — `dry_run=True` means no files are changed until you explicitly opt in.

## Basic Usage

```python
import pyfs_watcher

# Preview renames (dry_run=True by default)
result = pyfs_watcher.bulk_rename("/photos", r"IMG_(\d+)", r"photo_\1")
for entry in result.renamed:
    print(f"  {entry.old_name} -> {entry.new_name}")
```

---

## Apply Renames

Set `dry_run=False` to actually rename files:

```python
result = pyfs_watcher.bulk_rename(
    "/photos", r"IMG_(\d+)", r"photo_\1", dry_run=False
)
print(f"Renamed {len(result.renamed)} files")
```

---

## Undo

If you need to revert, call `undo()` on the result:

```python
result = pyfs_watcher.bulk_rename("/photos", r"old_", "new_", dry_run=False)

# Oops, undo!
errors = result.undo()
if errors:
    for err in errors:
        print(f"Failed to undo: {err.path} — {err.message}")
```

!!! note
    `undo()` only works on non-dry-run results and can only be called once.

---

## Recursive Rename

By default, only files in the top-level directory are renamed. Use `recursive=True` to include subdirectories:

```python
result = pyfs_watcher.bulk_rename(
    "/data", r"\.bak$", ".backup", recursive=True, dry_run=False
)
```

---

## Filter by Glob

Only rename files matching a glob pattern:

```python
# Only rename .jpg files
result = pyfs_watcher.bulk_rename(
    "/photos", r"IMG_", "photo_", glob_pattern="*.jpg", dry_run=False
)
```

---

## Include Directories

By default, only files are renamed. Use `include_dirs=True` to also rename directories:

```python
result = pyfs_watcher.bulk_rename(
    "/data", r"old_", "new_", include_dirs=True, dry_run=False
)
```

When directories are included, they are processed bottom-up (deepest first) to avoid path invalidation.

---

## Regex Patterns

The `pattern` and `replacement` use Rust's `regex` syntax, which supports capture groups:

```python
# Swap first and last name
pyfs_watcher.bulk_rename("/docs", r"(\w+)_(\w+)", r"\2_\1", dry_run=False)

# Add prefix
pyfs_watcher.bulk_rename("/data", r"^", "backup_", dry_run=False)

# Change extension
pyfs_watcher.bulk_rename("/data", r"\.txt$", ".md", dry_run=False)
```

---

## RenameResult Properties

| Property | Type | Description |
|---|---|---|
| `renamed` | `list[RenameEntry]` | Successful renames (old/new paths and names) |
| `skipped` | `int` | Files that didn't match the pattern |
| `errors` | `list[RenameFileError]` | Per-file errors |
| `dry_run` | `bool` | Whether this was a preview |

---

## Error Handling

```python
try:
    result = pyfs_watcher.bulk_rename("/data", r"[invalid", "replacement")
except pyfs_watcher.RenameError as e:
    print(f"Rename failed: {e}")
```
