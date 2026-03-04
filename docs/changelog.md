# Changelog

All notable changes to pyfs-watcher are documented here.

---

## v0.3.0

**Breaking:** Reorganised the public Python API to keep the top-level namespace clean.

- **`pyfs_watcher.errors`** — All exception classes (`WalkError`, `HashError`, `CopyError`, etc.) now live in a dedicated `errors` submodule.
- **`pyfs_watcher.types`** — All dataclass/result types (`DiffResult`, `SearchMatch`, `SnapshotResult`, `DuEntry`, `RenamePreview`, `RenameResult`) now live in a dedicated `types` submodule.
- **Top-level exports** are now limited to the public functions (`walk`, `walk_collect`, `hash_file`, `hash_files`, `copy_files`, `move_files`, `find_duplicates`, `search`, `search_iter`, `diff_dirs`, `sync`, `snapshot`, `verify`, `disk_usage`, `bulk_rename`) and the `FileWatcher` class.

Migration:

```python
# Before (0.2.x)
from pyfs_watcher import WalkError, DiffResult

# After (0.3.0)
from pyfs_watcher.errors import WalkError
from pyfs_watcher.types import DiffResult
```

---

## v0.2.0

Six new feature modules, transforming pyfs-watcher from fast filesystem utils into a comprehensive filesystem toolkit:

- **Search** — Parallel content search (`search()`, `search_iter()`) using Rust's `regex` crate. Supports glob filtering, case-insensitive matching, context lines, max file size limits, and binary file detection.
- **Diff** — Directory comparison (`diff_dirs()`) with content-aware diffing and optional move detection. Reports added, removed, modified, unchanged, and moved files.
- **Sync** — Incremental directory sync (`sync()`) that only copies changed files. Supports `dry_run` preview, `delete_extra` cleanup, progress callbacks, and error-tolerant operation.
- **Snapshot** — File integrity monitoring (`snapshot()`, `verify()`) with JSON-based snapshots. Captures file hashes, sizes, mtimes, and permissions. Detects additions, removals, and modifications.
- **Disk Usage** — Parallel size calculation (`disk_usage()`) with per-child breakdown sorted by size. Supports glob filtering and hidden file control.
- **Rename** — Regex-based batch rename (`bulk_rename()`) with `dry_run=True` by default for safety. Supports recursive operation, directory renaming, and `undo()`.

Additional changes:

- Added shared `WalkFilter` infrastructure in `utils.rs` for consistent filtering across all features
- Added 6 new typed exception classes (`SearchError`, `DirDiffError`, `SyncError`, `SnapshotError`, `DiskUsageError`, `RenameError`)
- Full type stubs for all new classes and functions
- Fixed jwalk `skip_hidden` default — hidden files now correctly appear when `skip_hidden=False`
- New Cargo dependencies: `regex`, `serde`, `serde_json`, `chrono`

## v0.1.1

- Bumped project version
- Set up CI with GitHub Actions (lint, typecheck, Rust checks, tests)
- Added pre-commit hooks for code quality
- Renamed package to `pyfs-watcher` / `pyfs_watcher`
- Created virtual environment workflow for CI test execution

## v0.1.0

Initial release with five feature modules:

- **Walk** — Parallel recursive directory traversal (`walk()`, `walk_collect()`) powered by jwalk
- **Hash** — BLAKE3 and SHA-256 file hashing (`hash_file()`, `hash_files()`) with memory-mapped I/O and parallel batch processing
- **Copy/Move** — Bulk file copy and move (`copy_files()`, `move_files()`) with progress callbacks and cross-device move fallback
- **Watch** — Cross-platform filesystem watcher (`FileWatcher`, `async_watch()`) with debouncing and ignore patterns
- **Dedup** — Three-stage duplicate file detection (`find_duplicates()`) with size grouping, partial hash, and full hash

Additional features:

- Full type stubs (`py.typed`) for IDE autocompletion and static analysis
- Typed exception hierarchy (`FsWatcherError` and subclasses)
- Logging bridge from Rust to Python via `pyo3-log`
- GitHub Actions workflow for PyPI publishing
