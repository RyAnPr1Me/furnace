//! Command Progress Bar
//!
//! Tracks and displays progress of running commands in the terminal.
//!
//! # Features
//! - Visual progress indicator for long-running commands
//! - Spinner animation while command is executing
//! - Elapsed time display
//! - Command name display

use std::time::{Duration, Instant};

/// Progress bar state for tracking command execution
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// Whether progress bar is currently visible
    pub visible: bool,
    /// Command being executed (stored without allocation when possible)
    command: String,
    /// When the command started
    start_time: Option<Instant>,
    /// Current spinner frame (for animation)
    spinner_frame: usize,
    /// Cached elapsed seconds to avoid repeated formatting (Bug #17)
    cached_elapsed_secs: u64,
}

/// Bug #15: ASCII spinner characters that work on all terminals including Windows Conhost
const SPINNER_CHARS: &[char] = &['|', '/', '-', '\\'];

impl ProgressBar {
    /// Create a new progress bar
    #[must_use]
    pub fn new() -> Self {
        Self {
            visible: false,
            command: String::new(),
            start_time: None,
            spinner_frame: 0,
            cached_elapsed_secs: 0,
        }
    }

    /// Start tracking a command (Bug #24: takes &str to avoid clone)
    pub fn start_ref(&mut self, command: &str) {
        self.visible = true;
        self.command.clear();
        self.command.push_str(command);
        self.start_time = Some(Instant::now());
        self.spinner_frame = 0;
        self.cached_elapsed_secs = 0;
    }

    /// Start tracking a command (legacy API, takes ownership)
    pub fn start(&mut self, command: String) {
        self.visible = true;
        self.command = command;
        self.start_time = Some(Instant::now());
        self.spinner_frame = 0;
        self.cached_elapsed_secs = 0;
    }

    /// Stop tracking and hide progress bar
    pub fn stop(&mut self) {
        self.visible = false;
        self.command.clear();
        self.start_time = None;
        self.spinner_frame = 0;
        self.cached_elapsed_secs = 0;
    }

    /// Update spinner animation
    pub fn tick(&mut self) {
        if self.visible {
            self.spinner_frame = (self.spinner_frame + 1) % SPINNER_CHARS.len();
            // Update cached elapsed time
            if let Some(start) = self.start_time {
                self.cached_elapsed_secs = start.elapsed().as_secs();
            }
        }
    }

    /// Get current spinner character (Bug #15: ASCII-safe)
    #[must_use]
    pub fn spinner_char(&self) -> char {
        SPINNER_CHARS[self.spinner_frame]
    }

    /// Get elapsed time as formatted string (Bug #17: uses cached value)
    #[must_use]
    pub fn elapsed(&self) -> String {
        format_duration_secs(self.cached_elapsed_secs)
    }

    /// Get display text for progress bar
    #[must_use]
    pub fn display_text(&self) -> String {
        if self.visible {
            format!(
                "{} Running: {} ({})",
                self.spinner_char(),
                self.command,
                self.elapsed()
            )
        } else {
            String::new()
        }
    }

    /// Bug #16: Get display text with truncated command
    #[must_use]
    pub fn display_text_truncated(&self, max_cmd_len: usize) -> String {
        if self.visible {
            if self.command.len() > max_cmd_len {
                format!(
                    "{} Running: {}... ({})",
                    self.spinner_char(),
                    &self.command[..max_cmd_len.saturating_sub(3)],
                    self.elapsed()
                )
            } else {
                format!(
                    "{} Running: {} ({})",
                    self.spinner_char(),
                    &self.command,
                    self.elapsed()
                )
            }
        } else {
            String::new()
        }
    }

    /// Get the command being tracked
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Format duration for display (Bug #17: takes seconds directly to avoid allocation)
fn format_duration_secs(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{minutes}m {seconds}s")
    } else {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        format!("{hours}h {minutes}m")
    }
}

/// Format duration for display (legacy API - kept for future use)
#[must_use]
pub fn _format_duration(duration: Duration) -> String {
    format_duration_secs(duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_start_stop() {
        let mut pb = ProgressBar::new();
        assert!(!pb.visible);

        pb.start("ls -la".to_string());
        assert!(pb.visible);
        assert_eq!(pb.command(), "ls -la");
        assert!(pb.start_time.is_some());

        pb.stop();
        assert!(!pb.visible);
        assert!(pb.command().is_empty());
        assert!(pb.start_time.is_none());
    }

    #[test]
    fn test_progress_bar_start_ref() {
        let mut pb = ProgressBar::new();
        pb.start_ref("git status");
        assert!(pb.visible);
        assert_eq!(pb.command(), "git status");
    }

    #[test]
    fn test_spinner_animation() {
        let mut pb = ProgressBar::new();
        pb.start("test".to_string());

        let first_char = pb.spinner_char();
        pb.tick();
        let second_char = pb.spinner_char();

        assert_ne!(first_char, second_char);
    }

    #[test]
    fn test_spinner_is_ascii() {
        // Bug #15: Ensure all spinner chars are basic ASCII
        for c in SPINNER_CHARS {
            assert!(c.is_ascii(), "Spinner char '{c}' is not ASCII");
        }
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(_format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(_format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(_format_duration(Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_truncated_display() {
        let mut pb = ProgressBar::new();
        pb.start("very-long-command-that-should-be-truncated".to_string());

        let text = pb.display_text_truncated(10);
        assert!(text.contains("very-lo..."));
    }
}
