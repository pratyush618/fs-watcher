"""Type stubs for the Rust extension module."""

from __future__ import annotations

from collections.abc import Iterator, Sequence
from os import PathLike
from typing import (
    Callable,
    Literal,
)

# ──── Exceptions ────

class FsWatcherError(Exception):
    """Base exception for all pyfs_watcher errors."""

class WalkError(FsWatcherError):
    """Raised when a directory walk operation fails."""

class HashError(FsWatcherError):
    """Raised when a file hashing operation fails."""

class CopyError(FsWatcherError):
    """Raised when a copy or move operation fails."""

class WatchError(FsWatcherError):
    """Raised when a file watching operation fails."""

class SearchError(FsWatcherError):
    """Raised when a search operation fails."""

class DirDiffError(FsWatcherError):
    """Raised when a directory diff operation fails."""

class SyncError(FsWatcherError):
    """Raised when a sync operation fails."""

class SnapshotError(FsWatcherError):
    """Raised when a snapshot or verify operation fails."""

class DiskUsageError(FsWatcherError):
    """Raised when a disk usage operation fails."""

class RenameError(FsWatcherError):
    """Raised when a bulk rename operation fails."""

# ──── Walk ────

class WalkEntry:
    """A single entry discovered during directory traversal."""

    @property
    def path(self) -> str: ...
    @property
    def is_dir(self) -> bool: ...
    @property
    def is_file(self) -> bool: ...
    @property
    def is_symlink(self) -> bool: ...
    @property
    def depth(self) -> int: ...
    @property
    def file_size(self) -> int: ...
    def __repr__(self) -> str: ...

class WalkIter:
    """Streaming iterator over directory entries."""

    def __iter__(self) -> Iterator[WalkEntry]: ...
    def __next__(self) -> WalkEntry: ...

def walk(
    path: str | PathLike[str],
    *,
    max_depth: int | None = None,
    follow_symlinks: bool = False,
    sort: bool = False,
    skip_hidden: bool = False,
    file_type: Literal["file", "dir", "any"] = "any",
    glob_pattern: str | None = None,
) -> WalkIter: ...
def walk_collect(
    path: str | PathLike[str],
    *,
    max_depth: int | None = None,
    follow_symlinks: bool = False,
    sort: bool = False,
    skip_hidden: bool = False,
    file_type: Literal["file", "dir", "any"] = "any",
    glob_pattern: str | None = None,
) -> list[WalkEntry]: ...

# ──── Hash ────

class HashResult:
    """Result of hashing a single file."""

    @property
    def path(self) -> str: ...
    @property
    def hash_hex(self) -> str: ...
    @property
    def algorithm(self) -> str: ...
    @property
    def file_size(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

def hash_file(
    path: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    chunk_size: int = 1_048_576,
) -> HashResult: ...
def hash_files(
    paths: Sequence[str | PathLike[str]],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    chunk_size: int = 1_048_576,
    max_workers: int | None = None,
    callback: Callable[[HashResult], None] | None = None,
) -> list[HashResult]: ...

# ──── Copy / Move ────

class CopyProgress:
    """Snapshot of progress during a copy or move operation."""

    @property
    def src(self) -> str: ...
    @property
    def dst(self) -> str: ...
    @property
    def bytes_copied(self) -> int: ...
    @property
    def total_bytes(self) -> int: ...
    @property
    def files_completed(self) -> int: ...
    @property
    def total_files(self) -> int: ...
    @property
    def current_file(self) -> str: ...
    def __repr__(self) -> str: ...

def copy_files(
    sources: Sequence[str | PathLike[str]],
    destination: str | PathLike[str],
    *,
    overwrite: bool = False,
    preserve_metadata: bool = True,
    progress_callback: Callable[[CopyProgress], None] | None = None,
    callback_interval_ms: int = 100,
) -> list[str]: ...
def move_files(
    sources: Sequence[str | PathLike[str]],
    destination: str | PathLike[str],
    *,
    overwrite: bool = False,
    progress_callback: Callable[[CopyProgress], None] | None = None,
    callback_interval_ms: int = 100,
) -> list[str]: ...

# ──── Watch ────

class FileChange:
    """A single filesystem change event."""

    @property
    def path(self) -> str: ...
    @property
    def change_type(self) -> Literal["created", "modified", "deleted"]: ...
    @property
    def is_dir(self) -> bool: ...
    @property
    def timestamp(self) -> float: ...
    def __repr__(self) -> str: ...

class FileWatcher:
    """Cross-platform filesystem watcher with debouncing."""

    def __init__(
        self,
        path: str | PathLike[str],
        *,
        recursive: bool = True,
        debounce_ms: int = 500,
        ignore_patterns: Sequence[str] | None = None,
    ) -> None: ...
    def start(self) -> None: ...
    def stop(self) -> None: ...
    def poll_events(self, timeout_ms: int = 1000) -> list[FileChange]: ...
    def __enter__(self) -> FileWatcher: ...
    def __exit__(self, *args: object) -> None: ...
    def __iter__(self) -> Iterator[list[FileChange]]: ...
    def __next__(self) -> list[FileChange]: ...

# ──── Dedup ────

class DuplicateGroup:
    """A group of files that share identical content."""

    @property
    def hash_hex(self) -> str: ...
    @property
    def file_size(self) -> int: ...
    @property
    def paths(self) -> list[str]: ...
    @property
    def wasted_bytes(self) -> int: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...

def find_duplicates(
    paths: Sequence[str | PathLike[str]],
    *,
    recursive: bool = True,
    min_size: int = 1,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    partial_hash_size: int = 4096,
    max_workers: int | None = None,
    progress_callback: Callable[[str, int, int], None] | None = None,
) -> list[DuplicateGroup]: ...

# ──── Search ────

class SearchMatch:
    """A single match within a file."""

    @property
    def line_number(self) -> int: ...
    @property
    def line_text(self) -> str: ...
    @property
    def match_start(self) -> int: ...
    @property
    def match_end(self) -> int: ...
    @property
    def context_before(self) -> list[str]: ...
    @property
    def context_after(self) -> list[str]: ...
    def __repr__(self) -> str: ...

class SearchResult:
    """All matches found in a single file."""

    @property
    def path(self) -> str: ...
    @property
    def matches(self) -> list[SearchMatch]: ...
    @property
    def match_count(self) -> int: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...

class SearchIter:
    """Streaming iterator over search results."""

    def __iter__(self) -> Iterator[SearchResult]: ...
    def __next__(self) -> SearchResult: ...

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
) -> list[SearchResult]: ...
def search_iter(
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
) -> SearchIter: ...

# ──── Diff ────

class DiffEntry:
    """A file that differs between source and target."""

    @property
    def path(self) -> str: ...
    @property
    def source_size(self) -> int | None: ...
    @property
    def target_size(self) -> int | None: ...
    @property
    def source_hash(self) -> str | None: ...
    @property
    def target_hash(self) -> str | None: ...
    def __repr__(self) -> str: ...

class MovedEntry:
    """A file detected as moved (same content, different path)."""

    @property
    def source_path(self) -> str: ...
    @property
    def target_path(self) -> str: ...
    @property
    def hash_hex(self) -> str: ...
    @property
    def file_size(self) -> int: ...
    def __repr__(self) -> str: ...

class DirDiff:
    """Result of comparing two directories."""

    @property
    def added(self) -> list[DiffEntry]: ...
    @property
    def removed(self) -> list[DiffEntry]: ...
    @property
    def modified(self) -> list[DiffEntry]: ...
    @property
    def unchanged(self) -> list[DiffEntry]: ...
    @property
    def moved(self) -> list[MovedEntry]: ...
    @property
    def total_changes(self) -> int: ...
    def __repr__(self) -> str: ...

def diff_dirs(
    source: str | PathLike[str],
    target: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    compare_content: bool = True,
    skip_hidden: bool = False,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    detect_moves: bool = False,
    max_workers: int | None = None,
    progress_callback: Callable[[str], None] | None = None,
) -> DirDiff: ...

# ──── Sync ────

class SyncFileError:
    """A per-file error during sync."""

    @property
    def path(self) -> str: ...
    @property
    def message(self) -> str: ...
    def __repr__(self) -> str: ...

class SyncProgress:
    """Progress snapshot during sync."""

    @property
    def current_file(self) -> str: ...
    @property
    def files_completed(self) -> int: ...
    @property
    def total_files(self) -> int: ...
    @property
    def bytes_transferred(self) -> int: ...
    @property
    def stage(self) -> str: ...
    def __repr__(self) -> str: ...

class SyncResult:
    """Result of a sync operation."""

    @property
    def copied(self) -> list[str]: ...
    @property
    def deleted(self) -> list[str]: ...
    @property
    def skipped(self) -> list[str]: ...
    @property
    def total_bytes_transferred(self) -> int: ...
    @property
    def errors(self) -> list[SyncFileError]: ...
    def __repr__(self) -> str: ...

def sync(
    source: str | PathLike[str],
    target: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    delete_extra: bool = False,
    skip_hidden: bool = False,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    dry_run: bool = False,
    preserve_metadata: bool = True,
    max_workers: int | None = None,
    progress_callback: Callable[[SyncProgress], None] | None = None,
) -> SyncResult: ...

# ──── Snapshot ────

class SnapshotEntry:
    """A single entry in a filesystem snapshot."""

    @property
    def path(self) -> str: ...
    @property
    def hash_hex(self) -> str: ...
    @property
    def file_size(self) -> int: ...
    @property
    def mtime(self) -> float: ...
    @property
    def permissions(self) -> int: ...
    def __repr__(self) -> str: ...

class Snapshot:
    """A filesystem snapshot capturing hashes and metadata."""

    @property
    def root_path(self) -> str: ...
    @property
    def algorithm(self) -> str: ...
    @property
    def created_at(self) -> str: ...
    @property
    def total_files(self) -> int: ...
    @property
    def total_size(self) -> int: ...
    @property
    def entries(self) -> list[SnapshotEntry]: ...
    def save(self, path: str | PathLike[str]) -> None: ...
    @staticmethod
    def load(path: str | PathLike[str]) -> Snapshot: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...

class VerifyChange:
    """A change detected during verification."""

    @property
    def path(self) -> str: ...
    @property
    def change_type(self) -> Literal["added", "removed", "modified"]: ...
    @property
    def expected_hash(self) -> str | None: ...
    @property
    def actual_hash(self) -> str | None: ...
    @property
    def expected_size(self) -> int | None: ...
    @property
    def actual_size(self) -> int | None: ...
    def __repr__(self) -> str: ...

class VerifyResult:
    """Result of verifying a snapshot against the filesystem."""

    @property
    def ok(self) -> bool: ...
    @property
    def added(self) -> list[VerifyChange]: ...
    @property
    def removed(self) -> list[VerifyChange]: ...
    @property
    def modified(self) -> list[VerifyChange]: ...
    @property
    def errors(self) -> list[str]: ...
    def __repr__(self) -> str: ...

def snapshot(
    path: str | PathLike[str],
    *,
    algorithm: Literal["sha256", "blake3"] = "blake3",
    skip_hidden: bool = False,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    follow_symlinks: bool = False,
    max_workers: int | None = None,
    progress_callback: Callable[[str, int, int], None] | None = None,
) -> Snapshot: ...
def verify(
    snapshot_or_path: Snapshot | str | PathLike[str],
    *,
    max_workers: int | None = None,
    progress_callback: Callable[[str, int, int], None] | None = None,
) -> VerifyResult: ...

# ──── Disk Usage ────

class DiskUsageEntry:
    """A single entry in the disk usage breakdown."""

    @property
    def path(self) -> str: ...
    @property
    def size(self) -> int: ...
    @property
    def file_count(self) -> int: ...
    @property
    def dir_count(self) -> int: ...
    @property
    def is_dir(self) -> bool: ...
    def __repr__(self) -> str: ...

class DiskUsage:
    """Result of disk usage calculation."""

    @property
    def total_size(self) -> int: ...
    @property
    def total_files(self) -> int: ...
    @property
    def total_dirs(self) -> int: ...
    @property
    def children(self) -> list[DiskUsageEntry]: ...
    def __repr__(self) -> str: ...

def disk_usage(
    path: str | PathLike[str],
    *,
    max_depth: int | None = None,
    skip_hidden: bool = False,
    follow_symlinks: bool = False,
    glob_pattern: str | None = None,
    max_workers: int | None = None,
) -> DiskUsage: ...

# ──── Rename ────

class RenameEntry:
    """A single rename operation (old -> new)."""

    @property
    def old_path(self) -> str: ...
    @property
    def new_path(self) -> str: ...
    @property
    def old_name(self) -> str: ...
    @property
    def new_name(self) -> str: ...
    def __repr__(self) -> str: ...

class RenameFileError:
    """A per-file error during rename."""

    @property
    def path(self) -> str: ...
    @property
    def message(self) -> str: ...
    def __repr__(self) -> str: ...

class RenameResult:
    """Result of a bulk rename operation."""

    @property
    def renamed(self) -> list[RenameEntry]: ...
    @property
    def skipped(self) -> int: ...
    @property
    def errors(self) -> list[RenameFileError]: ...
    @property
    def dry_run(self) -> bool: ...
    def undo(self) -> list[RenameFileError]: ...
    def __repr__(self) -> str: ...

def bulk_rename(
    path: str | PathLike[str],
    pattern: str,
    replacement: str,
    *,
    recursive: bool = False,
    skip_hidden: bool = True,
    glob_pattern: str | None = None,
    max_depth: int | None = None,
    dry_run: bool = True,
    include_dirs: bool = False,
) -> RenameResult: ...
