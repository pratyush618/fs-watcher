# Dedup API

## find_duplicates()

```python
def find_duplicates(
    paths: Sequence[str | PathLike[str]],
    *,
    recursive: bool = True,
    min_size: int = 1,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    partial_hash_size: int = 4096,
    max_workers: int | None = None,
    progress_callback: Callable[[str, int, int], None] | None = None,
) -> list[DuplicateGroup]
```

Find duplicate files using a staged pipeline.

Efficiently identifies duplicates in three stages, each eliminating non-duplicates before the next expensive step:

1. **Size grouping** — files with unique sizes are eliminated.
2. **Partial hash** — first and last `partial_hash_size` bytes are compared.
3. **Full hash** — remaining candidates are fully hashed to confirm.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `paths` | `Sequence[str \| PathLike[str]]` | *required* | Directories or files to scan |
| `recursive` | `bool` | `True` | Whether to recurse into subdirectories |
| `min_size` | `int` | `1` | Ignore files smaller than this many bytes |
| `algorithm` | `Literal["sha256", "blake3"]` | `"blake3"` | Hash algorithm |
| `partial_hash_size` | `int` | `4096` | Bytes to read from head and tail for partial hashing |
| `max_workers` | `int \| None` | `None` | Max parallel threads (`None` = all cores) |
| `progress_callback` | `Callable[[str, int, int], None] \| None` | `None` | `(stage, processed, total)` callback |

### Progress Callback

The callback receives three arguments:

| Argument | Type | Description |
|---|---|---|
| `stage` | `str` | `"collecting"`, `"partial_hash"`, or `"full_hash"` |
| `processed` | `int` | Items processed so far in this stage |
| `total` | `int` | Total items in this stage |

### Returns

A `list` of [`DuplicateGroup`](#duplicategroup) objects sorted by `wasted_bytes` descending.

### Raises

- `HashError` — If hashing fails for any file.

### Example

```python
groups = pyfs_watcher.find_duplicates(
    ["/photos", "/backup"],
    min_size=1024,
    progress_callback=lambda stage, done, total: print(f"{stage}: {done}/{total}"),
)
for g in groups:
    print(f"{g.file_size}B x {len(g.paths)} copies = {g.wasted_bytes}B wasted")
    for path in g.paths:
        print(f"  {path}")
```

---

## DuplicateGroup

```python
class DuplicateGroup
```

A group of files that share identical content. Returned by `find_duplicates()`. Groups are sorted by `wasted_bytes` in descending order.

### Properties

| Property | Type | Description |
|---|---|---|
| `hash_hex` | `str` | Hex-encoded hash digest shared by all files |
| `file_size` | `int` | Size of each file in bytes |
| `paths` | `list[str]` | Absolute paths of the duplicate files |
| `wasted_bytes` | `int` | `file_size * (count - 1)` |

### Protocols

- `__len__() -> int` — Number of duplicate files in this group
- `__repr__() -> str`

### Example

```python
groups = pyfs_watcher.find_duplicates(["/data"])
for g in groups:
    print(f"Hash: {g.hash_hex[:16]}...")
    print(f"Size: {g.file_size:,} bytes each")
    print(f"Copies: {len(g)}")
    print(f"Wasted: {g.wasted_bytes:,} bytes")
    for p in g.paths:
        print(f"  {p}")
```
