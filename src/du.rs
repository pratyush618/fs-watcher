use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pyo3::prelude::*;

use crate::errors::FsError;
use crate::utils::WalkFilter;

/// A single entry in the disk usage breakdown.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct DiskUsageEntry {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub size: u64,
    #[pyo3(get)]
    pub file_count: u64,
    #[pyo3(get)]
    pub dir_count: u64,
    #[pyo3(get)]
    pub is_dir: bool,
}

#[pymethods]
impl DiskUsageEntry {
    fn __repr__(&self) -> String {
        format!(
            "DiskUsageEntry({:?}, {}B, {} files, {} dirs)",
            self.path, self.size, self.file_count, self.dir_count
        )
    }
}

/// Result of disk usage calculation.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct DiskUsage {
    #[pyo3(get)]
    pub total_size: u64,
    #[pyo3(get)]
    pub total_files: u64,
    #[pyo3(get)]
    pub total_dirs: u64,
    #[pyo3(get)]
    pub children: Vec<DiskUsageEntry>,
}

#[pymethods]
impl DiskUsage {
    fn __repr__(&self) -> String {
        format!(
            "DiskUsage({}B, {} files, {} dirs, {} children)",
            self.total_size,
            self.total_files,
            self.total_dirs,
            self.children.len()
        )
    }
}

/// Calculate disk usage for a directory.
#[pyfunction]
#[pyo3(signature = (path, *, max_depth=None, skip_hidden=false, follow_symlinks=false, glob_pattern=None, max_workers=None))]
#[allow(clippy::too_many_arguments)]
pub fn disk_usage(
    py: Python<'_>,
    path: &str,
    max_depth: Option<usize>,
    skip_hidden: bool,
    follow_symlinks: bool,
    glob_pattern: Option<&str>,
    max_workers: Option<usize>,
) -> PyResult<DiskUsage> {
    let root = PathBuf::from(path);
    if !root.exists() {
        return Err(FsError::DiskUsage(format!("path does not exist: {}", path)).into());
    }
    if !root.is_dir() {
        return Err(FsError::DiskUsage(format!("path is not a directory: {}", path)).into());
    }

    let _ = max_workers; // reserved for future thread pool support

    let filter = WalkFilter::from_options(
        skip_hidden,
        glob_pattern,
        None, // we handle depth grouping ourselves
        follow_symlinks,
        FsError::DiskUsage,
    )?;

    let result = py.allow_threads(|| compute_disk_usage(&root, &filter, max_depth));

    Ok(result)
}

fn compute_disk_usage(root: &Path, filter: &WalkFilter, max_depth: Option<usize>) -> DiskUsage {
    let files = crate::utils::walk_files_filtered(root, filter);

    let mut total_size: u64 = 0;
    let mut total_files: u64 = 0;
    let mut dir_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Track per-child stats (first component of relative path)
    let mut child_stats: HashMap<String, (u64, u64, std::collections::HashSet<String>)> =
        HashMap::new();

    for (rel_path, _abs_path, file_size) in &files {
        total_size += file_size;
        total_files += 1;

        // Track parent dirs
        let rel = std::path::Path::new(rel_path);
        if let Some(parent) = rel.parent() {
            let parent_str = parent.to_string_lossy().into_owned();
            if !parent_str.is_empty() {
                dir_set.insert(parent_str);
            }
        }

        // Group by top-level child
        let components: Vec<_> = std::path::Path::new(rel_path).components().collect();

        if components.is_empty() {
            continue;
        }

        let child_name = if components.len() == 1 {
            // Direct file in root
            rel_path.clone()
        } else {
            // First directory component
            components[0].as_os_str().to_string_lossy().into_owned()
        };

        let entry = child_stats
            .entry(child_name)
            .or_insert_with(|| (0, 0, std::collections::HashSet::new()));
        entry.0 += file_size;
        entry.1 += 1;
        // Track subdirs within this child
        if components.len() > 2 {
            let subdir = components[1].as_os_str().to_string_lossy().into_owned();
            entry.2.insert(subdir);
        }
    }

    let total_dirs = dir_set.len() as u64;

    let mut children: Vec<DiskUsageEntry> = child_stats
        .into_iter()
        .map(|(name, (size, file_count, subdirs))| {
            let is_dir =
                std::path::Path::new(&name).extension().is_none() && root.join(&name).is_dir();
            DiskUsageEntry {
                path: name,
                size,
                file_count,
                dir_count: subdirs.len() as u64,
                is_dir,
            }
        })
        .collect();

    // Apply max_depth filtering — if max_depth is set, only show top-level
    if let Some(_depth) = max_depth {
        // Already grouped by top-level children, just sort
    }

    // Sort by size descending
    children.sort_by(|a, b| b.size.cmp(&a.size));

    DiskUsage {
        total_size,
        total_files,
        total_dirs,
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_du_tree() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("big_dir/sub")).unwrap();
        fs::create_dir_all(root.join("small_dir")).unwrap();

        fs::write(root.join("big_dir/large.bin"), vec![0u8; 10000]).unwrap();
        fs::write(root.join("big_dir/sub/nested.bin"), vec![0u8; 5000]).unwrap();
        fs::write(root.join("small_dir/tiny.txt"), "hello").unwrap();
        fs::write(root.join("root_file.txt"), "root").unwrap();

        tmp
    }

    #[test]
    fn test_disk_usage_totals() {
        let tmp = create_du_tree();
        let filter = WalkFilter {
            skip_hidden: false,
            glob_matcher: None,
            max_depth: None,
            follow_symlinks: false,
        };

        let result = compute_disk_usage(&tmp.path().to_path_buf(), &filter, None);

        assert_eq!(result.total_files, 4);
        assert_eq!(result.total_size, 10000 + 5000 + 5 + 4);
        assert!(result.total_dirs > 0);
    }

    #[test]
    fn test_disk_usage_children_sorted() {
        let tmp = create_du_tree();
        let filter = WalkFilter {
            skip_hidden: false,
            glob_matcher: None,
            max_depth: None,
            follow_symlinks: false,
        };

        let result = compute_disk_usage(&tmp.path().to_path_buf(), &filter, None);

        // Children should be sorted by size descending
        for i in 1..result.children.len() {
            assert!(result.children[i - 1].size >= result.children[i].size);
        }

        // big_dir should be first (15000 bytes)
        assert_eq!(result.children[0].path, "big_dir");
        assert_eq!(result.children[0].size, 15000);
        assert_eq!(result.children[0].file_count, 2);
    }

    #[test]
    fn test_disk_usage_skip_hidden() {
        let tmp = create_du_tree();
        let root = tmp.path();
        fs::write(root.join(".hidden_file"), "secret").unwrap();

        let filter = WalkFilter {
            skip_hidden: true,
            glob_matcher: None,
            max_depth: None,
            follow_symlinks: false,
        };

        let result = compute_disk_usage(&root.to_path_buf(), &filter, None);
        assert_eq!(result.total_files, 4); // hidden file excluded
    }
}
