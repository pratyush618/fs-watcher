# Diff

Compare two directories to find added, removed, modified, and optionally moved files. Uses hash-based content comparison by default.

## Basic Usage

```python
import pyfs_watcher

diff = pyfs_watcher.diff_dirs("/original", "/copy")
print(f"Added:     {len(diff.added)}")
print(f"Removed:   {len(diff.removed)}")
print(f"Modified:  {len(diff.modified)}")
print(f"Unchanged: {len(diff.unchanged)}")
```

---

## How It Works

1. Walk both directories and build file maps keyed by relative path
2. Files only in source → **removed**; only in target → **added**
3. Files in both with different sizes → **modified**
4. Same-size files → hash both and compare → **modified** or **unchanged**

---

## Move Detection

Enable `detect_moves=True` to identify files that were renamed or relocated:

```python
diff = pyfs_watcher.diff_dirs("/before", "/after", detect_moves=True)
for m in diff.moved:
    print(f"  {m.source_path} -> {m.target_path}")
```

Move detection works by hashing removed and added files, then matching them by hash (greedy 1:1). Detected moves are excluded from the added/removed lists.

---

## Skip Content Comparison

For a fast size-only comparison:

```python
diff = pyfs_watcher.diff_dirs("/src", "/dst", compare_content=False)
# Same-size files are assumed unchanged
```

---

## Filtering

```python
# Skip hidden files
diff = pyfs_watcher.diff_dirs("/src", "/dst", skip_hidden=True)

# Only compare Python files
diff = pyfs_watcher.diff_dirs("/src", "/dst", glob_pattern="*.py")

# Limit depth
diff = pyfs_watcher.diff_dirs("/src", "/dst", max_depth=3)
```

---

## Progress Callback

```python
def on_progress(stage):
    print(f"Stage: {stage}")

diff = pyfs_watcher.diff_dirs("/src", "/dst", progress_callback=on_progress)
```

Stages: `"walking_source"`, `"walking_target"`, `"comparing"`, `"detecting_moves"`.

---

## DirDiff Properties

| Property | Type | Description |
|---|---|---|
| `added` | `list[DiffEntry]` | Files only in target |
| `removed` | `list[DiffEntry]` | Files only in source |
| `modified` | `list[DiffEntry]` | Files with different content |
| `unchanged` | `list[DiffEntry]` | Identical files |
| `moved` | `list[MovedEntry]` | Files detected as moved |
| `total_changes` | `int` | `added + removed + modified + moved` |

---

## Error Handling

```python
try:
    diff = pyfs_watcher.diff_dirs("/src", "/dst")
except pyfs_watcher.DirDiffError as e:
    print(f"Diff failed: {e}")
```
