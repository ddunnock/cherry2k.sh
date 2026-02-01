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
//! - [`proposal`] - Extract file write proposals from AI responses
//! - [`scope`] - Project scope detection and validation
//! - [`security`] - Secrets detection and path validation

mod detector;
mod diff;
mod proposal;
mod reader;
mod scope;
mod security;
mod writer;

pub use detector::{detect_file_references, is_file_reference};
pub use diff::{display_new_file_preview, generate_diff, has_changes};
pub use proposal::{extract_file_proposals, FileProposal};
pub use reader::{FileReader, ReadResult};
pub use scope::{find_project_root, ProjectScope};
pub use security::{is_secrets_file, validate_write_path, ValidationResult};
pub use writer::{write_file_with_approval, write_multiple_files, WriteResult};
