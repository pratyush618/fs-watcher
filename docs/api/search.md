# Search API

## search()

```python
def search(
    path: str | PathLike[str],
    pattern: str,
    *,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    skip_hidden: bool = True,
    ignore_case: bool = False,
    max_count: int | None = None,
    max_filesize: int | None = None,
    context_lines: int = 0,
    follow_symlinks: bool = False,
    max_workers: int | None = None,
) -> list[SearchResult]
```

Search file contents using regex patterns, returning all results at once.

Walks the directory tree in parallel, searches each text file for the regex pattern, and returns all matching files with their matches.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Root directory to search |
| `pattern` | `str` | *required* | Regex pattern to search for |
| `glob_pattern` | `str \| None` | `None` | Only search files matching this glob |
| `max_depth` | `int \| None` | `None` | Maximum recursion depth |
| `skip_hidden` | `bool` | `True` | Skip files starting with `.` |
| `ignore_case` | `bool` | `False` | Case-insensitive matching |
| `max_count` | `int \| None` | `None` | Max matches per file |
| `max_filesize` | `int \| None` | `None` | Skip files larger than this (bytes) |
| `context_lines` | `int` | `0` | Lines of context before/after each match |
| `follow_symlinks` | `bool` | `False` | Follow symbolic links |
| `max_workers` | `int \| None` | `None` | Max parallel threads |

### Returns

A `list` of [`SearchResult`](#searchresult) objects, one per file with matches.

### Raises

- `SearchError` — If the path does not exist or the regex is invalid.

### Example

```python
results = pyfs_watcher.search("/project", r"TODO|FIXME", glob_pattern="*.py")
for r in results:
    for m in r.matches:
        print(f"{r.path}:{m.line_number}: {m.line_text.strip()}")
```

---

## search_iter()

```python
def search_iter(
    path: str | PathLike[str],
    pattern: str,
    *,
    # same kwargs as search()
) -> SearchIter
```

Streaming variant of `search()`. Returns an iterator that yields `SearchResult` objects as files are found.

### Example

```python
for result in pyfs_watcher.search_iter("/project", r"FIXME"):
    print(f"{result.path}: {result.match_count} matches")
```

---

## SearchResult

```python
class SearchResult
```

All matches found in a single file.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Absolute path of the file |
| `matches` | `list[SearchMatch]` | All matches in this file |
| `match_count` | `int` | Number of matches |

### Protocols

- `__len__() -> int` — Returns `match_count`
- `__repr__() -> str`

---

## SearchMatch

```python
class SearchMatch
```

A single match within a file.

### Properties

| Property | Type | Description |
|---|---|---|
| `line_number` | `int` | 1-based line number |
| `line_text` | `str` | Full text of the matching line |
| `match_start` | `int` | Start offset of the match within the line |
| `match_end` | `int` | End offset of the match within the line |
| `context_before` | `list[str]` | Lines before the match |
| `context_after` | `list[str]` | Lines after the match |

---

## SearchIter

```python
class SearchIter
```

Streaming iterator over search results. Yields `SearchResult` objects.

### Protocols

- `__iter__()` — Returns self
- `__next__() -> SearchResult` — Yields next result or raises `StopIteration`
