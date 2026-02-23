use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use globset::{Glob, GlobMatcher};
use jwalk::WalkDir;
use pyo3::prelude::*;

use crate::errors::FsError;

/// A single directory entry returned to Python.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct WalkEntry {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub is_dir: bool,
    #[pyo3(get)]
    pub is_file: bool,
    #[pyo3(get)]
    pub is_symlink: bool,
    #[pyo3(get)]
    pub depth: usize,
    #[pyo3(get)]
    pub file_size: u64,
}

#[pymethods]
impl WalkEntry {
    fn __repr__(&self) -> String {
        let kind = if self.is_dir {
            "dir"
        } else if self.is_symlink {
            "symlink"
        } else {
            "file"
        };
        format!("WalkEntry({:?}, {}, {}B)", self.path, kind, self.file_size)
    }
}

/// Iterator that yields WalkEntry objects from a background walk thread.
#[pyclass]
pub struct WalkIter {
    receiver: mpsc::Receiver<Result<WalkEntry, String>>,
    done: bool,
}

#[pymethods]
impl WalkIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<WalkEntry>> {
        if self.done {
            return Ok(None);
        }

        // Check for Ctrl+C
        py.check_signals()?;

        match self.receiver.recv() {
            Ok(Ok(entry)) => Ok(Some(entry)),
            Ok(Err(msg)) => {
                // Log the error but continue iteration
                log::warn!("walk error: {}", msg);
                // Try the next entry
                self.__next__(py)
            }
            Err(_) => {
                // Channel disconnected - walk is done
                self.done = true;
                Ok(None)
            }
        }
    }
}

/// Options parsed from Python kwargs for the walk.
struct WalkOptions {
    max_depth: Option<usize>,
    follow_symlinks: bool,
    sort: bool,
    skip_hidden: bool,
    file_type: FileTypeFilter,
    glob_matcher: Option<GlobMatcher>,
}

#[derive(Clone, Copy, PartialEq)]
enum FileTypeFilter {
    Any,
    File,
    Dir,
}

/// Walk a directory tree, yielding WalkEntry objects via a streaming iterator.
///
/// Uses jwalk for parallel directory reading, significantly faster than os.walk().
#[pyfunction]
#[pyo3(signature = (path, *, max_depth=None, follow_symlinks=false, sort=false, skip_hidden=false, file_type="any", glob_pattern=None))]
#[allow(clippy::too_many_arguments)]
pub fn walk(
    py: Python<'_>,
    path: &str,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    sort: bool,
    skip_hidden: bool,
    file_type: &str,
    glob_pattern: Option<&str>,
) -> PyResult<WalkIter> {
    let root = PathBuf::from(path);
    if !root.exists() {
        return Err(FsError::Walk(format!("path does not exist: {}", path)).into());
    }
    if !root.is_dir() {
        return Err(FsError::Walk(format!("path is not a directory: {}", path)).into());
    }

    let opts = parse_walk_options(
        max_depth,
        follow_symlinks,
        sort,
        skip_hidden,
        file_type,
        glob_pattern,
    )?;
    let (sender, receiver) = mpsc::channel();

    // Spawn background thread for the walk
    py.allow_threads(|| {
        thread::spawn(move || {
            run_walk(root, opts, sender);
        });
    });

    Ok(WalkIter {
        receiver,
        done: false,
    })
}

/// Walk a directory tree and collect all results into a list.
///
/// Faster than walk() when you need all entries, because it avoids per-item
/// GIL overhead by running the entire traversal in Rust.
#[pyfunction]
#[pyo3(signature = (path, *, max_depth=None, follow_symlinks=false, sort=false, skip_hidden=false, file_type="any", glob_pattern=None))]
#[allow(clippy::too_many_arguments)]
pub fn walk_collect(
    py: Python<'_>,
    path: &str,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    sort: bool,
    skip_hidden: bool,
    file_type: &str,
    glob_pattern: Option<&str>,
) -> PyResult<Vec<WalkEntry>> {
    let root = PathBuf::from(path);
    if !root.exists() {
        return Err(FsError::Walk(format!("path does not exist: {}", path)).into());
    }
    if !root.is_dir() {
        return Err(FsError::Walk(format!("path is not a directory: {}", path)).into());
    }

    let opts = parse_walk_options(
        max_depth,
        follow_symlinks,
        sort,
        skip_hidden,
        file_type,
        glob_pattern,
    )?;

    let results = py.allow_threads(|| collect_walk(root, opts));

    Ok(results)
}

fn parse_walk_options(
    max_depth: Option<usize>,
    follow_symlinks: bool,
    sort: bool,
    skip_hidden: bool,
    file_type: &str,
    glob_pattern: Option<&str>,
) -> PyResult<WalkOptions> {
    let file_type = match file_type {
        "any" => FileTypeFilter::Any,
        "file" => FileTypeFilter::File,
        "dir" => FileTypeFilter::Dir,
        other => {
            return Err(FsError::Walk(format!(
                "invalid file_type: {:?}, expected \"any\", \"file\", or \"dir\"",
                other
            ))
            .into())
        }
    };

    let glob_matcher = match glob_pattern {
        Some(pattern) => {
            let glob = Glob::new(pattern)
                .map_err(|e| FsError::Walk(format!("invalid glob pattern {:?}: {}", pattern, e)))?;
            Some(glob.compile_matcher())
        }
        None => None,
    };

    Ok(WalkOptions {
        max_depth,
        follow_symlinks,
        sort,
        skip_hidden,
        file_type,
        glob_matcher,
    })
}

fn should_include(entry: &jwalk::DirEntry<((), ())>, opts: &WalkOptions) -> bool {
    let file_name = entry.file_name().to_string_lossy();

    // Skip hidden files/dirs (starting with '.')
    if opts.skip_hidden && file_name.starts_with('.') {
        return false;
    }

    // Apply file type filter
    let ft = entry.file_type();
    match opts.file_type {
        FileTypeFilter::File => {
            if !ft.is_file() {
                return false;
            }
        }
        FileTypeFilter::Dir => {
            if !ft.is_dir() {
                return false;
            }
        }
        FileTypeFilter::Any => {}
    }

    // Apply glob pattern
    if let Some(ref matcher) = opts.glob_matcher {
        if !matcher.is_match(file_name.as_ref()) {
            return false;
        }
    }

    true
}

fn dir_entry_to_walk_entry(entry: &jwalk::DirEntry<((), ())>) -> WalkEntry {
    let ft = entry.file_type();
    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);

    WalkEntry {
        path: entry.path().to_string_lossy().into_owned(),
        is_dir: ft.is_dir(),
        is_file: ft.is_file(),
        is_symlink: ft.is_symlink(),
        depth: entry.depth,
        file_size,
    }
}

fn build_walkdir(root: PathBuf, opts: &WalkOptions) -> WalkDir {
    let mut walkdir = WalkDir::new(&root).follow_links(opts.follow_symlinks);

    if let Some(depth) = opts.max_depth {
        walkdir = walkdir.max_depth(depth);
    }

    if opts.sort {
        walkdir = walkdir.sort(true);
    }

    walkdir
}

fn run_walk(root: PathBuf, opts: WalkOptions, sender: mpsc::Sender<Result<WalkEntry, String>>) {
    let walkdir = build_walkdir(root, &opts);

    for result in walkdir {
        match result {
            Ok(entry) => {
                // Skip the root directory itself (depth 0)
                if entry.depth == 0 {
                    continue;
                }
                if should_include(&entry, &opts) {
                    let walk_entry = dir_entry_to_walk_entry(&entry);
                    if sender.send(Ok(walk_entry)).is_err() {
                        // Receiver dropped (Python iterator was garbage collected)
                        return;
                    }
                }
            }
            Err(e) => {
                if sender.send(Err(e.to_string())).is_err() {
                    return;
                }
            }
        }
    }
}

fn collect_walk(root: PathBuf, opts: WalkOptions) -> Vec<WalkEntry> {
    let walkdir = build_walkdir(root, &opts);
    let mut results = Vec::new();

    for entry in walkdir.into_iter().flatten() {
        if entry.depth == 0 {
            continue;
        }
        if should_include(&entry, &opts) {
            results.push(dir_entry_to_walk_entry(&entry));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("a/b/c")).unwrap();
        fs::create_dir_all(root.join("d")).unwrap();
        fs::write(root.join("a/file1.txt"), "hello").unwrap();
        fs::write(root.join("a/b/file2.py"), "world").unwrap();
        fs::write(root.join("a/b/c/file3.txt"), "deep").unwrap();
        fs::write(root.join("d/file4.rs"), "rust").unwrap();
        fs::write(root.join("top.txt"), "top").unwrap();
        fs::write(root.join("extra.log"), "logs").unwrap();

        tmp
    }

    #[test]
    fn test_collect_all() {
        let tmp = create_test_tree();
        let opts = WalkOptions {
            max_depth: None,
            follow_symlinks: false,
            sort: true,
            skip_hidden: false,
            file_type: FileTypeFilter::Any,
            glob_matcher: None,
        };

        let results = collect_walk(tmp.path().to_path_buf(), opts);
        // 4 dirs (a, a/b, a/b/c, d) + 6 files = 10 entries
        assert_eq!(results.len(), 10, "got {} entries", results.len());
    }

    #[test]
    fn test_collect_files_only() {
        let tmp = create_test_tree();
        let opts = WalkOptions {
            max_depth: None,
            follow_symlinks: false,
            sort: true,
            skip_hidden: false,
            file_type: FileTypeFilter::File,
            glob_matcher: None,
        };

        let results = collect_walk(tmp.path().to_path_buf(), opts);
        assert!(results.iter().all(|e| e.is_file));
        assert_eq!(results.len(), 6);
    }

    #[test]
    fn test_skip_hidden() {
        let tmp = create_test_tree();
        let opts = WalkOptions {
            max_depth: None,
            follow_symlinks: false,
            sort: true,
            skip_hidden: true,
            file_type: FileTypeFilter::File,
            glob_matcher: None,
        };

        let results = collect_walk(tmp.path().to_path_buf(), opts);
        // skip_hidden filters dotfiles from results
        assert!(results.iter().all(|e| {
            let name = std::path::Path::new(&e.path)
                .file_name()
                .unwrap()
                .to_string_lossy();
            !name.starts_with('.')
        }));
        assert_eq!(results.len(), 6);
    }

    #[test]
    fn test_glob_filter() {
        let tmp = create_test_tree();
        let glob = Glob::new("*.txt").unwrap().compile_matcher();
        let opts = WalkOptions {
            max_depth: None,
            follow_symlinks: false,
            sort: true,
            skip_hidden: false,
            file_type: FileTypeFilter::File,
            glob_matcher: Some(glob),
        };

        let results = collect_walk(tmp.path().to_path_buf(), opts);
        assert!(results.iter().all(|e| e.path.ends_with(".txt")));
        assert_eq!(results.len(), 3); // file1.txt, file3.txt, top.txt
    }

    #[test]
    fn test_max_depth() {
        let tmp = create_test_tree();
        let opts = WalkOptions {
            max_depth: Some(1),
            follow_symlinks: false,
            sort: true,
            skip_hidden: false,
            file_type: FileTypeFilter::Any,
            glob_matcher: None,
        };

        let results = collect_walk(tmp.path().to_path_buf(), opts);
        assert!(results.iter().all(|e| e.depth <= 1));
    }
}
