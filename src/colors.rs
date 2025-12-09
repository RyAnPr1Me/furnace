use anyhow::{Context, Result};
use std::fmt;

/// 24-bit true color support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrueColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TrueColor {
    /// Create a new true color from RGB values
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create from hex string (#RRGGBB or RRGGBB)
    ///
    /// # Errors
    /// Returns an error if the hex string is not exactly 6 characters or contains invalid hex digits
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
    #[must_use]
    pub fn to_ansi_fg(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Convert to ANSI escape sequence for background
    #[must_use]
    pub fn to_ansi_bg(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Blend two colors together using linear interpolation
    ///
    /// This function performs color blending with sub-pixel accuracy using
    /// proper rounding. It's optimized to use FMA (Fused Multiply-Add) when
    /// available on the CPU for better performance.
    ///
    /// # Arguments
    /// * `other` - The color to blend with
    /// * `factor` - Blend factor (0.0 = all self, 1.0 = all other)
    ///
    /// # Returns
    /// Blended color
    ///
    /// # Formula
    /// `result = self * (1 - factor) + other * factor`
    ///
    /// # Example
    /// ```ignore
    /// let red = TrueColor::new(255, 0, 0);
    /// let blue = TrueColor::new(0, 0, 255);
    /// let purple = red.blend(blue, 0.5);  // 50% blend = purple
    /// ```
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    #[must_use]
    pub fn blend(self, other: Self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            // Use mul_add for efficient FMA (Fused Multiply-Add) instruction
            // Formula: self * (1 - factor) + other * factor
            // Restructured as: (other - self) * factor + self for proper FMA usage
            // Note: Casts are safe because we clamp factor and round before casting
            r: (f32::from(other.r) - f32::from(self.r))
                .mul_add(factor, f32::from(self.r))
                .round() as u8,
            g: (f32::from(other.g) - f32::from(self.g))
                .mul_add(factor, f32::from(self.g))
                .round() as u8,
            b: (f32::from(other.b) - f32::from(self.b))
                .mul_add(factor, f32::from(self.b))
                .round() as u8,
        }
    }

    /// Lighten color by factor
    #[must_use]
    pub fn lighten(self, factor: f32) -> Self {
        let white = Self::new(255, 255, 255);
        self.blend(white, factor)
    }

    /// Darken color by factor
    #[must_use]
    pub fn darken(self, factor: f32) -> Self {
        let black = Self::new(0, 0, 0);
        self.blend(black, factor)
    }

    /// Get luminance (0.0 - 1.0)
    #[must_use]
    pub fn luminance(self) -> f32 {
        // Use nested mul_add for hardware FMA (Fused Multiply-Add) optimization
        // Original formula: (0.299*r + 0.587*g + 0.114*b) / 255.0
        // Nested as: 0.299*r + (0.587*g + (0.114*b + 0.0))
        // This leverages single-instruction FMA on modern CPUs for better performance and accuracy
        // Trade-off: slightly reduced readability for measurable performance gains in tight loops
        0.299f32.mul_add(
            f32::from(self.r),
            0.587f32.mul_add(f32::from(self.g), 0.114f32.mul_add(f32::from(self.b), 0.0)),
        ) / 255.0
    }

    /// Check if color is light
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
    /// Create default dark palette with cool red/black theme (no runtime unwrap/panic)
    #[must_use]
    pub fn default_dark() -> Self {
        // Use const values - these are compile-time verified, no runtime unwrap needed
        Self {
            black: TrueColor::new(0x00, 0x00, 0x00),   // #000000
            red: TrueColor::new(0xCC, 0x55, 0x55),     // #CC5555 - Darker, cooler red
            green: TrueColor::new(0x5A, 0x8A, 0x6A),   // #5A8A6A - Muted green
            yellow: TrueColor::new(0xB8, 0x98, 0x60),  // #B89860 - Darker yellow
            blue: TrueColor::new(0x6A, 0x7A, 0x9A),    // #6A7A9A - Cool blue-gray
            magenta: TrueColor::new(0xB0, 0x5A, 0x7A), // #B05A7A - Dark magenta-red
            cyan: TrueColor::new(0x5A, 0x8A, 0x8A),    // #5A8A8A - Dark teal
            white: TrueColor::new(0xC0, 0xB0, 0xB0),   // #C0B0B0 - Slightly reddish gray

            bright_black: TrueColor::new(0x3A, 0x2A, 0x2A), // #3A2A2A - Dark reddish-gray
            bright_red: TrueColor::new(0xDD, 0x66, 0x66),   // #DD6666 - Medium cool red
            bright_green: TrueColor::new(0x6A, 0x9A, 0x7A), // #6A9A7A - Muted bright green
            bright_yellow: TrueColor::new(0xC8, 0xA8, 0x70), // #C8A870 - Muted gold
            bright_blue: TrueColor::new(0x7A, 0x8A, 0xAA),  // #7A8AAA - Cool light blue
            bright_magenta: TrueColor::new(0xC0, 0x6A, 0x8A), // #C06A8A - Bright magenta-red
            bright_cyan: TrueColor::new(0x6A, 0x9A, 0x9A),  // #6A9A9A - Muted cyan
            bright_white: TrueColor::new(0xD0, 0xC0, 0xC0), // #D0C0C0 - Light reddish-gray

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

    /// Get color by 256-color index (optimized with inline and match)
    #[must_use]
    #[inline]
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
                .get(usize::from(i))
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
