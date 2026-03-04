# Watch

Monitor directories for filesystem changes with cross-platform native events, debouncing, and both sync and async interfaces.

## Synchronous Watching

### Context manager + iterator

The recommended way to use `FileWatcher`:

```python
import pyfs_watcher

with pyfs_watcher.FileWatcher("/data", debounce_ms=500) as watcher:
    for changes in watcher:
        for change in changes:
            print(f"{change.change_type}: {change.path}")
```

The watcher starts automatically when entering the context manager and stops when exiting. The iterator blocks until a batch of debounced events is ready.

### Manual start/stop

```python
watcher = pyfs_watcher.FileWatcher("/data")
watcher.start()

try:
    while True:
        events = watcher.poll_events(timeout_ms=1000)
        if events:
            for e in events:
                print(e.path, e.change_type)
except KeyboardInterrupt:
    pass
finally:
    watcher.stop()
```

---

## Async Watching

Use `async_watch()` for integration with asyncio event loops:

```python
import pyfs_watcher

async def monitor():
    async for changes in pyfs_watcher.async_watch("/data"):
        for change in changes:
            print(f"{change.change_type}: {change.path}")

import asyncio
asyncio.run(monitor())
```

`async_watch()` wraps `FileWatcher` in an async generator, polling for events in a thread executor to avoid blocking the event loop.

### Async parameters

```python
async for changes in pyfs_watcher.async_watch(
    "/data",
    recursive=True,
    debounce_ms=300,
    ignore_patterns=["*.tmp"],
    poll_interval_ms=100,  # How often to check for events
):
    process(changes)
```

---

## Debouncing

The `debounce_ms` parameter controls the quiet period before events are delivered. This batches rapid successive changes into a single notification:

```python
# Fast response (may get multiple batches for a single save)
watcher = pyfs_watcher.FileWatcher("/data", debounce_ms=100)

# Balanced (default)
watcher = pyfs_watcher.FileWatcher("/data", debounce_ms=500)

# Slow, fewer notifications (good for expensive reactions)
watcher = pyfs_watcher.FileWatcher("/data", debounce_ms=2000)
```

!!! tip "Choosing debounce_ms"
    - **100–200ms** for dev servers and hot-reload scenarios
    - **500ms** (default) for general-purpose monitoring
    - **1000–2000ms** when each reaction is expensive (e.g., running a full build)

---

## Ignore Patterns

Filter out noise with glob patterns:

```python
with pyfs_watcher.FileWatcher(
    "/project",
    ignore_patterns=[
        "*.tmp",
        "*.pyc",
        ".git/**",
        "__pycache__/**",
        "*.swp",
    ],
) as watcher:
    for changes in watcher:
        # Only meaningful changes reach here
        for c in changes:
            print(c.path, c.change_type)
```

---

## FileChange Properties

Each event provides:

| Property | Type | Description |
|---|---|---|
| `path` | `str` | Absolute path of the changed file/directory |
| `change_type` | `str` | `"created"`, `"modified"`, or `"deleted"` |
| `is_dir` | `bool` | Whether the path is a directory |
| `timestamp` | `float` | Unix timestamp when the change was detected |

### Change types

- **`created`** — A new file or directory appeared
- **`modified`** — An existing file's content or metadata changed
- **`deleted`** — A file or directory was removed

---

## Recursive vs Non-Recursive

```python
# Watch all subdirectories (default)
watcher = pyfs_watcher.FileWatcher("/data", recursive=True)

# Watch only the top-level directory
watcher = pyfs_watcher.FileWatcher("/data", recursive=False)
```

---

## Error Handling

```python
try:
    with pyfs_watcher.FileWatcher("/nonexistent") as w:
        for changes in w:
            pass
except pyfs_watcher.WatchError as e:
    print(f"Watch failed: {e}")
```

---

## Common Patterns

### Dev server reload

```python
import subprocess

with pyfs_watcher.FileWatcher(
    "/project/src",
    debounce_ms=200,
    ignore_patterns=["*.pyc", "__pycache__/**"],
) as watcher:
    for changes in watcher:
        py_changes = [c for c in changes if c.path.endswith(".py")]
        if py_changes:
            print("Python files changed, reloading...")
            subprocess.run(["python", "manage.py", "runserver"])
```

### Async log tailing

```python
async def tail_logs():
    async for changes in pyfs_watcher.async_watch(
        "/var/log/myapp",
        debounce_ms=100,
        ignore_patterns=["*.gz"],
    ):
        for c in changes:
            if c.change_type == "modified":
                print(f"Log updated: {c.path}")
```
