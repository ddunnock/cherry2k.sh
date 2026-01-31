//! Line-buffered output for streaming responses
//!
//! Provides smooth streaming output by buffering until complete lines
//! are available, then printing whole lines at once. This prevents
//! character-by-character output which can appear janky.

use std::io::{self, Stdout, Write};

use termimad::crossterm::style::Color;

use super::retro::retro_color_scheme;

/// ANSI escape code for the retro green color (bright green, ANSI 10)
const RETRO_GREEN: &str = "\x1b[38;5;10m";

/// ANSI escape code to reset colors
const ANSI_RESET: &str = "\x1b[0m";

/// Build the color prefix for retro mode output.
///
/// Returns the ANSI escape code if the color scheme uses AnsiValue,
/// empty string otherwise.
fn color_prefix() -> &'static str {
    let colors = retro_color_scheme();
    if matches!(colors.text, Color::AnsiValue(10)) {
        RETRO_GREEN
    } else {
        ""
    }
}

/// Line-buffered writer for streaming AI responses.
///
/// Buffers incoming chunks until a newline is encountered, then prints
/// the complete line. This provides smoother visual output compared to
/// character-by-character printing.
///
/// Applies retro 8-bit green color styling to output for the classic
/// terminal aesthetic.
///
/// # Example
///
/// ```no_run
/// use cherry2k::output::StreamWriter;
///
/// let mut writer = StreamWriter::new();
/// writer.write_chunk("Hello, ").unwrap();
/// writer.write_chunk("world!\n").unwrap();  // Prints "Hello, world!" in green
/// writer.write_chunk("Partial").unwrap();   // Buffered
/// writer.flush().unwrap();                   // Prints "Partial" in green
/// ```
pub struct StreamWriter {
    buffer: String,
    stdout: Stdout,
    /// Whether retro colors are enabled
    use_retro_colors: bool,
}

impl StreamWriter {
    /// Create a new line-buffered stream writer with retro colors enabled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            stdout: io::stdout(),
            use_retro_colors: true,
        }
    }

    /// Create a new line-buffered stream writer without retro colors.
    #[must_use]
    pub fn new_plain() -> Self {
        Self {
            buffer: String::new(),
            stdout: io::stdout(),
            use_retro_colors: false,
        }
    }

    /// Write a chunk of text to the stream.
    ///
    /// Text is buffered until a newline is encountered. When a newline
    /// is found, all text up to and including the newline is printed
    /// and the buffer is drained.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to stdout fails.
    pub fn write_chunk(&mut self, chunk: &str) -> io::Result<()> {
        self.buffer.push_str(chunk);

        // Print all complete lines
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer.drain(..=newline_pos).collect::<String>();
            self.write_styled(&line)?;
            self.stdout.flush()?;
        }

        Ok(())
    }

    /// Flush any remaining buffered content.
    ///
    /// Call this after streaming is complete to ensure any partial
    /// line (without trailing newline) is printed.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to stdout fails.
    pub fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            let remaining = std::mem::take(&mut self.buffer);
            self.write_styled(&remaining)?;
            self.stdout.flush()?;
        }
        Ok(())
    }

    /// Write text with optional retro color styling.
    fn write_styled(&mut self, text: &str) -> io::Result<()> {
        if self.use_retro_colors {
            write!(self.stdout, "{}{}{}", color_prefix(), text, ANSI_RESET)
        } else {
            write!(self.stdout, "{text}")
        }
    }

    /// Check if there is buffered content that hasn't been printed.
    #[must_use]
    pub fn has_buffered_content(&self) -> bool {
        !self.buffer.is_empty()
    }
}

impl Default for StreamWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StreamWriter {
    /// Ensures ANSI reset is written on drop to prevent color bleeding.
    ///
    /// Errors are intentionally ignored because:
    /// 1. Panicking in `Drop` is problematic (can abort during unwinding)
    /// 2. There's no meaningful recovery for stdout write failures at this point
    /// 3. The worst case is color bleeding, which is cosmetic
    fn drop(&mut self) {
        if self.use_retro_colors {
            let _ = write!(self.stdout, "{}", ANSI_RESET);
            let _ = self.stdout.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_writer_can_be_created() {
        let writer = StreamWriter::new();
        assert!(!writer.has_buffered_content());
    }

    #[test]
    fn stream_writer_buffers_partial_lines() {
        let mut writer = StreamWriter::new();
        writer.write_chunk("hello").unwrap();
        assert!(writer.has_buffered_content());
    }

    #[test]
    fn stream_writer_flushes_on_newline() {
        let mut writer = StreamWriter::new();
        writer.write_chunk("hello\n").unwrap();
        // After newline, buffer should be empty
        assert!(!writer.has_buffered_content());
    }

    #[test]
    fn stream_writer_handles_multiple_lines() {
        let mut writer = StreamWriter::new();
        writer.write_chunk("line1\nline2\npartial").unwrap();
        // Only "partial" should remain in buffer
        assert!(writer.has_buffered_content());
    }

    #[test]
    fn stream_writer_flush_clears_buffer() {
        let mut writer = StreamWriter::new();
        writer.write_chunk("partial").unwrap();
        assert!(writer.has_buffered_content());
        writer.flush().unwrap();
        assert!(!writer.has_buffered_content());
    }

    #[test]
    fn stream_writer_default_impl() {
        let writer = StreamWriter::default();
        assert!(!writer.has_buffered_content());
    }

    #[test]
    fn stream_writer_plain_mode() {
        let writer = StreamWriter::new_plain();
        assert!(!writer.has_buffered_content());
        assert!(!writer.use_retro_colors);
    }

    #[test]
    fn stream_writer_retro_mode_by_default() {
        let writer = StreamWriter::new();
        assert!(writer.use_retro_colors);
    }
}
