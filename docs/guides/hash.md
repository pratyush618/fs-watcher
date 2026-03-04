# Hash

Hash files using BLAKE3 or SHA-256 with automatic memory-mapped I/O and parallel batch processing.

## Algorithms

| Algorithm | Speed | Output | Use Case |
|---|---|---|---|
| `blake3` (default) | ~10x faster than SHA-256 | 64-char hex | Deduplication, integrity checks, caching |
| `sha256` | Standard speed | 64-char hex | Interoperability, compliance, verification |

BLAKE3 is the default because it is cryptographically secure **and** extremely fast — it leverages SIMD and multithreading internally.

---

## Single File Hashing

```python
import pyfs_watcher

result = pyfs_watcher.hash_file("large.iso")
print(result.hash_hex)       # "d74981efa70a0c880b..."
print(result.algorithm)      # "blake3"
print(result.file_size)      # 4294967296
```

### Choosing an algorithm

```python
# BLAKE3 (default, fastest)
result = pyfs_watcher.hash_file("data.bin")

# SHA-256 (for compatibility)
result = pyfs_watcher.hash_file("data.bin", algorithm="sha256")
```

### Custom chunk size

```python
# Smaller chunks for memory-constrained environments
result = pyfs_watcher.hash_file("data.bin", chunk_size=65536)
```

The default chunk size is 1 MB (`1_048_576` bytes). Files larger than 4 MB automatically use memory-mapped I/O regardless of chunk size.

---

## Parallel Batch Hashing

Hash many files at once using all available CPU cores:

```python
paths = ["file1.bin", "file2.bin", "file3.bin"]
results = pyfs_watcher.hash_files(paths, algorithm="blake3")

for r in results:
    print(f"{r.path}: {r.hash_hex}")
```

!!! note
    The order of results may differ from the input order because files are processed in parallel.

### Progress callback

```python
def on_hash(result):
    print(f"Hashed: {result.path} ({result.file_size} bytes)")

results = pyfs_watcher.hash_files(paths, callback=on_hash)
```

### Limiting workers

```python
# Use at most 4 threads
results = pyfs_watcher.hash_files(paths, max_workers=4)
```

By default, `max_workers=None` uses all available CPU cores via Rayon.

---

## Memory-Mapped I/O

For files larger than 4 MB, pyfs-watcher automatically uses memory-mapped I/O (`mmap`) instead of buffered reads. This allows the OS to manage page caching efficiently, which is particularly beneficial when hashing large files.

You don't need to configure this — it happens automatically.

---

## HashResult as Dict Key

`HashResult` supports equality comparison and hashing based on the digest and algorithm, so you can use instances in sets and as dictionary keys:

```python
results = pyfs_watcher.hash_files(paths)

# Group files by hash
by_hash = {}
for r in results:
    by_hash.setdefault(r.hash_hex, []).append(r.path)

# Find duplicates
duplicates = {h: paths for h, paths in by_hash.items() if len(paths) > 1}
```

!!! tip
    For full duplicate detection with a three-stage pipeline, use [`find_duplicates()`](../api/dedup.md) instead.

---

## Error Handling

```python
try:
    result = pyfs_watcher.hash_file("/nonexistent")
except FileNotFoundError:
    print("File does not exist")
except pyfs_watcher.HashError as e:
    print(f"Hashing failed: {e}")
```

When using `hash_files()`, individual file errors are included in the raised `HashError` rather than silently skipped.

---

## Performance Tips

- Use BLAKE3 (the default) for maximum throughput.
- Use `hash_files()` for batch operations — parallel processing saturates disk I/O better than sequential calls.
- The 4 MB mmap threshold is tuned for modern SSDs. No configuration needed.
- Limit `max_workers` if you want to leave CPU headroom for other tasks.
