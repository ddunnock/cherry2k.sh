//! Terminal output utilities for Cherry2K CLI
//!
//! This module provides user-facing output components:
//! - [`ResponseSpinner`] - Animated waiting indicator while awaiting AI response
//! - [`StreamWriter`] - Line-buffered output for streaming responses
//! - [`display_error`] - Boxed error display with actionable guidance
//! - [`render_markdown`] - Terminal markdown rendering with plain mode

mod error_box;
mod markdown;
mod spinner;
mod stream_writer;

pub use error_box::display_error;
pub use markdown::render_markdown;
pub use spinner::ResponseSpinner;
pub use stream_writer::StreamWriter;
