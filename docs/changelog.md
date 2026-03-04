# Changelog

All notable changes to pyfs-watcher are documented here.

---

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
