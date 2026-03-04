"""pyfs_watcher - Rust-powered filesystem toolkit."""

from pyfs_watcher._core import (
    CopyError,
    # Copy/Move
    CopyProgress,
    # Diff
    DiffEntry,
    DirDiff,
    DirDiffError,
    # Disk Usage
    DiskUsage,
    DiskUsageEntry,
    DiskUsageError,
    # Dedup
    DuplicateGroup,
    FileChange,
    # Watch
    FileWatcher,
    # Exceptions
    FsWatcherError,
    HashError,
    # Hash
    HashResult,
    # Rename
    RenameEntry,
    RenameError,
    RenameFileError,
    RenameResult,
    # Search
    SearchError,
    SearchIter,
    SearchMatch,
    SearchResult,
    # Snapshot
    Snapshot,
    SnapshotEntry,
    SnapshotError,
    # Sync
    SyncError,
    SyncFileError,
    SyncProgress,
    SyncResult,
    VerifyChange,
    VerifyResult,
    # Walk
    WalkEntry,
    WalkError,
    WatchError,
    bulk_rename,
    copy_files,
    diff_dirs,
    disk_usage,
    find_duplicates,
    hash_file,
    hash_files,
    move_files,
    search,
    search_iter,
    snapshot,
    sync,
    verify,
    walk,
    walk_collect,
)
from pyfs_watcher.watch import async_watch

__all__ = [
    # Exceptions
    "FsWatcherError",
    "WalkError",
    "HashError",
    "CopyError",
    "WatchError",
    "SearchError",
    "DirDiffError",
    "SyncError",
    "SnapshotError",
    "DiskUsageError",
    "RenameError",
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
    # Search
    "SearchMatch",
    "SearchResult",
    "SearchIter",
    "search",
    "search_iter",
    # Diff
    "DiffEntry",
    "MovedEntry",
    "DirDiff",
    "diff_dirs",
    # Sync
    "SyncFileError",
    "SyncProgress",
    "SyncResult",
    "sync",
    # Snapshot
    "SnapshotEntry",
    "Snapshot",
    "VerifyChange",
    "VerifyResult",
    "snapshot",
    "verify",
    # Disk Usage
    "DiskUsageEntry",
    "DiskUsage",
    "disk_usage",
    # Rename
    "RenameEntry",
    "RenameFileError",
    "RenameResult",
    "bulk_rename",
]
