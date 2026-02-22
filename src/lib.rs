#![allow(unexpected_cfgs)]

use pyo3::prelude::*;

mod copy;
mod dedup;
mod errors;
mod hash;
mod utils;
mod walk;
mod watch;

/// fs_watcher._core - Rust-powered filesystem toolkit.
#[pymodule]
fn _core(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize Rust log -> Python logging bridge
    pyo3_log::init();

    // Exceptions
    m.add("FsWatcherError", py.get_type_bound::<errors::FsWatcherError>())?;
    m.add("WalkError", py.get_type_bound::<errors::WalkError>())?;
    m.add("HashError", py.get_type_bound::<errors::HashError>())?;
    m.add("CopyError", py.get_type_bound::<errors::CopyError>())?;
    m.add("WatchError", py.get_type_bound::<errors::WatchError>())?;

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

    Ok(())
}
