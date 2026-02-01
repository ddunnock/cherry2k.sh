//! File operations for Cherry2K CLI
//!
//! Provides smart file reference detection and safe file reading with size limits.
//!
//! # Modules
//!
//! - [`detector`] - Detect file references in user messages
//! - [`reader`] - Safe file reading with size and binary checks

mod detector;
mod reader;

pub use detector::{detect_file_references, is_file_reference};
pub use reader::{FileReader, ReadResult};
