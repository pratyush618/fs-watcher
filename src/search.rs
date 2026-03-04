use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use pyo3::prelude::*;
use rayon::prelude::*;
use regex::Regex;

use crate::errors::FsError;
use crate::utils::WalkFilter;

/// A single match within a file.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct SearchMatch {
    #[pyo3(get)]
    pub line_number: usize,
    #[pyo3(get)]
    pub line_text: String,
    #[pyo3(get)]
    pub match_start: usize,
    #[pyo3(get)]
    pub match_end: usize,
    #[pyo3(get)]
    pub context_before: Vec<String>,
    #[pyo3(get)]
    pub context_after: Vec<String>,
}

#[pymethods]
impl SearchMatch {
    fn __repr__(&self) -> String {
        format!(
            "SearchMatch(line={}, col={}..{})",
            self.line_number, self.match_start, self.match_end
        )
    }
}

/// All matches found in a single file.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct SearchResult {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub matches: Vec<SearchMatch>,
    #[pyo3(get)]
    pub match_count: usize,
}

#[pymethods]
impl SearchResult {
    fn __repr__(&self) -> String {
        format!(
            "SearchResult({:?}, {} matches)",
            self.path, self.match_count
        )
    }

    fn __len__(&self) -> usize {
        self.match_count
    }
}

/// Streaming iterator for search results.
#[pyclass]
pub struct SearchIter {
    receiver: mpsc::Receiver<SearchResult>,
    done: bool,
}

#[pymethods]
impl SearchIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<SearchResult>> {
        if self.done {
            return Ok(None);
        }
        py.check_signals()?;
        match self.receiver.recv() {
            Ok(result) => Ok(Some(result)),
            Err(_) => {
                self.done = true;
                Ok(None)
            }
        }
    }
}

/// Search file contents using regex, returning all results at once.
#[pyfunction]
#[pyo3(signature = (path, pattern, *, glob_pattern=None, max_depth=None, skip_hidden=true,
                     ignore_case=false, max_count=None, max_filesize=None,
                     context_lines=0, follow_symlinks=false, max_workers=None))]
#[allow(clippy::too_many_arguments)]
pub fn search(
    py: Python<'_>,
    path: &str,
    pattern: &str,
    glob_pattern: Option<&str>,
    max_depth: Option<usize>,
    skip_hidden: bool,
    ignore_case: bool,
    max_count: Option<usize>,
    max_filesize: Option<u64>,
    context_lines: usize,
    follow_symlinks: bool,
    max_workers: Option<usize>,
) -> PyResult<Vec<SearchResult>> {
    let root = PathBuf::from(path);
    if !root.exists() {
        return Err(FsError::Search(format!("path does not exist: {}", path)).into());
    }

    let regex_pattern = if ignore_case {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    let re = Regex::new(&regex_pattern)
        .map_err(|e| FsError::Search(format!("invalid regex pattern: {}", e)))?;

    let filter = WalkFilter::from_options(
        skip_hidden,
        glob_pattern,
        max_depth,
        follow_symlinks,
        FsError::Search,
    )?;

    let results = py.allow_threads(|| {
        let files = crate::utils::walk_files_filtered(&root, &filter);

        // Filter by max_filesize
        let files: Vec<_> = files
            .into_iter()
            .filter(|(_, _, size)| {
                if let Some(max) = max_filesize {
                    *size <= max
                } else {
                    true
                }
            })
            .collect();

        let search_fn = |(_rel, abs_path, _size): &(String, PathBuf, u64)| -> Option<SearchResult> {
            search_file(abs_path, &re, context_lines, max_count)
        };

        let results: Vec<SearchResult> = if let Some(workers) = max_workers {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .ok()?;
            pool.install(|| files.par_iter().filter_map(search_fn).collect())
        } else {
            files.par_iter().filter_map(search_fn).collect()
        };

        Some(results)
    });

    Ok(results.unwrap_or_default())
}

/// Search file contents using regex, streaming results via iterator.
#[pyfunction]
#[pyo3(signature = (path, pattern, *, glob_pattern=None, max_depth=None, skip_hidden=true,
                     ignore_case=false, max_count=None, max_filesize=None,
                     context_lines=0, follow_symlinks=false, max_workers=None))]
#[allow(clippy::too_many_arguments)]
pub fn search_iter(
    py: Python<'_>,
    path: &str,
    pattern: &str,
    glob_pattern: Option<&str>,
    max_depth: Option<usize>,
    skip_hidden: bool,
    ignore_case: bool,
    max_count: Option<usize>,
    max_filesize: Option<u64>,
    context_lines: usize,
    follow_symlinks: bool,
    max_workers: Option<usize>,
) -> PyResult<SearchIter> {
    let root = PathBuf::from(path);
    if !root.exists() {
        return Err(FsError::Search(format!("path does not exist: {}", path)).into());
    }

    let regex_pattern = if ignore_case {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    let re = Regex::new(&regex_pattern)
        .map_err(|e| FsError::Search(format!("invalid regex pattern: {}", e)))?;

    let filter = WalkFilter::from_options(
        skip_hidden,
        glob_pattern,
        max_depth,
        follow_symlinks,
        FsError::Search,
    )?;

    let (sender, receiver) = mpsc::channel();

    let max_filesize_val = max_filesize;
    let max_workers_val = max_workers;

    py.allow_threads(|| {
        thread::spawn(move || {
            let files = crate::utils::walk_files_filtered(&root, &filter);

            let files: Vec<_> = files
                .into_iter()
                .filter(|(_, _, size)| {
                    if let Some(max) = max_filesize_val {
                        *size <= max
                    } else {
                        true
                    }
                })
                .collect();

            let results: Vec<SearchResult> = if let Some(workers) = max_workers_val {
                if let Ok(pool) = rayon::ThreadPoolBuilder::new().num_threads(workers).build() {
                    pool.install(|| {
                        files
                            .par_iter()
                            .filter_map(|(_rel, abs_path, _size)| {
                                search_file(abs_path, &re, context_lines, max_count)
                            })
                            .collect()
                    })
                } else {
                    return;
                }
            } else {
                files
                    .par_iter()
                    .filter_map(|(_rel, abs_path, _size)| {
                        search_file(abs_path, &re, context_lines, max_count)
                    })
                    .collect()
            };

            for result in results {
                if sender.send(result).is_err() {
                    return;
                }
            }
        });
    });

    Ok(SearchIter {
        receiver,
        done: false,
    })
}

/// Check if a file is likely binary by scanning the first 8KB for null bytes.
fn is_binary(path: &PathBuf) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return true,
    };
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 8192];
    match reader.read(&mut buf) {
        Ok(n) => buf[..n].contains(&0),
        Err(_) => true,
    }
}

/// Search a single file for regex matches.
fn search_file(
    path: &PathBuf,
    re: &Regex,
    context_lines: usize,
    max_count: Option<usize>,
) -> Option<SearchResult> {
    if is_binary(path) {
        return None;
    }

    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    let mut matches = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        if let Some(mat) = re.find(line) {
            let context_before: Vec<String> = if context_lines > 0 && idx > 0 {
                let start = idx.saturating_sub(context_lines);
                lines[start..idx].to_vec()
            } else {
                Vec::new()
            };

            let context_after: Vec<String> = if context_lines > 0 {
                let end = (idx + 1 + context_lines).min(lines.len());
                lines[idx + 1..end].to_vec()
            } else {
                Vec::new()
            };

            matches.push(SearchMatch {
                line_number: idx + 1,
                line_text: line.clone(),
                match_start: mat.start(),
                match_end: mat.end(),
                context_before,
                context_after,
            });

            if let Some(max) = max_count {
                if matches.len() >= max {
                    break;
                }
            }
        }
    }

    if matches.is_empty() {
        return None;
    }

    let match_count = matches.len();
    Some(SearchResult {
        path: path.to_string_lossy().into_owned(),
        matches,
        match_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_search_tree() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(
            root.join("hello.txt"),
            "Hello World\nfoo bar\nHello Again\n",
        )
        .unwrap();
        fs::write(root.join("code.py"), "def hello():\n    print('hello')\n").unwrap();
        fs::write(root.join("empty.txt"), "").unwrap();
        fs::write(root.join("binary.bin"), &[0u8, 1, 2, 0, 3]).unwrap();

        tmp
    }

    #[test]
    fn test_search_file_basic() {
        let tmp = create_search_tree();
        let re = Regex::new("Hello").unwrap();
        let result = search_file(&tmp.path().join("hello.txt"), &re, 0, None);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.match_count, 2);
        assert_eq!(r.matches[0].line_number, 1);
        assert_eq!(r.matches[1].line_number, 3);
    }

    #[test]
    fn test_search_file_context() {
        let tmp = create_search_tree();
        let re = Regex::new("foo").unwrap();
        let result = search_file(&tmp.path().join("hello.txt"), &re, 1, None);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.matches[0].context_before.len(), 1);
        assert_eq!(r.matches[0].context_after.len(), 1);
    }

    #[test]
    fn test_search_skips_binary() {
        let tmp = create_search_tree();
        let re = Regex::new(".*").unwrap();
        let result = search_file(&tmp.path().join("binary.bin"), &re, 0, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_search_max_count() {
        let tmp = create_search_tree();
        let re = Regex::new("Hello").unwrap();
        let result = search_file(&tmp.path().join("hello.txt"), &re, 0, Some(1));
        assert!(result.is_some());
        assert_eq!(result.unwrap().match_count, 1);
    }

    #[test]
    fn test_is_binary() {
        let tmp = create_search_tree();
        assert!(!is_binary(&tmp.path().join("hello.txt")));
        assert!(is_binary(&tmp.path().join("binary.bin")));
    }
}
