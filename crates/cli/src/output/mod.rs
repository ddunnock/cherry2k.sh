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
//! - [`display_suggested_command`] - Command display with bash syntax highlighting

mod command_display;
mod error_box;
mod markdown;
mod retro;
mod spinner;
mod stream_writer;

pub use command_display::display_suggested_command;
pub use error_box::{display_error, display_provider_error};
pub use markdown::render_markdown;
pub use retro::{RetroColors, apply_retro_skin, retro_color_scheme};
pub use spinner::ResponseSpinner;
pub use stream_writer::StreamWriter;
