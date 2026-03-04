# Watch API

## FileWatcher

```python
class FileWatcher
```

Cross-platform filesystem watcher with debouncing. Watches a directory for file changes and delivers batched, debounced events. Supports both context-manager and manual start/stop usage.

### Constructor

```python
FileWatcher(
    path: str | PathLike[str],
    *,
    recursive: bool = True,
    debounce_ms: int = 500,
    ignore_patterns: Sequence[str] | None = None,
)
```

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Directory to watch |
| `recursive` | `bool` | `True` | Whether to watch subdirectories |
| `debounce_ms` | `int` | `500` | Minimum quiet time in ms before delivering events |
| `ignore_patterns` | `Sequence[str] \| None` | `None` | Glob patterns for paths to ignore (e.g. `["*.tmp", ".git/**"]`) |

### Methods

#### `start()`

```python
def start(self) -> None
```

Start watching for filesystem events.

#### `stop()`

```python
def stop(self) -> None
```

Stop watching and release resources.

#### `poll_events()`

```python
def poll_events(self, timeout_ms: int = 1000) -> list[FileChange]
```

Poll for pending events, blocking up to `timeout_ms` milliseconds.

| Parameter | Type | Default | Description |
|---|---|---|---|
| `timeout_ms` | `int` | `1000` | Maximum time to wait for events |

**Returns:** A list of [`FileChange`](#filechange) events (empty if the timeout expires).

### Protocols

- `__enter__() -> FileWatcher` — Start watching
- `__exit__(*args) -> None` — Stop watching
- `__iter__() -> Iterator[list[FileChange]]` — Iterate over event batches
- `__next__() -> list[FileChange]` — Get next event batch

### Example

```python
# Context manager (recommended)
with pyfs_watcher.FileWatcher("/data", debounce_ms=500) as w:
    for changes in w:
        for c in changes:
            print(c.path, c.change_type)

# Manual usage
watcher = pyfs_watcher.FileWatcher("/data")
watcher.start()
try:
    events = watcher.poll_events(timeout_ms=2000)
finally:
    watcher.stop()
```

---

## FileChange

```python
class FileChange
```

A single filesystem change event detected by `FileWatcher`.

### Properties

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Absolute path of the changed file or directory |
| `change_type` | `Literal["created", "modified", "deleted"]` | Type of change |
| `is_dir` | `bool` | Whether the changed path is a directory |
| `timestamp` | `float` | Unix timestamp (seconds since epoch) when the change was detected |

### Change Types

| Value | Description |
|---|---|
| `"created"` | A new file or directory appeared |
| `"modified"` | Content or metadata of an existing file changed |
| `"deleted"` | A file or directory was removed |

### Example

```python
with pyfs_watcher.FileWatcher("/data") as w:
    for changes in w:
        for c in changes:
            if c.change_type == "created" and not c.is_dir:
                print(f"New file: {c.path} at {c.timestamp}")
```

---

## async_watch()

```python
async def async_watch(
    path: str | PathLike[str],
    *,
    recursive: bool = True,
    debounce_ms: int = 500,
    ignore_patterns: Sequence[str] | None = None,
    poll_interval_ms: int = 100,
) -> AsyncIterator[list[FileChange]]
```

Async generator that yields batches of file changes. Wraps `FileWatcher` and polls for events using `asyncio.run_in_executor()`.

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `path` | `str \| PathLike[str]` | *required* | Directory to watch |
| `recursive` | `bool` | `True` | Whether to watch subdirectories |
| `debounce_ms` | `int` | `500` | Minimum quiet time in ms before delivering events |
| `ignore_patterns` | `Sequence[str] \| None` | `None` | Glob patterns for paths to ignore |
| `poll_interval_ms` | `int` | `100` | How often to poll for events (ms) |

### Returns

An `AsyncIterator[list[FileChange]]` yielding batches of change events.

### Example

```python
import asyncio
import pyfs_watcher

async def monitor():
    async for changes in pyfs_watcher.async_watch("/data", debounce_ms=300):
        for c in changes:
            print(f"{c.change_type}: {c.path}")

asyncio.run(monitor())
```
