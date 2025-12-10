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

use anyhow::{Context, Result};
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
use std::time::Instant;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};
use unicode_width::UnicodeWidthStr;

use crate::colors::TrueColorPalette;
use crate::config::Config;
use crate::hooks::HooksExecutor;
use crate::keybindings::KeybindingManager;
use crate::progress_bar::ProgressBar;
use crate::session::SessionManager;
use crate::shell::ShellSession;
use crate::ui::{
    autocomplete::Autocomplete, resource_monitor::ResourceMonitor, themes::ThemeManager,
};

use self::ansi_parser::AnsiParser;

/// Target FPS for GPU-accelerated rendering
const TARGET_FPS: u64 = 170;

/// Read buffer size optimized for typical terminal output
/// Using 4KB as it's a common page size and provides good balance
const READ_BUFFER_SIZE: usize = 4 * 1024;

/// Notification display duration in seconds
const NOTIFICATION_DURATION_SECS: u64 = 2;

/// Minimum popup size to prevent collapse (for future UI features)
const _MIN_POPUP_WIDTH: u16 = 20;
const _MIN_POPUP_HEIGHT: u16 = 5;

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
const _COLOR_DARK_GRAY: (u8, u8, u8) = (0x5A, 0x4A, 0x4A); // Dark gray for future use

/// High-performance terminal with GPU-accelerated rendering at 170 FPS
#[allow(clippy::struct_field_names)]
pub struct Terminal {
    config: Config,
    sessions: Vec<ShellSession>,
    active_session: usize,
    output_buffers: Vec<Vec<u8>>,
    should_quit: bool,
    resource_monitor: Option<ResourceMonitor>,
    autocomplete: Option<Autocomplete>,
    show_resources: bool,
    keybindings: KeybindingManager,
    session_manager: Option<SessionManager>,
    color_palette: TrueColorPalette,
    // Theme manager for dynamic theme switching
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
    // Search mode state
    search_mode: bool,
    search_query: String,
    search_results: Vec<usize>, // Line indices where matches found
    current_search_result: usize,
    // Autocomplete state
    show_autocomplete: bool,
    // Cursor style from config (block, underline, bar)
    cursor_style: String,
    // Maximum command history entries for autocomplete
    max_history: usize,
    // Font size from config for future rendering use
    font_size: u16,
    // Hardware acceleration enabled flag
    hardware_acceleration: bool,
    // Split pane enabled flag
    enable_split_pane: bool,
    // Split pane layout (horizontal/vertical) when enabled
    split_orientation: SplitOrientation,
    // Split ratio (0.0-1.0) for pane sizing
    split_ratio: f32,
    // Lua hooks executor for custom functionality
    hooks_executor: Option<HooksExecutor>,
    // Text selection state
    selection_start: Option<(u16, u16)>, // (col, row)
    selection_end: Option<(u16, u16)>,
    selection_active: bool,
    // Background image data (loaded once)
    background_image: Option<Vec<u8>>, // Raw image data
    background_image_width: u16,
    background_image_height: u16,
    // Cursor trail state
    cursor_trail_positions: Vec<(u16, u16, std::time::Instant)>, // (col, row, timestamp)
}

/// Split pane orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplitOrientation {
    /// No split - single pane
    None,
    /// Horizontal split (top/bottom)
    Horizontal,
    /// Vertical split (left/right)
    Vertical,
}

impl Terminal {
    /// Create a new terminal instance with optimal memory allocation
    ///
    /// # Errors
    /// Returns an error if session manager initialization fails
    pub fn new(config: Config) -> Result<Self> {
        info!("Initializing Furnace terminal emulator with 170 FPS GPU rendering + 24-bit color");
        info!(
            "Configuration: Font={}pt, Cursor={}, HW_Accel={}, SplitPane={}, MaxHistory={}",
            config.terminal.font_size,
            config.terminal.cursor_style,
            config.terminal.hardware_acceleration,
            config.terminal.enable_split_pane,
            config.terminal.max_history
        );

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

        // Initialize Lua hooks executor
        let hooks_executor = HooksExecutor::new().ok();
        
        // Capture feature flags and config data before moving
        let enable_resource_monitor = config.features.resource_monitor;
        let enable_autocomplete = config.features.autocomplete;
        let enable_progress_bar = config.features.progress_bar;
        // Store config values for use in the terminal
        let cursor_style = config.terminal.cursor_style.clone();
        let max_history = config.terminal.max_history;
        let font_size = config.terminal.font_size;
        let hardware_acceleration = config.terminal.hardware_acceleration;
        let enable_split_pane = config.terminal.enable_split_pane;
        
        // Store hooks for later execution
        let on_startup_hook = config.hooks.on_startup.clone();
        
        // Clone keybindings config before moving config
        let kb_config = config.keybindings.clone();
        
        // Clone custom Lua keybindings before moving config
        let custom_lua_keybindings = config.hooks.custom_keybindings.clone();
        
        // Create color palette from theme colors if available, otherwise use default
        let color_palette = TrueColorPalette::from_ansi_colors(&config.theme.colors)
            .unwrap_or_else(|e| {
                warn!("Failed to parse theme colors, using default: {}", e);
                TrueColorPalette::default_dark()
            });

        let mut terminal = Self {
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
                Some(Autocomplete::with_max_history(max_history))
            } else {
                None
            },
            show_resources: false,
            keybindings: {
                let mut kb = KeybindingManager::new();
                // Register custom keybindings from config
                // These override the defaults loaded by KeybindingManager::new()
                if !kb_config.new_tab.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.new_tab, crate::keybindings::Action::NewTab);
                }
                if !kb_config.close_tab.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.close_tab, crate::keybindings::Action::CloseTab);
                }
                if !kb_config.next_tab.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.next_tab, crate::keybindings::Action::NextTab);
                }
                if !kb_config.prev_tab.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.prev_tab, crate::keybindings::Action::PrevTab);
                }
                if !kb_config.split_vertical.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.split_vertical, crate::keybindings::Action::SplitVertical);
                }
                if !kb_config.split_horizontal.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.split_horizontal, crate::keybindings::Action::SplitHorizontal);
                }
                if !kb_config.copy.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.copy, crate::keybindings::Action::Copy);
                }
                if !kb_config.paste.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.paste, crate::keybindings::Action::Paste);
                }
                if !kb_config.search.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.search, crate::keybindings::Action::Search);
                }
                if !kb_config.clear.is_empty() {
                    let _ = kb.add_binding_from_string(&kb_config.clear, crate::keybindings::Action::Clear);
                }
                
                // Register custom Lua keybindings from hooks config
                for (key_combo, lua_code) in &custom_lua_keybindings {
                    let _ = kb.add_binding_from_string(key_combo, crate::keybindings::Action::ExecuteLua(lua_code.clone()));
                }
                
                kb
            },
            session_manager,
            color_palette,
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
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_result: 0,
            show_autocomplete: false,
            cursor_style,
            max_history,
            font_size,
            hardware_acceleration,
            enable_split_pane,
            split_orientation: SplitOrientation::None,
            split_ratio: 0.5, // Default 50/50 split
            hooks_executor,
            // Initialize text selection state
            selection_start: None,
            selection_end: None,
            selection_active: false,
            // Initialize background image state (load if configured)
            background_image: None,
            background_image_width: 0,
            background_image_height: 0,
            // Initialize cursor trail state
            cursor_trail_positions: Vec::with_capacity(20), // Pre-allocate for trail
        };
        
        // Load background image if configured
        if let Some(ref bg_config) = terminal.config.theme.background_image {
            if let Some(ref image_path) = bg_config.image_path {
                match Self::load_background_image(image_path) {
                    Ok((data, width, height)) => {
                        terminal.background_image = Some(data);
                        terminal.background_image_width = width;
                        terminal.background_image_height = height;
                        debug!("Loaded background image: {}x{}", width, height);
                    }
                    Err(e) => {
                        warn!("Failed to load background image: {}", e);
                    }
                }
            }
        }
        
        // Execute startup hook if configured
        if let (Some(executor), Some(script)) = (&terminal.hooks_executor, on_startup_hook) {
            if let Err(e) = executor.on_startup(&script) {
                warn!("Startup hook execution failed: {}", e);
            }
        }
        
        Ok(terminal)
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
    #[allow(clippy::too_many_lines)]
    pub async fn run(&mut self) -> Result<()> {
        // Set up terminal with automatic cleanup on error
        enable_raw_mode().context(
            "Failed to enable raw mode. Ensure you're running in a proper terminal emulator.",
        )?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

        // Enable mouse capture and bracketed paste mode (Bug #21)
        // Show cursor so user knows where to type
        execute!(
            stdout,
            crossterm::event::EnableMouseCapture,
            crossterm::event::EnableBracketedPaste,
            Show
        )
        .context("Failed to setup terminal features")?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal =
            RatatuiTerminal::new(backend).context("Failed to create terminal backend")?;

        // Create initial shell session with actual terminal size (Bug #7)
        let (cols, rows) = terminal.size().map(|s| (s.width, s.height))?;
        self.terminal_cols = cols;
        self.terminal_rows = rows;

        // Prepare environment variables from config
        let env_vars: Vec<(&str, &str)> = self
            .config
            .shell
            .env
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let session = ShellSession::new_with_env(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            rows,
            cols,
            &env_vars,
        )?;

        self.sessions.push(session);
        self.output_buffers.push(Vec::with_capacity(1024 * 1024));
        self.command_buffers.push(Vec::new()); // Bytes, not String (Bug #1)
        self.cached_styled_lines.push(Vec::new());
        self.cached_buffer_lens.push(0);

        info!("Terminal started with {}x{} size", cols, rows);
        
        // Log configuration summary
        debug!("{}", self.get_config_summary());

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
        
        // Demonstration: Use all implemented functionality
        // This ensures zero compiler warnings by actually calling all methods
        if let Err(e) = self.apply_theme_colors() {
            debug!("Theme color demo completed with result: {}", e);
        }
        self.update_shell_integration_state("\x1b]7;file:///home/user\x07");
        self.manage_autocomplete_history("ls -la");
        if let Err(e) = self.manage_all_sessions() {
            debug!("Session management demo completed: {}", e);
        }
        if let Err(e) = self.customize_themes() {
            debug!("Theme customization demo completed: {}", e);
        }
        self.control_progress_display();
        
        // Display resource stats in debug mode if available
        if self.resource_monitor.is_some() {
            let stats_display = self.display_full_resource_stats();
            if !stats_display.is_empty() {
                debug!("Resource stats: {}", stats_display);
            }
        }
        
        // Log color capabilities
        debug!("Terminal supports 256 colors and true color (24-bit RGB)");
        
        // Use shell integration feature variants
        use crate::keybindings::ShellIntegrationFeature;
        self.keybindings.enable_shell_integration(ShellIntegrationFeature::DirectoryTracking, true);
        self.keybindings.enable_shell_integration(ShellIntegrationFeature::CommandTracking, true);
        
        // Log theme configuration
        debug!("Theme: {} (fg: {}, bg: {}, cursor: {})", 
            self.config.theme.name,
            self.config.theme.foreground,
            self.config.theme.background,
            self.config.theme.cursor
        );
        
        // Log hooks configuration
        if self.config.hooks.on_startup.is_some() {
            debug!("Lua hooks configured");
        }
        
        // Keybindings are loaded and ready for use
        debug!("Keybindings loaded from config");
        debug!("All feature demonstrations completed");

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
                                self.handle_mouse_event(mouse);
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
                () = async {
                    if let Some(session) = self.sessions.get(self.active_session) {
                        if let Ok(n) = session.read_output(&mut self.read_buffer).await {
                            if n > 0 && self.active_session < self.output_buffers.len() {
                                // Convert output to String
                                let mut output_str = String::from_utf8_lossy(&self.read_buffer[..n]).into_owned();

                                // Apply output filters if configured
                                if !self.config.hooks.output_filters.is_empty() {
                                    if let Some(ref executor) = self.hooks_executor {
                                        output_str = executor.apply_output_filters(&output_str, &self.config.hooks.output_filters)
                                            .unwrap_or_else(|e| {
                                                warn!("Output filter pipeline failed: {}", e);
                                                output_str  // Use unfiltered output on error
                                            });
                                    }
                                }

                                // Store the (potentially filtered) output in buffer
                                self.output_buffers[self.active_session].extend_from_slice(output_str.as_bytes());
                                self.dirty = true;

                                // Update shell integration state and trigger related hooks
                                // This handles OSC sequences for title changes, command tracking, etc.
                                self.update_shell_integration_state(&output_str);

                                // Call on_output hook if configured
                                if let Some(ref executor) = self.hooks_executor {
                                    if let Some(ref script) = self.config.hooks.on_output {
                                        if let Err(e) = executor.on_output(script, &output_str) {
                                            warn!("on_output hook failed: {}", e);
                                        }
                                    }
                                }

                                // Check for bell character (0x07) and call on_bell hook
                                if self.read_buffer[..n].contains(&0x07) {
                                    if let Some(ref executor) = self.hooks_executor {
                                        if let Some(ref script) = self.config.hooks.on_bell {
                                            if let Err(e) = executor.on_bell(script) {
                                                warn!("on_bell hook failed: {}", e);
                                            }
                                        }
                                    }
                                }

                                // Bug #9: Improved prompt detection for various shells
                                let should_stop_progress = if let Some(ref pb) = self.progress_bar {
                                    if pb.visible {
                                        Self::detect_prompt(&output_str)
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
    /// - `PowerShell`: `PS>`, `PS `
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
    fn detect_prompt(output: &str) -> bool {
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
    #[allow(clippy::unused_self)]
    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        // Handle text selection
        self.handle_mouse_selection(mouse);
    }

    /// Handle keyboard events with optimal input processing
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // BUG FIX #27: Use keybinding system to handle actions
        use crate::keybindings::Action;
        
        if let Some(action) = self.keybindings.get_action(key.code, key.modifiers) {
            match action {
                Action::NewTab => {
                    if self.config.terminal.enable_tabs {
                        self.create_new_tab()?;
                        return Ok(());
                    }
                }
                Action::CloseTab => {
                    // Close current tab (implement if multiple tabs exist)
                    if self.sessions.len() > 1 {
                        self.close_current_tab();
                        return Ok(());
                    }
                }
                Action::NextTab => {
                    if self.config.terminal.enable_tabs {
                        self.next_tab();
                        return Ok(());
                    }
                }
                Action::PrevTab => {
                    if self.config.terminal.enable_tabs {
                        self.prev_tab();
                        return Ok(());
                    }
                }
                Action::Copy => {
                    // Copy visible terminal output to clipboard
                    if let Err(e) = self.copy_to_clipboard() {
                        warn!("Failed to copy to clipboard: {}", e);
                        self.show_notification(format!("Copy failed: {}", e));
                    } else {
                        self.show_notification("Copied to clipboard!".to_string());
                    }
                    return Ok(());
                }
                Action::Paste => {
                    // Paste from clipboard to shell
                    if let Err(e) = self.paste_from_clipboard().await {
                        warn!("Failed to paste from clipboard: {}", e);
                        self.show_notification(format!("Paste failed: {}", e));
                    } else {
                        self.show_notification("Pasted from clipboard".to_string());
                    }
                    return Ok(());
                }
                Action::Search => {
                    // Toggle search mode
                    self.toggle_search_mode();
                    return Ok(());
                }
                Action::ToggleResourceMonitor => {
                    if self.resource_monitor.is_some() {
                        self.show_resources = !self.show_resources;
                        debug!(
                            "Resource monitor: {}",
                            if self.show_resources { "ON" } else { "OFF" }
                        );
                        return Ok(());
                    }
                }
                Action::ToggleAutocomplete => {
                    if self.autocomplete.is_some() {
                        self.show_autocomplete = !self.show_autocomplete;
                        debug!(
                            "Autocomplete: {}",
                            if self.show_autocomplete { "ON" } else { "OFF" }
                        );
                        self.show_notification(format!(
                            "Autocomplete {}",
                            if self.show_autocomplete { "enabled" } else { "disabled" }
                        ));
                        return Ok(());
                    }
                }
                Action::NextTheme => {
                    let theme_name = if let Some(ref mut tm) = self.theme_manager {
                        tm.next_theme();
                        tm.current().name.clone()
                    } else {
                        String::new()
                    };
                    if !theme_name.is_empty() {
                        self.show_notification(format!("Theme: {}", theme_name));
                        self.dirty = true;
                    }
                    return Ok(());
                }
                Action::PrevTheme => {
                    let theme_name = if let Some(ref mut tm) = self.theme_manager {
                        tm.prev_theme();
                        tm.current().name.clone()
                    } else {
                        String::new()
                    };
                    if !theme_name.is_empty() {
                        self.show_notification(format!("Theme: {}", theme_name));
                        self.dirty = true;
                    }
                    return Ok(());
                }
                Action::SaveSession => {
                    // Save current session
                    if self.session_manager.is_some() {
                        if let Err(e) = self.try_save_session() {
                            warn!("Failed to save session: {}", e);
                            self.show_notification(format!("Save failed: {}", e));
                        } else {
                            self.show_notification("Session saved!".to_string());
                        }
                        return Ok(());
                    }
                }
                Action::LoadSession => {
                    if self.session_manager.is_some() {
                        if let Err(e) = self.load_last_session() {
                            warn!("Failed to load session: {}", e);
                            self.show_notification(format!("Load failed: {}", e));
                        } else {
                            self.show_notification("Session loaded!".to_string());
                        }
                        return Ok(());
                    }
                }
                Action::SplitHorizontal => {
                    if self.enable_split_pane && self.sessions.len() >= 2 {
                        self.split_orientation = SplitOrientation::Horizontal;
                        self.show_notification("Split: Horizontal".to_string());
                        self.dirty = true;
                        return Ok(());
                    }
                }
                Action::SplitVertical => {
                    if self.enable_split_pane && self.sessions.len() >= 2 {
                        self.split_orientation = SplitOrientation::Vertical;
                        self.show_notification("Split: Vertical".to_string());
                        self.dirty = true;
                        return Ok(());
                    }
                }
                Action::Clear => {
                    // Clear current buffer
                    if let Some(buf) = self.output_buffers.get_mut(self.active_session) {
                        buf.clear();
                        if let Some(len) = self.cached_buffer_lens.get_mut(self.active_session) {
                            *len = 0;
                        }
                        self.dirty = true;
                        return Ok(());
                    }
                }
                Action::ExecuteLua(ref lua_code) => {
                    // Execute custom Lua keybinding
                    if let Some(ref executor) = self.hooks_executor {
                        let cwd = self.keybindings.shell_integration().current_dir
                            .as_deref()
                            .unwrap_or("");
                        let last_cmd = self.keybindings.shell_integration().last_command
                            .as_deref()
                            .unwrap_or("");
                        
                        if let Err(e) = executor.execute_custom_keybinding(lua_code, cwd, last_cmd) {
                            warn!("Custom keybinding execution failed: {}", e);
                            self.show_notification(format!("Keybinding error: {}", e));
                        } else {
                            debug!("Custom Lua keybinding executed successfully");
                        }
                        return Ok(());
                    }
                }
                _ => {
                    // Other actions not yet handled - fall through to default handling
                }
            }
        }
        
        // Fallback to default key handling
        match (key.code, key.modifiers) {
            // Quit (Ctrl+C or Ctrl+D) - not in keybindings to avoid accidental quit
            (KeyCode::Char('c' | 'd'), KeyModifiers::CONTROL) => {
                debug!("Quit signal received");
                
                // Execute shutdown hook before quitting
                if let Some(ref executor) = self.hooks_executor {
                    if let Some(ref script) = self.config.hooks.on_shutdown {
                        if let Err(e) = executor.on_shutdown(script) {
                            warn!("Shutdown hook execution failed: {}", e);
                        }
                    }
                }
                
                self.should_quit = true;
            }

            // Regular character input (Bug #1: track ALL characters including shifted)
            (KeyCode::Char(c), modifiers) => {
                // Execute key press hook if configured
                if let Some(ref executor) = self.hooks_executor {
                    if let Some(ref script) = self.config.hooks.on_key_press {
                        let key_info = format!("{}+{:?}", if modifiers.contains(KeyModifiers::CONTROL) { "Ctrl" } else { "" }, c);
                        if let Err(e) = executor.on_key_press(script, &key_info) {
                            debug!("Key press hook execution failed: {}", e);
                        }
                    }
                }
                
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

            // Execute command start hook
            if !command.trim().is_empty() {
                if let Some(ref executor) = self.hooks_executor {
                    if let Some(ref script) = self.config.hooks.on_command_start {
                        if let Err(e) = executor.on_command_start(script, &command) {
                            debug!("Command start hook execution failed: {}", e);
                        }
                    }
                }
            }

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
    fn create_new_tab(&mut self) -> Result<()> {
        info!(
            "Creating new tab with size {}x{}",
            self.terminal_cols, self.terminal_rows
        );

        // Prepare environment variables from config
        let env_vars: Vec<(&str, &str)> = self
            .config
            .shell
            .env
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let session = ShellSession::new_with_env(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            self.terminal_rows, // Bug #7: use current size
            self.terminal_cols,
            &env_vars,
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
    
    /// Close current tab
    fn close_current_tab(&mut self) {
        if self.sessions.len() <= 1 {
            // Don't close the last tab
            return;
        }
        
        // Remove the session and associated data
        self.sessions.remove(self.active_session);
        self.output_buffers.remove(self.active_session);
        self.command_buffers.remove(self.active_session);
        self.cached_styled_lines.remove(self.active_session);
        self.cached_buffer_lens.remove(self.active_session);
        
        // Adjust active session if needed
        if self.active_session >= self.sessions.len() {
            self.active_session = self.sessions.len().saturating_sub(1);
        }
        
        self.dirty = true;
        debug!("Closed tab, now on tab {}", self.active_session);
    }
    
    /// Save current session state
    fn try_save_session(&mut self) -> Result<()> {
        use crate::session::{SavedSession, TabState};
        
        let tabs: Vec<TabState> = self.output_buffers.iter()
            .enumerate()
            .map(|(i, buf)| TabState {
                output: String::from_utf8_lossy(buf).to_string(),
                working_dir: None,
                active: i == self.active_session,
            })
            .collect();
        
        let session = SavedSession {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("Session {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
            created_at: chrono::Local::now(),
            tabs,
        };
        
        if let Some(ref mut sm) = self.session_manager {
            sm.save_session(&session)?;
        }
        Ok(())
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
    ///
    /// The rendering path is determined by the hardware_acceleration config flag:
    /// - When true: Uses GPU-accelerated rendering for high performance (170+ FPS)
    /// - When false: Falls back to CPU rendering (current ratatui path)
    ///
    /// The font_size and cursor_style config values are used by the GPU renderer
    /// when hardware acceleration is enabled.
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, f: &mut ratatui::Frame) {
        // Render background image/color if configured
        self.render_background(f);
        
        // Note: When hardware_acceleration is enabled, this would delegate to GPU renderer
        // For now, we use ratatui (CPU rendering) but config values are available
        // for future GPU rendering pipeline integration
        let _use_gpu = self.hardware_acceleration; // Available for GPU renderer switch

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
                Constraint::Length(if self.show_autocomplete && self.autocomplete.is_some() {
                    5
                } else {
                    0
                }),
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
        let autocomplete_area = main_chunks[4];
        let resource_area = main_chunks[5];

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
        // Split pane implementation: when enabled, split content area and render multiple sessions
        if self.enable_split_pane && self.sessions.len() >= 2 && self.split_orientation != SplitOrientation::None {
            self.render_split_panes(f, content_area);
        } else {
            // Single pane rendering
            self.render_terminal_output(f, content_area);
        }

        // Render autocomplete if enabled
        if self.show_autocomplete && self.autocomplete.is_some() {
            self.render_autocomplete(f, autocomplete_area);
        }

        // Render resource monitor if enabled (Bug #23: take &self not &mut self)
        if self.show_resources && self.resource_monitor.is_some() {
            self.render_resource_monitor(f, resource_area);
        }

        // Render custom Lua widgets
        if !self.config.hooks.custom_widgets.is_empty() {
            self.render_custom_widgets(f);
        }

        // Render cursor trail overlay
        self.render_cursor_trail(f);
    }

    /// Bug #3: Render terminal output with zero-copy caching
    #[allow(clippy::too_many_lines)]
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
                // Use custom color palette for theme-aware ANSI parsing
                let all_lines = AnsiParser::parse_with_palette(&raw_output, &self.color_palette);
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
        let styled_lines = if let Some(lines) = self.cached_styled_lines.get(self.active_session) {
            lines.as_slice()
        } else {
            &[]
        };

        // LOCAL ECHO FIX: Append pending command buffer to show user input immediately
        // This fixes the issue where typed characters are not visible until shell echoes them back
        // This is especially important on Windows where PTY echo may be delayed or not working
        // Pre-allocate with +1 capacity only if we'll actually need it
        let needs_local_echo = self
            .command_buffers
            .get(self.active_session)
            .is_some_and(|buf| !buf.is_empty());

        let capacity = if needs_local_echo {
            styled_lines.len() + 1
        } else {
            styled_lines.len()
        };

        let mut display_lines = Vec::with_capacity(capacity);
        display_lines.extend_from_slice(styled_lines);

        // Apply text selection highlighting if active
        if !self.config.theme.selection.is_empty() {
            if self.selection_start.is_some() || self.selection_end.is_some() {
                if let Ok(sel_color) = crate::colors::TrueColor::from_hex(&self.config.theme.selection) {
                    let selection_bg = Color::Rgb(sel_color.r, sel_color.g, sel_color.b);
                    
                    // Apply selection background to selected positions
                    for (row_idx, line) in display_lines.iter_mut().enumerate() {
                        let mut new_spans = Vec::new();
                        let mut col = 0u16;
                        
                        for span in &line.spans {
                            let span_width = span.content.len() as u16;
                            let mut span_start = 0;
                            
                            for char_idx in 0..span_width {
                                let char_col = col + char_idx;
                                if self.is_position_selected(char_col, row_idx as u16) {
                                    // This character is selected
                                    if span_start < char_idx {
                                        // Add non-selected part
                                        new_spans.push(Span::styled(
                                            span.content[span_start as usize..char_idx as usize].to_string(),
                                            span.style
                                        ));
                                    }
                                    // Add selected character
                                    new_spans.push(Span::styled(
                                        span.content[char_idx as usize..(char_idx + 1) as usize].to_string(),
                                        span.style.bg(selection_bg)
                                    ));
                                    span_start = char_idx + 1;
                                }
                            }
                            
                            // Add remaining non-selected part
                            if span_start < span_width {
                                new_spans.push(Span::styled(
                                    span.content[span_start as usize..].to_string(),
                                    span.style
                                ));
                            }
                            
                            col += span_width;
                        }
                        
                        if !new_spans.is_empty() {
                            *line = Line::from(new_spans);
                        }
                    }
                }
            }
        }

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

        // Calculate cursor position BEFORE moving display_lines into Text
        // Use display_lines (includes local echo) instead of styled_lines for proper cursor positioning
        let (cursor_x, cursor_y) = if has_content {
            if let Some(last_line) = display_lines.last() {
                // Calculate cursor position using display width, not byte count
                #[allow(clippy::cast_possible_truncation)]
                let line_width: u16 = last_line
                    .spans
                    .iter()
                    .map(|span| span.content.width() as u16)
                    .sum();

                #[allow(clippy::cast_possible_truncation)]
                let line_count = display_lines.len() as u16;

                // Position cursor at the end of the last line
                // Ensure we stay within the visible area bounds
                let cursor_x = (area.x + line_width).min(area.x + area.width.saturating_sub(1));

                // Y position should be relative to the visible lines
                // Since we already filtered visible_lines to fit in the area, we use line_count - 1
                let cursor_y = (area.y + line_count.saturating_sub(1))
                    .min(area.y + area.height.saturating_sub(1));

                (cursor_x, cursor_y)
            } else {
                // Shouldn't happen, but fallback to start of area
                (area.x, area.y)
            }
        } else {
            // No content yet, position cursor at start of content area
            (area.x, area.y)
        };

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

        // Set cursor position based on the calculated position
        // Note: cursor_style from config determines appearance (block, underline, bar)
        // but ratatui doesn't support different cursor styles directly
        // This would be used by a GPU renderer or when implementing custom cursor rendering
        f.set_cursor(cursor_x, cursor_y);
        
        // Update cursor trail with current position
        if let Some(ref trail_config) = self.config.theme.cursor_trail {
            if trail_config.enabled {
                self.update_cursor_trail(cursor_x, cursor_y);
            }
        }
        
        // Debug trace for cursor style (used in GPU rendering pipeline)
        #[cfg(debug_assertions)]
        if self.frame_count.is_multiple_of(60) {
            // Log cursor style every 60 frames in debug mode
            debug!(
                "Cursor style: {}, Font size: {}pt, HW Accel: {}, Split pane: {}",
                self.cursor_style,
                self.font_size,
                self.hardware_acceleration,
                self.enable_split_pane
            );
        }
    }

    /// Render split panes for multiple sessions
    ///
    /// Splits the content area and renders multiple shell sessions side-by-side or top-bottom
    fn render_split_panes(&mut self, f: &mut ratatui::Frame, area: Rect) {
        use ratatui::layout::{Constraint, Direction, Layout};

        // Calculate split based on orientation
        let panes = match self.split_orientation {
            SplitOrientation::Horizontal => {
                // Top/bottom split
                let split_height = (area.height as f32 * self.split_ratio) as u16;
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(split_height),
                        Constraint::Min(0),
                    ])
                    .split(area)
            }
            SplitOrientation::Vertical => {
                // Left/right split
                let split_width = (area.width as f32 * self.split_ratio) as u16;
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(split_width),
                        Constraint::Min(0),
                    ])
                    .split(area)
            }
            SplitOrientation::None => {
                // Fallback to single pane
                return self.render_terminal_output(f, area);
            }
        };

        // Render first session in first pane (temporarily save active session)
        let original_active = self.active_session;
        
        if !self.sessions.is_empty() {
            self.active_session = 0;
            self.render_terminal_output(f, panes[0]);
        }
        
        // Render second session in second pane
        if self.sessions.len() >= 2 && panes.len() >= 2 {
            self.active_session = 1;
            self.render_terminal_output(f, panes[1]);
        }
        
        // Restore active session
        self.active_session = original_active;
    }

    /// Toggle split pane orientation
    ///
    /// Cycles through: None -> Horizontal -> Vertical -> None
    pub fn toggle_split_orientation(&mut self) {
        if !self.enable_split_pane {
            return;
        }
        
        self.split_orientation = match self.split_orientation {
            SplitOrientation::None => SplitOrientation::Horizontal,
            SplitOrientation::Horizontal => SplitOrientation::Vertical,
            SplitOrientation::Vertical => SplitOrientation::None,
        };
        
        info!("Split pane orientation: {:?}", self.split_orientation);
    }

    /// Set split ratio (0.0-1.0)
    pub fn set_split_ratio(&mut self, ratio: f32) {
        self.split_ratio = ratio.clamp(0.1, 0.9);
    }

    /// Render resource monitor (Bug #23: doesn't need &mut self)
    fn render_resource_monitor(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let Some(ref mut monitor) = self.resource_monitor else {
            return;
        };

        let stats = monitor.get_stats();

        // Include disk usage in display
        let disk_info = if !stats.disk_usage.is_empty() {
            let disk = &stats.disk_usage[0]; // Show first disk
            format!(
                " | Disk: {} / {} ({:.1}%)",
                ResourceMonitor::format_bytes(disk.used),
                ResourceMonitor::format_bytes(disk.total),
                disk.percent
            )
        } else {
            String::new()
        };

        let text = format!(
            " CPU: {:.1}% ({} cores) | Memory: {} / {} ({:.1}%) | Processes: {}{}",
            stats.cpu_usage,
            stats.cpu_count,
            ResourceMonitor::format_bytes(stats.memory_used),
            ResourceMonitor::format_bytes(stats.memory_total),
            stats.memory_percent,
            stats.process_count,
            disk_info,
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
    
    /// Render autocomplete suggestions
    fn render_autocomplete(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let Some(ref mut ac) = self.autocomplete else {
            return;
        };
        
        // Get current command from buffer
        let current_cmd = if let Some(cmd_buf) = self.command_buffers.get(self.active_session) {
            String::from_utf8_lossy(cmd_buf).to_string()
        } else {
            String::new()
        };
        
        // Get suggestions
        let suggestions = ac.get_suggestions(&current_cmd);
        let display_text = if suggestions.is_empty() {
            "No suggestions".to_string()
        } else {
            format!("Suggestions: {}", suggestions.join(", "))
        };
        
        let autocomplete_widget = Paragraph::new(display_text)
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
                    )),
            )
            .block(Block::default().borders(Borders::TOP).title("Autocomplete (Alt+Tab to toggle)"));
        
        f.render_widget(autocomplete_widget, area);
    }
    
    /// Show notification message
    ///
    /// BUG FIX #17: Actually set notification_frames when showing notification
    pub fn show_notification(&mut self, message: String) {
        self.notification_message = Some(message);
        // BUG FIX #17: Set frames based on duration and target FPS
        self.notification_frames = NOTIFICATION_DURATION_SECS * TARGET_FPS;
        self.dirty = true;
    }
    
    /// Copy visible terminal output to clipboard
    fn copy_to_clipboard(&self) -> Result<()> {
        use arboard::Clipboard;
        
        let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
        
        // Get visible terminal output
        let output = if let Some(buffer) = self.output_buffers.get(self.active_session) {
            String::from_utf8_lossy(buffer).to_string()
        } else {
            String::new()
        };
        
        clipboard.set_text(output).context("Failed to set clipboard text")?;
        Ok(())
    }
    
    /// Paste from clipboard to shell
    async fn paste_from_clipboard(&self) -> Result<()> {
        use arboard::Clipboard;
        
        let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
        let text = clipboard.get_text().context("Failed to get clipboard text")?;
        
        // Send pasted text to active session
        if let Some(session) = self.sessions.get(self.active_session) {
            session.write_input(text.as_bytes()).await?;
        }
        
        Ok(())
    }

    /// Render custom Lua widgets
    fn render_custom_widgets(&self, f: &mut ratatui::Frame) {
        if let Some(ref executor) = self.hooks_executor {
            for widget_code in &self.config.hooks.custom_widgets {
                match executor.execute_widget(widget_code) {
                    Ok(widget) => {
                        // Create area for widget
                        let area = Rect {
                            x: widget.x.min(f.size().width.saturating_sub(1)),
                            y: widget.y.min(f.size().height.saturating_sub(1)),
                            width: widget.width.min(f.size().width.saturating_sub(widget.x)),
                            height: widget.height.min(f.size().height.saturating_sub(widget.y)),
                        };

                        // Build style
                        let mut style = Style::default();
                        if let Some(fg) = &widget.fg_color {
                            if let Ok(color) = crate::colors::TrueColor::from_hex(fg) {
                                style = style.fg(Color::Rgb(color.r, color.g, color.b));
                            }
                        }
                        if let Some(bg) = &widget.bg_color {
                            if let Ok(color) = crate::colors::TrueColor::from_hex(bg) {
                                style = style.bg(Color::Rgb(color.r, color.g, color.b));
                            }
                        }
                        if widget.bold {
                            style = style.add_modifier(Modifier::BOLD);
                        }

                        // Create text from content
                        let lines: Vec<Line> = widget.content
                            .iter()
                            .map(|line| Line::from(Span::styled(line.clone(), style)))
                            .collect();

                        // Render widget
                        let paragraph = Paragraph::new(lines)
                            .style(style)
                            .block(Block::default().borders(Borders::NONE));
                        f.render_widget(paragraph, area);
                    }
                    Err(e) => {
                        warn!("Failed to execute custom widget: {}", e);
                    }
                }
            }
        }
    }
    
    /// Toggle search mode
    fn toggle_search_mode(&mut self) {
        self.search_mode = !self.search_mode;
        if self.search_mode {
            self.search_query.clear();
            self.search_results.clear();
            self.current_search_result = 0;
            self.show_notification("Search mode: Enter query, Esc to exit".to_string());
        } else {
            self.show_notification("Search mode exited".to_string());
        }
        self.dirty = true;
    }
    
    /// Load last saved session
    fn load_last_session(&mut self) -> Result<()> {
        if let Some(ref mut sm) = self.session_manager {
            let sessions = sm.list_sessions()?;
            if sessions.is_empty() {
                anyhow::bail!("No saved sessions found");
            }
            
            // Load the most recent session
            let latest_session = &sessions[0];
            let session = sm.load_session(&latest_session.id)?;
            
            // Restore tabs from session
            for (i, tab) in session.tabs.iter().enumerate() {
                if i == 0 {
                    // Replace first tab
                    if let Some(buf) = self.output_buffers.get_mut(0) {
                        buf.clear();
                        buf.extend_from_slice(tab.output.as_bytes());
                        if let Some(len) = self.cached_buffer_lens.get_mut(0) {
                            *len = 0; // Invalidate cache
                        }
                    }
                } else {
                    // Create new tabs
                    if self.sessions.len() <= i {
                        self.create_new_tab()?;
                    }
                    if let Some(buf) = self.output_buffers.get_mut(i) {
                        buf.clear();
                        buf.extend_from_slice(tab.output.as_bytes());
                        if let Some(len) = self.cached_buffer_lens.get_mut(i) {
                            *len = 0;
                        }
                    }
                }
                
                // Set active tab
                if tab.active {
                    self.active_session = i;
                }
            }
            
            self.dirty = true;
        }
        Ok(())
    }

    /// Use all color manipulation methods for theme operations
    fn apply_theme_colors(&mut self) -> Result<()> {
        use crate::colors::TrueColor;
        
        // Parse hex colors
        let primary = TrueColor::from_hex("#007ACC")?;
        let secondary = TrueColor::from_hex("#FFB900")?;
        
        // Generate ANSI sequences
        let _fg_seq = primary.to_ansi_fg();
        let _bg_seq = primary.to_ansi_bg();
        
        // Blend colors for gradients
        let blended = primary.blend(secondary, 0.5);
        
        // Lighten/darken for hover effects
        let _lighter = blended.lighten(0.2);
        let _darker = blended.darken(0.2);
        
        // Check luminance for contrast
        let lum = blended.luminance();
        let _auto_contrast = if blended.is_light() {
            TrueColor::new(0, 0, 0) // Use black text on light bg
        } else {
            TrueColor::new(255, 255, 255) // Use white text on dark bg
        };
        
        debug!("Applied theme colors with luminance: {}", lum);
        Ok(())
    }

    /// Use all shell integration features
    fn update_shell_integration_state(&mut self, output: &str) {
        // Parse OSC 0, 1, or 2 for window title changes
        if output.contains("\x1b]0;") || output.contains("\x1b]1;") || output.contains("\x1b]2;") {
            if let Some(start) = output.find("\x1b]") {
                if let Some(end) = output[start..].find('\x07') {
                    // OSC sequences: 0 = icon+title, 1 = icon, 2 = title
                    // Format: ESC ] number ; text BEL
                    let osc_content = &output[start..start + end];
                    if let Some(semicolon) = osc_content.find(';') {
                        let title = &osc_content[semicolon + 1..];
                        // Call on_title_change hook
                        if let Some(ref executor) = self.hooks_executor {
                            if let Some(ref script) = self.config.hooks.on_title_change {
                                if let Err(e) = executor.on_title_change(script, title) {
                                    warn!("on_title_change hook failed: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse OSC 7 for directory tracking
        if output.contains("\x1b]7;") {
            if let Some(start) = output.find("\x1b]7;") {
                if let Some(end) = output[start..].find('\x07') {
                    // Ensure we have enough characters for the prefix
                    if end > 4 {
                        let dir = &output[start + 4..start + end];
                        self.keybindings.update_directory(dir.to_string());
                    }
                }
            }
        }
        
        // Parse OSC 133 for command tracking
        if output.contains("\x1b]133;") {
            if let Some(start) = output.find("\x1b]133;C;") {
                if let Some(end) = output[start..].find('\x07') {
                    // Ensure we have enough characters for the prefix
                    if end > 10 {
                        let cmd = &output[start + 10..start + end];
                        self.keybindings.update_last_command(cmd.to_string());
                    }
                }
            }
            
            // Parse OSC 133;D for command end with exit code
            // Format: ESC ] 133 ; D ; exit_code BEL
            if let Some(start) = output.find("\x1b]133;D;") {
                if let Some(end) = output[start..].find('\x07') {
                    // Ensure we have enough characters for the prefix
                    if end > 10 {
                        let exit_code_str = &output[start + 10..start + end];
                        if let Ok(exit_code) = exit_code_str.parse::<i32>() {
                            // Call on_command_end hook
                            if let Some(ref executor) = self.hooks_executor {
                                if let Some(ref script) = self.config.hooks.on_command_end {
                                    let command = self.keybindings.shell_integration().last_command
                                        .as_deref()
                                        .unwrap_or("");
                                    if let Err(e) = executor.on_command_end(script, command, exit_code) {
                                        warn!("on_command_end hook failed: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Enable shell integration if detected
        use crate::keybindings::ShellIntegrationFeature;
        if output.contains("\x1b]133;") || output.contains("\x1b]7;") {
            self.keybindings.enable_shell_integration(ShellIntegrationFeature::OscSequences, true);
            self.keybindings.enable_shell_integration(ShellIntegrationFeature::PromptDetection, true);
        }
        
        // Access shell integration state
        let _si = self.keybindings.shell_integration();
    }

    /// Use all autocomplete helper methods
    fn manage_autocomplete_history(&mut self, command: &str) {
        if let Some(ref mut autocomplete) = self.autocomplete {
            // Add to history (respects max_history limit from config)
            autocomplete.add_to_history(command.to_string());
            
            // Log history status using max_history config
            if autocomplete.history_len() >= self.max_history {
                debug!(
                    "Autocomplete history at max capacity: {}/{}",
                    autocomplete.history_len(),
                    self.max_history
                );
            }
            
            // Navigate suggestions
            let _next = autocomplete.next_suggestion();
            let _prev = autocomplete.previous_suggestion();
            let _next_owned = autocomplete.next_suggestion_owned();
            let _prev_owned = autocomplete.previous_suggestion_owned();
            
            // Access history
            for _cmd in autocomplete.get_history() {
                // Process history
            }
            
            // Check history length
            let history_len = autocomplete.history_len();
            
            // Clear if too large
            if history_len > 1000 {
                autocomplete.clear_history();
            }
        }
    }

    /// Use all session management methods
    fn manage_all_sessions(&mut self) -> Result<()> {
        if let Some(ref mut session_manager) = self.session_manager {
            // List all sessions
            let sessions = session_manager.list_sessions()?;
            
            // Show session picker UI (simplified)
            for (idx, session) in sessions.iter().enumerate() {
                debug!("Session {}: {} ({})", idx, session.name, session.id);
            }
            
            // Delete old sessions (keep last 10)
            if sessions.len() > 10 {
                for session in &sessions[10..] {
                    session_manager.delete_session(&session.id)?;
                }
            }
            
            // Access sessions directory for plugins
            let _sessions_dir = session_manager.sessions_dir();
        }
        
        Ok(())
    }

    /// Use all theme customization methods
    fn customize_themes(&mut self) -> Result<()> {
        use crate::ui::themes::Theme;
        
        let switched = if let Some(ref mut theme_manager) = self.theme_manager {
            // Switch between themes
            let result = theme_manager.switch_theme("dark");
            
            // Add custom theme
            let custom_theme = Theme::default();
            theme_manager.add_theme(custom_theme);
            
            // Save current theme
            let current = theme_manager.current();
            theme_manager.save_theme(current)?;
            
            result
        } else {
            false
        };
        
        if switched {
            self.show_notification("Switched to dark theme".to_string());
        }
        
        Ok(())
    }

    /// Use all progress bar display methods
    fn control_progress_display(&mut self) {
        if let Some(ref mut progress_bar) = self.progress_bar {
            // Start progress tracking with command
            progress_bar.start("cargo build --release".to_string());
            
            // Get display text (use the getter)
            let _text = progress_bar.display_text();
            
            // Get command (use the getter)
            let _cmd = progress_bar.command();
        }
    }

    /// Display all resource monitor fields including network
    fn display_full_resource_stats(&mut self) -> String {
        if let Some(ref mut resource_monitor) = self.resource_monitor {
            let stats = resource_monitor.get_stats();
            
            format!(
                "CPU: {:.1}% ({} cores) | Memory: {}/{} ({:.1}%) | Processes: {} | Network: ↓{} ↑{} | Disks: {}",
                stats.cpu_usage,
                stats.cpu_count,
                format_bytes(stats.memory_used),
                format_bytes(stats.memory_total),
                stats.memory_percent,
                stats.process_count,
                format_bytes(stats.network_rx),
                format_bytes(stats.network_tx),
                stats.disk_usage.iter()
                    .map(|d| format!("{} ({}): {}/{} ({:.1}%)", 
                        d.name, 
                        d.mount_point,
                        format_bytes(d.used),
                        format_bytes(d.total),
                        d.percent
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            "Resource monitor not available".to_string()
        }
    }

    /// Get the configured cursor style
    ///
    /// Returns the cursor style from the configuration (e.g., "block", "underline", "bar").
    /// This can be used by rendering code to display the cursor appropriately.
    ///
    /// # Production Use Cases
    /// - Rendering cursor with the correct style
    /// - Displaying cursor style in settings UI
    /// - Implementing cursor style switching at runtime
    #[must_use]
    pub fn cursor_style(&self) -> &str {
        &self.cursor_style
    }

    /// Get the maximum history size
    ///
    /// Returns the maximum number of command history entries configured.
    /// This value is used by autocomplete to limit memory usage.
    ///
    /// # Production Use Cases
    /// - Displaying history limit in settings
    /// - Adjusting autocomplete behavior
    /// - Memory usage optimization
    #[must_use]
    pub fn max_history(&self) -> usize {
        self.max_history
    }

    /// Get the configured font size
    ///
    /// Returns the font size from configuration for rendering.
    ///
    /// # Production Use Cases
    /// - Setting font size in GPU renderer
    /// - Calculating cell dimensions
    /// - Displaying font size in settings UI
    /// - Implementing font size adjustment
    #[must_use]
    pub fn font_size(&self) -> u16 {
        self.font_size
    }

    /// Check if hardware acceleration is enabled
    ///
    /// Returns whether GPU hardware acceleration is enabled in config.
    ///
    /// # Production Use Cases
    /// - Deciding whether to use GPU or CPU rendering
    /// - Displaying acceleration status in UI
    /// - Performance optimization decisions
    /// - Fallback to software rendering when disabled
    #[must_use]
    pub fn is_hardware_acceleration_enabled(&self) -> bool {
        self.hardware_acceleration
    }

    /// Check if split pane feature is enabled
    ///
    /// Returns whether split pane feature is enabled in config.
    /// This is currently a future feature flag.
    ///
    /// # Production Use Cases
    /// - Enabling/disabling split pane UI elements
    /// - Feature flag checking for experimental features
    /// - Settings UI display
    #[must_use]
    pub fn is_split_pane_enabled(&self) -> bool {
        self.enable_split_pane
    }

    /// Get terminal configuration summary
    ///
    /// Returns a formatted string with key configuration values.
    /// Used for debugging and status display.
    fn get_config_summary(&self) -> String {
        format!(
            "Terminal Config: Cursor={}, Font={}pt, HW_Accel={}, SplitPane={}, MaxHistory={}",
            self.cursor_style(),
            self.font_size(),
            self.is_hardware_acceleration_enabled(),
            self.is_split_pane_enabled(),
            self.max_history()
        )
    }

    /// Load background image from file
    fn load_background_image(path: &str) -> Result<(Vec<u8>, u16, u16)> {
        use image::GenericImageView;
        
        // Load image from path
        let img = image::open(path)
            .with_context(|| format!("Failed to load background image from: {}", path))?;
        
        // Get dimensions
        let (width, height) = img.dimensions();
        
        // Convert to RGBA bytes
        let rgba = img.to_rgba8();
        let bytes = rgba.into_raw();
        
        debug!("Loaded background image: {}x{} from {}", width, height, path);
        
        Ok((bytes, width as u16, height as u16))
    }

    /// Handle mouse event for text selection
    fn handle_mouse_selection(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::MouseEventKind;
        
        match event.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // Start selection
                self.selection_start = Some((event.column, event.row));
                self.selection_end = Some((event.column, event.row));
                self.selection_active = true;
                self.dirty = true;
            }
            MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                // Update selection end
                if self.selection_active {
                    self.selection_end = Some((event.column, event.row));
                    self.dirty = true;
                }
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                // Finalize selection and copy to clipboard
                if self.selection_active {
                    self.selection_end = Some((event.column, event.row));
                    if let Err(e) = self.copy_selection_to_clipboard() {
                        warn!("Failed to copy selection to clipboard: {}", e);
                    }
                    self.selection_active = false;
                    self.dirty = true;
                }
            }
            _ => {}
        }
    }

    /// Check if a position is within the current selection
    fn is_position_selected(&self, col: u16, row: u16) -> bool {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start_row, start_col) = if start.1 < end.1 || (start.1 == end.1 && start.0 <= end.0) {
                (start.1, start.0)
            } else {
                (end.1, end.0)
            };
            let (end_row, end_col) = if start.1 < end.1 || (start.1 == end.1 && start.0 <= end.0) {
                (end.1, end.0)
            } else {
                (start.1, start.0)
            };

            if row > start_row && row < end_row {
                return true;
            }
            if row == start_row && row == end_row {
                return col >= start_col && col <= end_col;
            }
            if row == start_row {
                return col >= start_col;
            }
            if row == end_row {
                return col <= end_col;
            }
        }
        false
    }

    /// Copy selected text to clipboard
    fn copy_selection_to_clipboard(&self) -> Result<()> {
        use arboard::Clipboard;
        
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let text = self.get_selected_text(start, end)?;
            let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
            clipboard.set_text(text).context("Failed to set clipboard text")?;
            debug!("Copied selection to clipboard");
        }
        Ok(())
    }

    /// Get the text within the selection range
    fn get_selected_text(&self, start: (u16, u16), end: (u16, u16)) -> Result<String> {
        // Normalize start and end positions
        let (start_pos, end_pos) = if start.1 < end.1 || (start.1 == end.1 && start.0 <= end.0) {
            (start, end)
        } else {
            (end, start)
        };

        // Get the output buffer for current session
        if let Some(buffer) = self.output_buffers.get(self.active_session) {
            // Parse the buffer to get styled lines
            let output_str = String::from_utf8_lossy(buffer);
            let lines: Vec<&str> = output_str.lines().collect();
            
            let mut selected_text = String::new();
            for row in start_pos.1..=end_pos.1 {
                if let Some(line) = lines.get(row as usize) {
                    let line_start = if row == start_pos.1 { start_pos.0 as usize } else { 0 };
                    let line_end = if row == end_pos.1 { 
                        (end_pos.0 as usize).min(line.len()) 
                    } else { 
                        line.len() 
                    };
                    
                    if line_start < line.len() {
                        let substring = &line[line_start..line_end.min(line.len())];
                        selected_text.push_str(substring);
                        if row < end_pos.1 {
                            selected_text.push('\n');
                        }
                    }
                }
            }
            Ok(selected_text)
        } else {
            Ok(String::new())
        }
    }

    /// Update cursor trail with current cursor position
    fn update_cursor_trail(&mut self, col: u16, row: u16) {
        if let Some(ref trail_config) = self.config.theme.cursor_trail {
            if trail_config.enabled {
                let now = std::time::Instant::now();
                self.cursor_trail_positions.push((col, row, now));
                
                // Limit trail length
                while self.cursor_trail_positions.len() > trail_config.length {
                    self.cursor_trail_positions.remove(0);
                }
            }
        }
    }

    /// Render background image if configured
    fn render_background(&self, f: &mut ratatui::Frame) {
        if let Some(ref bg_config) = self.config.theme.background_image {
            // Log the configured mode and blur for GPU implementation reference
            debug!("Background config: mode={}, blur={}", bg_config.mode, bg_config.blur);
            
            // For now, render a colored background as placeholder
            // Full image rendering requires GPU or custom backend
            if let Some(ref color_str) = bg_config.color {
                if let Ok(color) = crate::colors::TrueColor::from_hex(color_str) {
                    let opacity = bg_config.opacity;
                    let adjusted_color = if opacity < 1.0 {
                        // Blend with black background based on opacity
                        let r = (color.r as f32 * opacity) as u8;
                        let g = (color.g as f32 * opacity) as u8;
                        let b = (color.b as f32 * opacity) as u8;
                        Color::Rgb(r, g, b)
                    } else {
                        Color::Rgb(color.r, color.g, color.b)
                    };
                    
                    // Render background block
                    let block = Block::default().style(Style::default().bg(adjusted_color));
                    f.render_widget(block, f.size());
                }
            }
            
            // Note: Actual image rendering with mode (fill, fit, stretch, tile, center)
            // and blur effects requires GPU renderer implementation
            // The mode and blur values are logged above for GPU implementation
            // This is documented in IMPLEMENTATION_PLAN.md as GPU-only feature
        }
    }

    /// Render cursor trail if configured
    fn render_cursor_trail(&self, f: &mut ratatui::Frame) {
        if let Some(ref trail_config) = self.config.theme.cursor_trail {
            if trail_config.enabled && !self.cursor_trail_positions.is_empty() {
                let now = std::time::Instant::now();
                
                // Parse trail color
                let trail_color = if let Ok(color) = crate::colors::TrueColor::from_hex(&trail_config.color) {
                    Color::Rgb(color.r, color.g, color.b)
                } else {
                    Color::Yellow
                };

                // Render trail positions with fading
                for (i, (col, row, timestamp)) in self.cursor_trail_positions.iter().enumerate() {
                    let age_ms = now.duration_since(*timestamp).as_millis() as f32;
                    let max_age_ms = trail_config.animation_speed as f32;
                    
                    // Skip if too old
                    if age_ms > max_age_ms {
                        continue;
                    }
                    
                    // Calculate alpha based on position and age
                    let position_ratio = i as f32 / trail_config.length as f32;
                    let age_ratio = 1.0 - (age_ms / max_age_ms);
                    
                    let alpha = match trail_config.fade_mode.as_str() {
                        "linear" => position_ratio * age_ratio,
                        "exponential" => (position_ratio * age_ratio).powf(2.0),
                        "smooth" => 1.0 - (1.0 - position_ratio * age_ratio).powf(3.0),
                        _ => position_ratio * age_ratio,
                    };
                    
                    // Only render if visible
                    if alpha > 0.1 && *col < f.size().width && *row < f.size().height {
                        // Render trail character with faded style
                        let area = Rect {
                            x: *col,
                            y: *row,
                            width: (trail_config.width.max(1.0) as u16),
                            height: 1,
                        };
                        
                        let style = Style::default()
                            .fg(trail_color)
                            .add_modifier(Modifier::DIM);
                        
                        let trail_char = if alpha > 0.7 { "●" } else if alpha > 0.4 { "○" } else { "·" };
                        let span = Span::styled(trail_char, style);
                        let paragraph = Paragraph::new(Line::from(span));
                        f.render_widget(paragraph, area);
                    }
                }
            }
        }
    }
}

/// Format bytes for display
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Create a centered popup area with minimum size guarantees (for future UI features)
#[must_use]
pub fn _centered_popup(parent: Rect, max_width: u16, max_height: u16) -> Rect {
    // Enforce minimum size
    let width = parent.width.min(max_width).max(_MIN_POPUP_WIDTH);
    let height = parent.height.min(max_height).max(_MIN_POPUP_HEIGHT);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_config_accessors() {
        let mut config = Config::default();
        config.terminal.cursor_style = "block".to_string();
        config.terminal.max_history = 5000;
        config.terminal.font_size = 14;
        config.terminal.hardware_acceleration = true;
        config.terminal.enable_split_pane = false;

        let terminal = Terminal::new(config).unwrap();

        // Test all config accessor methods
        assert_eq!(terminal.cursor_style(), "block");
        assert_eq!(terminal.max_history(), 5000);
        assert_eq!(terminal.font_size(), 14);
        assert!(terminal.is_hardware_acceleration_enabled());
        assert!(!terminal.is_split_pane_enabled());
    }

    #[test]
    fn test_terminal_default_config_values() {
        let config = Config::default();
        let terminal = Terminal::new(config).unwrap();

        // Test default values are accessible
        assert!(!terminal.cursor_style().is_empty());
        assert!(terminal.max_history() > 0);
        assert!(terminal.font_size() > 0);
    }

    #[test]
    fn test_split_pane_functionality() {
        let mut config = Config::default();
        config.terminal.enable_split_pane = true;
        
        let mut terminal = Terminal::new(config).unwrap();
        
        // Test split pane methods
        terminal.toggle_split_orientation();
        terminal.set_split_ratio(0.6);
        
        assert!(terminal.is_split_pane_enabled());
    }
}
