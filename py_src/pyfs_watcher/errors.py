"""Exception classes for pyfs_watcher."""

from pyfs_watcher._core import (
    CopyError,
    DirDiffError,
    DiskUsageError,
    FsWatcherError,
    HashError,
    RenameError,
    SearchError,
    SnapshotError,
    SyncError,
    WalkError,
    WatchError,
)

__all__ = [
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
]
