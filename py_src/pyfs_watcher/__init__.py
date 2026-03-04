"""pyfs_watcher - Rust-powered filesystem toolkit."""

from pyfs_watcher._core import (
    FileWatcher,
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
    "FileWatcher",
    "async_watch",
    "bulk_rename",
    "copy_files",
    "diff_dirs",
    "disk_usage",
    "find_duplicates",
    "hash_file",
    "hash_files",
    "move_files",
    "search",
    "search_iter",
    "snapshot",
    "sync",
    "verify",
    "walk",
    "walk_collect",
]
