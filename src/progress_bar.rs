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
    /// Command being executed
    pub command: String,
    /// When the command started
    pub start_time: Option<Instant>,
    /// Current spinner frame (for animation)
    pub spinner_frame: usize,
    /// Spinner characters
    spinner_chars: Vec<char>,
}

impl ProgressBar {
    /// Create a new progress bar
    #[must_use]
    pub fn new() -> Self {
        Self {
            visible: false,
            command: String::new(),
            start_time: None,
            spinner_frame: 0,
            spinner_chars: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
        }
    }

    /// Start tracking a command
    pub fn start(&mut self, command: String) {
        self.visible = true;
        self.command = command;
        self.start_time = Some(Instant::now());
        self.spinner_frame = 0;
    }

    /// Stop tracking and hide progress bar
    pub fn stop(&mut self) {
        self.visible = false;
        self.command.clear();
        self.start_time = None;
        self.spinner_frame = 0;
    }

    /// Update spinner animation
    pub fn tick(&mut self) {
        if self.visible {
            self.spinner_frame = (self.spinner_frame + 1) % self.spinner_chars.len();
        }
    }

    /// Get current spinner character
    #[must_use]
    pub fn spinner_char(&self) -> char {
        self.spinner_chars[self.spinner_frame]
    }

    /// Get elapsed time as formatted string
    #[must_use]
    pub fn elapsed(&self) -> String {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            format_duration(elapsed)
        } else {
            String::from("0s")
        }
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
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Format duration for display
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_start_stop() {
        let mut pb = ProgressBar::new();
        assert!(!pb.visible);

        pb.start("ls -la".to_string());
        assert!(pb.visible);
        assert_eq!(pb.command, "ls -la");
        assert!(pb.start_time.is_some());

        pb.stop();
        assert!(!pb.visible);
        assert!(pb.command.is_empty());
        assert!(pb.start_time.is_none());
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
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
    }
}
