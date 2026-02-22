use std::fs::File;

use memmap2::Mmap;

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
