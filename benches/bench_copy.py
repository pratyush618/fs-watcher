"""Benchmark: pyfs_watcher.copy_files vs shutil.copy2"""

import os
import shutil
import sys
import tempfile
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import pyfs_watcher


def create_test_files(directory: str, count: int, size: int) -> list[str]:
    paths = []
    for i in range(count):
        path = os.path.join(directory, f"file_{i}.bin")
        with open(path, "wb") as f:
            f.write(os.urandom(size))
        paths.append(path)
    return paths


def bench_shutil(paths: list[str], dest: str):
    start = time.perf_counter()
    for path in paths:
        shutil.copy2(path, dest)
    elapsed = time.perf_counter() - start
    return elapsed


def bench_pyfs_watcher(paths: list[str], dest: str):
    start = time.perf_counter()
    pyfs_watcher.copy_files(paths, dest)
    elapsed = time.perf_counter() - start
    return elapsed


if __name__ == "__main__":
    count = 20
    size = 50 * 1024 * 1024  # 50 MB each

    with tempfile.TemporaryDirectory() as tmpdir:
        src_dir = os.path.join(tmpdir, "src")
        os.makedirs(src_dir)

        print(f"Creating {count} files of {size // 1024 // 1024} MB each...\n")
        paths = create_test_files(src_dir, count, size)
        total_mb = count * size / 1024 / 1024

        dst1 = os.path.join(tmpdir, "dst_shutil")
        os.makedirs(dst1)
        time_shutil = bench_shutil(paths, dst1)
        print(f"shutil.copy2:       {time_shutil:.3f}s ({total_mb / time_shutil:.0f} MB/s)")

        dst2 = os.path.join(tmpdir, "dst_fsw")
        os.makedirs(dst2)
        time_fsw = bench_pyfs_watcher(paths, dst2)
        print(f"pyfs_watcher.copy:    {time_fsw:.3f}s ({total_mb / time_fsw:.0f} MB/s)")

        print(f"\nSpeedup: {time_shutil / time_fsw:.1f}x")
