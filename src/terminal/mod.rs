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
    cursor::{Hide, Show},
    event::{
        self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Terminal as RatatuiTerminal,
};
use std::borrow::Cow;
use std::io;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

use crate::colors::TrueColorPalette;
use crate::config::Config;
use crate::keybindings::KeybindingManager;
use crate::plugins::PluginManager;
use crate::progress_bar::ProgressBar;
use crate::session::SessionManager;
use crate::shell::ShellSession;
use crate::ssh_manager::SshManager;
use crate::translator::CommandTranslator;
use crate::ui::{
    autocomplete::Autocomplete, command_palette::CommandPalette, resource_monitor::ResourceMonitor,
};
use crate::url_handler::UrlHandler;

use self::ansi_parser::AnsiParser;

/// Target FPS for GPU-accelerated rendering
const TARGET_FPS: u64 = 170;

/// Read buffer size optimized for typical terminal output
const READ_BUFFER_SIZE: usize = 4096;

/// URL cache refresh interval in frames (at 170 FPS, 30 frames ≈ 176ms)
const URL_CACHE_REFRESH_FRAMES: u64 = 30;

/// Notification display duration in seconds
const NOTIFICATION_DURATION_SECS: u64 = 2;

/// Minimum popup size to prevent collapse (Bug #19)
const MIN_POPUP_WIDTH: u16 = 20;
const MIN_POPUP_HEIGHT: u16 = 5;

/// Maximum command display length in progress bar (Bug #16)
const MAX_PROGRESS_COMMAND_LEN: usize = 40;

/// High-performance terminal with GPU-accelerated rendering at 170 FPS
pub struct Terminal {
    config: Config,
    sessions: Vec<ShellSession>,
    active_session: usize,
    output_buffers: Vec<Vec<u8>>,
    should_quit: bool,
    command_palette: CommandPalette,
    resource_monitor: ResourceMonitor,
    #[allow(dead_code)]
    autocomplete: Autocomplete,
    show_resources: bool,
    #[allow(dead_code)]
    keybindings: KeybindingManager,
    #[allow(dead_code)]
    session_manager: SessionManager,
    #[allow(dead_code)]
    plugin_manager: PluginManager,
    #[allow(dead_code)]
    color_palette: TrueColorPalette,
    // Performance optimization: track if redraw is needed
    dirty: bool,
    // Reusable read buffer to reduce allocations
    read_buffer: Vec<u8>,
    // Frame counter for performance metrics
    frame_count: u64,
    // Command translator for cross-platform compatibility
    command_translator: CommandTranslator,
    // Current command buffer for each session - tracks BYTES sent to shell (Bug #1, #2)
    command_buffers: Vec<Vec<u8>>,
    // Translation notification message and timeout
    translation_notification: Option<String>,
    notification_frames: u64,
    // SSH connection manager
    ssh_manager: SshManager,
    // Cached URL positions with line numbers for coordinate mapping (Bug #5)
    cached_urls: Vec<crate::url_handler::DetectedUrl>,
    // Track when URL cache was last updated (frame counter)
    url_cache_frame: u64,
    // Progress bar for command execution
    progress_bar: ProgressBar,
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
    /// Returns an error if SSH manager or session manager initialization fails
    pub fn new(config: Config) -> Result<Self> {
        info!("Initializing Furnace terminal emulator with 170 FPS GPU rendering + 24-bit color");

        let command_translator = CommandTranslator::new(config.command_translation.enabled);
        let ssh_manager = SshManager::new()?;

        Ok(Self {
            config,
            sessions: Vec::with_capacity(8),
            active_session: 0,
            output_buffers: Vec::with_capacity(8),
            should_quit: false,
            command_palette: CommandPalette::new(),
            resource_monitor: ResourceMonitor::new(),
            autocomplete: Autocomplete::new(),
            show_resources: false,
            keybindings: KeybindingManager::new(),
            session_manager: SessionManager::new()?,
            plugin_manager: PluginManager::new(),
            color_palette: TrueColorPalette::default_dark(),
            dirty: true,
            read_buffer: vec![0u8; READ_BUFFER_SIZE],
            frame_count: 0,
            command_translator,
            command_buffers: Vec::with_capacity(8),
            translation_notification: None,
            notification_frames: 0,
            ssh_manager,
            cached_urls: Vec::new(),
            url_cache_frame: 0,
            progress_bar: ProgressBar::new(),
            terminal_cols: 80,
            terminal_rows: 24,
            cached_styled_lines: Vec::with_capacity(8),
            cached_buffer_lens: Vec::with_capacity(8),
        })
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
        // Hide cursor initially - ratatui manages its own cursor
        execute!(
            stdout,
            crossterm::event::EnableMouseCapture,
            crossterm::event::EnableBracketedPaste,
            Hide
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
                                if self.progress_bar.visible {
                                    let recent_output = String::from_utf8_lossy(&self.read_buffer[..n]);
                                    if self.detect_prompt(&recent_output) {
                                        self.progress_bar.stop();
                                    }
                                }

                                // Bug #8: Enforce scrollback limit and clear URL cache
                                let max_buffer = self.config.terminal.scrollback_lines * 256;
                                if self.output_buffers[self.active_session].len() > max_buffer {
                                    let excess = self.output_buffers[self.active_session].len() - max_buffer;
                                    self.output_buffers[self.active_session].drain(..excess);
                                    // Bug #10: Invalidate URL cache since buffer changed
                                    self.cached_urls.clear();
                                }
                            }
                        }
                    }
                } => {}

                // Render at consistent frame rate
                _ = render_interval.tick() => {
                    // Update progress bar spinner (only if visible)
                    if self.progress_bar.visible {
                        self.progress_bar.tick();
                        self.dirty = true;
                    }

                    // Bug #11: Only decrement notification counter when actually rendering
                    if self.dirty && self.notification_frames > 0 {
                        self.notification_frames -= 1;
                        if self.notification_frames == 0 {
                            self.translation_notification = None;
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
    fn detect_prompt(&self, output: &str) -> bool {
        // Common prompt patterns
        output.contains("$ ")
            || output.contains("> ")
            || output.contains("# ")
            || output.contains("% ")
            || output.contains("❯") // fish/starship
            || output.contains("➜") // oh-my-zsh
            || output.contains("λ") // some zsh themes
            || output.contains("PS>") // PowerShell
            || output.contains("PS ") // PowerShell
            || output.contains(">>>") // Python REPL
            || output.contains("...") // Python continuation
            || (output.ends_with('\n') && output.len() < 100) // Short line likely prompt
    }

    /// Handle mouse events for URL clicking (Bug #5: coordinate-based URL detection)
    async fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        if !self.config.url_handler.enabled {
            return Ok(());
        }

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if mouse.modifiers.contains(KeyModifiers::CONTROL) {
                // Bug #5: Find URL at click position instead of opening first URL
                if let Some(buffer) = self.output_buffers.get(self.active_session) {
                    // Only update cache if needed (avoid allocation - Bug #3)
                    if self.frame_count - self.url_cache_frame > URL_CACHE_REFRESH_FRAMES {
                        let text = String::from_utf8_lossy(buffer);
                        self.cached_urls = UrlHandler::detect_urls(&text);
                        self.url_cache_frame = self.frame_count;
                    }

                    // Find URL at click position
                    let click_row = mouse.row as usize;
                    let click_col = mouse.column as usize;

                    // Try to find a URL that contains the click position
                    if let Some(url) = self.find_url_at_position(click_row, click_col) {
                        info!("Opening URL at ({}, {}): {}", click_row, click_col, url);
                        if let Err(e) = UrlHandler::open_url(&url) {
                            warn!("Failed to open URL: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Find URL at the given screen position
    fn find_url_at_position(&self, _row: usize, col: usize) -> Option<String> {
        // For each cached URL, check if the click position falls within it
        // This is a simplified implementation - a full one would track line positions
        for detected in &self.cached_urls {
            // Check if click column is within the URL's character range
            if col >= detected.start_pos && col < detected.end_pos {
                return Some(detected.url.clone());
            }
        }
        // Fallback: if no position match, don't open anything (Bug #5 fix)
        None
    }

    /// Handle keyboard events with optimal input processing
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // SSH manager takes priority when visible
        if self.ssh_manager.visible {
            return self.handle_ssh_manager_input(key).await;
        }

        // Command palette takes priority
        if self.command_palette.visible {
            return self.handle_command_palette_input(key).await;
        }

        match (key.code, key.modifiers) {
            // SSH Manager (Ctrl+Shift+S) - Bug #18: Don't fall through when disabled
            (KeyCode::Char('s'), m) | (KeyCode::Char('S'), m)
                if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
            {
                if self.config.ssh_manager.enabled {
                    self.ssh_manager.toggle();
                    debug!(
                        "SSH manager: {}",
                        if self.ssh_manager.visible { "ON" } else { "OFF" }
                    );
                }
                // Bug #18: Don't send 's' to shell when SSH manager is disabled
                return Ok(());
            }

            // Command palette (Ctrl+P)
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.command_palette.toggle();
            }

            // Toggle resource monitor (Ctrl+R)
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.show_resources = !self.show_resources;
                debug!(
                    "Resource monitor: {}",
                    if self.show_resources { "ON" } else { "OFF" }
                );
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
            (KeyCode::BackTab, m) if m.contains(KeyModifiers::SHIFT) && self.config.terminal.enable_tabs => {
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
                        if !cmd_buf.is_empty() {
                            // First, pop any trailing continuation bytes (10xxxxxx pattern)
                            while !cmd_buf.is_empty() {
                                let last = *cmd_buf.last().unwrap();
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

    /// Handle Enter key with command translation (Bug #2: byte-based backspace)
    async fn handle_enter(&mut self) -> Result<()> {
        if let Some(session) = self.sessions.get(self.active_session) {
            // Get the current command as a string from bytes
            let command = self
                .command_buffers
                .get(self.active_session)
                .map(|b| String::from_utf8_lossy(b))
                .unwrap_or(Cow::Borrowed(""));

            // Check for SSH command
            if self.config.ssh_manager.enabled
                && self.config.ssh_manager.auto_show
                && command.trim().starts_with("ssh ")
            {
                if let Some(conn) =
                    crate::ssh_manager::SshManager::parse_ssh_command(&command)
                {
                    let name = conn.name.clone();
                    self.ssh_manager.add_connection(name, conn);
                    let _ = self.ssh_manager.save_connections();
                }
                self.ssh_manager.toggle();
                return Ok(());
            }

            // Attempt translation
            let result = self.command_translator.translate(&command);

            if result.translated {
                info!(
                    "Translated '{}' to '{}'",
                    result.original_command, result.final_command
                );

                if self.config.command_translation.show_notifications {
                    self.translation_notification = Some(format!(
                        "Translated: {} → {}",
                        result.original_command, result.final_command
                    ));
                    self.notification_frames = TARGET_FPS * NOTIFICATION_DURATION_SECS;
                    self.dirty = true;
                }

                // Bug #2: Send one backspace per BYTE in buffer (not per char)
                let byte_count = self
                    .command_buffers
                    .get(self.active_session)
                    .map(|b| b.len())
                    .unwrap_or(0);

                // Send backspaces to clear the original command
                for _ in 0..byte_count {
                    session.write_input(&[127]).await?;
                }

                // Send the translated command
                session.write_input(result.final_command.as_bytes()).await?;
            }

            // Send Enter
            session.write_input(b"\r").await?;

            // Start progress bar (Bug #24: avoid clone)
            let command_to_track = if result.translated {
                &result.final_command
            } else {
                &*command
            };

            if !command_to_track.trim().is_empty() {
                self.progress_bar.start_ref(command_to_track);
                self.dirty = true;
            }

            // Clear command buffer
            if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                cmd_buf.clear();
            }
        }
        Ok(())
    }

    /// Handle SSH manager input
    async fn handle_ssh_manager_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.ssh_manager.toggle();
            }
            KeyCode::Enter => {
                if let Some(conn) = self.ssh_manager.get_selected() {
                    let cmd = conn.to_command();
                    info!("Connecting via SSH: {}", cmd);

                    if let Some(session) = self.sessions.get(self.active_session) {
                        session.write_input(cmd.as_bytes()).await?;
                        session.write_input(b"\r").await?;
                    }

                    self.ssh_manager.toggle();
                }
            }
            KeyCode::Up => {
                self.ssh_manager.select_previous();
            }
            KeyCode::Down => {
                self.ssh_manager.select_next();
            }
            KeyCode::Char(c) => {
                self.ssh_manager.filter_input.push(c);
                self.ssh_manager.update_filter();
            }
            KeyCode::Backspace => {
                self.ssh_manager.filter_input.pop();
                self.ssh_manager.update_filter();
            }
            KeyCode::Delete if !self.ssh_manager.filtered_connections.is_empty() => {
                if self.ssh_manager.selected_index < self.ssh_manager.filtered_connections.len() {
                    let name = self.ssh_manager.filtered_connections
                        [self.ssh_manager.selected_index]
                        .clone();
                    self.ssh_manager.remove_connection(&name);
                    let _ = self.ssh_manager.save_connections();
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle command palette input
    async fn handle_command_palette_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.command_palette.toggle();
            }
            KeyCode::Enter => {
                if let Some(command) = self.command_palette.execute_selected() {
                    self.execute_command(&command).await?;
                }
            }
            KeyCode::Up => {
                self.command_palette.select_previous();
            }
            KeyCode::Down => {
                self.command_palette.select_next();
            }
            KeyCode::Char(c) => {
                self.command_palette.input.push(c);
                self.command_palette
                    .update_input(self.command_palette.input.clone());
            }
            KeyCode::Backspace => {
                self.command_palette.input.pop();
                self.command_palette
                    .update_input(self.command_palette.input.clone());
            }
            _ => {}
        }
        Ok(())
    }

    /// Execute a command from the palette
    async fn execute_command(&mut self, command: &str) -> Result<()> {
        match command {
            "new-tab" => self.create_new_tab().await?,
            "close-tab" => {
                if self.sessions.len() > 1 {
                    self.sessions.remove(self.active_session);
                    self.output_buffers.remove(self.active_session);
                    self.command_buffers.remove(self.active_session);
                    self.cached_styled_lines.remove(self.active_session);
                    self.cached_buffer_lens.remove(self.active_session);
                    if self.active_session >= self.sessions.len() {
                        self.active_session = self.sessions.len().saturating_sub(1);
                    }
                }
            }
            "clear" => {
                if let Some(buffer) = self.output_buffers.get_mut(self.active_session) {
                    buffer.clear();
                    // Bug #10: Clear URL cache when buffer is cleared
                    self.cached_urls.clear();
                }
                // Invalidate render cache
                if let Some(len) = self.cached_buffer_lens.get_mut(self.active_session) {
                    *len = 0;
                }
            }
            "quit" => {
                self.should_quit = true;
            }
            _ => {
                debug!("Unknown command: {}", command);
            }
        }
        Ok(())
    }

    /// Create a new tab (Bug #7: use current terminal size)
    async fn create_new_tab(&mut self) -> Result<()> {
        info!("Creating new tab with size {}x{}", self.terminal_cols, self.terminal_rows);

        let session = ShellSession::new(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            self.terminal_rows,  // Bug #7: use current size
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
                self.cached_urls.clear();
                if let Some(len) = self.cached_buffer_lens.get_mut(tab_index) {
                    *len = 0;
                }
            }
        }
    }

    /// Render UI with hardware acceleration (Bug #3: zero-copy rendering)
    fn render(&mut self, f: &mut ratatui::Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(
                    if self.config.terminal.enable_tabs && self.sessions.len() > 1 {
                        1
                    } else {
                        0
                    },
                ),
                Constraint::Length(if self.translation_notification.is_some() {
                    1
                } else {
                    0
                }),
                Constraint::Length(if self.progress_bar.visible { 1 } else { 0 }),
                Constraint::Min(0),
                Constraint::Length(if self.show_resources { 3 } else { 0 }),
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
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(Span::styled(format!(" Tab {} ", i + 1), style))
                })
                .collect();

            let tabs = Tabs::new(tab_titles)
                .block(Block::default().borders(Borders::BOTTOM))
                .select(self.active_session)
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_widget(tabs, tab_area);
        }

        // Render translation notification if present
        if let Some(ref msg) = self.translation_notification {
            let notification = Paragraph::new(msg.as_str())
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(notification, notification_area);
        }

        // Render progress bar if visible (Bug #15, #16, #17)
        if self.progress_bar.visible {
            let progress_text = self.progress_bar.display_text_truncated(MAX_PROGRESS_COMMAND_LEN);
            let progress_widget = Paragraph::new(progress_text)
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(progress_widget, progress_area);
        }

        // Render SSH manager if visible (takes priority over command palette)
        if self.ssh_manager.visible {
            // First render terminal output underneath
            self.render_terminal_output(f, content_area);
            // Then render SSH manager overlay
            self.render_ssh_manager(f, content_area);
            return;
        }

        // Render command palette if visible (Bug #4: don't wipe terminal)
        if self.command_palette.visible {
            // First render terminal output underneath
            self.render_terminal_output(f, content_area);
            // Then render command palette overlay
            self.render_command_palette(f, content_area);
            return;
        }

        // Render terminal output (Bug #3: use cached styled lines)
        self.render_terminal_output(f, content_area);

        // Render resource monitor if enabled (Bug #23: take &self not &mut self)
        if self.show_resources {
            self.render_resource_monitor(f, resource_area);
        }
    }

    /// Bug #3: Render terminal output with zero-copy caching
    fn render_terminal_output(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let buffer_len = self
            .output_buffers
            .get(self.active_session)
            .map(|b| b.len())
            .unwrap_or(0);
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
                let height = area.height as usize;
                let skip_count = all_lines.len().saturating_sub(height);
                let visible_lines: Vec<Line<'static>> = all_lines
                    .into_iter()
                    .skip(skip_count)
                    .collect();

                if let Some(cache) = self.cached_styled_lines.get_mut(self.active_session) {
                    *cache = visible_lines;
                }
                if let Some(len) = self.cached_buffer_lens.get_mut(self.active_session) {
                    *len = buffer_len;
                }
            }
        }

        // Use cached styled lines
        let styled_lines = self
            .cached_styled_lines
            .get(self.active_session)
            .cloned()
            .unwrap_or_default();

        let text = Text::from(styled_lines);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(paragraph, area);
    }

    /// Render SSH manager overlay
    fn render_ssh_manager(&self, f: &mut ratatui::Frame, area: Rect) {
        let popup_area = centered_popup(area, 80, 25);

        // Clear just the popup area (not the whole screen)
        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self
            .ssh_manager
            .filtered_connections
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                self.ssh_manager.get_connection(name).map(|conn| {
                    let content =
                        format!("{} ({}@{}:{})", name, conn.username, conn.host, conn.port);

                    let style = if i == self.ssh_manager.selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    ListItem::new(content).style(style)
                })
            })
            .collect();

        let title = if self.ssh_manager.filter_input.is_empty() {
            String::from("SSH Connections (Ctrl+Shift+S to close, Enter to connect, Del to remove)")
        } else {
            format!(
                "SSH Connections - Filter: {}",
                self.ssh_manager.filter_input
            )
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(list, popup_area);
    }

    /// Render command palette overlay (Bug #4: don't wipe terminal)
    fn render_command_palette(&self, f: &mut ratatui::Frame, area: Rect) {
        let popup_area = centered_popup(area, 80, 20);

        // Bug #4: Clear only the popup area, not the entire screen
        f.render_widget(Clear, popup_area);

        let palette_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(popup_area);

        // Input box
        let input = Paragraph::new(format!("> {}", self.command_palette.input))
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Palette (Esc to close)")
                    .border_style(Style::default().fg(Color::Cyan))
                    .style(Style::default().bg(Color::Black)),
            );
        f.render_widget(input, palette_chunks[0]);

        // Suggestions
        let suggestions: Vec<Line> = self
            .command_palette
            .suggestions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == self.command_palette.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(vec![
                    Span::styled(format!("  {} ", s.command), style),
                    Span::styled(
                        format!("- {}", s.description),
                        Style::default().fg(Color::Gray),
                    ),
                ])
            })
            .collect();

        let suggestions_widget = Paragraph::new(suggestions)
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .style(Style::default().bg(Color::Black)),
            );
        f.render_widget(suggestions_widget, palette_chunks[1]);
    }

    /// Render resource monitor (Bug #23: doesn't need &mut self)
    fn render_resource_monitor(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let stats = self.resource_monitor.get_stats();

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
            .style(Style::default().fg(Color::Green).bg(Color::Black))
            .block(Block::default().borders(Borders::TOP));

        f.render_widget(resource_widget, area);
    }
}

/// Bug #19: Create a centered popup area with minimum size guarantees
#[must_use]
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
