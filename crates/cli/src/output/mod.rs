//! Terminal output utilities for Cherry2K CLI
//!
//! This module provides user-facing output components:
//! - [`ResponseSpinner`] - Animated waiting indicator while awaiting AI response
//! - [`StreamWriter`] - Line-buffered output for streaming responses
//! - [`display_error`] - Boxed error display for generic errors
//! - [`display_provider_error`] - Boxed error display with ProviderError-specific guidance
//! - [`render_markdown`] - Terminal markdown rendering with plain mode
//! - [`retro_color_scheme`] - 8-bit retro color palette for terminal output
//! - [`apply_retro_skin`] - Apply retro colors to markdown rendering

mod error_box;
mod markdown;
mod retro;
mod spinner;
mod stream_writer;

pub use error_box::{display_error, display_provider_error};
pub use markdown::render_markdown;
pub use retro::{apply_retro_skin, retro_color_scheme, RetroColors};
pub use spinner::ResponseSpinner;
pub use stream_writer::StreamWriter;
