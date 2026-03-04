use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use pyo3::prelude::*;
use rayon::prelude::*;

use crate::errors::FsError;
use crate::hash::{self, Algorithm};
use crate::utils::WalkFilter;

/// A per-file error during sync.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct SyncFileError {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub message: String,
}

#[pymethods]
impl SyncFileError {
    fn __repr__(&self) -> String {
        format!("SyncFileError({:?}, {:?})", self.path, self.message)
    }
}

/// Progress snapshot during sync.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct SyncProgress {
    #[pyo3(get)]
    pub current_file: String,
    #[pyo3(get)]
    pub files_completed: usize,
    #[pyo3(get)]
    pub total_files: usize,
    #[pyo3(get)]
    pub bytes_transferred: u64,
    #[pyo3(get)]
    pub stage: String,
}

#[pymethods]
impl SyncProgress {
    fn __repr__(&self) -> String {
        format!(
            "SyncProgress({}, {}/{} files, {}B)",
            self.stage, self.files_completed, self.total_files, self.bytes_transferred
        )
    }
}

/// Result of a sync operation.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct SyncResult {
    #[pyo3(get)]
    pub copied: Vec<String>,
    #[pyo3(get)]
    pub deleted: Vec<String>,
    #[pyo3(get)]
    pub skipped: Vec<String>,
    #[pyo3(get)]
    pub total_bytes_transferred: u64,
    #[pyo3(get)]
    pub errors: Vec<SyncFileError>,
}

#[pymethods]
impl SyncResult {
    fn __repr__(&self) -> String {
        format!(
            "SyncResult(copied={}, deleted={}, skipped={}, errors={})",
            self.copied.len(),
            self.deleted.len(),
            self.skipped.len(),
            self.errors.len()
        )
    }
}

struct FileInfo {
    abs_path: PathBuf,
    size: u64,
}

/// Synchronize source directory to target directory.
#[pyfunction]
#[pyo3(signature = (source, target, *, algorithm="blake3", delete_extra=false,
                     skip_hidden=false, glob_pattern=None, max_depth=None,
                     dry_run=false, preserve_metadata=true, max_workers=None,
                     progress_callback=None))]
#[allow(clippy::too_many_arguments)]
pub fn sync(
    py: Python<'_>,
    source: &str,
    target: &str,
    algorithm: &str,
    delete_extra: bool,
    skip_hidden: bool,
    glob_pattern: Option<&str>,
    max_depth: Option<usize>,
    dry_run: bool,
    preserve_metadata: bool,
    max_workers: Option<usize>,
    progress_callback: Option<PyObject>,
) -> PyResult<SyncResult> {
    let src_root = PathBuf::from(source);
    let tgt_root = PathBuf::from(target);

    if !src_root.is_dir() {
        return Err(FsError::Sync(format!("source is not a directory: {}", source)).into());
    }

    // Create target if it doesn't exist
    if !tgt_root.exists() {
        if !dry_run {
            fs::create_dir_all(&tgt_root)?;
        }
    } else if !tgt_root.is_dir() {
        return Err(FsError::Sync(format!("target is not a directory: {}", target)).into());
    }

    let algo = Algorithm::from_str(algorithm)?;
    let _ = max_workers; // reserved for future use

    let filter =
        WalkFilter::from_options(skip_hidden, glob_pattern, max_depth, false, FsError::Sync)?;

    // Phase 1: Walk both dirs
    report_progress(py, &progress_callback, "walking", "", 0, 0, 0)?;

    let src_files = py.allow_threads(|| crate::utils::walk_files_filtered(&src_root, &filter));
    let src_map: HashMap<String, FileInfo> = src_files
        .into_iter()
        .map(|(rel, abs, size)| {
            (
                rel,
                FileInfo {
                    abs_path: abs,
                    size,
                },
            )
        })
        .collect();

    let tgt_files = if tgt_root.exists() {
        py.allow_threads(|| crate::utils::walk_files_filtered(&tgt_root, &filter))
    } else {
        Vec::new()
    };
    let tgt_map: HashMap<String, FileInfo> = tgt_files
        .into_iter()
        .map(|(rel, abs, size)| {
            (
                rel,
                FileInfo {
                    abs_path: abs,
                    size,
                },
            )
        })
        .collect();

    // Phase 2: Diff
    report_progress(py, &progress_callback, "comparing", "", 0, src_map.len(), 0)?;

    let mut to_copy: Vec<(String, PathBuf, u64)> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut to_hash: Vec<(String, PathBuf, PathBuf)> = Vec::new();

    for (rel_path, src_info) in &src_map {
        match tgt_map.get(rel_path) {
            None => {
                to_copy.push((rel_path.clone(), src_info.abs_path.clone(), src_info.size));
            }
            Some(tgt_info) => {
                if src_info.size != tgt_info.size {
                    to_copy.push((rel_path.clone(), src_info.abs_path.clone(), src_info.size));
                } else {
                    // Same size — hash to check
                    to_hash.push((
                        rel_path.clone(),
                        src_info.abs_path.clone(),
                        tgt_info.abs_path.clone(),
                    ));
                }
            }
        }
    }

    // Hash same-size files
    let hash_results: Vec<(String, PathBuf, bool)> = py.allow_threads(|| {
        to_hash
            .par_iter()
            .map(|(rel, src_path, tgt_path)| {
                let src_hash = hash::hash_file_internal(src_path, algo, 1_048_576)
                    .ok()
                    .map(|r| r.hash_hex);
                let tgt_hash = hash::hash_file_internal(tgt_path, algo, 1_048_576)
                    .ok()
                    .map(|r| r.hash_hex);
                let same = src_hash.is_some() && src_hash == tgt_hash;
                (rel.clone(), src_path.clone(), same)
            })
            .collect()
    });

    for (rel, src_path, same) in hash_results {
        if same {
            skipped.push(rel);
        } else {
            let size = src_map.get(&rel).map(|f| f.size).unwrap_or(0);
            to_copy.push((rel, src_path, size));
        }
    }

    // Files to delete (in target but not in source)
    let to_delete: Vec<String> = if delete_extra {
        tgt_map
            .keys()
            .filter(|k| !src_map.contains_key(*k))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    // Phase 3: Execute
    let total_files = to_copy.len() + to_delete.len();
    report_progress(py, &progress_callback, "syncing", "", 0, total_files, 0)?;

    let mut copied = Vec::new();
    let mut errors = Vec::new();
    let mut total_bytes: u64 = 0;

    for (idx, (rel_path, src_path, _size)) in to_copy.iter().enumerate() {
        py.check_signals()?;

        let dst_path = tgt_root.join(rel_path);

        if dry_run {
            copied.push(rel_path.clone());
            continue;
        }

        // Create parent dirs
        if let Some(parent) = dst_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                errors.push(SyncFileError {
                    path: rel_path.clone(),
                    message: format!("failed to create directory: {}", e),
                });
                continue;
            }
        }

        match copy_file(src_path, &dst_path) {
            Ok(bytes) => {
                total_bytes += bytes;
                copied.push(rel_path.clone());

                if preserve_metadata {
                    if let Ok(metadata) = fs::metadata(src_path) {
                        let _ = fs::set_permissions(&dst_path, metadata.permissions());
                    }
                }
            }
            Err(e) => {
                errors.push(SyncFileError {
                    path: rel_path.clone(),
                    message: e.to_string(),
                });
            }
        }

        report_progress(
            py,
            &progress_callback,
            "syncing",
            rel_path,
            idx + 1,
            total_files,
            total_bytes,
        )?;
    }

    // Delete extra files
    let mut deleted = Vec::new();
    for rel_path in &to_delete {
        py.check_signals()?;

        if dry_run {
            deleted.push(rel_path.clone());
            continue;
        }

        let tgt_path = tgt_root.join(rel_path);
        match fs::remove_file(&tgt_path) {
            Ok(()) => deleted.push(rel_path.clone()),
            Err(e) => errors.push(SyncFileError {
                path: rel_path.clone(),
                message: format!("failed to delete: {}", e),
            }),
        }
    }

    Ok(SyncResult {
        copied,
        deleted,
        skipped,
        total_bytes_transferred: total_bytes,
        errors,
    })
}

fn report_progress(
    py: Python<'_>,
    callback: &Option<PyObject>,
    stage: &str,
    current_file: &str,
    files_completed: usize,
    total_files: usize,
    bytes_transferred: u64,
) -> PyResult<()> {
    py.check_signals()?;
    if let Some(ref cb) = callback {
        let progress = SyncProgress {
            current_file: current_file.to_string(),
            files_completed,
            total_files,
            bytes_transferred,
            stage: stage.to_string(),
        };
        let py_progress = Py::new(py, progress)?;
        cb.call1(py, (py_progress,))?;
    }
    Ok(())
}

fn copy_file(src: &PathBuf, dst: &std::path::Path) -> Result<u64, FsError> {
    let src_file = fs::File::open(src)?;
    let dst_file = fs::File::create(dst)?;
    let mut reader = BufReader::with_capacity(256 * 1024, src_file);
    let mut writer = BufWriter::with_capacity(256 * 1024, dst_file);
    let mut buf = vec![0u8; 256 * 1024];
    let mut total: u64 = 0;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n])?;
        total += n as u64;
    }
    writer.flush()?;
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_sync_dirs() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source");
        let tgt = tmp.path().join("target");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&tgt).unwrap();

        // File to copy (new)
        fs::write(src.join("new.txt"), "new content").unwrap();

        // File unchanged
        fs::write(src.join("same.txt"), "same content").unwrap();
        fs::write(tgt.join("same.txt"), "same content").unwrap();

        // File modified
        fs::write(src.join("modified.txt"), "updated").unwrap();
        fs::write(tgt.join("modified.txt"), "original").unwrap();

        // File only in target (extra)
        fs::write(tgt.join("extra.txt"), "extra file").unwrap();

        (tmp, src, tgt)
    }

    #[test]
    fn test_copy_file() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.txt");
        let dst = tmp.path().join("dst.txt");
        fs::write(&src, "hello world").unwrap();

        let bytes = copy_file(&src, &dst).unwrap();
        assert_eq!(bytes, 11);
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello world");
    }

    #[test]
    fn test_sync_basic() {
        let (tmp, src, tgt) = create_sync_dirs();
        let algo = Algorithm::Blake3;
        let filter = WalkFilter {
            skip_hidden: false,
            glob_matcher: None,
            max_depth: None,
            follow_symlinks: false,
        };

        let src_files = crate::utils::walk_files_filtered(&src, &filter);
        let tgt_files = crate::utils::walk_files_filtered(&tgt, &filter);

        // Verify source has expected files
        assert_eq!(src_files.len(), 3);
        assert_eq!(tgt_files.len(), 3);

        // new.txt should not be in target yet
        assert!(!tgt.join("new.txt").exists());

        // After sync would copy new.txt and modified.txt
        let _ = (tmp, algo); // keep alive
    }
}
