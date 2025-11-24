use anyhow::{Context, Result};
use std::fmt;

/// 24-bit true color support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Public API for color system
pub struct TrueColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TrueColor {
    /// Create a new true color from RGB values
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create from hex string (#RRGGBB or RRGGBB)
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');
        
        if hex.len() != 6 {
            anyhow::bail!("Invalid hex color: must be 6 characters");
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16)
            .context("Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .context("Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .context("Invalid blue component")?;
        
        Ok(Self::new(r, g, b))
    }

    /// Convert to hex string
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Convert to ANSI escape sequence for foreground
    #[allow(dead_code)] // Public API
    pub fn to_ansi_fg(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Convert to ANSI escape sequence for background
    #[allow(dead_code)] // Public API
    pub fn to_ansi_bg(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Blend with another color
    #[allow(dead_code)] // Public API
    pub fn blend(&self, other: &TrueColor, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: ((self.r as f32) * (1.0 - factor) + (other.r as f32) * factor) as u8,
            g: ((self.g as f32) * (1.0 - factor) + (other.g as f32) * factor) as u8,
            b: ((self.b as f32) * (1.0 - factor) + (other.b as f32) * factor) as u8,
        }
    }

    /// Lighten color by factor
    #[allow(dead_code)] // Public API
    pub fn lighten(&self, factor: f32) -> Self {
        let white = TrueColor::new(255, 255, 255);
        self.blend(&white, factor)
    }

    /// Darken color by factor
    #[allow(dead_code)] // Public API
    pub fn darken(&self, factor: f32) -> Self {
        let black = TrueColor::new(0, 0, 0);
        self.blend(&black, factor)
    }

    /// Get luminance (0.0 - 1.0)
    #[allow(dead_code)] // Public API
    pub fn luminance(&self) -> f32 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }

    /// Check if color is light
    #[allow(dead_code)] // Public API
    pub fn is_light(&self) -> bool {
        self.luminance() > 0.5
    }
}

impl fmt::Display for TrueColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Color palette with 24-bit color support
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API for color palette
pub struct TrueColorPalette {
    // ANSI colors (16 colors)
    pub black: TrueColor,
    pub red: TrueColor,
    pub green: TrueColor,
    pub yellow: TrueColor,
    pub blue: TrueColor,
    pub magenta: TrueColor,
    pub cyan: TrueColor,
    pub white: TrueColor,
    
    pub bright_black: TrueColor,
    pub bright_red: TrueColor,
    pub bright_green: TrueColor,
    pub bright_yellow: TrueColor,
    pub bright_blue: TrueColor,
    pub bright_magenta: TrueColor,
    pub bright_cyan: TrueColor,
    pub bright_white: TrueColor,
    
    // Extended 256 color palette
    pub extended: Vec<TrueColor>,
}

impl TrueColorPalette {
    /// Create default palette
    pub fn default_dark() -> Self {
        Self {
            black: TrueColor::from_hex("#000000").unwrap(),
            red: TrueColor::from_hex("#FF5555").unwrap(),
            green: TrueColor::from_hex("#50FA7B").unwrap(),
            yellow: TrueColor::from_hex("#F1FA8C").unwrap(),
            blue: TrueColor::from_hex("#BD93F9").unwrap(),
            magenta: TrueColor::from_hex("#FF79C6").unwrap(),
            cyan: TrueColor::from_hex("#8BE9FD").unwrap(),
            white: TrueColor::from_hex("#BFBFBF").unwrap(),
            
            bright_black: TrueColor::from_hex("#4D4D4D").unwrap(),
            bright_red: TrueColor::from_hex("#FF6E67").unwrap(),
            bright_green: TrueColor::from_hex("#5AF78E").unwrap(),
            bright_yellow: TrueColor::from_hex("#F4F99D").unwrap(),
            bright_blue: TrueColor::from_hex("#CAA9FA").unwrap(),
            bright_magenta: TrueColor::from_hex("#FF92D0").unwrap(),
            bright_cyan: TrueColor::from_hex("#9AEDFE").unwrap(),
            bright_white: TrueColor::from_hex("#E6E6E6").unwrap(),
            
            extended: Self::generate_256_palette(),
        }
    }

    /// Generate 256 color palette (for xterm compatibility)
    fn generate_256_palette() -> Vec<TrueColor> {
        let mut palette = Vec::with_capacity(256);
        
        // First 16 colors are the standard ANSI colors (handled separately)
        for _ in 0..16 {
            palette.push(TrueColor::new(0, 0, 0));
        }
        
        // 216 color cube (6x6x6)
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let r_val = if r > 0 { r * 40 + 55 } else { 0 };
                    let g_val = if g > 0 { g * 40 + 55 } else { 0 };
                    let b_val = if b > 0 { b * 40 + 55 } else { 0 };
                    palette.push(TrueColor::new(r_val, g_val, b_val));
                }
            }
        }
        
        // 24 grayscale colors
        for i in 0..24 {
            let gray = i * 10 + 8;
            palette.push(TrueColor::new(gray, gray, gray));
        }
        
        palette
    }

    /// Get color by 256-color index
    #[allow(dead_code)] // Public API
    pub fn get_256(&self, index: u8) -> TrueColor {
        match index {
            0 => self.black,
            1 => self.red,
            2 => self.green,
            3 => self.yellow,
            4 => self.blue,
            5 => self.magenta,
            6 => self.cyan,
            7 => self.white,
            8 => self.bright_black,
            9 => self.bright_red,
            10 => self.bright_green,
            11 => self.bright_yellow,
            12 => self.bright_blue,
            13 => self.bright_magenta,
            14 => self.bright_cyan,
            15 => self.bright_white,
            i => self.extended.get(i as usize).copied()
                .unwrap_or(TrueColor::new(0, 0, 0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_color_from_hex() {
        let color = TrueColor::from_hex("#FF8800").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 136);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_true_color_to_hex() {
        let color = TrueColor::new(255, 136, 0);
        assert_eq!(color.to_hex(), "#FF8800");
    }

    #[test]
    fn test_ansi_sequences() {
        let color = TrueColor::new(255, 0, 0);
        assert_eq!(color.to_ansi_fg(), "\x1b[38;2;255;0;0m");
        assert_eq!(color.to_ansi_bg(), "\x1b[48;2;255;0;0m");
    }

    #[test]
    fn test_color_blending() {
        let red = TrueColor::new(255, 0, 0);
        let blue = TrueColor::new(0, 0, 255);
        let purple = red.blend(&blue, 0.5);
        
        assert_eq!(purple.r, 127);
        assert_eq!(purple.b, 127);
    }

    #[test]
    fn test_luminance() {
        let white = TrueColor::new(255, 255, 255);
        let black = TrueColor::new(0, 0, 0);
        
        assert!(white.is_light());
        assert!(!black.is_light());
    }
}
