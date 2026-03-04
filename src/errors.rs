use pyo3::create_exception;
use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;
use std::io::ErrorKind;

// Exception hierarchy: all inherit from FsWatcherError
create_exception!(
    pyfs_watcher._core,
    FsWatcherError,
    pyo3::exceptions::PyException
);
create_exception!(pyfs_watcher._core, WalkError, FsWatcherError);
create_exception!(pyfs_watcher._core, HashError, FsWatcherError);
create_exception!(pyfs_watcher._core, CopyError, FsWatcherError);
create_exception!(pyfs_watcher._core, WatchError, FsWatcherError);
create_exception!(pyfs_watcher._core, SearchError, FsWatcherError);
create_exception!(pyfs_watcher._core, DirDiffError, FsWatcherError);
create_exception!(pyfs_watcher._core, SyncError, FsWatcherError);
create_exception!(pyfs_watcher._core, SnapshotError, FsWatcherError);
create_exception!(pyfs_watcher._core, DiskUsageError, FsWatcherError);
create_exception!(pyfs_watcher._core, RenameError, FsWatcherError);

/// Internal error type that converts to PyErr.
#[derive(Debug)]
pub enum FsError {
    Io(std::io::Error),
    Walk(String),
    Hash(String),
    Copy(String),
    Watch(String),
    Search(String),
    DirDiff(String),
    Sync(String),
    Snapshot(String),
    DiskUsage(String),
    Rename(String),
}

impl From<FsError> for PyErr {
    fn from(err: FsError) -> PyErr {
        match err {
            FsError::Io(e) => match e.kind() {
                ErrorKind::NotFound => PyFileNotFoundError::new_err(e.to_string()),
                ErrorKind::PermissionDenied => PyPermissionError::new_err(e.to_string()),
                _ => PyOSError::new_err(e.to_string()),
            },
            FsError::Walk(msg) => WalkError::new_err(msg),
            FsError::Hash(msg) => HashError::new_err(msg),
            FsError::Copy(msg) => CopyError::new_err(msg),
            FsError::Watch(msg) => WatchError::new_err(msg),
            FsError::Search(msg) => SearchError::new_err(msg),
            FsError::DirDiff(msg) => DirDiffError::new_err(msg),
            FsError::Sync(msg) => SyncError::new_err(msg),
            FsError::Snapshot(msg) => SnapshotError::new_err(msg),
            FsError::DiskUsage(msg) => DiskUsageError::new_err(msg),
            FsError::Rename(msg) => RenameError::new_err(msg),
        }
    }
}

impl From<std::io::Error> for FsError {
    fn from(e: std::io::Error) -> Self {
        FsError::Io(e)
    }
}

impl From<notify_debouncer_full::notify::Error> for FsError {
    fn from(e: notify_debouncer_full::notify::Error) -> Self {
        FsError::Watch(e.to_string())
    }
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::Io(e) => write!(f, "{}", e),
            FsError::Walk(msg) => write!(f, "{}", msg),
            FsError::Hash(msg) => write!(f, "{}", msg),
            FsError::Copy(msg) => write!(f, "{}", msg),
            FsError::Watch(msg) => write!(f, "{}", msg),
            FsError::Search(msg) => write!(f, "{}", msg),
            FsError::DirDiff(msg) => write!(f, "{}", msg),
            FsError::Sync(msg) => write!(f, "{}", msg),
            FsError::Snapshot(msg) => write!(f, "{}", msg),
            FsError::DiskUsage(msg) => write!(f, "{}", msg),
            FsError::Rename(msg) => write!(f, "{}", msg),
        }
    }
}
