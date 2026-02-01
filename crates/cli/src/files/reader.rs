//! Safe file reading with size limits
//!
//! Provides file reading with automatic detection of:
//! - Large files (> 50KB warning, > 500KB hard limit)
//! - Binary files (null byte detection)
//! - Read errors

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Large file threshold - files above this size trigger a warning (50KB)
pub const LARGE_FILE_THRESHOLD: u64 = 50_000;

/// Maximum file size - files above this size are rejected (500KB)
pub const MAX_FILE_SIZE: u64 = 500_000;

/// Number of bytes to check for binary content detection (8KB)
const BINARY_CHECK_BYTES: usize = 8192;

/// Result of attempting to read a file
#[derive(Debug, PartialEq, Eq)]
pub enum ReadResult {
    /// File was read successfully
    Content(String),

    /// File exceeds the maximum size limit
    TooLarge { path: PathBuf, size: u64 },

    /// File appears to be binary (contains null bytes)
    Binary { path: PathBuf },

    /// File read failed
    Error { path: PathBuf, error: String },
}

/// File reader with safety checks
pub struct FileReader;

impl FileReader {
    /// Read a file with full safety checks.
    ///
    /// Performs the following checks:
    /// 1. File size validation (rejects > MAX_FILE_SIZE)
    /// 2. Binary content detection (null byte check)
    /// 3. Safe UTF-8 text reading
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use cherry2k::files::{FileReader, ReadResult};
    ///
    /// let result = FileReader::read_file(Path::new("main.rs"));
    /// match result {
    ///     Ok(ReadResult::Content(text)) => println!("File content: {}", text),
    ///     Ok(ReadResult::TooLarge { path, size }) => {
    ///         println!("File {} is too large: {} bytes", path.display(), size);
    ///     }
    ///     Ok(ReadResult::Binary { path }) => {
    ///         println!("File {} is binary", path.display());
    ///     }
    ///     Ok(ReadResult::Error { path, error }) => {
    ///         println!("Error reading {}: {}", path.display(), error);
    ///     }
    ///     Err(e) => println!("IO error: {}", e),
    /// }
    /// ```
    pub fn read_file(path: &Path) -> Result<ReadResult, io::Error> {
        let path_buf = path.to_path_buf();

        // Check file size
        let size = Self::check_file_size(path)?;
        if size > MAX_FILE_SIZE {
            return Ok(ReadResult::TooLarge {
                path: path_buf,
                size,
            });
        }

        // Check if binary
        if Self::is_binary(path)? {
            return Ok(ReadResult::Binary { path: path_buf });
        }

        // Read as text
        match fs::read_to_string(path) {
            Ok(content) => Ok(ReadResult::Content(content)),
            Err(e) => Ok(ReadResult::Error {
                path: path_buf,
                error: e.to_string(),
            }),
        }
    }

    /// Read a file without size or binary checks.
    ///
    /// Use this when you know the file is safe to read (e.g., config files).
    pub fn read_file_unchecked(path: &Path) -> Result<String, io::Error> {
        fs::read_to_string(path)
    }

    /// Get the size of a file in bytes.
    pub fn check_file_size(path: &Path) -> Result<u64, io::Error> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }

    /// Check if a file is likely binary.
    ///
    /// Uses two heuristics:
    /// 1. File extension check (common binary extensions)
    /// 2. Null byte detection in first BINARY_CHECK_BYTES bytes
    pub fn is_binary(path: &Path) -> Result<bool, io::Error> {
        // Check extension first (fast path)
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            let binary_extensions = [
                "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", // Images
                "mp3", "mp4", "wav", "avi", "mov", "mkv", "flac", // Media
                "exe", "dll", "so", "dylib", "bin", // Executables
                "zip", "tar", "gz", "7z", "rar", "bz2", // Archives
                "pdf", "doc", "docx", "xls", "xlsx", // Documents
                "wasm", "o", "a", // Compiled code
            ];

            if binary_extensions.contains(&ext_str.as_str()) {
                return Ok(true);
            }
        }

        // Read first BINARY_CHECK_BYTES and check for null bytes
        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0u8; BINARY_CHECK_BYTES];
        let bytes_read = file.read(&mut buffer)?;

        // Check for null bytes in the read portion
        Ok(buffer[..bytes_read].contains(&0))
    }

    /// Check if a file is considered large.
    ///
    /// Returns true if file size exceeds LARGE_FILE_THRESHOLD.
    pub fn is_large(path: &Path) -> Result<bool, io::Error> {
        let size = Self::check_file_size(path)?;
        Ok(size > LARGE_FILE_THRESHOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn reads_small_text_file() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!").unwrap();

        let result = FileReader::read_file(&file_path).unwrap();
        match result {
            ReadResult::Content(content) => {
                assert_eq!(content, "Hello, world!");
            }
            _ => panic!("Expected Content variant"),
        }
    }

    #[test]
    fn detects_too_large_file() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("large.txt");

        // Create a file larger than MAX_FILE_SIZE
        let large_content = "x".repeat((MAX_FILE_SIZE + 1000) as usize);
        fs::write(&file_path, large_content).unwrap();

        let result = FileReader::read_file(&file_path).unwrap();
        match result {
            ReadResult::TooLarge { path, size } => {
                assert!(path.ends_with("large.txt"));
                assert!(size > MAX_FILE_SIZE);
            }
            _ => panic!("Expected TooLarge variant"),
        }
    }

    #[test]
    fn detects_binary_file_by_null_bytes() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("binary.dat");

        // Write binary content with null bytes
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"Hello\x00World\x00Binary").unwrap();

        let result = FileReader::read_file(&file_path).unwrap();
        match result {
            ReadResult::Binary { path } => {
                assert!(path.ends_with("binary.dat"));
            }
            _ => panic!("Expected Binary variant, got {:?}", result),
        }
    }

    #[test]
    fn detects_binary_file_by_extension() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("image.png");
        fs::write(&file_path, "not really png but has extension").unwrap();

        let is_binary = FileReader::is_binary(&file_path).unwrap();
        assert!(is_binary);
    }

    #[test]
    fn returns_error_for_nonexistent_file() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = FileReader::read_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn check_file_size_works() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("sized.txt");
        fs::write(&file_path, "12345").unwrap();

        let size = FileReader::check_file_size(&file_path).unwrap();
        assert_eq!(size, 5);
    }

    #[test]
    fn is_large_detects_large_files() {
        let temp_dir = setup_test_dir();

        // Small file
        let small_path = temp_dir.path().join("small.txt");
        fs::write(&small_path, "small").unwrap();
        assert!(!FileReader::is_large(&small_path).unwrap());

        // Large file
        let large_path = temp_dir.path().join("large.txt");
        let large_content = "x".repeat((LARGE_FILE_THRESHOLD + 1000) as usize);
        fs::write(&large_path, large_content).unwrap();
        assert!(FileReader::is_large(&large_path).unwrap());
    }

    #[test]
    fn read_file_unchecked_reads_without_checks() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("unchecked.txt");
        fs::write(&file_path, "test content").unwrap();

        let content = FileReader::read_file_unchecked(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn is_binary_rejects_text_files() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("text.rs");
        fs::write(&file_path, "fn main() { println!(\"Hello\"); }").unwrap();

        let is_binary = FileReader::is_binary(&file_path).unwrap();
        assert!(!is_binary);
    }

    #[test]
    fn multiple_binary_extensions_detected() {
        let temp_dir = setup_test_dir();

        let extensions = ["exe", "dll", "zip", "mp3", "jpg"];
        for ext in &extensions {
            let file_path = temp_dir.path().join(format!("file.{}", ext));
            fs::write(&file_path, "content").unwrap();

            let is_binary = FileReader::is_binary(&file_path).unwrap();
            assert!(is_binary, "Extension .{} should be detected as binary", ext);
        }
    }
}
