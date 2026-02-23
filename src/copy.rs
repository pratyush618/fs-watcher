use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use pyo3::prelude::*;

use crate::errors::FsError;

/// Progress information for a copy/move operation.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct CopyProgress {
    #[pyo3(get)]
    pub src: String,
    #[pyo3(get)]
    pub dst: String,
    #[pyo3(get)]
    pub bytes_copied: u64,
    #[pyo3(get)]
    pub total_bytes: u64,
    #[pyo3(get)]
    pub files_completed: usize,
    #[pyo3(get)]
    pub total_files: usize,
    #[pyo3(get)]
    pub current_file: String,
}

#[pymethods]
impl CopyProgress {
    fn __repr__(&self) -> String {
        let pct = if self.total_bytes > 0 {
            (self.bytes_copied as f64 / self.total_bytes as f64) * 100.0
        } else {
            100.0
        };
        format!(
            "CopyProgress({:.1}%, {}/{} files, {:?})",
            pct, self.files_completed, self.total_files, self.current_file
        )
    }
}

/// Copy files/directories to a destination.
#[pyfunction]
#[pyo3(signature = (sources, destination, *, overwrite=false, preserve_metadata=true, progress_callback=None, callback_interval_ms=100))]
pub fn copy_files(
    py: Python<'_>,
    sources: Vec<String>,
    destination: &str,
    overwrite: bool,
    preserve_metadata: bool,
    progress_callback: Option<PyObject>,
    callback_interval_ms: u64,
) -> PyResult<Vec<String>> {
    let dst_path = PathBuf::from(destination);
    let src_paths: Vec<PathBuf> = sources.iter().map(PathBuf::from).collect();

    // Calculate total bytes and collect file list
    let mut all_operations: Vec<(PathBuf, PathBuf, u64)> = Vec::new();
    let mut total_bytes: u64 = 0;

    for src in &src_paths {
        if !src.exists() {
            return Err(FsError::Copy(format!("source does not exist: {:?}", src)).into());
        }

        if src.is_file() {
            let size = fs::metadata(src)?.len();
            let dest_file = if dst_path.is_dir() {
                dst_path.join(src.file_name().unwrap_or_default())
            } else {
                dst_path.clone()
            };
            all_operations.push((src.clone(), dest_file, size));
            total_bytes += size;
        } else if src.is_dir() {
            collect_dir_operations(src, &dst_path, &mut all_operations, &mut total_bytes)?;
        }
    }

    let total_files = all_operations.len();
    let mut result_paths = Vec::with_capacity(total_files);
    let mut bytes_copied_total: u64 = 0;
    let mut files_completed: usize = 0;

    for (src, dst, size) in &all_operations {
        // Check for Ctrl+C
        py.check_signals()?;

        // Check overwrite
        if dst.exists() && !overwrite {
            return Err(FsError::Copy(format!(
                "destination already exists: {:?} (use overwrite=True)",
                dst
            ))
            .into());
        }

        // Ensure parent directory exists
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy with progress
        let bytes = copy_single_file(
            py,
            src,
            dst,
            *size,
            bytes_copied_total,
            total_bytes,
            files_completed,
            total_files,
            progress_callback.as_ref(),
            callback_interval_ms,
        )?;

        bytes_copied_total += bytes;
        files_completed += 1;

        if preserve_metadata {
            if let Ok(metadata) = fs::metadata(src) {
                let _ = fs::set_permissions(dst, metadata.permissions());
            }
        }

        result_paths.push(dst.to_string_lossy().into_owned());
    }

    // Final callback
    if let Some(ref cb) = progress_callback {
        let progress = CopyProgress {
            src: String::new(),
            dst: destination.to_string(),
            bytes_copied: bytes_copied_total,
            total_bytes,
            files_completed,
            total_files,
            current_file: String::new(),
        };
        let py_progress = Py::new(py, progress)?;
        cb.call1(py, (py_progress,))?;
    }

    Ok(result_paths)
}

/// Move files/directories to a destination.
/// Uses rename when on the same filesystem, falls back to copy+delete.
#[pyfunction]
#[pyo3(signature = (sources, destination, *, overwrite=false, progress_callback=None, callback_interval_ms=100))]
pub fn move_files(
    py: Python<'_>,
    sources: Vec<String>,
    destination: &str,
    overwrite: bool,
    progress_callback: Option<PyObject>,
    callback_interval_ms: u64,
) -> PyResult<Vec<String>> {
    let dst_path = PathBuf::from(destination);
    let src_paths: Vec<PathBuf> = sources.iter().map(PathBuf::from).collect();
    let mut result_paths = Vec::with_capacity(src_paths.len());

    for src in &src_paths {
        py.check_signals()?;

        if !src.exists() {
            return Err(FsError::Copy(format!("source does not exist: {:?}", src)).into());
        }

        let dest_file = if dst_path.is_dir() {
            dst_path.join(src.file_name().unwrap_or_default())
        } else {
            dst_path.clone()
        };

        if dest_file.exists() && !overwrite {
            return Err(FsError::Copy(format!(
                "destination already exists: {:?} (use overwrite=True)",
                dest_file
            ))
            .into());
        }

        // Try rename first (instant if same filesystem)
        match fs::rename(src, &dest_file) {
            Ok(()) => {
                result_paths.push(dest_file.to_string_lossy().into_owned());
            }
            Err(e) => {
                // EXDEV (cross-device link) or other error: fall back to copy+delete
                if e.raw_os_error() == Some(libc::EXDEV) || cfg!(windows) {
                    let cb_clone = progress_callback.as_ref().map(|cb| cb.clone_ref(py));
                    let copied = copy_files(
                        py,
                        vec![src.to_string_lossy().into_owned()],
                        &dest_file.to_string_lossy(),
                        overwrite,
                        true,
                        cb_clone,
                        callback_interval_ms,
                    )?;

                    // Delete source after successful copy
                    if src.is_dir() {
                        fs::remove_dir_all(src)?;
                    } else {
                        fs::remove_file(src)?;
                    }

                    result_paths.extend(copied);
                } else {
                    return Err(FsError::Copy(format!(
                        "failed to move {:?} to {:?}: {}",
                        src, dest_file, e
                    ))
                    .into());
                }
            }
        }
    }

    Ok(result_paths)
}

fn collect_dir_operations(
    src_dir: &Path,
    dst_base: &Path,
    operations: &mut Vec<(PathBuf, PathBuf, u64)>,
    total_bytes: &mut u64,
) -> Result<(), FsError> {
    let dir_name = src_dir.file_name().unwrap_or_default();
    let dst_dir = dst_base.join(dir_name);

    for entry in jwalk::WalkDir::new(src_dir).sort(true) {
        let entry = entry.map_err(|e| FsError::Copy(e.to_string()))?;
        if entry.file_type().is_file() {
            let entry_path = entry.path().to_path_buf();
            let rel = entry_path
                .strip_prefix(src_dir)
                .map_err(|e| FsError::Copy(e.to_string()))?;
            let dst_file = dst_dir.join(rel);
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            operations.push((entry_path, dst_file, size));
            *total_bytes += size;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn copy_single_file(
    py: Python<'_>,
    src: &Path,
    dst: &Path,
    _file_size: u64,
    bytes_copied_before: u64,
    total_bytes: u64,
    files_completed: usize,
    total_files: usize,
    callback: Option<&PyObject>,
    callback_interval_ms: u64,
) -> PyResult<u64> {
    let src_file = fs::File::open(src)?;
    let dst_file = fs::File::create(dst)?;
    let mut reader = BufReader::with_capacity(256 * 1024, src_file);
    let mut writer = BufWriter::with_capacity(256 * 1024, dst_file);
    let mut buf = vec![0u8; 256 * 1024];
    let mut bytes_this_file: u64 = 0;
    let mut last_callback = Instant::now();
    let interval = std::time::Duration::from_millis(callback_interval_ms);

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| FsError::Copy(e.to_string()))?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buf[..n])
            .map_err(|e| FsError::Copy(e.to_string()))?;
        bytes_this_file += n as u64;

        // Throttled progress callback
        if let Some(cb) = callback {
            if last_callback.elapsed() >= interval {
                let progress = CopyProgress {
                    src: src.to_string_lossy().into_owned(),
                    dst: dst.to_string_lossy().into_owned(),
                    bytes_copied: bytes_copied_before + bytes_this_file,
                    total_bytes,
                    files_completed,
                    total_files,
                    current_file: src
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                };
                let py_progress = Py::new(py, progress)?;
                cb.call1(py, (py_progress,))?;
                last_callback = Instant::now();
            }
        }
    }

    writer.flush().map_err(|e| FsError::Copy(e.to_string()))?;
    Ok(bytes_this_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_collect_dir_operations() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.txt"), "aaa").unwrap();
        fs::write(src.join("sub/b.txt"), "bbb").unwrap();

        let dst = tmp.path().join("dst");
        let mut ops = Vec::new();
        let mut total = 0;

        collect_dir_operations(&src, &dst, &mut ops, &mut total).unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(total, 6); // 3 + 3 bytes
    }
}
