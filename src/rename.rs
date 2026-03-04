use std::fs;
use std::path::PathBuf;

use pyo3::prelude::*;
use regex::Regex;

use crate::errors::FsError;
use crate::utils::WalkFilter;

/// A single rename operation (old -> new).
#[pyclass(frozen)]
#[derive(Clone)]
pub struct RenameEntry {
    #[pyo3(get)]
    pub old_path: String,
    #[pyo3(get)]
    pub new_path: String,
    #[pyo3(get)]
    pub old_name: String,
    #[pyo3(get)]
    pub new_name: String,
}

#[pymethods]
impl RenameEntry {
    fn __repr__(&self) -> String {
        format!("RenameEntry({:?} -> {:?})", self.old_name, self.new_name)
    }
}

/// A per-file error during rename.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct RenameFileError {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub message: String,
}

#[pymethods]
impl RenameFileError {
    fn __repr__(&self) -> String {
        format!("RenameFileError({:?}, {:?})", self.path, self.message)
    }
}

/// Result of a bulk rename operation.
#[pyclass]
#[derive(Clone)]
pub struct RenameResult {
    #[pyo3(get)]
    pub renamed: Vec<RenameEntry>,
    #[pyo3(get)]
    pub skipped: usize,
    #[pyo3(get)]
    pub errors: Vec<RenameFileError>,
    #[pyo3(get)]
    pub dry_run: bool,
    undo_available: bool,
}

#[pymethods]
impl RenameResult {
    fn __repr__(&self) -> String {
        format!(
            "RenameResult(renamed={}, skipped={}, errors={}, dry_run={})",
            self.renamed.len(),
            self.skipped,
            self.errors.len(),
            self.dry_run
        )
    }

    /// Undo all successful renames (only works if dry_run was false).
    fn undo(&mut self) -> PyResult<Vec<RenameFileError>> {
        if self.dry_run {
            return Err(FsError::Rename("cannot undo a dry-run operation".to_string()).into());
        }
        if !self.undo_available {
            return Err(FsError::Rename(
                "undo already performed or no renames to undo".to_string(),
            )
            .into());
        }

        let mut errors = Vec::new();

        // Reverse renames in reverse order
        for entry in self.renamed.iter().rev() {
            if let Err(e) = fs::rename(&entry.new_path, &entry.old_path) {
                errors.push(RenameFileError {
                    path: entry.new_path.clone(),
                    message: format!("failed to undo rename: {}", e),
                });
            }
        }

        self.undo_available = false;
        Ok(errors)
    }
}

/// Rename files matching a regex pattern.
#[pyfunction]
#[pyo3(signature = (path, pattern, replacement, *, recursive=false, skip_hidden=true,
                     glob_pattern=None, max_depth=None, dry_run=true, include_dirs=false))]
#[allow(clippy::too_many_arguments)]
pub fn bulk_rename(
    py: Python<'_>,
    path: &str,
    pattern: &str,
    replacement: &str,
    recursive: bool,
    skip_hidden: bool,
    glob_pattern: Option<&str>,
    max_depth: Option<usize>,
    dry_run: bool,
    include_dirs: bool,
) -> PyResult<RenameResult> {
    let root = PathBuf::from(path);
    if !root.is_dir() {
        return Err(FsError::Rename(format!("path is not a directory: {}", path)).into());
    }

    let re = Regex::new(pattern)
        .map_err(|e| FsError::Rename(format!("invalid regex pattern: {}", e)))?;

    let max_depth_val = if recursive { max_depth } else { Some(1) };

    let filter = WalkFilter::from_options(
        skip_hidden,
        glob_pattern,
        max_depth_val,
        false,
        FsError::Rename,
    )?;

    let replacement = replacement.to_string();

    let result = py
        .allow_threads(|| execute_rename(&root, &re, &replacement, &filter, dry_run, include_dirs));

    Ok(result)
}

fn execute_rename(
    root: &PathBuf,
    re: &Regex,
    replacement: &str,
    filter: &WalkFilter,
    dry_run: bool,
    include_dirs: bool,
) -> RenameResult {
    // Collect all entries (files and optionally dirs)
    let mut entries: Vec<(PathBuf, bool)> = Vec::new();

    let mut walkdir = jwalk::WalkDir::new(root)
        .follow_links(false)
        .skip_hidden(false); // We do our own hidden-file filtering
    if let Some(depth) = filter.max_depth {
        walkdir = walkdir.max_depth(depth);
    }

    for entry in walkdir.into_iter().flatten() {
        if entry.depth == 0 {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();
        if filter.skip_hidden && file_name.starts_with('.') {
            continue;
        }

        if let Some(ref matcher) = filter.glob_matcher {
            if !matcher.is_match(file_name.as_ref()) {
                continue;
            }
        }

        let is_dir = entry.file_type().is_dir();
        if is_dir && !include_dirs {
            continue;
        }
        if !is_dir && !entry.file_type().is_file() {
            continue;
        }

        entries.push((entry.path().to_path_buf(), is_dir));
    }

    // Sort: process directories bottom-up (deepest first) when include_dirs
    if include_dirs {
        entries.sort_by(|a, b| {
            let a_depth = a.0.components().count();
            let b_depth = b.0.components().count();
            b_depth.cmp(&a_depth) // deepest first
        });
    }

    let mut renamed = Vec::new();
    let mut skipped: usize = 0;
    let mut errors = Vec::new();

    for (abs_path, _is_dir) in &entries {
        let file_name = match abs_path.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => {
                skipped += 1;
                continue;
            }
        };

        if !re.is_match(&file_name) {
            skipped += 1;
            continue;
        }

        let new_name = re.replace_all(&file_name, replacement).into_owned();

        if new_name == file_name {
            skipped += 1;
            continue;
        }

        let new_path = abs_path.with_file_name(&new_name);

        if !dry_run {
            if let Err(e) = fs::rename(abs_path, &new_path) {
                errors.push(RenameFileError {
                    path: abs_path.to_string_lossy().into_owned(),
                    message: e.to_string(),
                });
                continue;
            }
        }

        renamed.push(RenameEntry {
            old_path: abs_path.to_string_lossy().into_owned(),
            new_path: new_path.to_string_lossy().into_owned(),
            old_name: file_name,
            new_name,
        });
    }

    RenameResult {
        renamed,
        skipped,
        errors,
        dry_run,
        undo_available: !dry_run,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bulk_rename_dry_run() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(root.join("photo_001.jpg"), "img1").unwrap();
        fs::write(root.join("photo_002.jpg"), "img2").unwrap();
        fs::write(root.join("document.pdf"), "doc").unwrap();

        let re = Regex::new(r"photo_(\d+)").unwrap();
        let filter = WalkFilter {
            skip_hidden: true,
            glob_matcher: None,
            max_depth: Some(1),
            follow_symlinks: false,
        };

        let result = execute_rename(&root.to_path_buf(), &re, "img_$1", &filter, true, false);

        assert_eq!(result.renamed.len(), 2);
        assert_eq!(result.skipped, 1); // document.pdf
        assert!(result.dry_run);

        // Files should NOT have been renamed (dry run)
        assert!(root.join("photo_001.jpg").exists());
        assert!(root.join("photo_002.jpg").exists());
    }

    #[test]
    fn test_bulk_rename_actual() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(root.join("old_a.txt"), "a").unwrap();
        fs::write(root.join("old_b.txt"), "b").unwrap();

        let re = Regex::new(r"old_").unwrap();
        let filter = WalkFilter {
            skip_hidden: true,
            glob_matcher: None,
            max_depth: Some(1),
            follow_symlinks: false,
        };

        let result = execute_rename(&root.to_path_buf(), &re, "new_", &filter, false, false);

        assert_eq!(result.renamed.len(), 2);
        assert!(!result.dry_run);

        // Files should have been renamed
        assert!(root.join("new_a.txt").exists());
        assert!(root.join("new_b.txt").exists());
        assert!(!root.join("old_a.txt").exists());
        assert!(!root.join("old_b.txt").exists());
    }

    #[test]
    fn test_rename_with_dirs() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir(root.join("old_dir")).unwrap();
        fs::write(root.join("old_dir/file.txt"), "content").unwrap();

        let re = Regex::new(r"old_").unwrap();
        let filter = WalkFilter {
            skip_hidden: true,
            glob_matcher: None,
            max_depth: Some(1),
            follow_symlinks: false,
        };

        let result = execute_rename(&root.to_path_buf(), &re, "new_", &filter, false, true);

        assert_eq!(result.renamed.len(), 1); // just the dir
        assert!(root.join("new_dir").exists());
        assert!(root.join("new_dir/file.txt").exists());
    }
}
