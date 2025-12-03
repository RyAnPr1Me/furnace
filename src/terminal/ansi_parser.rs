//! ANSI Escape Code Parser for Terminal Output
//!
//! This module parses ANSI escape sequences and converts them to ratatui styled text.
//! It supports:
//! - SGR (Select Graphic Rendition) for colors and text attributes
//! - Standard 16 colors (8 normal + 8 bright)
//! - 256-color palette
//! - 24-bit true color (RGB)
//! - Text attributes (bold, italic, underline, etc.)

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use vte::{Params, Parser, Perform};

/// Convert a u16 color value to u8, clamping to valid range
/// This is marked inline to allow the compiler to optimize it away when possible
#[inline]
#[must_use]
const fn to_color_u8(value: u16) -> u8 {
    if value > 255 {
        255
    } else {
        value as u8
    }
}

/// ANSI parser that converts escape sequences to styled ratatui spans
pub struct AnsiParser {
    /// Current style being applied
    current_style: Style,
    /// Accumulated text with current style
    current_text: String,
    /// Completed spans for the current line
    current_line_spans: Vec<Span<'static>>,
    /// Completed lines
    lines: Vec<Line<'static>>,
}

impl AnsiParser {
    /// Create a new ANSI parser
    pub fn new() -> Self {
        Self {
            current_style: Style::default().fg(Color::White).bg(Color::Black),
            current_text: String::new(),
            current_line_spans: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Parse ANSI-encoded text and return styled lines
    pub fn parse(text: &str) -> Vec<Line<'static>> {
        let mut parser = Parser::new();
        let mut performer = AnsiParser::new();

        // VTE 0.15 expects a slice of bytes
        parser.advance(&mut performer, text.as_bytes());

        // Flush any remaining content
        performer.flush_text();
        performer.flush_line();

        performer.lines
    }

    /// Flush accumulated text to a span
    fn flush_text(&mut self) {
        if !self.current_text.is_empty() {
            let text = std::mem::take(&mut self.current_text);
            self.current_line_spans
                .push(Span::styled(text, self.current_style));
        }
    }

    /// Flush current line spans to a line
    fn flush_line(&mut self) {
        self.flush_text();
        if self.current_line_spans.is_empty() {
            // Empty line
            self.lines.push(Line::from(""));
        } else {
            let spans = std::mem::take(&mut self.current_line_spans);
            self.lines.push(Line::from(spans));
        }
    }

    /// Parse SGR (Select Graphic Rendition) parameters
    fn handle_sgr(&mut self, params: &Params) {
        let mut iter = params.iter();

        while let Some(param) = iter.next() {
            if param.is_empty() {
                continue;
            }

            match param[0] {
                // Reset
                0 => {
                    self.current_style = Style::default().fg(Color::White).bg(Color::Black);
                }
                // Bold
                1 => {
                    self.current_style = self.current_style.add_modifier(Modifier::BOLD);
                }
                // Dim/Faint
                2 => {
                    self.current_style = self.current_style.add_modifier(Modifier::DIM);
                }
                // Italic
                3 => {
                    self.current_style = self.current_style.add_modifier(Modifier::ITALIC);
                }
                // Underline
                4 => {
                    self.current_style = self.current_style.add_modifier(Modifier::UNDERLINED);
                }
                // Blink (slow)
                5 => {
                    self.current_style = self.current_style.add_modifier(Modifier::SLOW_BLINK);
                }
                // Blink (rapid)
                6 => {
                    self.current_style = self.current_style.add_modifier(Modifier::RAPID_BLINK);
                }
                // Reverse video
                7 => {
                    self.current_style = self.current_style.add_modifier(Modifier::REVERSED);
                }
                // Hidden
                8 => {
                    self.current_style = self.current_style.add_modifier(Modifier::HIDDEN);
                }
                // Strikethrough
                9 => {
                    self.current_style = self.current_style.add_modifier(Modifier::CROSSED_OUT);
                }
                // Normal intensity (not bold, not dim)
                22 => {
                    self.current_style = self
                        .current_style
                        .remove_modifier(Modifier::BOLD)
                        .remove_modifier(Modifier::DIM);
                }
                // Not italic
                23 => {
                    self.current_style = self.current_style.remove_modifier(Modifier::ITALIC);
                }
                // Not underlined
                24 => {
                    self.current_style = self.current_style.remove_modifier(Modifier::UNDERLINED);
                }
                // Not blinking
                25 => {
                    self.current_style = self
                        .current_style
                        .remove_modifier(Modifier::SLOW_BLINK)
                        .remove_modifier(Modifier::RAPID_BLINK);
                }
                // Not reversed
                27 => {
                    self.current_style = self.current_style.remove_modifier(Modifier::REVERSED);
                }
                // Not hidden
                28 => {
                    self.current_style = self.current_style.remove_modifier(Modifier::HIDDEN);
                }
                // Not strikethrough
                29 => {
                    self.current_style = self.current_style.remove_modifier(Modifier::CROSSED_OUT);
                }
                // Standard foreground colors (30-37)
                30 => self.current_style = self.current_style.fg(Color::Black),
                31 => self.current_style = self.current_style.fg(Color::Red),
                32 => self.current_style = self.current_style.fg(Color::Green),
                33 => self.current_style = self.current_style.fg(Color::Yellow),
                34 => self.current_style = self.current_style.fg(Color::Blue),
                35 => self.current_style = self.current_style.fg(Color::Magenta),
                36 => self.current_style = self.current_style.fg(Color::Cyan),
                37 => self.current_style = self.current_style.fg(Color::White),
                // Extended foreground color (256-color or RGB)
                38 => {
                    if let Some(next) = iter.next() {
                        if !next.is_empty() {
                            match next[0] {
                                // 256-color palette
                                5 => {
                                    if let Some(color_param) = iter.next() {
                                        if !color_param.is_empty() {
                                            self.current_style = self
                                                .current_style
                                                .fg(Color::Indexed(to_color_u8(color_param[0])));
                                        }
                                    }
                                }
                                // 24-bit RGB
                                2 => {
                                    let r = iter.next().and_then(|p| p.first().copied());
                                    let g = iter.next().and_then(|p| p.first().copied());
                                    let b = iter.next().and_then(|p| p.first().copied());
                                    if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                        self.current_style = self.current_style.fg(Color::Rgb(
                                            to_color_u8(r),
                                            to_color_u8(g),
                                            to_color_u8(b),
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // Default foreground color
                39 => {
                    self.current_style = self.current_style.fg(Color::White);
                }
                // Standard background colors (40-47)
                40 => self.current_style = self.current_style.bg(Color::Black),
                41 => self.current_style = self.current_style.bg(Color::Red),
                42 => self.current_style = self.current_style.bg(Color::Green),
                43 => self.current_style = self.current_style.bg(Color::Yellow),
                44 => self.current_style = self.current_style.bg(Color::Blue),
                45 => self.current_style = self.current_style.bg(Color::Magenta),
                46 => self.current_style = self.current_style.bg(Color::Cyan),
                47 => self.current_style = self.current_style.bg(Color::White),
                // Extended background color (256-color or RGB)
                48 => {
                    if let Some(next) = iter.next() {
                        if !next.is_empty() {
                            match next[0] {
                                // 256-color palette
                                5 => {
                                    if let Some(color_param) = iter.next() {
                                        if !color_param.is_empty() {
                                            self.current_style = self
                                                .current_style
                                                .bg(Color::Indexed(to_color_u8(color_param[0])));
                                        }
                                    }
                                }
                                // 24-bit RGB
                                2 => {
                                    let r = iter.next().and_then(|p| p.first().copied());
                                    let g = iter.next().and_then(|p| p.first().copied());
                                    let b = iter.next().and_then(|p| p.first().copied());
                                    if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                        self.current_style = self.current_style.bg(Color::Rgb(
                                            to_color_u8(r),
                                            to_color_u8(g),
                                            to_color_u8(b),
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // Default background color
                49 => {
                    self.current_style = self.current_style.bg(Color::Black);
                }
                // Bright foreground colors (90-97)
                90 => self.current_style = self.current_style.fg(Color::DarkGray),
                91 => self.current_style = self.current_style.fg(Color::LightRed),
                92 => self.current_style = self.current_style.fg(Color::LightGreen),
                93 => self.current_style = self.current_style.fg(Color::LightYellow),
                94 => self.current_style = self.current_style.fg(Color::LightBlue),
                95 => self.current_style = self.current_style.fg(Color::LightMagenta),
                96 => self.current_style = self.current_style.fg(Color::LightCyan),
                97 => self.current_style = self.current_style.fg(Color::White),
                // Bright background colors (100-107)
                100 => self.current_style = self.current_style.bg(Color::DarkGray),
                101 => self.current_style = self.current_style.bg(Color::LightRed),
                102 => self.current_style = self.current_style.bg(Color::LightGreen),
                103 => self.current_style = self.current_style.bg(Color::LightYellow),
                104 => self.current_style = self.current_style.bg(Color::LightBlue),
                105 => self.current_style = self.current_style.bg(Color::LightMagenta),
                106 => self.current_style = self.current_style.bg(Color::LightCyan),
                107 => self.current_style = self.current_style.bg(Color::White),
                _ => {}
            }
        }
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Perform for AnsiParser {
    fn print(&mut self, c: char) {
        self.current_text.push(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // Newline
            b'\n' => {
                self.flush_line();
            }
            // Carriage return - typically ignore, handled with newline
            b'\r' => {}
            // Tab
            b'\t' => {
                self.current_text.push_str("    "); // 4-space tab
            }
            // Backspace
            0x08 => {
                self.current_text.pop();
            }
            // Bell - ignore
            0x07 => {}
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences - not commonly needed for basic terminal display
    }

    fn put(&mut self, _byte: u8) {
        // DCS data - not commonly needed
    }

    fn unhook(&mut self) {
        // End of DCS sequence
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences (operating system commands) - often used for window titles
        // We ignore these for now
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            // SGR - Select Graphic Rendition (colors and attributes)
            'm' => {
                self.flush_text(); // Style change, flush current text
                self.handle_sgr(params);
            }
            // Erase in Line (K) - clear current line content
            'K' => {
                self.flush_text();
                // Get the parameter (default is 0)
                let param = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(0);
                match param {
                    // 0: Clear from cursor to end of line (default)
                    0 => {
                        // Clear remaining text on current line
                        self.current_text.clear();
                    }
                    // 1: Clear from start of line to cursor
                    1 => {
                        // Clear all spans on current line
                        self.current_line_spans.clear();
                        self.current_text.clear();
                    }
                    // 2: Clear entire line
                    2 => {
                        self.current_line_spans.clear();
                        self.current_text.clear();
                    }
                    _ => {}
                }
            }
            // Erase in Display (J) - clear screen
            'J' => {
                self.flush_text();
                self.flush_line();
                let param = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(0);
                match param {
                    // 0: Clear from cursor to end of display
                    0 => {
                        // Just clear current line for simplicity
                        self.current_line_spans.clear();
                        self.current_text.clear();
                    }
                    // 1: Clear from start of display to cursor
                    1 => {
                        // Clear all previous lines
                        self.lines.clear();
                        self.current_line_spans.clear();
                        self.current_text.clear();
                    }
                    // 2: Clear entire display
                    2 | 3 => {
                        // Clear everything
                        self.lines.clear();
                        self.current_line_spans.clear();
                        self.current_text.clear();
                    }
                    _ => {}
                }
            }
            // Cursor movement and other CSI sequences - ignore for display
            'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'L' | 'M' | 'P' | 'S'
            | 'T' | 'X' | 'd' | 'f' | 'g' | 'h' | 'l' | 'n' | 'r' | 's' | 'u' => {
                // These are cursor/screen control - ignore for basic display
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // Simple escape sequences - mostly cursor control, ignore for display
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_color_u8() {
        // Normal values should pass through
        assert_eq!(to_color_u8(0), 0);
        assert_eq!(to_color_u8(128), 128);
        assert_eq!(to_color_u8(255), 255);

        // Values > 255 should be clamped
        assert_eq!(to_color_u8(256), 255);
        assert_eq!(to_color_u8(500), 255);
        assert_eq!(to_color_u8(u16::MAX), 255);
    }

    #[test]
    fn test_plain_text() {
        let lines = AnsiParser::parse("Hello, World!");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_newlines() {
        let lines = AnsiParser::parse("Line 1\nLine 2\nLine 3");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_basic_color() {
        let lines = AnsiParser::parse("\x1b[31mRed Text\x1b[0m");
        assert_eq!(lines.len(), 1);
        // Verify the span has red color
        if let Some(span) = lines[0].spans.first() {
            assert_eq!(span.style.fg, Some(Color::Red));
        }
    }

    #[test]
    fn test_bold() {
        let lines = AnsiParser::parse("\x1b[1mBold\x1b[0m");
        assert_eq!(lines.len(), 1);
        if let Some(span) = lines[0].spans.first() {
            assert!(span.style.add_modifier.contains(Modifier::BOLD));
        }
    }

    #[test]
    fn test_256_color() {
        let lines = AnsiParser::parse("\x1b[38;5;196mBright Red\x1b[0m");
        assert_eq!(lines.len(), 1);
        if let Some(span) = lines[0].spans.first() {
            assert_eq!(span.style.fg, Some(Color::Indexed(196)));
        }
    }

    #[test]
    fn test_rgb_color() {
        let lines = AnsiParser::parse("\x1b[38;2;255;128;64mOrange\x1b[0m");
        assert_eq!(lines.len(), 1);
        if let Some(span) = lines[0].spans.first() {
            assert_eq!(span.style.fg, Some(Color::Rgb(255, 128, 64)));
        }
    }

    #[test]
    fn test_multiple_attributes() {
        let lines = AnsiParser::parse("\x1b[1;31;4mBold Red Underline\x1b[0m");
        assert_eq!(lines.len(), 1);
        if let Some(span) = lines[0].spans.first() {
            assert_eq!(span.style.fg, Some(Color::Red));
            assert!(span.style.add_modifier.contains(Modifier::BOLD));
            assert!(span.style.add_modifier.contains(Modifier::UNDERLINED));
        }
    }

    #[test]
    fn test_erase_in_line() {
        // Test ESC[K (clear to end of line)
        let lines = AnsiParser::parse("Hello\x1b[KWorld");
        assert_eq!(lines.len(), 1);
        // Should show "HelloWorld" since we can't accurately track cursor position
        // but the K sequence shouldn't cause a crash
    }

    #[test]
    fn test_erase_in_display() {
        // Test ESC[2J (clear screen)
        let lines = AnsiParser::parse("Line1\nLine2\x1b[2JLine3");
        // After clear screen, only content after clear should remain
        assert!(!lines.is_empty());
    }
}
