//! GPU text rendering utilities

use super::{CellStyle, GpuCell};

/// Convert ANSI color code to RGBA
pub fn ansi_to_rgba(code: u8) -> [f32; 4] {
    // Standard 16 ANSI colors
    match code {
        0 => [0.0, 0.0, 0.0, 1.0],       // Black
        1 => [0.8, 0.0, 0.0, 1.0],       // Red
        2 => [0.0, 0.8, 0.0, 1.0],       // Green
        3 => [0.8, 0.8, 0.0, 1.0],       // Yellow
        4 => [0.0, 0.0, 0.8, 1.0],       // Blue
        5 => [0.8, 0.0, 0.8, 1.0],       // Magenta
        6 => [0.0, 0.8, 0.8, 1.0],       // Cyan
        7 => [0.75, 0.75, 0.75, 1.0],    // White
        8 => [0.5, 0.5, 0.5, 1.0],       // Bright Black
        9 => [1.0, 0.0, 0.0, 1.0],       // Bright Red
        10 => [0.0, 1.0, 0.0, 1.0],      // Bright Green
        11 => [1.0, 1.0, 0.0, 1.0],      // Bright Yellow
        12 => [0.0, 0.0, 1.0, 1.0],      // Bright Blue
        13 => [1.0, 0.0, 1.0, 1.0],      // Bright Magenta
        14 => [0.0, 1.0, 1.0, 1.0],      // Bright Cyan
        15 => [1.0, 1.0, 1.0, 1.0],      // Bright White
        // 256-color palette (16-231: 6x6x6 color cube)
        16..=231 => {
            let idx = code - 16;
            let r = ((idx / 36) % 6) as f32 / 5.0;
            let g = ((idx / 6) % 6) as f32 / 5.0;
            let b = (idx % 6) as f32 / 5.0;
            [r, g, b, 1.0]
        }
        // 232-255: grayscale
        232..=255 => {
            let gray = ((code - 232) as f32 * 10.0 + 8.0) / 255.0;
            [gray, gray, gray, 1.0]
        }
    }
}

/// Convert RGB to RGBA
pub fn rgb_to_rgba(r: u8, g: u8, b: u8) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
}

/// Convert hex color string to RGBA
pub fn hex_to_rgba(hex: &str) -> Option<[f32; 4]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(rgb_to_rgba(r, g, b))
}

/// Parse terminal output into GPU cells
pub fn parse_terminal_output(output: &str, cols: usize) -> Vec<GpuCell> {
    let mut cells = Vec::with_capacity(cols * 50);
    let mut current_fg = [1.0f32, 1.0, 1.0, 1.0]; // White
    let mut current_bg = [0.0f32, 0.0, 0.0, 1.0]; // Black
    let mut current_style = CellStyle::empty();
    
    let mut col = 0;
    let mut chars = output.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => {
                // Parse ANSI escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    let mut params = Vec::new();
                    let mut current_param = String::new();
                    
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() || ch == ';' {
                            chars.next();
                            if ch == ';' {
                                if !current_param.is_empty() {
                                    params.push(current_param.parse::<u8>().unwrap_or(0));
                                    current_param.clear();
                                }
                            } else {
                                current_param.push(ch);
                            }
                        } else {
                            break;
                        }
                    }
                    
                    if !current_param.is_empty() {
                        params.push(current_param.parse::<u8>().unwrap_or(0));
                    }
                    
                    // Get final character
                    if let Some(cmd) = chars.next() {
                        match cmd {
                            'm' => {
                                // SGR (Select Graphic Rendition)
                                for param in params {
                                    match param {
                                        0 => {
                                            current_fg = [1.0, 1.0, 1.0, 1.0];
                                            current_bg = [0.0, 0.0, 0.0, 1.0];
                                            current_style = CellStyle::empty();
                                        }
                                        1 => current_style.insert(CellStyle::BOLD),
                                        2 => current_style.insert(CellStyle::DIM),
                                        3 => current_style.insert(CellStyle::ITALIC),
                                        4 => current_style.insert(CellStyle::UNDERLINE),
                                        5 | 6 => current_style.insert(CellStyle::BLINK),
                                        7 => current_style.insert(CellStyle::REVERSE),
                                        8 => current_style.insert(CellStyle::HIDDEN),
                                        9 => current_style.insert(CellStyle::STRIKETHROUGH),
                                        30..=37 => current_fg = ansi_to_rgba(param - 30),
                                        39 => current_fg = [1.0, 1.0, 1.0, 1.0],
                                        40..=47 => current_bg = ansi_to_rgba(param - 40),
                                        49 => current_bg = [0.0, 0.0, 0.0, 1.0],
                                        90..=97 => current_fg = ansi_to_rgba(param - 90 + 8),
                                        100..=107 => current_bg = ansi_to_rgba(param - 100 + 8),
                                        _ => {}
                                    }
                                }
                            }
                            _ => {} // Ignore other sequences
                        }
                    }
                }
            }
            '\n' => {
                // Fill rest of line with spaces
                while col < cols {
                    cells.push(GpuCell {
                        char_code: ' ' as u32,
                        fg_color: current_fg,
                        bg_color: current_bg,
                        style: current_style,
                    });
                    col += 1;
                }
                col = 0;
            }
            '\r' => {
                col = 0;
            }
            '\t' => {
                // Tab to next 8-column boundary
                let spaces = 8 - (col % 8);
                for _ in 0..spaces {
                    if col < cols {
                        cells.push(GpuCell {
                            char_code: ' ' as u32,
                            fg_color: current_fg,
                            bg_color: current_bg,
                            style: current_style,
                        });
                        col += 1;
                    }
                }
            }
            _ if c.is_control() => {
                // Skip control characters
            }
            _ => {
                if col < cols {
                    cells.push(GpuCell {
                        char_code: c as u32,
                        fg_color: current_fg,
                        bg_color: current_bg,
                        style: current_style,
                    });
                    col += 1;
                }
            }
        }
    }
    
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_rgba() {
        let black = ansi_to_rgba(0);
        assert_eq!(black[0], 0.0);
        assert_eq!(black[3], 1.0);

        let white = ansi_to_rgba(15);
        assert_eq!(white[0], 1.0);
        assert_eq!(white[1], 1.0);
        assert_eq!(white[2], 1.0);
    }

    #[test]
    fn test_hex_to_rgba() {
        let color = hex_to_rgba("#FF0000").unwrap();
        assert!((color[0] - 1.0).abs() < 0.01);
        assert!((color[1] - 0.0).abs() < 0.01);
        assert!((color[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_simple_text() {
        let cells = parse_terminal_output("Hello", 80);
        assert_eq!(cells.len(), 5);
        assert_eq!(cells[0].char_code, 'H' as u32);
    }
}
