# API Reference

Complete reference for all public symbols exported by `pyfs_watcher`.

## Summary

| Symbol | Category | Description |
|---|---|---|
| [`walk()`](walk.md#walk) | Walk | Streaming directory traversal |
| [`walk_collect()`](walk.md#walk_collect) | Walk | Collect all entries at once |
| [`WalkEntry`](walk.md#walkentry) | Walk | Single directory entry |
| [`WalkIter`](walk.md#walkiter) | Walk | Streaming walk iterator |
| [`hash_file()`](hash.md#hash_file) | Hash | Hash a single file |
| [`hash_files()`](hash.md#hash_files) | Hash | Hash multiple files in parallel |
| [`HashResult`](hash.md#hashresult) | Hash | Hash result with metadata |
| [`copy_files()`](copy.md#copy_files) | Copy/Move | Copy files with progress |
| [`move_files()`](copy.md#move_files) | Copy/Move | Move files with smart fallback |
| [`CopyProgress`](copy.md#copyprogress) | Copy/Move | Progress snapshot |
| [`FileWatcher`](watch.md#filewatcher) | Watch | Filesystem watcher |
| [`FileChange`](watch.md#filechange) | Watch | Single change event |
| [`async_watch()`](watch.md#async_watch) | Watch | Async watch generator |
| [`find_duplicates()`](dedup.md#find_duplicates) | Dedup | Find duplicate files |
| [`DuplicateGroup`](dedup.md#duplicategroup) | Dedup | Group of duplicate files |

## Exceptions

| Exception | Description |
|---|---|
| [`FsWatcherError`](exceptions.md#fswatchererror) | Base exception for all errors |
| [`WalkError`](exceptions.md#walkerror) | Directory walk failure |
| [`HashError`](exceptions.md#hasherror) | Hashing failure |
| [`CopyError`](exceptions.md#copyerror) | Copy/move failure |
| [`WatchError`](exceptions.md#watcherror) | File watching failure |

## Import

All symbols are available from the top-level package:

```python
from pyfs_watcher import (
    walk, walk_collect, WalkEntry,
    hash_file, hash_files, HashResult,
    copy_files, move_files, CopyProgress,
    FileWatcher, FileChange, async_watch,
    find_duplicates, DuplicateGroup,
    FsWatcherError, WalkError, HashError, CopyError, WatchError,
)
```
