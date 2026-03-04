use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::types::PyString;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::errors::FsError;
use crate::hash::{self, Algorithm};
use crate::utils::WalkFilter;

/// (rel_path, actual_hash, actual_size, expected_hash, expected_size)
type VerifyRow = (String, Option<String>, Option<u64>, String, u64);

/// A single entry in a filesystem snapshot.
#[pyclass(frozen)]
#[derive(Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub hash_hex: String,
    #[pyo3(get)]
    pub file_size: u64,
    #[pyo3(get)]
    pub mtime: f64,
    #[pyo3(get)]
    pub permissions: u32,
}

#[pymethods]
impl SnapshotEntry {
    fn __repr__(&self) -> String {
        format!(
            "SnapshotEntry({:?}, {}B, {})",
            self.path,
            self.file_size,
            &self.hash_hex[..16.min(self.hash_hex.len())]
        )
    }
}

/// Serializable snapshot data for JSON persistence.
#[derive(Serialize, Deserialize)]
struct SnapshotData {
    root_path: String,
    algorithm: String,
    created_at: String,
    total_files: usize,
    total_size: u64,
    entries: Vec<SnapshotEntry>,
}

/// A filesystem snapshot capturing hashes and metadata.
#[pyclass]
#[derive(Clone)]
pub struct Snapshot {
    #[pyo3(get)]
    pub root_path: String,
    #[pyo3(get)]
    pub algorithm: String,
    #[pyo3(get)]
    pub created_at: String,
    #[pyo3(get)]
    pub total_files: usize,
    #[pyo3(get)]
    pub total_size: u64,
    #[pyo3(get)]
    pub entries: Vec<SnapshotEntry>,
}

#[pymethods]
impl Snapshot {
    fn __repr__(&self) -> String {
        format!(
            "Snapshot({:?}, {} files, {}B, {})",
            self.root_path, self.total_files, self.total_size, self.created_at
        )
    }

    fn __len__(&self) -> usize {
        self.total_files
    }

    /// Save snapshot to a JSON file.
    fn save(&self, path: &str) -> PyResult<()> {
        let data = SnapshotData {
            root_path: self.root_path.clone(),
            algorithm: self.algorithm.clone(),
            created_at: self.created_at.clone(),
            total_files: self.total_files,
            total_size: self.total_size,
            entries: self.entries.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| FsError::Snapshot(format!("failed to serialize snapshot: {}", e)))?;

        fs::write(path, json)?;
        Ok(())
    }

    /// Load a snapshot from a JSON file.
    #[staticmethod]
    fn load(path: &str) -> PyResult<Snapshot> {
        let json = fs::read_to_string(path)?;
        let data: SnapshotData = serde_json::from_str(&json)
            .map_err(|e| FsError::Snapshot(format!("failed to parse snapshot: {}", e)))?;

        Ok(Snapshot {
            root_path: data.root_path,
            algorithm: data.algorithm,
            created_at: data.created_at,
            total_files: data.total_files,
            total_size: data.total_size,
            entries: data.entries,
        })
    }
}

/// A change detected during verification.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct VerifyChange {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub change_type: String,
    #[pyo3(get)]
    pub expected_hash: Option<String>,
    #[pyo3(get)]
    pub actual_hash: Option<String>,
    #[pyo3(get)]
    pub expected_size: Option<u64>,
    #[pyo3(get)]
    pub actual_size: Option<u64>,
}

#[pymethods]
impl VerifyChange {
    fn __repr__(&self) -> String {
        format!("VerifyChange({:?}, {:?})", self.path, self.change_type)
    }
}

/// Result of verifying a snapshot against the filesystem.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct VerifyResult {
    #[pyo3(get)]
    pub ok: bool,
    #[pyo3(get)]
    pub added: Vec<VerifyChange>,
    #[pyo3(get)]
    pub removed: Vec<VerifyChange>,
    #[pyo3(get)]
    pub modified: Vec<VerifyChange>,
    #[pyo3(get)]
    pub errors: Vec<String>,
}

#[pymethods]
impl VerifyResult {
    fn __repr__(&self) -> String {
        format!(
            "VerifyResult(ok={}, added={}, removed={}, modified={}, errors={})",
            self.ok,
            self.added.len(),
            self.removed.len(),
            self.modified.len(),
            self.errors.len()
        )
    }
}

/// Create a snapshot of a directory.
#[pyfunction]
#[pyo3(signature = (path, *, algorithm="blake3", skip_hidden=false, glob_pattern=None,
                     max_depth=None, follow_symlinks=false, max_workers=None,
                     progress_callback=None))]
#[allow(clippy::too_many_arguments)]
pub fn snapshot(
    py: Python<'_>,
    path: &str,
    algorithm: &str,
    skip_hidden: bool,
    glob_pattern: Option<&str>,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    max_workers: Option<usize>,
    progress_callback: Option<PyObject>,
) -> PyResult<Snapshot> {
    let root = PathBuf::from(path);
    if !root.is_dir() {
        return Err(FsError::Snapshot(format!("path is not a directory: {}", path)).into());
    }

    let algo = Algorithm::from_str(algorithm)?;

    let filter = WalkFilter::from_options(
        skip_hidden,
        glob_pattern,
        max_depth,
        follow_symlinks,
        FsError::Snapshot,
    )?;

    // Walk
    if let Some(ref cb) = progress_callback {
        cb.call1(py, ("walking", 0usize, 0usize))?;
    }

    let files = py.allow_threads(|| crate::utils::walk_files_filtered(&root, &filter));
    let total = files.len();

    // Hash + metadata
    if let Some(ref cb) = progress_callback {
        cb.call1(py, ("hashing", 0usize, total))?;
    }

    let entries: Vec<SnapshotEntry> = py.allow_threads(|| {
        let hash_fn = || -> Vec<SnapshotEntry> {
            files
                .par_iter()
                .filter_map(|(rel, abs, _size)| {
                    let hash_result = hash::hash_file_internal(abs, algo, 1_048_576).ok()?;

                    let metadata = fs::metadata(abs).ok()?;
                    let mtime = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0);

                    #[cfg(unix)]
                    let permissions = {
                        use std::os::unix::fs::PermissionsExt;
                        metadata.permissions().mode()
                    };
                    #[cfg(not(unix))]
                    let permissions = 0u32;

                    Some(SnapshotEntry {
                        path: rel.clone(),
                        hash_hex: hash_result.hash_hex,
                        file_size: hash_result.file_size,
                        mtime,
                        permissions,
                    })
                })
                .collect()
        };

        if let Some(workers) = max_workers {
            if let Ok(pool) = rayon::ThreadPoolBuilder::new().num_threads(workers).build() {
                pool.install(hash_fn)
            } else {
                hash_fn()
            }
        } else {
            hash_fn()
        }
    });

    let total_size: u64 = entries.iter().map(|e| e.file_size).sum();
    let created_at = chrono::Utc::now().to_rfc3339();

    if let Some(ref cb) = progress_callback {
        cb.call1(py, ("done", entries.len(), total))?;
    }

    Ok(Snapshot {
        root_path: path.to_string(),
        algorithm: algorithm.to_string(),
        created_at,
        total_files: entries.len(),
        total_size,
        entries,
    })
}

/// Verify a snapshot against the current filesystem state.
#[pyfunction]
#[pyo3(signature = (snapshot_or_path, *, max_workers=None, progress_callback=None))]
pub fn verify(
    py: Python<'_>,
    snapshot_or_path: &Bound<'_, PyAny>,
    max_workers: Option<usize>,
    progress_callback: Option<PyObject>,
) -> PyResult<VerifyResult> {
    // Accept either a Snapshot object or a path string
    let snap: Snapshot = if let Ok(s) = snapshot_or_path.extract::<Snapshot>() {
        s
    } else if let Ok(path_str) = snapshot_or_path.downcast::<PyString>() {
        Snapshot::load(&path_str.to_string())?
    } else {
        return Err(
            FsError::Snapshot("expected a Snapshot object or a path string".to_string()).into(),
        );
    };

    let algo = Algorithm::from_str(&snap.algorithm)?;
    let root = PathBuf::from(&snap.root_path);

    if !root.is_dir() {
        return Err(
            FsError::Snapshot(format!("snapshot root does not exist: {}", snap.root_path)).into(),
        );
    }

    // Build map from snapshot
    let snap_map: HashMap<String, &SnapshotEntry> =
        snap.entries.iter().map(|e| (e.path.clone(), e)).collect();

    // Re-walk the directory
    let filter = WalkFilter {
        skip_hidden: false,
        glob_matcher: None,
        max_depth: None,
        follow_symlinks: false,
    };

    if let Some(ref cb) = progress_callback {
        cb.call1(py, ("walking", 0usize, 0usize))?;
    }

    let current_files = py.allow_threads(|| crate::utils::walk_files_filtered(&root, &filter));
    let current_map: HashMap<String, PathBuf> = current_files
        .into_iter()
        .map(|(rel, abs, _)| (rel, abs))
        .collect();

    if let Some(ref cb) = progress_callback {
        cb.call1(py, ("verifying", 0usize, snap.entries.len()))?;
    }

    // Hash current files that exist in snapshot
    let to_verify: Vec<(String, PathBuf, String, u64)> = snap_map
        .iter()
        .filter_map(|(rel, entry)| {
            current_map.get(rel).map(|abs| {
                (
                    rel.clone(),
                    abs.clone(),
                    entry.hash_hex.clone(),
                    entry.file_size,
                )
            })
        })
        .collect();

    let verify_results: Vec<VerifyRow> = py.allow_threads(|| {
        let work = || -> Vec<VerifyRow> {
            to_verify
                .par_iter()
                .map(|(rel, abs, expected_hash, expected_size)| {
                    let result = hash::hash_file_internal(abs, algo, 1_048_576).ok();
                    let actual_hash = result.as_ref().map(|r| r.hash_hex.clone());
                    let actual_size = result.map(|r| r.file_size);
                    (
                        rel.clone(),
                        actual_hash,
                        actual_size,
                        expected_hash.clone(),
                        *expected_size,
                    )
                })
                .collect()
        };

        if let Some(workers) = max_workers {
            if let Ok(pool) = rayon::ThreadPoolBuilder::new().num_threads(workers).build() {
                pool.install(work)
            } else {
                work()
            }
        } else {
            work()
        }
    });

    let mut modified = Vec::new();
    let mut errors = Vec::new();

    for (rel, actual_hash, actual_size, expected_hash, expected_size) in verify_results {
        match (&actual_hash, actual_size) {
            (Some(ah), Some(asize)) => {
                if *ah != expected_hash || asize != expected_size {
                    modified.push(VerifyChange {
                        path: rel,
                        change_type: "modified".to_string(),
                        expected_hash: Some(expected_hash),
                        actual_hash,
                        expected_size: Some(expected_size),
                        actual_size: Some(asize),
                    });
                }
            }
            _ => {
                errors.push(format!("failed to hash: {}", rel));
            }
        }
    }

    // Removed files (in snapshot but not on disk)
    let removed: Vec<VerifyChange> = snap_map
        .keys()
        .filter(|k| !current_map.contains_key(*k))
        .map(|k| {
            let entry = snap_map[k];
            VerifyChange {
                path: k.clone(),
                change_type: "removed".to_string(),
                expected_hash: Some(entry.hash_hex.clone()),
                actual_hash: None,
                expected_size: Some(entry.file_size),
                actual_size: None,
            }
        })
        .collect();

    // Added files (on disk but not in snapshot)
    let added: Vec<VerifyChange> = current_map
        .keys()
        .filter(|k| !snap_map.contains_key(*k))
        .map(|k| VerifyChange {
            path: k.clone(),
            change_type: "added".to_string(),
            expected_hash: None,
            actual_hash: None,
            expected_size: None,
            actual_size: None,
        })
        .collect();

    let ok = added.is_empty() && removed.is_empty() && modified.is_empty() && errors.is_empty();

    if let Some(ref cb) = progress_callback {
        cb.call1(py, ("done", snap.entries.len(), snap.entries.len()))?;
    }

    Ok(VerifyResult {
        ok,
        added,
        removed,
        modified,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_snapshot_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(root.join("a.txt"), "hello").unwrap();
        fs::write(root.join("b.txt"), "world").unwrap();

        let filter = WalkFilter {
            skip_hidden: false,
            glob_matcher: None,
            max_depth: None,
            follow_symlinks: false,
        };

        let files = crate::utils::walk_files_filtered(root, &filter);
        let algo = Algorithm::Blake3;

        let entries: Vec<SnapshotEntry> = files
            .iter()
            .filter_map(|(rel, abs, _)| {
                let hr = hash::hash_file_internal(abs, algo, 1_048_576).ok()?;
                let metadata = fs::metadata(abs).ok()?;
                let mtime = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);
                Some(SnapshotEntry {
                    path: rel.clone(),
                    hash_hex: hr.hash_hex,
                    file_size: hr.file_size,
                    mtime,
                    permissions: 0,
                })
            })
            .collect();

        let snap = Snapshot {
            root_path: root.to_string_lossy().into_owned(),
            algorithm: "blake3".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            total_files: entries.len(),
            total_size: entries.iter().map(|e| e.file_size).sum(),
            entries,
        };

        // Save and load
        let snap_path = tmp.path().join("snapshot.json");
        let data = SnapshotData {
            root_path: snap.root_path.clone(),
            algorithm: snap.algorithm.clone(),
            created_at: snap.created_at.clone(),
            total_files: snap.total_files,
            total_size: snap.total_size,
            entries: snap.entries.clone(),
        };

        let json = serde_json::to_string_pretty(&data).unwrap();
        fs::write(&snap_path, &json).unwrap();

        let loaded_json = fs::read_to_string(&snap_path).unwrap();
        let loaded: SnapshotData = serde_json::from_str(&loaded_json).unwrap();

        assert_eq!(loaded.total_files, 2);
        assert_eq!(loaded.algorithm, "blake3");
    }
}
