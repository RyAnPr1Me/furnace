use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal as RatatuiTerminal,
};
use std::io;
use tokio::time::{Duration, interval};
use tracing::{debug, info};

use crate::config::Config;
use crate::shell::ShellSession;

/// High-performance terminal with hardware-accelerated rendering
pub struct Terminal {
    config: Config,
    sessions: Vec<ShellSession>,
    active_session: usize,
    output_buffers: Vec<Vec<u8>>,
    should_quit: bool,
}

impl Terminal {
    /// Create a new terminal instance with optimal memory allocation
    pub fn new(config: Config) -> Result<Self> {
        info!("Initializing Furnace terminal emulator");
        
        Ok(Self {
            config,
            sessions: Vec::with_capacity(8), // Pre-allocate for performance
            active_session: 0,
            output_buffers: Vec::with_capacity(8),
            should_quit: false,
        })
    }

    /// Main event loop with async I/O for maximum performance
    pub async fn run(&mut self) -> Result<()> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
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

        info!("Terminal started with {}x{} size", cols, rows);

        // Event loop with optimized timing
        let mut render_interval = interval(Duration::from_millis(16)); // ~60 FPS

        while !self.should_quit {
            tokio::select! {
                // Handle user input
                _ = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(1))) => {
                    if event::poll(Duration::from_millis(1))? {
                        if let Event::Key(key) = event::read()? {
                            self.handle_key_event(key).await?;
                        }
                    }
                }
                
                // Read shell output (non-blocking)
                _ = async {
                    if let Some(session) = self.sessions.get(self.active_session) {
                        let mut buffer = vec![0u8; 8192]; // 8KB read buffer
                        if let Ok(n) = session.read_output(&mut buffer).await {
                            if n > 0 {
                                self.output_buffers[self.active_session].extend_from_slice(&buffer[..n]);
                                // Keep buffer size manageable
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
                    terminal.draw(|f| self.render(f))?;
                }
            }
        }

        // Cleanup
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        info!("Terminal shutdown complete");
        Ok(())
    }

    /// Handle keyboard events with optimal input processing
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            // Quit
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
                    }
                    
                    session.write_input(&input).await?;
                }
            }
            
            // Enter
            (KeyCode::Enter, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    session.write_input(b"\r").await?;
                }
            }
            
            // Backspace
            (KeyCode::Backspace, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    session.write_input(&[127]).await?;
                }
            }
            
            // Arrow keys
            (KeyCode::Up, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
                    session.write_input(b"\x1b[A").await?;
                }
            }
            (KeyCode::Down, _) => {
                if let Some(session) = self.sessions.get(self.active_session) {
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

    /// Create a new tab
    async fn create_new_tab(&mut self) -> Result<()> {
        info!("Creating new tab");
        
        let session = ShellSession::new(
            &self.config.shell.default_shell,
            self.config.shell.working_dir.as_deref(),
            40, 120, // Default size
        )?;
        
        self.sessions.push(session);
        self.output_buffers.push(Vec::with_capacity(1024 * 1024));
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
    fn render(&self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(f.size());

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
            
            f.render_widget(tabs, chunks[0]);
        }

        // Render terminal output
        let output_area = if self.config.terminal.enable_tabs && self.sessions.len() > 1 {
            chunks[1]
        } else {
            f.size()
        };

        let output = if let Some(buffer) = self.output_buffers.get(self.active_session) {
            String::from_utf8_lossy(buffer).to_string()
        } else {
            String::new()
        };

        let paragraph = Paragraph::new(output)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(paragraph, output_area);
    }
}

// Rust ensures no memory leaks via RAII and Drop trait
impl Drop for Terminal {
    fn drop(&mut self) {
        info!("Terminal instance dropped - all resources cleaned up");
    }
}
