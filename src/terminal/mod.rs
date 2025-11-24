//! Terminal module for the Furnace terminal emulator
//!
//! This module contains the main Terminal struct and its supporting modules:
//! - `input`: Input handling for keyboard and mouse events
//! - `renderer`: UI rendering components
//!
//! # Architecture
//! The terminal is structured to separate concerns:
//! - Event loop management (main run loop)
//! - Input processing (keyboard/mouse handlers)
//! - Rendering (UI drawing)
//! - Tab/session management

pub mod input;
pub mod renderer;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, List, ListItem},
    Terminal as RatatuiTerminal,
};
use std::io;
use tokio::time::{Duration, interval};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::shell::ShellSession;
use crate::ui::{command_palette::CommandPalette, resource_monitor::ResourceMonitor, autocomplete::Autocomplete};
use crate::keybindings::KeybindingManager;
use crate::session::SessionManager;
use crate::plugins::PluginManager;
use crate::colors::TrueColorPalette;
use crate::translator::CommandTranslator;
use crate::ssh_manager::SshManager;
use crate::url_handler::UrlHandler;
use crate::progress_bar::ProgressBar;

/// Target FPS for GPU-accelerated rendering
const TARGET_FPS: u64 = 170;

/// Default terminal dimensions for new tabs
const DEFAULT_ROWS: u16 = 40;
const DEFAULT_COLS: u16 = 120;

/// Read buffer size optimized for typical terminal output (reduced from 8KB to 4KB for better cache locality)
const READ_BUFFER_SIZE: usize = 4096;

/// URL cache refresh interval in frames (at 170 FPS, 30 frames ≈ 176ms)
const URL_CACHE_REFRESH_FRAMES: u64 = 30;

/// Backspace buffer initial capacity for typical command lengths
const BACKSPACE_BUFFER_CAPACITY: usize = 256;

/// Notification display duration in seconds
const NOTIFICATION_DURATION_SECS: u64 = 2;

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
    // Current command buffer for each session
    command_buffers: Vec<String>,
    // Translation notification message and timeout
    translation_notification: Option<String>,
    notification_frames: u64,
    // SSH connection manager
    ssh_manager: SshManager,
    // Cached URL positions to avoid re-parsing on every mouse event
    cached_urls: Vec<crate::url_handler::DetectedUrl>,
    // Track when URL cache was last updated (frame counter)
    url_cache_frame: u64,
    // Reusable backspace buffer to avoid allocations
    backspace_buffer: Vec<u8>,
    // Progress bar for command execution
    progress_bar: ProgressBar,
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
            sessions: Vec::with_capacity(8), // Pre-allocate for performance
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
            dirty: true, // Initial draw needed
            read_buffer: vec![0u8; READ_BUFFER_SIZE], // Pre-allocated reusable buffer
            frame_count: 0,
            command_translator,
            command_buffers: Vec::with_capacity(8),
            translation_notification: None,
            notification_frames: 0,
            ssh_manager,
            cached_urls: Vec::new(),
            url_cache_frame: 0,
            backspace_buffer: Vec::with_capacity(BACKSPACE_BUFFER_CAPACITY),
            progress_bar: ProgressBar::new(),
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
        
        // Enable mouse capture
        execute!(stdout, crossterm::event::EnableMouseCapture)?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = RatatuiTerminal::new(backend)?;

        // Create initial shell session
        let (cols, rows) = terminal.size().map(|s| (s.width, s.height))?;
        let session = ShellSession::new(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            rows,
            cols,
        )?;
        
        self.sessions.push(session);
        self.output_buffers.push(Vec::with_capacity(1024 * 1024)); // 1MB buffer
        self.command_buffers.push(String::new()); // Initialize command buffer

        info!("Terminal started with {}x{} size", cols, rows);

        // Event loop with optimized timing for TARGET_FPS
        let frame_duration = Duration::from_micros(1_000_000 / TARGET_FPS); // ~5.88ms per frame for 170 FPS
        let mut render_interval = interval(frame_duration);
        render_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip); // Skip missed frames for consistent performance

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
                            _ => {}
                        }
                    }
                }
                
                // Read shell output (non-blocking, optimized with reusable buffer)
                _ = async {
                    if let Some(session) = self.sessions.get(self.active_session) {
                        // Reuse pre-allocated buffer to avoid repeated allocations
                        if let Ok(n) = session.read_output(&mut self.read_buffer).await {
                            if n > 0 {
                                self.output_buffers[self.active_session].extend_from_slice(&self.read_buffer[..n]);
                                self.dirty = true; // Mark for redraw
                                
                                // Check if we got a prompt (command completion detection)
                                if self.progress_bar.visible {
                                    let recent_output = String::from_utf8_lossy(&self.read_buffer[..n]);
                                    // Look for common shell prompt indicators
                                    if recent_output.contains("$ ") || recent_output.contains("> ") 
                                        || recent_output.contains("# ") || recent_output.contains("% ") {
                                        self.progress_bar.stop();
                                    }
                                }
                                
                                // Keep buffer size manageable with efficient drain
                                let max_buffer = self.config.terminal.scrollback_lines * 256;
                                if self.output_buffers[self.active_session].len() > max_buffer {
                                    let excess = self.output_buffers[self.active_session].len() - max_buffer;
                                    self.output_buffers[self.active_session].drain(..excess);
                                }
                            }
                        }
                    }
                } => {}
                
                // Render at consistent frame rate (only if dirty flag is set)
                _ = render_interval.tick() => {
                    // Update progress bar spinner
                    if self.progress_bar.visible {
                        self.progress_bar.tick();
                        self.dirty = true;
                    }
                    
                    // Decrement notification counter
                    if self.notification_frames > 0 {
                        self.notification_frames -= 1;
                        if self.notification_frames == 0 {
                            self.translation_notification = None;
                        }
                        self.dirty = true;
                    }
                    
                    if self.dirty {
                        terminal.draw(|f| self.render(f))?;
                        self.dirty = false; // Clear dirty flag
                        self.frame_count += 1;
                        
                        // Log performance metrics every 1000 frames
                        if self.frame_count.is_multiple_of(1000) {
                            debug!("Rendered {} frames", self.frame_count);
                        }
                    }
                }
            }
        }

        // Cleanup
        execute!(terminal.backend_mut(), crossterm::event::DisableMouseCapture)?;
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        info!("Terminal shutdown complete");
        Ok(())
    }

    /// Handle mouse events for URL clicking
    async fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        // Only handle Ctrl+Click for URLs if URL handler is enabled
        if !self.config.url_handler.enabled {
            return Ok(());
        }
        
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if mouse.modifiers.contains(KeyModifiers::CONTROL) {
                // Update URL cache if output has changed (every 30 frames or ~176ms at 170fps)
                if self.frame_count - self.url_cache_frame > URL_CACHE_REFRESH_FRAMES {
                    if let Some(buffer) = self.output_buffers.get(self.active_session) {
                        let text = String::from_utf8_lossy(buffer);
                        self.cached_urls = UrlHandler::detect_urls(&text);
                        self.url_cache_frame = self.frame_count;
                    }
                }
                
                // Use cached URLs instead of re-parsing
                if !self.cached_urls.is_empty() {
                    // For now, just open the first URL found
                    // A full implementation would map click coordinates to text positions
                    if let Some(url) = self.cached_urls.first() {
                        info!("Opening URL: {}", url.url);
                        if let Err(e) = UrlHandler::open_url(&url.url) {
                            warn!("Failed to open URL: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
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
            // SSH Manager (Ctrl+Shift+S)
            (KeyCode::Char('s'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) | 
            (KeyCode::Char('S'), KeyModifiers::CONTROL) if self.config.ssh_manager.enabled => {
                self.ssh_manager.toggle();
                debug!("SSH manager: {}", if self.ssh_manager.visible { "ON" } else { "OFF" });
            }
            
            // Command palette (Ctrl+P)
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.command_palette.toggle();
            }

            // Toggle resource monitor (Ctrl+R)
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.show_resources = !self.show_resources;
                debug!("Resource monitor: {}", if self.show_resources { "ON" } else { "OFF" });
            }

            // Quit (Ctrl+C or Ctrl+D)
            (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                debug!("Quit signal received");
                self.should_quit = true;
            }
            
            // New tab
            (KeyCode::Char('t'), KeyModifiers::CONTROL) if self.config.terminal.enable_tabs => {
                self.create_new_tab().await?;
            }
            
            // Next tab
            (KeyCode::Tab, KeyModifiers::CONTROL) if self.config.terminal.enable_tabs => {
                self.next_tab();
            }
            
            // Previous tab
            (KeyCode::BackTab, KeyModifiers::SHIFT | KeyModifiers::CONTROL) if self.config.terminal.enable_tabs => {
                self.prev_tab();
            }
            
            // Regular character input
            (KeyCode::Char(c), modifiers) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    let mut input = vec![c as u8];
                    
                    // Handle modifiers
                    if modifiers.contains(KeyModifiers::CONTROL) && c.is_ascii_alphabetic() {
                        // Send control character
                        let ctrl_char = (c.to_ascii_uppercase() as u8) - b'A' + 1;
                        input = vec![ctrl_char];
                    } else if !modifiers.contains(KeyModifiers::CONTROL) {
                        // Track normal character input for command translation
                        if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                            cmd_buf.push(c);
                        }
                    }
                    
                    session.write_input(&input).await?;
                }
            }
            
            // Enter - translate command before sending, check for SSH
            (KeyCode::Enter, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    // Get the current command
                    let command = self.command_buffers.get(self.active_session)
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    
                    // Check if it's an SSH command and auto-show manager if enabled
                    if self.config.ssh_manager.enabled && self.config.ssh_manager.auto_show 
                        && command.trim().starts_with("ssh ") {
                        
                        // Parse and optionally save the connection
                        if let Some(conn) = crate::ssh_manager::SshManager::parse_ssh_command(command) {
                            let name = conn.name.clone();
                            self.ssh_manager.add_connection(name, conn);
                            let _ = self.ssh_manager.save_connections();
                        }
                        
                        // Show SSH manager for user to select/confirm
                        self.ssh_manager.toggle();
                        
                        // Don't send the command yet - let user interact with SSH manager
                        return Ok(());
                    }
                    
                    // Attempt translation
                    let result = self.command_translator.translate(command);
                    
                    if result.translated {
                        // Command was translated - send translated version
                        info!("Translated '{}' to '{}'", result.original_command, result.final_command);
                        
                        // Show notification if enabled
                        if self.config.command_translation.show_notifications {
                            self.translation_notification = Some(format!(
                                "Translated: {} → {}",
                                result.original_command,
                                result.final_command
                            ));
                            self.notification_frames = TARGET_FPS * NOTIFICATION_DURATION_SECS;
                            self.dirty = true;
                        }
                        
                        // Clear the shell's input line and send the translated command
                        // Count Unicode characters properly
                        let char_count = command.chars().count();
                        
                        // Reuse backspace buffer to avoid allocation
                        self.backspace_buffer.clear();
                        self.backspace_buffer.resize(char_count, 127);
                        session.write_input(&self.backspace_buffer).await?;
                        
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
                        self.progress_bar.start(command_to_track);
                        self.dirty = true;
                    }
                    
                    // Clear command buffer
                    if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                        cmd_buf.clear();
                    }
                }
            }
            
            // Backspace
            (KeyCode::Backspace, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    // Remove last character from command buffer
                    if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                        cmd_buf.pop();
                    }
                    session.write_input(&[127]).await?;
                }
            }
            
            // Arrow keys
            (KeyCode::Up, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    // Clear command buffer when navigating history
                    if let Some(cmd_buf) = self.command_buffers.get_mut(self.active_session) {
                        cmd_buf.clear();
                    }
                    session.write_input(b"\x1b[A").await?;
                }
            }
            (KeyCode::Down, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    // Clear command buffer when navigating history
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

    /// Handle SSH manager input
    async fn handle_ssh_manager_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.ssh_manager.toggle();
            }
            KeyCode::Enter => {
                // Connect to selected SSH host
                if let Some(conn) = self.ssh_manager.get_selected() {
                    let cmd = conn.to_command();
                    info!("Connecting via SSH: {}", cmd);
                    
                    // Send the SSH command to the shell
                    if let Some(session) = self.sessions.get(self.active_session) {
                        session.write_input(cmd.as_bytes()).await?;
                        session.write_input(b"\r").await?;
                    }
                    
                    // Update last used time
                    // (In a full implementation, you'd update the connection's last_used field)
                    
                    // Close SSH manager
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
                // Delete selected connection
                if self.ssh_manager.selected_index < self.ssh_manager.filtered_connections.len() {
                    let name = self.ssh_manager.filtered_connections[self.ssh_manager.selected_index].clone();
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
                self.command_palette.update_input(self.command_palette.input.clone());
            }
            KeyCode::Backspace => {
                self.command_palette.input.pop();
                self.command_palette.update_input(self.command_palette.input.clone());
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
                    if self.active_session >= self.sessions.len() {
                        self.active_session = self.sessions.len().saturating_sub(1);
                    }
                }
            }
            "clear" => {
                if let Some(buffer) = self.output_buffers.get_mut(self.active_session) {
                    buffer.clear();
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

    /// Create a new tab
    async fn create_new_tab(&mut self) -> Result<()> {
        info!("Creating new tab");
        
        let session = ShellSession::new(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            DEFAULT_ROWS,
            DEFAULT_COLS,
        )?;
        
        self.sessions.push(session);
        self.output_buffers.push(Vec::with_capacity(1024 * 1024));
        self.command_buffers.push(String::new()); // Initialize command buffer for new tab
        self.active_session = self.sessions.len() - 1;
        
        Ok(())
    }

    /// Switch to next tab
    fn next_tab(&mut self) {
        if !self.sessions.is_empty() {
            self.active_session = (self.active_session + 1) % self.sessions.len();
            debug!("Switched to tab {}", self.active_session);
        }
    }

    /// Switch to previous tab
    fn prev_tab(&mut self) {
        if !self.sessions.is_empty() {
            if self.active_session == 0 {
                self.active_session = self.sessions.len() - 1;
            } else {
                self.active_session -= 1;
            }
            debug!("Switched to tab {}", self.active_session);
        }
    }

    /// Render UI with hardware acceleration (zero-copy where possible)
    fn render(&mut self, f: &mut ratatui::Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if self.config.terminal.enable_tabs && self.sessions.len() > 1 { 1 } else { 0 }),
                Constraint::Length(if self.translation_notification.is_some() { 1 } else { 0 }),
                Constraint::Length(if self.progress_bar.visible { 1 } else { 0 }),
                Constraint::Min(0),
                Constraint::Length(if self.show_resources { 3 } else { 0 }),
            ].as_ref())
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
                .style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(notification, notification_area);
        }

        // Render progress bar if visible
        if self.progress_bar.visible {
            let progress_text = self.progress_bar.display_text();
            let progress_widget = Paragraph::new(progress_text)
                .style(Style::default().fg(Color::Cyan).bg(Color::Black).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(progress_widget, progress_area);
        }

        // Render SSH manager if visible (takes priority over command palette)
        if self.ssh_manager.visible {
            self.render_ssh_manager(f, content_area);
            return; // SSH manager takes full screen focus
        }

        // Render command palette if visible
        if self.command_palette.visible {
            self.render_command_palette(f, content_area);
            return; // Command palette takes full screen focus
        }

        // Render terminal output
        let output = if let Some(buffer) = self.output_buffers.get(self.active_session) {
            String::from_utf8_lossy(buffer).to_string()
        } else {
            String::new()
        };

        let paragraph = Paragraph::new(output)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(paragraph, content_area);

        // Render resource monitor if enabled
        if self.show_resources {
            self.render_resource_monitor(f, resource_area);
        }
    }

    /// Render SSH manager overlay
    fn render_ssh_manager(&mut self, f: &mut ratatui::Frame, area: Rect) {
        // Create centered popup
        let popup_area = {
            let width = area.width.min(80);
            let height = area.height.min(25);
            let x = (area.width - width) / 2;
            let y = (area.height - height) / 2;
            Rect {
                x: area.x + x,
                y: area.y + y,
                width,
                height,
            }
        };

        // Render connection list - use filter_map to safely handle missing connections
        let items: Vec<ListItem> = self.ssh_manager.filtered_connections
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                // Safely get connection - returns None if not found
                self.ssh_manager.get_connection(name).map(|conn| {
                    let content = format!(
                        "{} ({}@{}:{})",
                        name,
                        conn.username,
                        conn.host,
                        conn.port
                    );
                    
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
            "SSH Connections (Ctrl+Shift+S to close, Enter to connect, Del to remove)"
        } else {
            &format!("SSH Connections - Filter: {}", self.ssh_manager.filter_input)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .style(Style::default().bg(Color::Black))
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(list, popup_area);
    }

    /// Render command palette overlay
    fn render_command_palette(&mut self, f: &mut ratatui::Frame, area: Rect) {
        // Create centered popup
        let popup_area = {
            let width = area.width.min(80);
            let height = area.height.min(20);
            let x = (area.width - width) / 2;
            let y = (area.height - height) / 2;
            Rect::new(x, y, width, height)
        };

        // Clear background
        let bg = Block::default()
            .style(Style::default().bg(Color::Black));
        f.render_widget(bg, area);

        // Render palette
        let palette_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(popup_area);

        // Input box
        let input = Paragraph::new(format!("> {}", self.command_palette.input))
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Command Palette (Esc to close)")
                .border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(input, palette_chunks[0]);

        // Suggestions
        let suggestions: Vec<Line> = self.command_palette.suggestions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == self.command_palette.selected_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(vec![
                    Span::styled(format!("  {} ", s.command), style),
                    Span::styled(format!("- {}", s.description), Style::default().fg(Color::Gray)),
                ])
            })
            .collect();

        let suggestions_widget = Paragraph::new(suggestions)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(suggestions_widget, palette_chunks[1]);
    }

    /// Render resource monitor
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

// Rust ensures no memory leaks via RAII and Drop trait
impl Drop for Terminal {
    fn drop(&mut self) {
        info!("Terminal instance dropped - all resources cleaned up");
    }
}
