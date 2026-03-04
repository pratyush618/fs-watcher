---
hide:
  - navigation
---

# pyfs-watcher

[![PyPI](https://img.shields.io/pypi/v/pyfs_watcher)](https://pypi.org/project/pyfs_watcher/)
[![Python](https://img.shields.io/pypi/pyversions/pyfs_watcher)](https://pypi.org/project/pyfs_watcher/)
[![License](https://img.shields.io/pypi/l/pyfs_watcher)](https://github.com/pratyush618/pyfs-watcher/blob/master/LICENSE)
[![CI](https://github.com/pratyush618/pyfs-watcher/actions/workflows/ci.yml/badge.svg)](https://github.com/pratyush618/pyfs-watcher/actions/workflows/ci.yml)

**Rust-powered filesystem toolkit for Python.**

Fast recursive directory listing, parallel file hashing, bulk copy/move with progress, cross-platform file watching, and file deduplication — all from a single, typed Python package.

---

## Why pyfs-watcher?

- **Performance** — Core operations run in Rust with parallel execution via Rayon, bypassing the GIL. Walk directories and hash files 10x faster than pure Python.
- **Type Safety** — Full type stubs (`py.typed`) ship with the package. Every function, class, and parameter has type annotations for IDE autocompletion and mypy/pyright checking.
- **Cross-Platform** — Works on Linux, macOS, and Windows. File watching uses native OS APIs (inotify, FSEvents, ReadDirectoryChangesW).
- **Batteries Included** — Five feature modules cover the most common filesystem operations: walk, hash, copy/move, watch, and dedup.

---

## Features

<div class="grid cards" markdown>

- **Walk**

    ---

    Parallel recursive directory traversal powered by jwalk. Stream entries one-by-one or collect them all at once.

    [Walk guide →](guides/walk.md)

- **Hash**

    ---

    BLAKE3 and SHA-256 hashing with automatic memory-mapped I/O for large files. Parallel batch hashing across all cores.

    [Hash guide →](guides/hash.md)

- **Copy / Move**

    ---

    Bulk file copy and move with real-time progress callbacks. Smart cross-device move with automatic fallback.

    [Copy/Move guide →](guides/copy-move.md)

- **Watch**

    ---

    Cross-platform filesystem watcher with debouncing. Supports both synchronous iteration and async generators.

    [Watch guide →](guides/watch.md)

- **Dedup**

    ---

    Three-stage duplicate finder: size grouping, partial hash, then full hash. Finds duplicates across multiple directories.

    [Dedup guide →](guides/dedup.md)

</div>

---

## Quick Install

```bash
pip install pyfs_watcher
```

---

## At a Glance

=== "Walk"

    ```python
    import pyfs_watcher

    for entry in pyfs_watcher.walk("/data", file_type="file", glob_pattern="*.py"):
        print(entry.path, entry.file_size)
    ```

=== "Hash"

    ```python
    result = pyfs_watcher.hash_file("large.iso", algorithm="blake3")
    print(result.hash_hex)
    ```

=== "Copy/Move"

    ```python
    pyfs_watcher.copy_files(
        ["src/a.bin", "src/b.bin"], "/backup",
        progress_callback=lambda p: print(f"{p.bytes_copied / p.total_bytes:.0%}"),
    )
    ```

=== "Watch"

    ```python
    with pyfs_watcher.FileWatcher("/data", debounce_ms=500) as w:
        for changes in w:
            for c in changes:
                print(c.path, c.change_type)
    ```

=== "Dedup"

    ```python
    groups = pyfs_watcher.find_duplicates(["/photos", "/backup"], min_size=1024)
    for g in groups:
        print(f"{g.file_size}B x {len(g.paths)} copies")
    ```

---

[Get Started →](getting-started.md){ .md-button .md-button--primary }
[API Reference](api/index.md){ .md-button }
