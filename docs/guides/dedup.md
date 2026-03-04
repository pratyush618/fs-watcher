# Dedup

Find duplicate files efficiently using a three-stage pipeline that eliminates non-duplicates early, avoiding unnecessary I/O.

## How It Works

```mermaid
graph LR
    A[Collect Files] --> B[Group by Size]
    B --> C[Partial Hash<br/>first + last 4KB]
    C --> D[Full Hash]
    D --> E[Duplicate Groups]
```

Each stage filters out unique files before proceeding to the next, more expensive step:

1. **Size grouping** — Files with a unique size cannot be duplicates. Only size-matched groups continue.
2. **Partial hash** — The first and last `partial_hash_size` bytes (default 4KB) are hashed. Files with unique partial hashes are eliminated.
3. **Full hash** — Remaining candidates are fully hashed to confirm they are true duplicates.

This means that for a directory with 10,000 files where only 50 are duplicates, the full hash only runs on a small subset — not all 10,000.

---

## Basic Usage

```python
import pyfs_watcher

groups = pyfs_watcher.find_duplicates(["/photos", "/backup"])

for group in groups:
    print(f"\n{group.file_size:,} bytes x {len(group.paths)} copies "
          f"= {group.wasted_bytes:,} bytes wasted")
    for path in group.paths:
        print(f"  {path}")
```

---

## Multiple Directories

Scan across multiple directories to find duplicates that span locations:

```python
groups = pyfs_watcher.find_duplicates([
    "/home/user/Documents",
    "/home/user/Downloads",
    "/home/user/Desktop",
])
```

---

## Minimum File Size

Skip small files that aren't worth deduplicating:

```python
# Only files >= 1 KB
groups = pyfs_watcher.find_duplicates(["/data"], min_size=1024)

# Only files >= 1 MB
groups = pyfs_watcher.find_duplicates(["/data"], min_size=1_048_576)
```

The default `min_size=1` includes all non-empty files.

---

## Progress Tracking

The `progress_callback` receives three arguments: the stage name, the number of items processed, and the total:

```python
def on_progress(stage, processed, total):
    pct = processed / total * 100 if total else 0
    print(f"  [{stage}] {processed}/{total} ({pct:.0f}%)")

groups = pyfs_watcher.find_duplicates(
    ["/photos"],
    progress_callback=on_progress,
)
```

Stages reported:

| Stage | Description |
|---|---|
| `"collecting"` | Scanning directories and grouping by file size |
| `"partial_hash"` | Hashing first + last bytes of size-matched files |
| `"full_hash"` | Fully hashing remaining candidates |

---

## Algorithm and Tuning

```python
groups = pyfs_watcher.find_duplicates(
    ["/data"],
    algorithm="blake3",         # or "sha256"
    partial_hash_size=4096,     # Bytes from head + tail for partial hash
    max_workers=4,              # Limit parallel threads
)
```

- **`algorithm`** — BLAKE3 (default) is ~10x faster than SHA-256.
- **`partial_hash_size`** — Increasing this reduces false positives at the partial-hash stage but reads more data. The default 4096 bytes is a good balance.
- **`max_workers`** — Limits the Rayon thread pool size. `None` (default) uses all cores.

---

## DuplicateGroup Properties

| Property | Type | Description |
|---|---|---|
| `hash_hex` | `str` | Hex digest shared by all files in the group |
| `file_size` | `int` | Size of each file in bytes |
| `paths` | `list[str]` | Absolute paths of the duplicate files |
| `wasted_bytes` | `int` | `file_size * (count - 1)` |

Groups are returned sorted by `wasted_bytes` in descending order, so the biggest space savings appear first.

`len(group)` returns the number of duplicate files.

---

## Recipes

### Print a summary report

```python
groups = pyfs_watcher.find_duplicates(["/data"])

total_wasted = sum(g.wasted_bytes for g in groups)
total_groups = len(groups)
total_dupes = sum(len(g) for g in groups)

print(f"Found {total_dupes} duplicate files in {total_groups} groups")
print(f"Total wasted space: {total_wasted / 1_048_576:.1f} MB")
```

### Find the biggest duplicates

```python
# Groups are already sorted by wasted_bytes descending
top5 = groups[:5]
for g in top5:
    print(f"{g.wasted_bytes / 1_048_576:.1f} MB wasted — {len(g.paths)} copies")
```

---

## Error Handling

```python
try:
    groups = pyfs_watcher.find_duplicates(["/data"])
except pyfs_watcher.HashError as e:
    print(f"Dedup failed: {e}")
```

Files that cannot be read (permission denied, etc.) are skipped during the collection stage.
