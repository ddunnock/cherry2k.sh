//! File operations for Cherry2K CLI
//!
//! Provides smart file reference detection and safe file reading with size limits.
//!
//! # Modules
//!
//! - [`detector`] - Detect file references in user messages
//! - [`reader`] - Safe file reading with size and binary checks
//! - [`diff`] - Unified diff generation with colored output
//! - [`writer`] - File writing with approval flow

mod detector;
mod diff;
mod reader;
mod writer;

pub use detector::{detect_file_references, is_file_reference};
pub use diff::{display_new_file_preview, generate_diff, has_changes};
pub use reader::{FileReader, ReadResult};
pub use writer::{write_file_with_approval, write_multiple_files, WriteResult};
