//! Safe file reading with size limits

/// Result of attempting to read a file
#[derive(Debug)]
pub enum ReadResult {
    /// File was read successfully
    Content(String),
}

/// File reader with safety checks
pub struct FileReader;
