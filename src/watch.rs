use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel as channel;

use globset::{Glob, GlobSet, GlobSetBuilder};
use notify_debouncer_full::notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use pyo3::prelude::*;

use crate::errors::FsError;

// Re-export the notify types through the debouncer's version to avoid version conflicts
use notify_debouncer_full::notify;

/// Represents a single file system change event.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct FileChange {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub change_type: String,
    #[pyo3(get)]
    pub is_dir: bool,
    #[pyo3(get)]
    pub timestamp: f64,
}

#[pymethods]
impl FileChange {
    fn __repr__(&self) -> String {
        let kind = if self.is_dir { "dir" } else { "file" };
        format!(
            "FileChange({:?}, {}, {})",
            self.path, self.change_type, kind
        )
    }
}

/// Watch a directory tree for changes with debouncing.
#[pyclass]
pub struct FileWatcher {
    path: PathBuf,
    recursive: bool,
    debounce_ms: u64,
    ignore_glob_set: Option<GlobSet>,
    receiver: Option<channel::Receiver<DebounceEventResult>>,
    debouncer: Option<Debouncer<notify::RecommendedWatcher, FileIdMap>>,
    running: Arc<AtomicBool>,
}

#[pymethods]
impl FileWatcher {
    #[new]
    #[pyo3(signature = (path, *, recursive=true, debounce_ms=500, ignore_patterns=None))]
    fn new(
        path: &str,
        recursive: bool,
        debounce_ms: u64,
        ignore_patterns: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let watch_path = PathBuf::from(path);
        if !watch_path.exists() {
            return Err(FsError::Watch(format!("path does not exist: {}", path)).into());
        }

        let ignore_glob_set = if let Some(patterns) = ignore_patterns {
            let mut builder = GlobSetBuilder::new();
            for pattern in &patterns {
                let glob = Glob::new(pattern).map_err(|e| {
                    FsError::Watch(format!("invalid ignore pattern {:?}: {}", pattern, e))
                })?;
                builder.add(glob);
            }
            Some(
                builder
                    .build()
                    .map_err(|e| FsError::Watch(format!("failed to build glob set: {}", e)))?,
            )
        } else {
            None
        };

        Ok(FileWatcher {
            path: watch_path,
            recursive,
            debounce_ms,
            ignore_glob_set,
            receiver: None,
            debouncer: None,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Start watching. Called automatically if using as context manager.
    fn start(&mut self) -> PyResult<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(()); // Already running
        }

        let (sender, receiver) = channel::unbounded();

        let mut debouncer = new_debouncer(
            Duration::from_millis(self.debounce_ms),
            None,
            move |result: DebounceEventResult| {
                let _ = sender.send(result);
            },
        )
        .map_err(|e| FsError::Watch(format!("failed to create watcher: {}", e)))?;

        let mode = if self.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        debouncer
            .watcher()
            .watch(&self.path, mode)
            .map_err(|e| FsError::Watch(format!("failed to watch {:?}: {}", self.path, e)))?;

        debouncer.cache().add_root(&self.path, mode);

        self.debouncer = Some(debouncer);
        self.receiver = Some(receiver);
        self.running.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// Stop watching and clean up resources.
    fn stop(&mut self) -> PyResult<()> {
        self.running.store(false, Ordering::SeqCst);
        self.debouncer.take();
        self.receiver.take();
        Ok(())
    }

    /// Poll for events with a timeout. Returns a list of FileChange.
    #[pyo3(signature = (timeout_ms=1000))]
    fn poll_events(&self, py: Python<'_>, timeout_ms: u64) -> PyResult<Vec<FileChange>> {
        let receiver = match &self.receiver {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        let result =
            py.allow_threads(|| receiver.recv_timeout(Duration::from_millis(timeout_ms)));

        match result {
            Ok(Ok(events)) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();

                let mut changes = Vec::new();
                for event in events {
                    let change_type = match event.kind {
                        notify::EventKind::Create(_) => "created",
                        notify::EventKind::Modify(_) => "modified",
                        notify::EventKind::Remove(_) => "deleted",
                        _ => continue,
                    };

                    for path in &event.paths {
                        // Apply ignore patterns
                        if let Some(ref glob_set) = self.ignore_glob_set {
                            if let Some(name) = path.file_name() {
                                if glob_set.is_match(name) {
                                    continue;
                                }
                            }
                        }

                        changes.push(FileChange {
                            path: path.to_string_lossy().into_owned(),
                            change_type: change_type.to_string(),
                            is_dir: path.is_dir(),
                            timestamp: now,
                        });
                    }
                }

                Ok(changes)
            }
            Ok(Err(errors)) => {
                for e in &errors {
                    log::warn!("watch error: {}", e);
                }
                Ok(Vec::new())
            }
            Err(channel::RecvTimeoutError::Timeout) => Ok(Vec::new()),
            Err(channel::RecvTimeoutError::Disconnected) => Ok(Vec::new()),
        }
    }

    fn __enter__(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        slf.start()?;
        Ok(slf)
    }

    #[pyo3(signature = (*_args))]
    fn __exit__(&mut self, _args: &Bound<'_, pyo3::types::PyTuple>) -> PyResult<()> {
        self.stop()
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Vec<FileChange>>> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(None);
        }

        loop {
            py.check_signals()?;

            if !self.running.load(Ordering::SeqCst) {
                return Ok(None);
            }

            let events = self.poll_events(py, 1000)?;
            if !events.is_empty() {
                return Ok(Some(events));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_set_building() {
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new("*.tmp").unwrap());
        builder.add(Glob::new("*.log").unwrap());
        let set = builder.build().unwrap();

        assert!(set.is_match("test.tmp"));
        assert!(set.is_match("error.log"));
        assert!(!set.is_match("data.txt"));
    }
}
