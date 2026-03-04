use std::fs::File;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};
use memmap2::Mmap;

use crate::errors::FsError;

/// Threshold above which we use mmap instead of buffered reads (4 MB).
pub const MMAP_THRESHOLD: u64 = 4 * 1024 * 1024;

/// Memory-map a file for read-only access.
///
/// # Safety
/// The caller must ensure the file is not modified while the mmap is alive.
/// This is acceptable for hashing operations where we control the access pattern.
pub fn mmap_file(file: &File) -> std::io::Result<Mmap> {
    unsafe { Mmap::map(file) }
}

/// Reusable filter for walking directories across features.
pub struct WalkFilter {
    pub skip_hidden: bool,
    pub glob_matcher: Option<GlobMatcher>,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
}

impl WalkFilter {
    /// Build a WalkFilter from common Python kwargs.
    pub fn from_options(
        skip_hidden: bool,
        glob_pattern: Option<&str>,
        max_depth: Option<usize>,
        follow_symlinks: bool,
        error_variant: fn(String) -> FsError,
    ) -> Result<Self, FsError> {
        let glob_matcher = match glob_pattern {
            Some(pattern) => {
                let glob = Glob::new(pattern).map_err(|e| {
                    error_variant(format!("invalid glob pattern {:?}: {}", pattern, e))
                })?;
                Some(glob.compile_matcher())
            }
            None => None,
        };

        Ok(WalkFilter {
            skip_hidden,
            glob_matcher,
            max_depth,
            follow_symlinks,
        })
    }
}

/// Walk a directory and return files matching the filter.
/// Returns (relative_path, absolute_path, file_size) for each file.
pub fn walk_files_filtered(root: &Path, filter: &WalkFilter) -> Vec<(String, PathBuf, u64)> {
    let mut walkdir = jwalk::WalkDir::new(root)
        .follow_links(filter.follow_symlinks)
        .skip_hidden(false); // We do our own hidden-file filtering

    if let Some(depth) = filter.max_depth {
        walkdir = walkdir.max_depth(depth);
    }

    let mut results = Vec::new();

    for entry in walkdir.into_iter().flatten() {
        if entry.depth == 0 {
            continue;
        }

        if !entry.file_type().is_file() {
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

        let abs_path = entry.path().to_path_buf();
        let rel_path = abs_path
            .strip_prefix(root)
            .unwrap_or(&abs_path)
            .to_string_lossy()
            .into_owned();
        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);

        results.push((rel_path, abs_path, file_size));
    }

    results
}
