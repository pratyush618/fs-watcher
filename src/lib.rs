#![allow(unexpected_cfgs)]
#![allow(clippy::useless_conversion)] // pyo3 proc macros generate .into() on PyResult returns

use pyo3::prelude::*;

mod copy;
mod dedup;
mod diff;
mod du;
mod errors;
mod hash;
mod rename;
mod search;
mod snapshot;
mod sync;
mod utils;
mod walk;
mod watch;

/// pyfs_watcher._core - Rust-powered filesystem toolkit.
#[pymodule]
fn _core(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize Rust log -> Python logging bridge
    pyo3_log::init();

    // Exceptions
    m.add(
        "FsWatcherError",
        py.get_type_bound::<errors::FsWatcherError>(),
    )?;
    m.add("WalkError", py.get_type_bound::<errors::WalkError>())?;
    m.add("HashError", py.get_type_bound::<errors::HashError>())?;
    m.add("CopyError", py.get_type_bound::<errors::CopyError>())?;
    m.add("WatchError", py.get_type_bound::<errors::WatchError>())?;
    m.add("SearchError", py.get_type_bound::<errors::SearchError>())?;
    m.add("DirDiffError", py.get_type_bound::<errors::DirDiffError>())?;
    m.add("SyncError", py.get_type_bound::<errors::SyncError>())?;
    m.add(
        "SnapshotError",
        py.get_type_bound::<errors::SnapshotError>(),
    )?;
    m.add(
        "DiskUsageError",
        py.get_type_bound::<errors::DiskUsageError>(),
    )?;
    m.add("RenameError", py.get_type_bound::<errors::RenameError>())?;

    // Walk
    m.add_class::<walk::WalkEntry>()?;
    m.add_class::<walk::WalkIter>()?;
    m.add_function(wrap_pyfunction!(walk::walk, m)?)?;
    m.add_function(wrap_pyfunction!(walk::walk_collect, m)?)?;

    // Hash
    m.add_class::<hash::HashResult>()?;
    m.add_function(wrap_pyfunction!(hash::hash_file, m)?)?;
    m.add_function(wrap_pyfunction!(hash::hash_files, m)?)?;

    // Copy/Move
    m.add_class::<copy::CopyProgress>()?;
    m.add_function(wrap_pyfunction!(copy::copy_files, m)?)?;
    m.add_function(wrap_pyfunction!(copy::move_files, m)?)?;

    // Watch
    m.add_class::<watch::FileWatcher>()?;
    m.add_class::<watch::FileChange>()?;

    // Dedup
    m.add_class::<dedup::DuplicateGroup>()?;
    m.add_function(wrap_pyfunction!(dedup::find_duplicates, m)?)?;

    // Search
    m.add_class::<search::SearchMatch>()?;
    m.add_class::<search::SearchResult>()?;
    m.add_class::<search::SearchIter>()?;
    m.add_function(wrap_pyfunction!(search::search, m)?)?;
    m.add_function(wrap_pyfunction!(search::search_iter, m)?)?;

    // Diff
    m.add_class::<diff::DiffEntry>()?;
    m.add_class::<diff::MovedEntry>()?;
    m.add_class::<diff::DirDiff>()?;
    m.add_function(wrap_pyfunction!(diff::diff_dirs, m)?)?;

    // Sync
    m.add_class::<sync::SyncFileError>()?;
    m.add_class::<sync::SyncProgress>()?;
    m.add_class::<sync::SyncResult>()?;
    m.add_function(wrap_pyfunction!(sync::sync, m)?)?;

    // Snapshot
    m.add_class::<snapshot::SnapshotEntry>()?;
    m.add_class::<snapshot::Snapshot>()?;
    m.add_class::<snapshot::VerifyChange>()?;
    m.add_class::<snapshot::VerifyResult>()?;
    m.add_function(wrap_pyfunction!(snapshot::snapshot, m)?)?;
    m.add_function(wrap_pyfunction!(snapshot::verify, m)?)?;

    // Disk Usage
    m.add_class::<du::DiskUsageEntry>()?;
    m.add_class::<du::DiskUsage>()?;
    m.add_function(wrap_pyfunction!(du::disk_usage, m)?)?;

    // Rename
    m.add_class::<rename::RenameEntry>()?;
    m.add_class::<rename::RenameFileError>()?;
    m.add_class::<rename::RenameResult>()?;
    m.add_function(wrap_pyfunction!(rename::bulk_rename, m)?)?;

    Ok(())
}
