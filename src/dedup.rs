use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use pyo3::prelude::*;
use rayon::prelude::*;

use crate::errors::FsError;
use crate::hash::{self, Algorithm};

/// A group of files that are duplicates of each other.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct DuplicateGroup {
    #[pyo3(get)]
    pub hash_hex: String,
    #[pyo3(get)]
    pub file_size: u64,
    #[pyo3(get)]
    pub paths: Vec<String>,
}

#[pymethods]
impl DuplicateGroup {
    /// Bytes wasted by keeping all copies instead of just one.
    #[getter]
    fn wasted_bytes(&self) -> u64 {
        if self.paths.len() <= 1 {
            return 0;
        }
        self.file_size * (self.paths.len() as u64 - 1)
    }

    fn __repr__(&self) -> String {
        format!(
            "DuplicateGroup({}B x {} copies, wasted={}B)",
            self.file_size,
            self.paths.len(),
            self.wasted_bytes()
        )
    }

    fn __len__(&self) -> usize {
        self.paths.len()
    }
}

/// Find duplicate files using a staged pipeline.
///
/// Pipeline:
/// 1. Walk all paths, group by file size
/// 2. Partial hash (first + last `partial_hash_size` bytes)
/// 3. Full hash only for files matching in steps 1 and 2
#[pyfunction]
#[pyo3(signature = (paths, *, recursive=true, min_size=1, algorithm="blake3", partial_hash_size=4096, max_workers=None, progress_callback=None))]
pub fn find_duplicates(
    py: Python<'_>,
    paths: Vec<String>,
    recursive: bool,
    min_size: u64,
    algorithm: &str,
    partial_hash_size: usize,
    max_workers: Option<usize>,
    progress_callback: Option<PyObject>,
) -> PyResult<Vec<DuplicateGroup>> {
    let algo = Algorithm::from_str(algorithm)?;

    // Build optional custom thread pool
    let pool = if let Some(workers) = max_workers {
        Some(
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .map_err(|e| FsError::Hash(format!("failed to create thread pool: {}", e)))?,
        )
    } else {
        None
    };

    // Stage 1: Collect files and group by size
    report_progress(py, &progress_callback, "collecting", 0, 0)?;

    let file_entries = py.allow_threads(|| collect_files(&paths, recursive, min_size))?;
    let total_files = file_entries.len();

    report_progress(py, &progress_callback, "size_grouping", 0, total_files)?;

    let size_groups = py.allow_threads(|| group_by_size(file_entries));
    let candidates_after_size: Vec<(u64, Vec<PathBuf>)> = size_groups
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect();

    let candidate_count: usize = candidates_after_size.iter().map(|(_, f)| f.len()).sum();
    report_progress(py, &progress_callback, "size_grouping", candidate_count, total_files)?;

    // Stage 2: Partial hash
    report_progress(py, &progress_callback, "partial_hash", 0, candidate_count)?;

    let partial_groups = py.allow_threads(|| {
        let work = || {
            partial_hash_stage(&candidates_after_size, algo, partial_hash_size)
        };
        match &pool {
            Some(p) => p.install(work),
            None => work(),
        }
    });

    let candidates_after_partial: Vec<(String, u64, Vec<PathBuf>)> = partial_groups
        .into_iter()
        .filter(|(_, _, files)| files.len() > 1)
        .collect();

    let partial_count: usize = candidates_after_partial.iter().map(|(_, _, f)| f.len()).sum();
    report_progress(py, &progress_callback, "partial_hash", partial_count, candidate_count)?;

    // Stage 3: Full hash
    report_progress(py, &progress_callback, "full_hash", 0, partial_count)?;

    let full_groups = py.allow_threads(|| {
        let work = || full_hash_stage(&candidates_after_partial, algo);
        match &pool {
            Some(p) => p.install(work),
            None => work(),
        }
    });

    let mut duplicates: Vec<DuplicateGroup> = full_groups
        .into_iter()
        .filter(|(_, _, files)| files.len() > 1)
        .map(|(hash_hex, size, files)| DuplicateGroup {
            hash_hex,
            file_size: size,
            paths: files.into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
        })
        .collect();

    // Sort by wasted bytes descending (worst offenders first)
    duplicates.sort_by(|a, b| b.wasted_bytes().cmp(&a.wasted_bytes()));

    let dup_count: usize = duplicates.iter().map(|g| g.paths.len()).sum();
    report_progress(py, &progress_callback, "full_hash", dup_count, partial_count)?;

    Ok(duplicates)
}

fn report_progress(
    py: Python<'_>,
    callback: &Option<PyObject>,
    stage: &str,
    processed: usize,
    total: usize,
) -> PyResult<()> {
    py.check_signals()?;
    if let Some(ref cb) = callback {
        cb.call1(py, (stage, processed, total))?;
    }
    Ok(())
}

fn collect_files(
    paths: &[String],
    recursive: bool,
    min_size: u64,
) -> Result<Vec<(PathBuf, u64)>, FsError> {
    let mut entries = Vec::new();

    for path_str in paths {
        let path = PathBuf::from(path_str);
        if path.is_file() {
            let size = fs::metadata(&path)?.len();
            if size >= min_size {
                entries.push((path, size));
            }
        } else if path.is_dir() {
            let mut walkdir = jwalk::WalkDir::new(&path);
            if !recursive {
                walkdir = walkdir.max_depth(1);
            }

            for entry in walkdir {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        if size >= min_size {
                            entries.push((entry.path().to_path_buf(), size));
                        }
                    }
                }
            }
        }
    }

    Ok(entries)
}

fn group_by_size(entries: Vec<(PathBuf, u64)>) -> HashMap<u64, Vec<PathBuf>> {
    let mut groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (path, size) in entries {
        groups.entry(size).or_default().push(path);
    }
    groups
}

fn partial_hash_stage(
    size_groups: &[(u64, Vec<PathBuf>)],
    algo: Algorithm,
    partial_size: usize,
) -> Vec<(String, u64, Vec<PathBuf>)> {
    let mut results: Vec<(String, u64, Vec<PathBuf>)> = Vec::new();

    for (size, files) in size_groups {
        // Hash all files in this group in parallel
        let hashes: Vec<(PathBuf, Option<String>)> = files
            .par_iter()
            .map(|path| {
                let hash = hash::partial_hash(path, algo, partial_size).ok();
                (path.clone(), hash)
            })
            .collect();

        // Group by partial hash
        let mut hash_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for (path, hash) in hashes {
            if let Some(h) = hash {
                hash_groups.entry(h).or_default().push(path);
            }
        }

        for (hash, paths) in hash_groups {
            results.push((hash, *size, paths));
        }
    }

    results
}

fn full_hash_stage(
    partial_groups: &[(String, u64, Vec<PathBuf>)],
    algo: Algorithm,
) -> Vec<(String, u64, Vec<PathBuf>)> {
    let mut results: Vec<(String, u64, Vec<PathBuf>)> = Vec::new();

    for (_partial_hash, size, files) in partial_groups {
        // Full-hash all files in this group in parallel
        let hashes: Vec<(PathBuf, Option<String>)> = files
            .par_iter()
            .map(|path| {
                let result = hash::hash_file_internal(path, algo, 1_048_576).ok();
                (path.clone(), result.map(|r| r.hash_hex))
            })
            .collect();

        // Group by full hash
        let mut hash_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for (path, hash) in hashes {
            if let Some(h) = hash {
                hash_groups.entry(h).or_default().push(path);
            }
        }

        for (hash, paths) in hash_groups {
            results.push((hash, *size, paths));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_group_by_size() {
        let entries = vec![
            (PathBuf::from("/a"), 100),
            (PathBuf::from("/b"), 100),
            (PathBuf::from("/c"), 200),
            (PathBuf::from("/d"), 200),
            (PathBuf::from("/e"), 300),
        ];

        let groups = group_by_size(entries);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[&100].len(), 2);
        assert_eq!(groups[&200].len(), 2);
        assert_eq!(groups[&300].len(), 1);
    }

    #[test]
    fn test_collect_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), "hello").unwrap();
        fs::write(tmp.path().join("b.txt"), "world").unwrap();
        fs::write(tmp.path().join("tiny"), "").unwrap(); // empty

        let entries =
            collect_files(&[tmp.path().to_string_lossy().into_owned()], true, 1).unwrap();
        assert_eq!(entries.len(), 2); // empty file filtered by min_size=1
    }

    #[test]
    fn test_full_pipeline() {
        let tmp = TempDir::new().unwrap();
        let content_a = vec![0u8; 10000];
        let content_b = vec![1u8; 10000];

        fs::write(tmp.path().join("dup_a1.bin"), &content_a).unwrap();
        fs::write(tmp.path().join("dup_a2.bin"), &content_a).unwrap();
        fs::write(tmp.path().join("dup_a3.bin"), &content_a).unwrap();
        fs::write(tmp.path().join("dup_b1.bin"), &content_b).unwrap();
        fs::write(tmp.path().join("dup_b2.bin"), &content_b).unwrap();
        fs::write(tmp.path().join("unique.bin"), &[2u8; 5000]).unwrap();

        let path_str = tmp.path().to_string_lossy().into_owned();
        let entries = collect_files(&[path_str], true, 1).unwrap();
        let size_groups: Vec<(u64, Vec<PathBuf>)> = group_by_size(entries)
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        // Should have one size group (10000 bytes with 5 files)
        assert_eq!(size_groups.len(), 1);
        assert_eq!(size_groups[0].1.len(), 5);

        let partial = partial_hash_stage(&size_groups, Algorithm::Blake3, 4096);
        // After partial hash: 2 groups (a's hash and b's hash)
        let partial_dup: Vec<_> = partial.iter().filter(|(_, _, f)| f.len() > 1).collect();
        assert_eq!(partial_dup.len(), 2);

        let full = full_hash_stage(
            &partial.iter().filter(|(_, _, f)| f.len() > 1).cloned().collect::<Vec<_>>(),
            Algorithm::Blake3,
        );
        let full_dup: Vec<_> = full.iter().filter(|(_, _, f)| f.len() > 1).collect();
        assert_eq!(full_dup.len(), 2);
    }
}
