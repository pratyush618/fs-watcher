# Hash API

## hash_file()

```python
def hash_file(
    path: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    chunk_size: int = 1_048_576,
) -> HashResult
```

Hash a single file.

Uses memory-mapped I/O for files larger than 4 MB and buffered reads for smaller files.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Path to the file to hash |
| `algorithm` | `Literal["sha256", "blake3"]` | `"blake3"` | Hash algorithm — `"blake3"` (~10x faster) or `"sha256"` |
| `chunk_size` | `int` | `1_048_576` | Read buffer size in bytes for buffered hashing |

### Returns

A [`HashResult`](#hashresult) with the hex digest and file metadata.

### Raises

- `HashError` — If hashing fails.
- `FileNotFoundError` — If the file does not exist.

### Example

```python
result = pyfs_watcher.hash_file("large.iso", algorithm="blake3")
print(result.hash_hex)  # "d74981efa70a0c880b..."
```

---

## hash_files()

```python
def hash_files(
    paths: Sequence[str | PathLike[str]],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    chunk_size: int = 1_048_576,
    max_workers: int | None = None,
    callback: Callable[[HashResult], None] | None = None,
) -> list[HashResult]
```

Hash multiple files in parallel using a Rayon thread pool.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `paths` | `Sequence[str \| PathLike[str]]` | *required* | Sequence of file paths to hash |
| `algorithm` | `Literal["sha256", "blake3"]` | `"blake3"` | Hash algorithm |
| `chunk_size` | `int` | `1_048_576` | Read buffer size in bytes |
| `max_workers` | `int \| None` | `None` | Max parallel threads (`None` = all cores) |
| `callback` | `Callable[[HashResult], None] \| None` | `None` | Called with each `HashResult` as it completes |

### Returns

A `list` of [`HashResult`](#hashresult) objects. Order may differ from input.

### Raises

- `HashError` — If the thread pool cannot be created.

### Example

```python
results = pyfs_watcher.hash_files(
    ["file1.bin", "file2.bin", "file3.bin"],
    algorithm="blake3",
    callback=lambda r: print(f"{r.path}: {r.hash_hex}"),
)
```

---

## HashResult

```python
class HashResult
```

Result of hashing a single file. Supports equality comparison and hashing based on the hex digest and algorithm, so instances can be used in sets and as dict keys.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Absolute path of the hashed file |
| `hash_hex` | `str` | Hex-encoded hash digest |
| `algorithm` | `str` | Algorithm used (`"sha256"` or `"blake3"`) |
| `file_size` | `int` | Size of the file in bytes |

### Protocols

- `__eq__(other) -> bool` — Compare by digest and algorithm
- `__hash__() -> int` — Hash by digest and algorithm
- `__repr__() -> str`

### Example

```python
result = pyfs_watcher.hash_file("data.bin")
print(result.hash_hex)    # "a1b2c3d4..."
print(result.algorithm)   # "blake3"
print(result.file_size)   # 1048576

# Use as dict key
cache = {result: "processed"}
```
