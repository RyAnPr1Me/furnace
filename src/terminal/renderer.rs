//! Rendering module for terminal UI
//!
//! Extracted from the main Terminal struct to improve modularity.
//! Handles all UI rendering operations.
//!
//! # Future Use
//! These functions are designed to be integrated with the main Terminal
//! when further refactoring is performed. They provide a cleaner separation
//! of rendering concerns.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, List, ListItem},
    Frame,
};

use crate::ssh_manager::SshManager;
use crate::ui::command_palette::CommandPalette;
use crate::ui::resource_monitor::ResourceMonitor;
use crate::progress_bar::ProgressBar;

/// Render tab bar
#[allow(dead_code)] // Public API for future refactoring
pub fn render_tabs(f: &mut Frame, area: Rect, sessions_count: usize, active_session: usize) {
    let tab_titles: Vec<Line> = (0..sessions_count)
        .map(|i| {
            let style = if i == active_session {
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
        .select(active_session)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    
    f.render_widget(tabs, area);
}

/// Render translation notification
#[allow(dead_code)] // Public API for future refactoring
pub fn render_notification(f: &mut Frame, area: Rect, msg: &str) {
    let notification = Paragraph::new(msg)
        .style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(notification, area);
}

/// Render progress bar
#[allow(dead_code)] // Public API for future refactoring
pub fn render_progress_bar(f: &mut Frame, area: Rect, progress_bar: &ProgressBar) {
    let progress_text = progress_bar.display_text();
    let progress_widget = Paragraph::new(progress_text)
        .style(Style::default().fg(Color::Cyan).bg(Color::Black).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(progress_widget, area);
}

/// Render SSH manager overlay
#[allow(dead_code)] // Public API for future refactoring
pub fn render_ssh_manager(f: &mut Frame, area: Rect, ssh_manager: &SshManager) {
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
    let items: Vec<ListItem> = ssh_manager.filtered_connections
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            // Safely get connection - returns None if not found
            ssh_manager.get_connection(name).map(|conn| {
                let content = format!(
                    "{} ({}@{}:{})",
                    name,
                    conn.username,
                    conn.host,
                    conn.port
                );
                
                let style = if i == ssh_manager.selected_index {
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

    // Create title string - format! only when filter is not empty
    let title = if ssh_manager.filter_input.is_empty() {
        String::from("SSH Connections (Ctrl+Shift+S to close, Enter to connect, Del to remove)")
    } else {
        format!("SSH Connections - Filter: {}", ssh_manager.filter_input)
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
#[allow(dead_code)] // Public API for future refactoring
pub fn render_command_palette(f: &mut Frame, area: Rect, command_palette: &CommandPalette) {
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
    let input = Paragraph::new(format!("> {}", command_palette.input))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Command Palette (Esc to close)")
            .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(input, palette_chunks[0]);

    // Suggestions
    let suggestions: Vec<Line> = command_palette.suggestions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == command_palette.selected_index {
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

/// Render terminal output
#[allow(dead_code)] // Public API for future refactoring
pub fn render_terminal_output(f: &mut Frame, area: Rect, output_buffers: &[Vec<u8>], active_session: usize) {
    let output = if let Some(buffer) = output_buffers.get(active_session) {
        String::from_utf8_lossy(buffer).to_string()
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(output)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(paragraph, area);
}

/// Render resource monitor
#[allow(dead_code)] // Public API for future refactoring
pub fn render_resource_monitor(f: &mut Frame, area: Rect, resource_monitor: &mut ResourceMonitor) {
    let stats = resource_monitor.get_stats();
    
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
