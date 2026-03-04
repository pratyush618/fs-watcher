# Search

Search file contents in parallel using regex patterns. Binary files are automatically skipped, and results can be collected at once or streamed.

## Basic Usage

```python
import pyfs_watcher

results = pyfs_watcher.search("/project", r"TODO")
for r in results:
    for m in r.matches:
        print(f"{r.path}:{m.line_number}: {m.line_text.strip()}")
```

---

## Filter by File Type

Use `glob_pattern` to restrict which files are searched:

```python
# Only Python files
results = pyfs_watcher.search("/project", r"import os", glob_pattern="*.py")

# Only markdown files
results = pyfs_watcher.search("/docs", r"deprecated", glob_pattern="*.md")
```

---

## Case-Insensitive Search

```python
results = pyfs_watcher.search("/project", r"error", ignore_case=True)
# Matches "Error", "ERROR", "error", etc.
```

---

## Context Lines

Show lines before and after each match, like `grep -C`:

```python
results = pyfs_watcher.search("/project", r"raise\s+\w+Error", context_lines=2)
for r in results:
    for m in r.matches:
        for line in m.context_before:
            print(f"  {line}")
        print(f"> {m.line_text}")
        for line in m.context_after:
            print(f"  {line}")
```

---

## Limit Results

```python
# Max 5 matches per file
results = pyfs_watcher.search("/project", r"TODO", max_count=5)

# Skip files larger than 1 MB
results = pyfs_watcher.search("/project", r"TODO", max_filesize=1_048_576)
```

---

## Streaming Mode

Use `search_iter()` for streaming results as they are found:

```python
for result in pyfs_watcher.search_iter("/project", r"FIXME"):
    print(f"{result.path}: {result.match_count} matches")
```

This is useful when searching large codebases and you want to start processing results before the full scan completes.

---

## Match Properties

Each `SearchMatch` provides detailed position information:

| Property | Type | Description |
|---|---|---|
| `line_number` | `int` | 1-based line number |
| `line_text` | `str` | Full text of the matching line |
| `match_start` | `int` | Start offset of the match within the line |
| `match_end` | `int` | End offset of the match within the line |
| `context_before` | `list[str]` | Lines before the match (if `context_lines > 0`) |
| `context_after` | `list[str]` | Lines after the match (if `context_lines > 0`) |

---

## Binary File Detection

Files are checked for null bytes in the first 8 KB. Binary files are automatically skipped to avoid noisy, meaningless matches.

---

## Error Handling

```python
try:
    results = pyfs_watcher.search("/data", r"pattern")
except pyfs_watcher.SearchError as e:
    print(f"Search failed: {e}")
```

Common errors:

- Invalid regex pattern
- Root path does not exist
