//! Line-buffered output for streaming responses
//!
//! Provides smooth streaming output by buffering until complete lines
//! are available, then printing whole lines at once. This prevents
//! character-by-character output which can appear janky.

use std::io::{self, Stdout, Write};

/// Line-buffered writer for streaming AI responses.
///
/// Buffers incoming chunks until a newline is encountered, then prints
/// the complete line. This provides smoother visual output compared to
/// character-by-character printing.
///
/// # Example
///
/// ```no_run
/// use cherry2k::output::StreamWriter;
///
/// let mut writer = StreamWriter::new();
/// writer.write_chunk("Hello, ").unwrap();
/// writer.write_chunk("world!\n").unwrap();  // Prints "Hello, world!"
/// writer.write_chunk("Partial").unwrap();   // Buffered
/// writer.flush().unwrap();                   // Prints "Partial"
/// ```
pub struct StreamWriter {
    buffer: String,
    stdout: Stdout,
}

impl StreamWriter {
    /// Create a new line-buffered stream writer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            stdout: io::stdout(),
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
            write!(self.stdout, "{line}")?;
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
            write!(self.stdout, "{remaining}")?;
            self.stdout.flush()?;
        }
        Ok(())
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
}
