# Rename API

## bulk_rename()

```python
def bulk_rename(
    path: str | PathLike[str],
    pattern: str,
    replacement: str,
    *,
    recursive: bool = False,
    skip_hidden: bool = True,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    dry_run: bool = True,
    include_dirs: bool = False,
) -> RenameResult
```

Rename files matching a regex pattern.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Directory containing files to rename |
| `pattern` | `str` | *required* | Regex pattern to match against filenames |
| `replacement` | `str` | *required* | Replacement string (supports capture groups like `\1`) |
| `recursive` | `bool` | `False` | Include files in subdirectories |
| `skip_hidden` | `bool` | `True` | Skip hidden files |
| `glob_pattern` | `str \| None` | `None` | Only rename files matching this glob |
| `max_depth` | `int \| None` | `None` | Maximum recursion depth (when recursive) |
| `dry_run` | `bool` | `True` | Preview renames without applying them |
| `include_dirs` | `bool` | `False` | Also rename directories |

### Returns

A [`RenameResult`](#renameresult) object.

### Raises

- `RenameError` — If the path is not a directory or the regex is invalid.

### Example

```python
# Preview
result = pyfs_watcher.bulk_rename("/photos", r"IMG_(\d+)", r"photo_\1")
for entry in result.renamed:
    print(f"{entry.old_name} -> {entry.new_name}")

# Apply
result = pyfs_watcher.bulk_rename("/photos", r"IMG_(\d+)", r"photo_\1", dry_run=False)
```

---

## RenameResult

```python
class RenameResult
```

Result of a bulk rename operation.

### Properties

| Property | Type | Description |
|---|---|---|
| `renamed` | `list[RenameEntry]` | Successful renames |
| `skipped` | `int` | Files that didn't match the pattern |
| `errors` | `list[RenameFileError]` | Per-file errors |
| `dry_run` | `bool` | Whether this was a preview |

### Methods

- `undo() -> list[RenameFileError]` — Reverse all renames. Only available on non-dry-run results. Can only be called once.

### Protocols

- `__repr__() -> str`

---

## RenameEntry

```python
class RenameEntry
```

A single rename operation (old -> new).

### Properties

| Property | Type | Description |
|---|---|---|
| `old_path` | `str` | Original absolute path |
| `new_path` | `str` | New absolute path |
| `old_name` | `str` | Original filename |
| `new_name` | `str` | New filename |

### Protocols

- `__repr__() -> str`

---

## RenameFileError

```python
class RenameFileError
```

A per-file error during rename.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Path of the file that failed |
| `message` | `str` | Error message |

### Protocols

- `__repr__() -> str`
