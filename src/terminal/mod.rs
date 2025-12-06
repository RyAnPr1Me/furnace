//! Terminal module for the Furnace terminal emulator
//!
//! This module contains the main Terminal struct and its supporting modules:
//! - `ansi_parser`: ANSI escape code parser for colors and styling
//!
//! # Architecture
//! The terminal is structured to separate concerns:
//! - Event loop management (main run loop)
//! - Input processing (keyboard/mouse handlers)
//! - Rendering (UI drawing)
//! - Tab/session management

pub mod ansi_parser;

use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal as RatatuiTerminal,
};
use std::borrow::Cow;
use std::io;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};
use unicode_width::UnicodeWidthStr;

use crate::colors::TrueColorPalette;
use crate::config::Config;
use crate::keybindings::KeybindingManager;
use crate::progress_bar::ProgressBar;
use crate::session::SessionManager;
use crate::shell::ShellSession;
use crate::ui::{
    autocomplete::Autocomplete, resource_monitor::ResourceMonitor,
    themes::ThemeManager,
};

use self::ansi_parser::AnsiParser;

/// Target FPS for GPU-accelerated rendering
const TARGET_FPS: u64 = 170;

/// Read buffer size optimized for typical terminal output
/// Using 4KB as it's a common page size and provides good balance
const READ_BUFFER_SIZE: usize = 4 * 1024;

/// Notification display duration in seconds
#[allow(dead_code)] // May be used for notifications feature
const NOTIFICATION_DURATION_SECS: u64 = 2;

/// Minimum popup size to prevent collapse (Bug #19)
const MIN_POPUP_WIDTH: u16 = 20;
const MIN_POPUP_HEIGHT: u16 = 5;

/// Maximum command display length in progress bar (Bug #16)
const MAX_PROGRESS_COMMAND_LEN: usize = 40;

/// Initial shell output timeout in milliseconds
const INITIAL_OUTPUT_TIMEOUT_MS: u64 = 1000;

/// Polling interval for initial output in milliseconds
const INITIAL_OUTPUT_POLL_INTERVAL_MS: u64 = 20;

/// Extra read attempts after receiving initial output
const EXTRA_READ_ATTEMPTS: usize = 5;

/// Delay between extra read attempts in milliseconds
const EXTRA_READ_DELAY_MS: u64 = 20;

/// Delay after sending newline to trigger prompt
const PROMPT_TRIGGER_DELAY_MS: u64 = 200;

/// Read attempts after sending newline to trigger prompt
const PROMPT_TRIGGER_READ_ATTEMPTS: usize = 10;

/// Delay after receiving first output to get full prompt
const INITIAL_OUTPUT_SETTLE_MS: u64 = 100;

/// Color constants for cool red/black theme
const COLOR_COOL_RED: (u8, u8, u8) = (0xDD, 0x66, 0x66); // Cool red accent
const COLOR_REDDISH_GRAY: (u8, u8, u8) = (0xC0, 0xB0, 0xB0); // Reddish-gray text
const COLOR_PURE_BLACK: (u8, u8, u8) = (0x00, 0x00, 0x00); // Pure black background
const COLOR_MUTED_GREEN: (u8, u8, u8) = (0x6A, 0x9A, 0x7A); // Muted green
const COLOR_MAGENTA_RED: (u8, u8, u8) = (0xB0, 0x5A, 0x7A); // Magenta-red
#[allow(dead_code)] // May be used for future UI features
const COLOR_DARK_GRAY: (u8, u8, u8) = (0x5A, 0x4A, 0x4A); // Dark gray for comments

/// High-performance terminal with GPU-accelerated rendering at 170 FPS
pub struct Terminal {
    config: Config,
    sessions: Vec<ShellSession>,
    active_session: usize,
    output_buffers: Vec<Vec<u8>>,
    should_quit: bool,
    resource_monitor: Option<ResourceMonitor>,
    #[allow(dead_code)] // Feature not yet implemented
    autocomplete: Option<Autocomplete>,
    show_resources: bool,
    #[allow(dead_code)]
    keybindings: KeybindingManager,
    #[allow(dead_code)] // Feature not yet implemented
    session_manager: Option<SessionManager>,
    #[allow(dead_code)]
    color_palette: TrueColorPalette,
    // Theme manager for dynamic theme switching
    #[allow(dead_code)] // May be used for future theming features
    theme_manager: Option<ThemeManager>,
    // Performance optimization: track if redraw is needed
    dirty: bool,
    // Reusable read buffer to reduce allocations
    read_buffer: Vec<u8>,
    // Frame counter for performance metrics
    frame_count: u64,
    // Current command buffer for each session - tracks BYTES sent to shell (Bug #1, #2)
    command_buffers: Vec<Vec<u8>>,
    // Notification message and timeout
    notification_message: Option<String>,
    notification_frames: u64,
    // Progress bar for command execution
    progress_bar: Option<ProgressBar>,
    // Current terminal size for proper tab creation (Bug #7)
    terminal_cols: u16,
    terminal_rows: u16,
    // Cached styled lines for zero-copy rendering (Bug #3)
    cached_styled_lines: Vec<Vec<Line<'static>>>,
    // Track buffer length when cache was built (for invalidation)
    cached_buffer_lens: Vec<usize>,
}

impl Terminal {
    /// Create a new terminal instance with optimal memory allocation
    ///
    /// # Errors
    /// Returns an error if session manager initialization fails
    pub fn new(config: Config) -> Result<Self> {
        info!("Initializing Furnace terminal emulator with 170 FPS GPU rendering + 24-bit color");

        // Initialize optional theme manager based on config
        let theme_manager = if config.features.theme_manager {
            match ThemeManager::default_themes_dir() {
                Ok(themes_dir) => match ThemeManager::with_themes_dir(&themes_dir) {
                    Ok(manager) => {
                        debug!(
                            "Theme manager initialized with custom themes from {:?}",
                            themes_dir
                        );
                        Some(manager)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to initialize theme manager with custom themes: {}",
                            e
                        );
                        Some(ThemeManager::new())
                    }
                },
                Err(e) => {
                    warn!("Could not determine themes directory: {}", e);
                    Some(ThemeManager::new())
                }
            }
        } else {
            None
        };

        // Initialize optional session manager
        let session_manager = if config.features.session_manager {
            Some(SessionManager::new()?)
        } else {
            None
        };

        // Capture feature flags before moving config
        let enable_resource_monitor = config.features.resource_monitor;
        let enable_autocomplete = config.features.autocomplete;
        let enable_progress_bar = config.features.progress_bar;

        Ok(Self {
            config,
            sessions: Vec::with_capacity(8),
            active_session: 0,
            output_buffers: Vec::with_capacity(8),
            should_quit: false,
            resource_monitor: if enable_resource_monitor {
                Some(ResourceMonitor::new())
            } else {
                None
            },
            autocomplete: if enable_autocomplete {
                Some(Autocomplete::new())
            } else {
                None
            },
            show_resources: false,
            keybindings: KeybindingManager::new(),
            session_manager,
            color_palette: TrueColorPalette::default_dark(),
            theme_manager,
            dirty: true,
            read_buffer: vec![0u8; READ_BUFFER_SIZE],
            frame_count: 0,
            command_buffers: Vec::with_capacity(8),
            notification_message: None,
            notification_frames: 0,
            progress_bar: if enable_progress_bar {
                Some(ProgressBar::new())
            } else {
                None
            },
            terminal_cols: 80,
            terminal_rows: 24,
            cached_styled_lines: Vec::with_capacity(8),
            cached_buffer_lens: Vec::with_capacity(8),
        })
    }

    /// Helper method to read shell output and store it in the buffer
    ///
    /// This function attempts to read from the shell multiple times with delays
    /// to capture all available output. This is particularly useful for:
    /// - Initial shell startup (capturing the prompt)
    /// - After sending commands (capturing output)
    /// - Handling slow or buffered output
    ///
    /// # Arguments
    /// * `max_attempts` - Maximum number of read attempts to make
    /// * `delay_ms` - Milliseconds to wait between read attempts
    ///
    /// # Returns
    /// Total number of bytes read across all attempts
    ///
    /// # Performance Note
    /// Each read is non-blocking, so this won't hang if there's no output.
    /// The delay allows time for the shell to produce output between reads.
    async fn read_and_store_output(&mut self, max_attempts: usize, delay_ms: u64) -> usize {
        let mut total_bytes = 0;

        // Safety check: Ensure both vectors are in sync to prevent index out of bounds
        // This can happen if sessions are created/destroyed but buffers aren't updated
        if self.active_session >= self.sessions.len()
            || self.active_session >= self.output_buffers.len()
        {
            warn!(
                "Active session index {} is out of bounds (sessions: {}, buffers: {})",
                self.active_session,
                self.sessions.len(),
                self.output_buffers.len()
            );
            return 0;
        }

        if let Some(session) = self.sessions.get(self.active_session) {
            for _ in 0..max_attempts {
                if let Ok(n) = session.read_output(&mut self.read_buffer).await {
                    if n > 0 {
                        self.output_buffers[self.active_session]
                            .extend_from_slice(&self.read_buffer[..n]);
                        self.dirty = true;
                        total_bytes += n;
                        debug!("Read {} bytes from shell", n);
                    }
                }
                if delay_ms > 0 {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }

        total_bytes
    }

    /// Main event loop with async I/O for maximum performance
    ///
    /// # Errors
    /// Returns an error if terminal setup, shell session creation, or event handling fails
    pub async fn run(&mut self) -> Result<()> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        // Enable mouse capture and bracketed paste mode (Bug #21)
        // Show cursor so user knows where to type
        execute!(
            stdout,
            crossterm::event::EnableMouseCapture,
            crossterm::event::EnableBracketedPaste,
            Show
        )?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = RatatuiTerminal::new(backend)?;

        // Create initial shell session with actual terminal size (Bug #7)
        let (cols, rows) = terminal.size().map(|s| (s.width, s.height))?;
        self.terminal_cols = cols;
        self.terminal_rows = rows;

        let session = ShellSession::new(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            rows,
            cols,
        )?;

        self.sessions.push(session);
        self.output_buffers.push(Vec::with_capacity(1024 * 1024));
        self.command_buffers.push(Vec::new()); // Bytes, not String (Bug #1)
        self.cached_styled_lines.push(Vec::new());
        self.cached_buffer_lens.push(0);

        info!("Terminal started with {}x{} size", cols, rows);

        // Wait for initial shell output (prompt) to ensure it's displayed
        // This prevents the blank screen issue on Windows PowerShell
        debug!("Waiting for initial shell output...");
        let initial_timeout = Duration::from_millis(INITIAL_OUTPUT_TIMEOUT_MS);
        let start_time = tokio::time::Instant::now();
        let mut received_output = false;

        // Poll for initial output with timeout
        while start_time.elapsed() < initial_timeout {
            // Try reading once
            let bytes_read = self.read_and_store_output(1, 0).await;

            if bytes_read > 0 {
                received_output = true;
                debug!("Received {} bytes of initial shell output", bytes_read);

                // Continue reading for a bit more to get the full prompt
                tokio::time::sleep(Duration::from_millis(INITIAL_OUTPUT_SETTLE_MS)).await;

                // Try to read more data that might be coming
                let additional = self
                    .read_and_store_output(EXTRA_READ_ATTEMPTS, EXTRA_READ_DELAY_MS)
                    .await;
                if additional > 0 {
                    debug!("Received additional {} bytes", additional);
                }
                break;
            }

            tokio::time::sleep(Duration::from_millis(INITIAL_OUTPUT_POLL_INTERVAL_MS)).await;
        }

        // If no output received, try sending a newline to trigger the prompt
        // This helps with shells like PowerShell that don't show a prompt until Enter is pressed
        if !received_output {
            warn!("No initial shell output received - sending newline to trigger prompt");
            if let Some(session) = self.sessions.get(self.active_session) {
                if let Err(e) = session.write_input(b"\r").await {
                    warn!("Failed to send initial newline: {}", e);
                } else {
                    // Wait a bit for the prompt to appear after sending newline
                    tokio::time::sleep(Duration::from_millis(PROMPT_TRIGGER_DELAY_MS)).await;

                    // Try reading again
                    let bytes_read = self
                        .read_and_store_output(
                            PROMPT_TRIGGER_READ_ATTEMPTS,
                            INITIAL_OUTPUT_POLL_INTERVAL_MS,
                        )
                        .await;

                    if bytes_read > 0 {
                        received_output = true;
                        debug!("Received {} bytes after sending newline", bytes_read);
                    }
                }
            }
        }

        if received_output {
            info!("Successfully captured initial shell output");
        } else {
            warn!("No initial shell output received - shell may be slow to start or not configured correctly");
        }

        // Always render the initial screen, even if empty
        // This ensures the user sees SOMETHING instead of a blank screen
        terminal.draw(|f| self.render(f))?;
        self.dirty = false;
        debug!("Initial render complete");

        // Event loop with optimized timing for TARGET_FPS
        let frame_duration = Duration::from_micros(1_000_000 / TARGET_FPS);
        let mut render_interval = interval(frame_duration);
        render_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        while !self.should_quit {
            tokio::select! {
                // Handle user input (higher priority)
                Ok(Ok(has_event)) = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(1))) => {
                    if has_event {
                        match event::read() {
                            Ok(Event::Key(key)) => {
                                self.handle_key_event(key).await?;
                                self.dirty = true;
                            }
                            Ok(Event::Mouse(mouse)) => {
                                self.handle_mouse_event(mouse).await?;
                                self.dirty = true;
                            }
                            Ok(Event::Resize(new_cols, new_rows)) => {
                                // Bug #20: Handle terminal resize
                                self.terminal_cols = new_cols;
                                self.terminal_rows = new_rows;
                                // Resize all PTYs
                                for session in &self.sessions {
                                    let _ = session.resize(new_rows, new_cols).await;
                                }
                                // Invalidate all caches
                                for len in &mut self.cached_buffer_lens {
                                    *len = 0;
                                }
                                self.dirty = true;
                            }
                            Ok(Event::Paste(text)) => {
                                // Bug #21: Handle bracketed paste - send directly without translation
                                if let Some(session) = self.sessions.get(self.active_session) {
                                    session.write_input(text.as_bytes()).await?;
                                    // Don't track pasted content in command buffer
                                }
                                self.dirty = true;
                            }
                            _ => {}
                        }
                    }
                }

                // Read shell output (non-blocking)
                _ = async {
                    if let Some(session) = self.sessions.get(self.active_session) {
                        if let Ok(n) = session.read_output(&mut self.read_buffer).await {
                            if n > 0 && self.active_session < self.output_buffers.len() {
                                self.output_buffers[self.active_session].extend_from_slice(&self.read_buffer[..n]);
                                self.dirty = true;

                                // Bug #9: Improved prompt detection for various shells
                                let should_stop_progress = if let Some(ref pb) = self.progress_bar {
                                    if pb.visible {
                                        let recent_output = String::from_utf8_lossy(&self.read_buffer[..n]);
                                        self.detect_prompt(&recent_output)
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };

                                if should_stop_progress {
                                    if let Some(ref mut pb) = self.progress_bar {
                                        pb.stop();
                                    }
                                }

                                // Bug #8: Enforce scrollback limit and clear URL cache
                                let max_buffer = self.config.terminal.scrollback_lines * 256;
                                if self.output_buffers[self.active_session].len() > max_buffer {
                                    let excess = self.output_buffers[self.active_session].len() - max_buffer;
                                    self.output_buffers[self.active_session].drain(..excess);
                                }
                            }
                        }
                    }
                } => {}

                // Render at consistent frame rate
                _ = render_interval.tick() => {
                    // Update progress bar spinner (only if visible)
                    if let Some(ref mut pb) = self.progress_bar {
                        if pb.visible {
                            pb.tick();
                            self.dirty = true;
                        }
                    }

                    // Bug #11: Only decrement notification counter when actually rendering
                    if self.dirty && self.notification_frames > 0 {
                        self.notification_frames -= 1;
                        if self.notification_frames == 0 {
                            self.notification_message = None;
                        }
                    }

                    if self.dirty {
                        terminal.draw(|f| self.render(f))?;
                        self.dirty = false;
                        self.frame_count += 1;

                        if self.frame_count.is_multiple_of(1000) {
                            debug!("Rendered {} frames", self.frame_count);
                        }
                    }
                }
            }
        }

        // Cleanup
        execute!(
            terminal.backend_mut(),
            crossterm::event::DisableMouseCapture,
            crossterm::event::DisableBracketedPaste,
            Show
        )?;
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        info!("Terminal shutdown complete");
        Ok(())
    }

    /// Bug #9: Detect shell prompts from various shells
    /// Detects shell prompts in terminal output
    ///
    /// This function identifies common shell prompt patterns to determine when
    /// a command has finished executing. It supports various shells and themes:
    ///
    /// # Supported Shells
    /// - Bash: `$ `, `# `
    /// - Zsh: `% `, `❯`, `➜`, `λ`
    /// - Fish: `❯`, `> `
    /// - PowerShell: `PS>`, `PS `
    /// - Python REPL: `>>>`, `...`
    ///
    /// # Detection Strategy
    /// 1. Check for explicit prompt characters
    /// 2. Heuristic: Short lines ending with newline are likely prompts
    ///
    /// # Arguments
    /// * `output` - Recent shell output to check for prompts
    ///
    /// # Returns
    /// `true` if a prompt pattern is detected, `false` otherwise
    fn detect_prompt(&self, output: &str) -> bool {
        // Check for common prompt patterns across different shells
        output.contains("$ ")   // Bash default
            || output.contains("> ")   // Generic shell
            || output.contains("# ")   // Root prompt
            || output.contains("% ")   // Zsh default
            || output.contains("❯")    // fish/starship
            || output.contains("➜")    // oh-my-zsh
            || output.contains("λ")    // some zsh themes
            || output.contains("PS>")  // PowerShell
            || output.contains("PS ")  // PowerShell alternative
            || output.contains(">>>")  // Python REPL
            || output.contains("...")  // Python continuation
            || (output.ends_with('\n') && output.len() < 100) // Heuristic: short line likely a prompt
    }

    /// Handle mouse events
    async fn handle_mouse_event(&mut self, _mouse: MouseEvent) -> Result<()> {
        // Mouse events currently not handled
        Ok(())
    }

    /// Handle keyboard events with optimal input processing
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            // Toggle resource monitor (Ctrl+R)
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                if self.resource_monitor.is_some() {
                    self.show_resources = !self.show_resources;
                    debug!(
                        "Resource monitor: {}",
                        if self.show_resources { "ON" } else { "OFF" }
                    );
                }
            }

            // Quit (Ctrl+C or Ctrl+D)
            (KeyCode::Char('c' | 'd'), KeyModifiers::CONTROL) => {
                debug!("Quit signal received");
                self.should_quit = true;
            }

            // New tab (Bug #7: use current terminal size)
            (KeyCode::Char('t'), KeyModifiers::CONTROL) if self.config.terminal.enable_tabs => {
                self.create_new_tab().await?;
            }

            // Next tab
            (KeyCode::Tab, KeyModifiers::CONTROL) if self.config.terminal.enable_tabs => {
                self.next_tab();
            }

            // Previous tab
            (KeyCode::BackTab, m)
                if m.contains(KeyModifiers::SHIFT) && self.config.terminal.enable_tabs =>
            {
                self.prev_tab();
            }

            // Regular character input (Bug #1: track ALL characters including shifted)
            (KeyCode::Char(c), modifiers) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    // Bug #1: Track the actual byte sent to shell, not the character
                    if modifiers.contains(KeyModifiers::CONTROL) && c.is_ascii_alphabetic() {
                        // Send control character - don't track in command buffer
                        let ctrl_char = (c.to_ascii_uppercase() as u8) - b'A' + 1;
                        session.write_input(&[ctrl_char]).await?;
                    } else {
                        // Bug #1: Track the actual character (including uppercase/symbols)
                        // Send the character as UTF-8 bytes (encode_utf8 uses stack efficiently)
                        let mut buf = [0u8; 4];
                        let s = c.encode_utf8(&mut buf);
                        session.write_input(s.as_bytes()).await?;

                        // Track bytes sent for backspace calculation (Bug #2)
                        if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                            cmd_buf.extend_from_slice(s.as_bytes());
                        }
                    }
                }
            }

            // Enter - translate command before sending
            (KeyCode::Enter, _) => {
                self.handle_enter().await?;
            }

            // Backspace (Bug #2: track byte removal properly)
            (KeyCode::Backspace, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    // Remove last UTF-8 character from command buffer (Bug #2)
                    if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                        // Pop one complete UTF-8 character from the end
                        // UTF-8 encoding: ASCII is 0xxxxxxx, lead bytes are 11xxxxxx, continuation bytes are 10xxxxxx
                        // First, pop any trailing continuation bytes (10xxxxxx pattern)
                        while let Some(&last) = cmd_buf.last() {
                            if (last & 0xC0) == 0x80 {
                                // This is a continuation byte, pop it
                                cmd_buf.pop();
                            } else {
                                // This is either ASCII or a lead byte, pop it and we're done
                                cmd_buf.pop();
                                break;
                            }
                        }
                    }
                    session.write_input(&[127]).await?;
                }
            }

            // Arrow keys - clear command buffer on history navigation
            (KeyCode::Up, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                        cmd_buf.clear();
                    }
                    session.write_input(b"\x1b[A").await?;
                }
            }
            (KeyCode::Down, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                        cmd_buf.clear();
                    }
                    session.write_input(b"\x1b[B").await?;
                }
            }
            (KeyCode::Right, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    session.write_input(b"\x1b[C").await?;
                }
            }
            (KeyCode::Left, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    session.write_input(b"\x1b[D").await?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle Enter key
    async fn handle_enter(&mut self) -> Result<()> {
        if let Some(session) = self.sessions.get(self.active_session) {
            // Get the current command as a string from bytes
            let command = self
                .command_buffers
                .get(self.active_session)
                .map_or(Cow::Borrowed(""), |b| String::from_utf8_lossy(b));

            // Send Enter
            session.write_input(b"\r").await?;

            // Start progress bar (Bug #24: avoid clone)
            if !command.trim().is_empty() {
                if let Some(ref mut pb) = self.progress_bar {
                    pb.start_ref(&command);
                    self.dirty = true;
                }
            }

            // Clear command buffer
            if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                cmd_buf.clear();
            }
        }
        Ok(())
    }

    /// Create a new tab (Bug #7: use current terminal size)
    async fn create_new_tab(&mut self) -> Result<()> {
        info!(
            "Creating new tab with size {}x{}",
            self.terminal_cols, self.terminal_rows
        );

        let session = ShellSession::new(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            self.terminal_rows, // Bug #7: use current size
            self.terminal_cols,
        )?;

        self.sessions.push(session);
        self.output_buffers.push(Vec::with_capacity(1024 * 1024));
        self.command_buffers.push(Vec::new());
        self.cached_styled_lines.push(Vec::new());
        self.cached_buffer_lens.push(0);
        self.active_session = self.sessions.len() - 1;

        Ok(())
    }

    /// Switch to next tab (Bug #8: enforce scrollback limit on switch)
    fn next_tab(&mut self) {
        if !self.sessions.is_empty() {
            // Bug #8: Enforce scrollback limit on current tab before switching
            self.enforce_scrollback_limit(self.active_session);

            self.active_session = (self.active_session + 1) % self.sessions.len();
            debug!("Switched to tab {}", self.active_session);
        }
    }

    /// Switch to previous tab (Bug #8: enforce scrollback limit on switch)
    fn prev_tab(&mut self) {
        if !self.sessions.is_empty() {
            // Bug #8: Enforce scrollback limit on current tab before switching
            self.enforce_scrollback_limit(self.active_session);

            if self.active_session == 0 {
                self.active_session = self.sessions.len() - 1;
            } else {
                self.active_session -= 1;
            }
            debug!("Switched to tab {}", self.active_session);
        }
    }

    /// Bug #8: Enforce scrollback limit on a specific tab
    fn enforce_scrollback_limit(&mut self, tab_index: usize) {
        if let Some(buffer) = self.output_buffers.get_mut(tab_index) {
            let max_buffer = self.config.terminal.scrollback_lines * 256;
            if buffer.len() > max_buffer {
                let excess = buffer.len() - max_buffer;
                buffer.drain(..excess);
                // Invalidate caches
                if let Some(len) = self.cached_buffer_lens.get_mut(tab_index) {
                    *len = 0;
                }
            }
        }
    }

    /// Render UI with hardware acceleration (Bug #3: zero-copy rendering)
    fn render(&mut self, f: &mut ratatui::Frame) {
        let progress_visible = self.progress_bar.as_ref().is_some_and(|pb| pb.visible);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(u16::from(
                    self.config.terminal.enable_tabs && self.sessions.len() > 1,
                )),
                Constraint::Length(u16::from(self.notification_message.is_some())),
                Constraint::Length(u16::from(progress_visible)),
                Constraint::Min(0),
                Constraint::Length(if self.show_resources && self.resource_monitor.is_some() {
                    3
                } else {
                    0
                }),
            ])
            .split(f.size());

        let tab_area = main_chunks[0];
        let notification_area = main_chunks[1];
        let progress_area = main_chunks[2];
        let content_area = main_chunks[3];
        let resource_area = main_chunks[4];

        // Render tabs if enabled
        if self.config.terminal.enable_tabs && self.sessions.len() > 1 {
            let tab_titles: Vec<Line> = (0..self.sessions.len())
                .map(|i| {
                    let style = if i == self.active_session {
                        Style::default()
                            .fg(Color::Rgb(
                                COLOR_COOL_RED.0,
                                COLOR_COOL_RED.1,
                                COLOR_COOL_RED.2,
                            ))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Rgb(
                            COLOR_REDDISH_GRAY.0,
                            COLOR_REDDISH_GRAY.1,
                            COLOR_REDDISH_GRAY.2,
                        ))
                    };
                    Line::from(Span::styled(format!(" Tab {} ", i + 1), style))
                })
                .collect();

            let tabs = Tabs::new(tab_titles)
                .block(Block::default().borders(Borders::BOTTOM))
                .select(self.active_session)
                .style(Style::default().fg(Color::Rgb(
                    COLOR_REDDISH_GRAY.0,
                    COLOR_REDDISH_GRAY.1,
                    COLOR_REDDISH_GRAY.2,
                )))
                .highlight_style(
                    Style::default()
                        .fg(Color::Rgb(
                            COLOR_COOL_RED.0,
                            COLOR_COOL_RED.1,
                            COLOR_COOL_RED.2,
                        ))
                        .add_modifier(Modifier::BOLD),
                );

            f.render_widget(tabs, tab_area);
        }

        // Render translation notification if present
        if let Some(ref msg) = self.notification_message {
            let notification = Paragraph::new(msg.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(
                            COLOR_MUTED_GREEN.0,
                            COLOR_MUTED_GREEN.1,
                            COLOR_MUTED_GREEN.2,
                        ))
                        .bg(Color::Rgb(
                            COLOR_PURE_BLACK.0,
                            COLOR_PURE_BLACK.1,
                            COLOR_PURE_BLACK.2,
                        ))
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(notification, notification_area);
        }

        // Render progress bar if visible (Bug #15, #16, #17)
        if let Some(ref pb) = self.progress_bar {
            if pb.visible {
                let progress_text = pb.display_text_truncated(MAX_PROGRESS_COMMAND_LEN);
                let progress_widget = Paragraph::new(progress_text)
                    .style(
                        Style::default()
                            .fg(Color::Rgb(
                                COLOR_MAGENTA_RED.0,
                                COLOR_MAGENTA_RED.1,
                                COLOR_MAGENTA_RED.2,
                            ))
                            .bg(Color::Rgb(
                                COLOR_PURE_BLACK.0,
                                COLOR_PURE_BLACK.1,
                                COLOR_PURE_BLACK.2,
                            ))
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(Block::default().borders(Borders::NONE));
                f.render_widget(progress_widget, progress_area);
            }
        }

        // Render terminal output (Bug #3: use cached styled lines)
        self.render_terminal_output(f, content_area);

        // Render resource monitor if enabled (Bug #23: take &self not &mut self)
        if self.show_resources && self.resource_monitor.is_some() {
            self.render_resource_monitor(f, resource_area);
        }
    }

    /// Bug #3: Render terminal output with zero-copy caching
    fn render_terminal_output(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let buffer_len = self
            .output_buffers
            .get(self.active_session)
            .map_or(0, std::vec::Vec::len);
        let cached_len = self
            .cached_buffer_lens
            .get(self.active_session)
            .copied()
            .unwrap_or(0);

        // Only reparse if buffer has changed (Bug #3: avoid massive allocation)
        if buffer_len != cached_len {
            if let Some(buffer) = self.output_buffers.get(self.active_session) {
                // Use String::from_utf8_lossy which returns Cow - doesn't allocate if valid UTF-8
                let raw_output = String::from_utf8_lossy(buffer);
                let all_lines = AnsiParser::parse(&raw_output);
                // Leave 1 line at bottom for breathing room (ensure prompt is visible)
                let height = (area.height as usize).saturating_sub(1).max(1);
                let skip_count = all_lines.len().saturating_sub(height);
                let visible_lines: Vec<Line<'static>> =
                    all_lines.into_iter().skip(skip_count).collect();

                if let Some(cache) = self.cached_styled_lines.get_mut(self.active_session) {
                    *cache = visible_lines;
                }
                if let Some(len) = self.cached_buffer_lens.get_mut(self.active_session) {
                    *len = buffer_len;
                }
            }
        }

        // Use cached styled lines - avoid clone by taking reference
        let styled_lines = self
            .cached_styled_lines
            .get(self.active_session)
            .map(|lines| lines.as_slice())
            .unwrap_or(&[]);

        // LOCAL ECHO FIX: Append pending command buffer to show user input immediately
        // This fixes the issue where typed characters are not visible until shell echoes them back
        // This is especially important on Windows where PTY echo may be delayed or not working
        // Pre-allocate with +1 capacity only if we'll actually need it
        let needs_local_echo = self.command_buffers
            .get(self.active_session)
            .is_some_and(|buf| !buf.is_empty());
        
        let capacity = if needs_local_echo {
            styled_lines.len() + 1
        } else {
            styled_lines.len()
        };
        
        let mut display_lines = Vec::with_capacity(capacity);
        display_lines.extend_from_slice(styled_lines);
        
        if let Some(cmd_buf) = self.command_buffers.get(self.active_session) {
            if !cmd_buf.is_empty() {
                // Convert command buffer to string for display (local echo)
                let pending_input = String::from_utf8_lossy(cmd_buf);

                // Check if the last line already ends with this input (shell echo is working)
                // to avoid duplicate display
                let should_display = if let Some(last_line) = display_lines.last() {
                    let last_line_text: String = last_line
                        .spans
                        .iter()
                        .map(|span| span.content.as_ref())
                        .collect();
                    // Only show local echo if the shell hasn't echoed it yet
                    !last_line_text.ends_with(pending_input.as_ref())
                } else {
                    true
                };

                if should_display {
                    // If we have lines already, append to the last line
                    if let Some(last_line) = display_lines.last_mut() {
                        // Add the pending input as a new span to the last line
                        // Use the same color as normal text for consistency
                        last_line.spans.push(Span::styled(
                            pending_input.into_owned(),
                            Style::default().fg(Color::Rgb(
                                COLOR_REDDISH_GRAY.0,
                                COLOR_REDDISH_GRAY.1,
                                COLOR_REDDISH_GRAY.2,
                            )),
                        ));
                    } else {
                        // No lines yet, create a new line with the pending input
                        display_lines.push(Line::from(Span::styled(
                            pending_input.into_owned(),
                            Style::default().fg(Color::Rgb(
                                COLOR_REDDISH_GRAY.0,
                                COLOR_REDDISH_GRAY.1,
                                COLOR_REDDISH_GRAY.2,
                            )),
                        )));
                    }
                }
            }
        }

        // If no content yet, show a placeholder prompt so users know where to type
        // This prevents confusion when the shell is slow to start
        let has_content = !display_lines.is_empty();
        let text = if has_content {
            Text::from(display_lines)
        } else {
            // Create a simple prompt-like line to indicate where the user can type
            // Use theme colors for consistency with other UI elements
            let prompt_line = Line::from(vec![Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Rgb(
                        COLOR_COOL_RED.0,
                        COLOR_COOL_RED.1,
                        COLOR_COOL_RED.2,
                    ))
                    .add_modifier(Modifier::BOLD),
            )]);

            Text::from(vec![prompt_line])
        };

        let paragraph = Paragraph::new(text)
            .style(
                Style::default()
                    .fg(Color::Rgb(
                        COLOR_REDDISH_GRAY.0,
                        COLOR_REDDISH_GRAY.1,
                        COLOR_REDDISH_GRAY.2,
                    ))
                    .bg(Color::Rgb(
                        COLOR_PURE_BLACK.0,
                        COLOR_PURE_BLACK.1,
                        COLOR_PURE_BLACK.2,
                    )),
            )
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(paragraph, area);

        // Set cursor position at the end of the visible content
        // Only position cursor if we have real content (not just placeholder)
        if has_content && !styled_lines.is_empty() {
            if let Some(last_line) = styled_lines.last() {
                // Calculate cursor position using display width, not byte count
                let line_width: u16 = last_line
                    .spans
                    .iter()
                    .map(|span| span.content.width() as u16)
                    .sum();

                let line_count = styled_lines.len() as u16;

                // Position cursor at the end of the last line
                // Ensure we stay within the visible area bounds
                let cursor_x = (area.x + line_width).min(area.x + area.width.saturating_sub(1));

                // Y position should be relative to the visible lines
                // Since we already filtered visible_lines to fit in the area, we use line_count - 1
                let cursor_y = (area.y + line_count.saturating_sub(1))
                    .min(area.y + area.height.saturating_sub(1));

                f.set_cursor(cursor_x, cursor_y);
            } else {
                // Shouldn't happen, but fallback to start of area
                f.set_cursor(area.x, area.y);
            }
        } else {
            // No real content yet, position cursor at start of content area
            f.set_cursor(area.x, area.y);
        }
    }

    /// Render command palette overlay (Bug #4: don't wipe terminal)
    /// Render resource monitor (Bug #23: doesn't need &mut self)
    fn render_resource_monitor(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let Some(ref mut monitor) = self.resource_monitor else {
            return;
        };

        let stats = monitor.get_stats();

        let text = format!(
            " CPU: {:.1}% ({} cores) | Memory: {} / {} ({:.1}%) | Processes: {} | Network: ↓{} ↑{} ",
            stats.cpu_usage,
            stats.cpu_count,
            ResourceMonitor::format_bytes(stats.memory_used),
            ResourceMonitor::format_bytes(stats.memory_total),
            stats.memory_percent,
            stats.process_count,
            ResourceMonitor::format_bytes(stats.network_rx),
            ResourceMonitor::format_bytes(stats.network_tx),
        );

        let resource_widget = Paragraph::new(text)
            .style(
                Style::default()
                    .fg(Color::Rgb(
                        COLOR_MUTED_GREEN.0,
                        COLOR_MUTED_GREEN.1,
                        COLOR_MUTED_GREEN.2,
                    ))
                    .bg(Color::Rgb(
                        COLOR_PURE_BLACK.0,
                        COLOR_PURE_BLACK.1,
                        COLOR_PURE_BLACK.2,
                    )),
            )
            .block(Block::default().borders(Borders::TOP));

        f.render_widget(resource_widget, area);
    }
}

/// Bug #19: Create a centered popup area with minimum size guarantees
#[must_use]
#[allow(dead_code)] // May be used for future UI features
pub fn centered_popup(parent: Rect, max_width: u16, max_height: u16) -> Rect {
    // Enforce minimum size (Bug #19)
    let width = parent.width.min(max_width).max(MIN_POPUP_WIDTH);
    let height = parent.height.min(max_height).max(MIN_POPUP_HEIGHT);

    // If parent is too small, just use parent size
    let width = width.min(parent.width);
    let height = height.min(parent.height);

    let x = parent.width.saturating_sub(width) / 2;
    let y = parent.height.saturating_sub(height) / 2;
    Rect {
        x: parent.x + x,
        y: parent.y + y,
        width,
        height,
    }
}
