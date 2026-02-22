"""fs_watcher - Rust-powered filesystem toolkit."""

from fs_watcher._core import (
    # Exceptions
    FsWatcherError,
    WalkError,
    HashError,
    CopyError,
    WatchError,
    # Walk
    WalkEntry,
    walk,
    walk_collect,
    # Hash
    HashResult,
    hash_file,
    hash_files,
    # Copy/Move
    CopyProgress,
    copy_files,
    move_files,
    # Watch
    FileWatcher,
    FileChange,
    # Dedup
    DuplicateGroup,
    find_duplicates,
)
from fs_watcher.watch import async_watch

__all__ = [
    # Exceptions
    "FsWatcherError",
    "WalkError",
    "HashError",
    "CopyError",
    "WatchError",
    # Walk
    "WalkEntry",
    "walk",
    "walk_collect",
    # Hash
    "HashResult",
    "hash_file",
    "hash_files",
    # Copy/Move
    "CopyProgress",
    "copy_files",
    "move_files",
    # Watch
    "FileWatcher",
    "FileChange",
    "async_watch",
    # Dedup
    "DuplicateGroup",
    "find_duplicates",
]
