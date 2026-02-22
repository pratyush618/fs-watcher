"""Benchmark: fs_watcher.walk vs os.walk"""

import os
import time
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import fs_watcher

TARGET = "/usr"


def bench_os_walk():
    start = time.perf_counter()
    count = sum(len(files) for _, _, files in os.walk(TARGET))
    elapsed = time.perf_counter() - start
    return count, elapsed


def bench_fs_watcher_collect():
    start = time.perf_counter()
    entries = fs_watcher.walk_collect(TARGET, file_type="file")
    elapsed = time.perf_counter() - start
    return len(entries), elapsed


def bench_fs_watcher_iter():
    start = time.perf_counter()
    count = sum(1 for _ in fs_watcher.walk(TARGET, file_type="file"))
    elapsed = time.perf_counter() - start
    return count, elapsed


if __name__ == "__main__":
    print(f"Benchmarking recursive walk of {TARGET}\n")

    count_os, time_os = bench_os_walk()
    print(f"os.walk:               {count_os:>8,} files in {time_os:.3f}s")

    count_collect, time_collect = bench_fs_watcher_collect()
    print(f"fs_watcher.walk_collect: {count_collect:>8,} files in {time_collect:.3f}s")

    count_iter, time_iter = bench_fs_watcher_iter()
    print(f"fs_watcher.walk (iter): {count_iter:>8,} files in {time_iter:.3f}s")

    print(f"\nSpeedup (collect): {time_os / time_collect:.1f}x")
    print(f"Speedup (iter):    {time_os / time_iter:.1f}x")
