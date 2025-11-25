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
    ///
    /// # Errors
    /// Returns an error if the hex string is not exactly 6 characters or contains invalid hex digits
    #[allow(dead_code)] // Public API for runtime color parsing
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            anyhow::bail!("Invalid hex color: must be 6 characters");
        }

        let r = u8::from_str_radix(&hex[0..2], 16).context("Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16).context("Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16).context("Invalid blue component")?;

        Ok(Self::new(r, g, b))
    }

    /// Convert to hex string
    #[must_use]
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Convert to ANSI escape sequence for foreground
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn to_ansi_fg(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Convert to ANSI escape sequence for background
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn to_ansi_bg(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Blend with another color (uses rounding instead of truncation for accuracy)
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn blend(self, other: Self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            // Use round() instead of truncation for more accurate color blending
            r: ((self.r as f32) * (1.0 - factor) + (other.r as f32) * factor).round() as u8,
            g: ((self.g as f32) * (1.0 - factor) + (other.g as f32) * factor).round() as u8,
            b: ((self.b as f32) * (1.0 - factor) + (other.b as f32) * factor).round() as u8,
        }
    }

    /// Lighten color by factor
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn lighten(self, factor: f32) -> Self {
        let white = Self::new(255, 255, 255);
        self.blend(white, factor)
    }

    /// Darken color by factor
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn darken(self, factor: f32) -> Self {
        let black = Self::new(0, 0, 0);
        self.blend(black, factor)
    }

    /// Get luminance (0.0 - 1.0)
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn luminance(self) -> f32 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }

    /// Check if color is light
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn is_light(self) -> bool {
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
    /// Create default palette using const colors (no runtime unwrap/panic)
    #[must_use]
    pub fn default_dark() -> Self {
        // Use const values - these are compile-time verified, no runtime unwrap needed
        Self {
            black: TrueColor::new(0x00, 0x00, 0x00),       // #000000
            red: TrueColor::new(0xFF, 0x55, 0x55),         // #FF5555
            green: TrueColor::new(0x50, 0xFA, 0x7B),       // #50FA7B
            yellow: TrueColor::new(0xF1, 0xFA, 0x8C),      // #F1FA8C
            blue: TrueColor::new(0xBD, 0x93, 0xF9),        // #BD93F9
            magenta: TrueColor::new(0xFF, 0x79, 0xC6),     // #FF79C6
            cyan: TrueColor::new(0x8B, 0xE9, 0xFD),        // #8BE9FD
            white: TrueColor::new(0xBF, 0xBF, 0xBF),       // #BFBFBF

            bright_black: TrueColor::new(0x4D, 0x4D, 0x4D),     // #4D4D4D
            bright_red: TrueColor::new(0xFF, 0x6E, 0x67),       // #FF6E67
            bright_green: TrueColor::new(0x5A, 0xF7, 0x8E),     // #5AF78E
            bright_yellow: TrueColor::new(0xF4, 0xF9, 0x9D),    // #F4F99D
            bright_blue: TrueColor::new(0xCA, 0xA9, 0xFA),      // #CAA9FA
            bright_magenta: TrueColor::new(0xFF, 0x92, 0xD0),   // #FF92D0
            bright_cyan: TrueColor::new(0x9A, 0xED, 0xFE),      // #9AEDFE
            bright_white: TrueColor::new(0xE6, 0xE6, 0xE6),     // #E6E6E6

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
            i => self
                .extended
                .get(i as usize)
                .copied()
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
        let purple = red.blend(blue, 0.5);

        // With rounding: 255 * 0.5 = 127.5 -> rounds to 128
        assert_eq!(purple.r, 128);
        assert_eq!(purple.b, 128);
    }

    #[test]
    fn test_luminance() {
        let white = TrueColor::new(255, 255, 255);
        let black = TrueColor::new(0, 0, 0);

        assert!(white.is_light());
        assert!(!black.is_light());
    }
}
