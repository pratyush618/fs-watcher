# fs_watcher

Rust-powered filesystem toolkit for Python. Fast recursive directory listing, parallel file hashing, bulk copy/move with progress, cross-platform file watching, and file deduplication.

## Install

```bash
pip install fs_watcher
```

**From source:**

```bash
pip install maturin
maturin develop
```

## Usage

### Walk directories (parallel, faster than os.walk)

```python
import fs_watcher

# Streaming iterator
for entry in fs_watcher.walk("/data", file_type="file", glob_pattern="*.py"):
    print(entry.path, entry.file_size)

# Bulk collect (faster when you need all results)
entries = fs_watcher.walk_collect("/data", max_depth=3, sort=True, skip_hidden=True)
```

### Hash files (parallel SHA256/BLAKE3)

```python
# Single file
result = fs_watcher.hash_file("large.iso", algorithm="blake3")
print(result.hash_hex)

# Parallel batch hashing
results = fs_watcher.hash_files(paths, algorithm="blake3", callback=lambda r: print(r.path))
```

### Copy/move with progress

```python
def on_progress(p):
    pct = p.bytes_copied / p.total_bytes * 100
    print(f"{pct:.0f}% - {p.current_file}")

fs_watcher.copy_files(sources, "/dest", progress_callback=on_progress)
fs_watcher.move_files(sources, "/dest")  # rename if same fs, copy+delete otherwise
```

### Watch for file changes

```python
# Sync
with fs_watcher.FileWatcher("/data", debounce_ms=500, ignore_patterns=["*.tmp"]) as w:
    for changes in w:
        for c in changes:
            print(c.path, c.change_type)  # "created", "modified", "deleted"

# Async
async for changes in fs_watcher.async_watch("/data"):
    for c in changes:
        print(c.path, c.change_type)
```

### Find duplicate files

```python
groups = fs_watcher.find_duplicates(
    ["/photos", "/backup"],
    min_size=1024,
    progress_callback=lambda stage, done, total: print(f"{stage}: {done}/{total}"),
)
for g in groups:
    print(f"{g.file_size}B x {len(g.paths)} copies = {g.wasted_bytes}B wasted")
    for path in g.paths:
        print(f"  {path}")
```

## API

All functions raise typed exceptions inheriting from `FsWatcherError`:

- `WalkError` - directory walk failures
- `HashError` - hashing failures
- `CopyError` - copy/move failures
- `WatchError` - file watching failures

Standard `FileNotFoundError` and `PermissionError` are raised for I/O errors.

## Development

```bash
# Setup
uv venv && source .venv/bin/activate
uv pip install maturin pytest pytest-asyncio pytest-timeout

# Build
maturin develop

# Test
cargo test        # Rust tests
pytest tests/     # Python tests

# Benchmark
python benches/bench_walk.py
python benches/bench_hash.py
```

## Tech

- Rust + PyO3 for Python bindings
- jwalk for parallel directory traversal
- BLAKE3/SHA-256 for hashing with rayon parallelism
- notify + debouncer for cross-platform file watching
- Staged dedup pipeline: size grouping -> partial hash -> full hash
