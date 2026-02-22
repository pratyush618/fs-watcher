use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;
use pyo3::create_exception;
use std::io::ErrorKind;

// Exception hierarchy: all inherit from FsWatcherError
create_exception!(fs_watcher._core, FsWatcherError, pyo3::exceptions::PyException);
create_exception!(fs_watcher._core, WalkError, FsWatcherError);
create_exception!(fs_watcher._core, HashError, FsWatcherError);
create_exception!(fs_watcher._core, CopyError, FsWatcherError);
create_exception!(fs_watcher._core, WatchError, FsWatcherError);

/// Internal error type that converts to PyErr.
#[derive(Debug)]
pub enum FsError {
    Io(std::io::Error),
    Walk(String),
    Hash(String),
    Copy(String),
    Watch(String),
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
        }
    }
}
