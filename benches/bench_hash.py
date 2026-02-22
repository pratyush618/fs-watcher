"""Benchmark: fs_watcher.hash vs hashlib"""

import hashlib
import os
import tempfile
import time
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import fs_watcher


def create_test_files(directory: str, count: int, size: int) -> list[str]:
    paths = []
    for i in range(count):
        path = os.path.join(directory, f"file_{i}.bin")
        with open(path, "wb") as f:
            f.write(os.urandom(size))
        paths.append(path)
    return paths


def bench_hashlib(paths: list[str]):
    start = time.perf_counter()
    for path in paths:
        h = hashlib.sha256()
        with open(path, "rb") as f:
            while chunk := f.read(1_048_576):
                h.update(chunk)
        h.hexdigest()
    elapsed = time.perf_counter() - start
    return elapsed


def bench_fs_watcher_sha256(paths: list[str]):
    start = time.perf_counter()
    fs_watcher.hash_files(paths, algorithm="sha256")
    elapsed = time.perf_counter() - start
    return elapsed


def bench_fs_watcher_blake3(paths: list[str]):
    start = time.perf_counter()
    fs_watcher.hash_files(paths, algorithm="blake3")
    elapsed = time.perf_counter() - start
    return elapsed


if __name__ == "__main__":
    count = 50
    size = 10 * 1024 * 1024  # 10 MB each

    with tempfile.TemporaryDirectory() as tmpdir:
        print(f"Creating {count} files of {size // 1024 // 1024} MB each...\n")
        paths = create_test_files(tmpdir, count, size)
        total_mb = count * size / 1024 / 1024

        time_hashlib = bench_hashlib(paths)
        print(f"hashlib (SHA256, sequential):     {time_hashlib:.3f}s ({total_mb / time_hashlib:.0f} MB/s)")

        time_sha256 = bench_fs_watcher_sha256(paths)
        print(f"fs_watcher (SHA256, parallel):     {time_sha256:.3f}s ({total_mb / time_sha256:.0f} MB/s)")

        time_blake3 = bench_fs_watcher_blake3(paths)
        print(f"fs_watcher (BLAKE3, parallel):     {time_blake3:.3f}s ({total_mb / time_blake3:.0f} MB/s)")

        print(f"\nSpeedup SHA256 (parallel vs seq): {time_hashlib / time_sha256:.1f}x")
        print(f"Speedup BLAKE3 vs hashlib SHA256: {time_hashlib / time_blake3:.1f}x")
