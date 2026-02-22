use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use digest::Digest;
use pyo3::prelude::*;
use rayon::prelude::*;

use crate::errors::FsError;
use crate::utils::{mmap_file, MMAP_THRESHOLD};

/// Result of hashing a file.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct HashResult {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub hash_hex: String,
    #[pyo3(get)]
    pub algorithm: String,
    #[pyo3(get)]
    pub file_size: u64,
}

#[pymethods]
impl HashResult {
    fn __repr__(&self) -> String {
        format!(
            "HashResult({:?}, {}={}, {}B)",
            self.path,
            self.algorithm,
            &self.hash_hex[..16.min(self.hash_hex.len())],
            self.file_size
        )
    }

    fn __eq__(&self, other: &HashResult) -> bool {
        self.hash_hex == other.hash_hex && self.algorithm == other.algorithm
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash_hex.hash(&mut hasher);
        self.algorithm.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Copy)]
pub enum Algorithm {
    Sha256,
    Blake3,
}

impl Algorithm {
    pub fn from_str(s: &str) -> Result<Self, FsError> {
        match s {
            "sha256" => Ok(Algorithm::Sha256),
            "blake3" => Ok(Algorithm::Blake3),
            other => Err(FsError::Hash(format!(
                "unknown algorithm {:?}, expected \"sha256\" or \"blake3\"",
                other
            ))),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Sha256 => "sha256",
            Algorithm::Blake3 => "blake3",
        }
    }
}

/// Hash a single file.
#[pyfunction]
#[pyo3(signature = (path, *, algorithm="blake3", chunk_size=1_048_576))]
pub fn hash_file(
    py: Python<'_>,
    path: &str,
    algorithm: &str,
    chunk_size: usize,
) -> PyResult<HashResult> {
    let algo = Algorithm::from_str(algorithm)?;
    let file_path = PathBuf::from(path);

    let result = py.allow_threads(|| hash_file_internal(&file_path, algo, chunk_size))?;
    Ok(result)
}

/// Hash multiple files in parallel using rayon.
#[pyfunction]
#[pyo3(signature = (paths, *, algorithm="blake3", chunk_size=1_048_576, max_workers=None, callback=None))]
pub fn hash_files(
    py: Python<'_>,
    paths: Vec<String>,
    algorithm: &str,
    chunk_size: usize,
    max_workers: Option<usize>,
    callback: Option<PyObject>,
) -> PyResult<Vec<HashResult>> {
    let algo = Algorithm::from_str(algorithm)?;
    let file_paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();

    if let Some(workers) = max_workers {
        // Build a custom rayon pool if max_workers is specified
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .map_err(|e| FsError::Hash(format!("failed to create thread pool: {}", e)))?;

        let results: Vec<Result<HashResult, FsError>> = py.allow_threads(|| {
            pool.install(|| {
                file_paths
                    .par_iter()
                    .map(|p| hash_file_internal(p, algo, chunk_size))
                    .collect()
            })
        });

        process_results(py, results, callback)
    } else {
        let results: Vec<Result<HashResult, FsError>> = py.allow_threads(|| {
            file_paths
                .par_iter()
                .map(|p| hash_file_internal(p, algo, chunk_size))
                .collect()
        });

        process_results(py, results, callback)
    }
}

fn process_results(
    py: Python<'_>,
    results: Vec<Result<HashResult, FsError>>,
    callback: Option<PyObject>,
) -> PyResult<Vec<HashResult>> {
    let mut out = Vec::with_capacity(results.len());

    for result in results {
        match result {
            Ok(hr) => {
                if let Some(ref cb) = callback {
                    let py_hr = Py::new(py, hr.clone())?;
                    cb.call1(py, (py_hr,))?;
                }
                out.push(hr);
            }
            Err(e) => {
                log::warn!("hash error: {}", e);
            }
        }
    }

    Ok(out)
}

/// Internal hash function that works entirely in Rust (no GIL needed).
pub fn hash_file_internal(
    path: &Path,
    algorithm: Algorithm,
    chunk_size: usize,
) -> Result<HashResult, FsError> {
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();

    let hash_hex = if file_size > MMAP_THRESHOLD {
        // Use mmap for large files
        let file = File::open(path)?;
        let mmap = mmap_file(&file)?;
        hash_bytes(&mmap, algorithm)
    } else {
        // Use buffered reads for smaller files
        hash_buffered(path, algorithm, chunk_size)?
    };

    Ok(HashResult {
        path: path.to_string_lossy().into_owned(),
        hash_hex,
        algorithm: algorithm.name().to_string(),
        file_size,
    })
}

fn hash_bytes(data: &[u8], algorithm: Algorithm) -> String {
    match algorithm {
        Algorithm::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        Algorithm::Blake3 => {
            let hash = blake3::hash(data);
            hash.to_hex().to_string()
        }
    }
}

fn hash_buffered(path: &Path, algorithm: Algorithm, chunk_size: usize) -> Result<String, FsError> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(chunk_size, file);
    let mut buf = vec![0u8; chunk_size];

    match algorithm {
        Algorithm::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        Algorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hasher.finalize().to_hex().to_string())
        }
    }
}

/// Hash the first and last `size` bytes of a file (for dedup partial hashing).
pub fn partial_hash(path: &Path, algorithm: Algorithm, size: usize) -> Result<String, FsError> {
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();

    if file_size <= (size * 2) as u64 {
        // File is small enough to hash entirely
        return hash_buffered(path, algorithm, size);
    }

    let mut file = File::open(path)?;
    let mut head = vec![0u8; size];
    let mut tail = vec![0u8; size];

    // Read head
    file.read_exact(&mut head)?;

    // Read tail
    file.seek(SeekFrom::End(-(size as i64)))?;
    file.read_exact(&mut tail)?;

    // Hash the concatenation of head + tail
    match algorithm {
        Algorithm::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&head);
            hasher.update(&tail);
            Ok(format!("{:x}", hasher.finalize()))
        }
        Algorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&head);
            hasher.update(&tail);
            Ok(hasher.finalize().to_hex().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_blake3_known_value() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello world").unwrap();
        f.flush().unwrap();
        let result = hash_file_internal(f.path(), Algorithm::Blake3, 1024).unwrap();
        assert_eq!(result.algorithm, "blake3");
        assert_eq!(
            result.hash_hex,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
        assert_eq!(result.file_size, 11);
    }

    #[test]
    fn test_sha256_known_value() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello world").unwrap();
        f.flush().unwrap();
        let result = hash_file_internal(f.path(), Algorithm::Sha256, 1024).unwrap();
        assert_eq!(result.algorithm, "sha256");
        assert_eq!(
            result.hash_hex,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_empty_file() {
        let f = NamedTempFile::new().unwrap();
        let result = hash_file_internal(f.path(), Algorithm::Blake3, 1024).unwrap();
        assert_eq!(result.file_size, 0);
        assert!(!result.hash_hex.is_empty());
    }

    #[test]
    fn test_partial_hash_small_file() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"tiny").unwrap();
        f.flush().unwrap();
        // For files smaller than 2*size, partial_hash falls back to full hash
        let partial = partial_hash(f.path(), Algorithm::Blake3, 4096).unwrap();
        let full = hash_file_internal(f.path(), Algorithm::Blake3, 4096).unwrap();
        assert_eq!(partial, full.hash_hex);
    }

    #[test]
    fn test_partial_hash_large_file() {
        let mut f = NamedTempFile::new().unwrap();
        let data = vec![0u8; 16384]; // 16KB
        f.write_all(&data).unwrap();
        f.flush().unwrap();
        let result = partial_hash(f.path(), Algorithm::Blake3, 4096).unwrap();
        assert!(!result.is_empty());
    }
}
