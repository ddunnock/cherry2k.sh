//! Animated spinner for waiting states
//!
//! Displays an animated spinner while waiting for AI provider responses.
//! The spinner provides visual feedback that the application is working.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Animated spinner shown while waiting for AI response.
///
/// # Example
///
/// ```no_run
/// use cherry2k::output::ResponseSpinner;
///
/// let spinner = ResponseSpinner::new();
/// spinner.start();
/// // ... wait for response ...
/// spinner.stop();
/// ```
pub struct ResponseSpinner {
    progress_bar: ProgressBar,
}

impl ResponseSpinner {
    /// Create a new response spinner with default message.
    #[must_use]
    pub fn new() -> Self {
        Self::with_message("Waiting for response...")
    }

    /// Create a new response spinner with a custom message.
    #[must_use]
    pub fn with_message(message: &str) -> Self {
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("valid spinner template"),
        );
        progress_bar.set_message(message.to_string());

        Self { progress_bar }
    }

    /// Start the spinner animation.
    ///
    /// Enables a steady tick at 100ms intervals for smooth animation.
    pub fn start(&self) {
        self.progress_bar
            .enable_steady_tick(Duration::from_millis(100));
    }

    /// Stop and clear the spinner.
    ///
    /// Removes the spinner from display, leaving the terminal clean
    /// for the actual response content.
    pub fn stop(&self) {
        self.progress_bar.finish_and_clear();
    }

    /// Update the spinner message.
    pub fn set_message(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }
}

impl Default for ResponseSpinner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_can_be_created() {
        let spinner = ResponseSpinner::new();
        // Just verify it doesn't panic
        spinner.start();
        spinner.stop();
    }

    #[test]
    fn spinner_with_custom_message() {
        let spinner = ResponseSpinner::with_message("Loading...");
        spinner.start();
        spinner.set_message("Almost done...");
        spinner.stop();
    }

    #[test]
    fn spinner_default_impl() {
        let spinner = ResponseSpinner::default();
        spinner.start();
        spinner.stop();
    }
}
