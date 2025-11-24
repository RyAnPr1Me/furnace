//! Input handling module for terminal
//!
//! Extracted from the main Terminal struct to improve modularity.
//! Handles keyboard and mouse input processing.
//!
//! # Future Use
//! These functions are designed to be integrated with the main Terminal
//! when further refactoring is performed.

use anyhow::Result;
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use tracing::{info, warn};

use crate::config::Config;
use crate::progress_bar::ProgressBar;
use crate::shell::ShellSession;
use crate::translator::CommandTranslator;
use crate::url_handler::UrlHandler;

/// URL cache refresh interval in frames (at 170 FPS, 30 frames ≈ 176ms)
const URL_CACHE_REFRESH_FRAMES: u64 = 30;

/// Backspace buffer initial capacity for typical command lengths
const BACKSPACE_BUFFER_CAPACITY: usize = 256;

/// Input handler state that can be shared with the terminal
#[derive(Debug)]
#[allow(dead_code)] // Public API for future refactoring
pub struct InputState {
    /// Current command buffer for each session
    pub command_buffers: Vec<String>,
    /// Translation notification message and timeout
    pub translation_notification: Option<String>,
    pub notification_frames: u64,
    /// Cached URL positions to avoid re-parsing on every mouse event
    pub cached_urls: Vec<crate::url_handler::DetectedUrl>,
    /// Track when URL cache was last updated (frame counter)
    pub url_cache_frame: u64,
    /// Reusable backspace buffer to avoid allocations
    pub backspace_buffer: Vec<u8>,
}

impl InputState {
    /// Create a new input state
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_buffers: Vec::with_capacity(8),
            translation_notification: None,
            notification_frames: 0,
            cached_urls: Vec::new(),
            url_cache_frame: 0,
            backspace_buffer: Vec::with_capacity(BACKSPACE_BUFFER_CAPACITY),
        }
    }

    /// Add a command buffer for a new session
    #[allow(dead_code)] // Public API
    pub fn add_session(&mut self) {
        self.command_buffers.push(String::new());
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle mouse events for URL clicking
///
/// Returns true if the event was handled and the terminal should be marked dirty
#[allow(dead_code)] // Public API for future refactoring
pub async fn handle_mouse_event(
    mouse: MouseEvent,
    config: &Config,
    output_buffers: &[Vec<u8>],
    active_session: usize,
    input_state: &mut InputState,
    frame_count: u64,
) -> Result<bool> {
    // Only handle Ctrl+Click for URLs if URL handler is enabled
    if !config.url_handler.enabled {
        return Ok(false);
    }

    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
        if mouse.modifiers.contains(KeyModifiers::CONTROL) {
            // Update URL cache if output has changed (every 30 frames or ~176ms at 170fps)
            if frame_count - input_state.url_cache_frame > URL_CACHE_REFRESH_FRAMES {
                if let Some(buffer) = output_buffers.get(active_session) {
                    let text = String::from_utf8_lossy(buffer);
                    input_state.cached_urls = UrlHandler::detect_urls(&text);
                    input_state.url_cache_frame = frame_count;
                }
            }

            // Use cached URLs instead of re-parsing
            if !input_state.cached_urls.is_empty() {
                // For now, just open the first URL found
                // TODO: A full implementation would map click coordinates to text positions
                if let Some(url) = input_state.cached_urls.first() {
                    info!("Opening URL: {}", url.url);
                    if let Err(e) = UrlHandler::open_url(&url.url) {
                        warn!("Failed to open URL: {}", e);
                    }
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}

/// Process a regular character input
#[allow(dead_code)] // Public API for future refactoring
pub async fn process_char_input(
    c: char,
    modifiers: KeyModifiers,
    session: &ShellSession,
    command_buffer: &mut String,
) -> Result<()> {
    let mut input = vec![c as u8];

    // Handle modifiers
    if modifiers.contains(KeyModifiers::CONTROL) && c.is_ascii_alphabetic() {
        // Send control character
        let ctrl_char = (c.to_ascii_uppercase() as u8) - b'A' + 1;
        input = vec![ctrl_char];
    } else if !modifiers.contains(KeyModifiers::CONTROL) {
        // Track normal character input for command translation
        command_buffer.push(c);
    }

    session.write_input(&input).await?;
    Ok(())
}

/// Process Enter key with command translation
#[allow(dead_code)] // Public API for future refactoring
pub async fn process_enter(
    session: &ShellSession,
    command_buffer: &mut String,
    translator: &CommandTranslator,
    config: &Config,
    backspace_buffer: &mut Vec<u8>,
    progress_bar: &mut ProgressBar,
) -> Result<Option<String>> {
    let command = command_buffer.as_str();

    // Attempt translation
    let result = translator.translate(command);
    let mut notification = None;

    if result.translated {
        // Command was translated - send translated version
        info!(
            "Translated '{}' to '{}'",
            result.original_command, result.final_command
        );

        // Show notification if enabled
        if config.command_translation.show_notifications {
            notification = Some(format!(
                "Translated: {} → {}",
                result.original_command, result.final_command
            ));
        }

        // Clear the shell's input line and send the translated command
        // Count Unicode characters properly
        let char_count = command.chars().count();

        // Reuse backspace buffer to avoid allocation
        backspace_buffer.clear();
        backspace_buffer.resize(char_count, 127);
        session.write_input(backspace_buffer).await?;

        // Then send the translated command
        session.write_input(result.final_command.as_bytes()).await?;
    }

    // Send Enter
    session.write_input(b"\r").await?;

    // Start progress bar for the command
    let command_to_track = if result.translated {
        result.final_command.clone()
    } else {
        command.to_string()
    };

    if !command_to_track.trim().is_empty() {
        progress_bar.start(command_to_track);
    }

    // Clear command buffer
    command_buffer.clear();

    Ok(notification)
}
