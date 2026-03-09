//! Glyph cache for GPU text rendering
//!
//! Caches rasterized glyphs in a texture atlas for efficient GPU rendering.
//! Uses fontdue for font rasterization to provide actual glyph bitmaps.

use std::collections::HashMap;

/// Glyph cache for efficient text rendering with fontdue font rasterization
///
/// Manages a texture atlas of pre-rendered glyphs for efficient GPU rendering.
#[allow(dead_code)] // Public API - used by GPU renderer consumers
pub struct GlyphCache {
    /// Map from character code to UV coordinates in atlas
    glyph_map: HashMap<u32, GlyphInfo>,
    /// Font for rasterization
    font: Option<fontdue::Font>,
    /// Font size
    font_size: f32,
    /// Font family name (used for font loading)
    font_family: String,
    /// Atlas dimensions
    atlas_size: u32,
    /// Current x position in atlas
    cursor_x: u32,
    /// Current y position in atlas  
    cursor_y: u32,
    /// Maximum height in current row
    row_height: u32,
    /// Atlas bitmap data (R8 format)
    atlas_data: Vec<u8>,
}

/// Information about a cached glyph
///
/// Contains UV coordinates and metrics for rendering a single glyph.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Public API - used by GPU renderer consumers
pub struct GlyphInfo {
    /// UV coordinates (x, y, width, height) normalized to 0-1
    pub uv: [f32; 4],
    /// Advance width in pixels
    pub advance: f32,
    /// Bearing (offset from baseline)
    pub bearing: [f32; 2],
    /// Glyph dimensions in pixels
    pub size: [f32; 2],
}

#[allow(dead_code)] // Public API - methods used by GPU text rendering consumers
impl GlyphCache {
    /// Create a new glyph cache with font loading
    ///
    /// BUG FIX #4: Implement actual font loading and rasterization
    pub fn new(font_size: f32, font_family: &str) -> Self {
        let font = Self::load_font(font_family);

        let atlas_size = 2048;
        let mut cache = Self {
            glyph_map: HashMap::with_capacity(256),
            font,
            font_size,
            font_family: font_family.to_string(),
            atlas_size,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            atlas_data: vec![0u8; (atlas_size * atlas_size) as usize],
        };

        // Pre-cache ASCII characters
        cache.precache_ascii();

        cache
    }

    /// Load font from system or embedded
    ///
    /// Tries the requested font first, then falls back to common monospace fonts
    /// available on various operating systems to ensure text is always rendered.
    fn load_font(font_family: &str) -> Option<fontdue::Font> {
        // Try the requested font family first
        let font_paths = Self::get_font_paths(font_family);

        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(font) = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
                {
                    tracing::info!("Loaded font from: {}", path);
                    return Some(font);
                }
            }
        }

        // Try common fallback monospace fonts on all platforms
        let fallback_families = [
            "DejaVuSansMono",
            "DejaVu Sans Mono",
            "LiberationMono",
            "Liberation Mono",
            "NotoSansMono",
            "Noto Sans Mono",
            "UbuntuMono",
            "Ubuntu Mono",
            "DroidSansMono",
            "Consolas",
            "Courier New",
            "FreeMono",
            "Menlo",
            "Monaco",
        ];

        for fallback in &fallback_families {
            if *fallback == font_family {
                continue; // Already tried this one
            }
            for path in &Self::get_font_paths(fallback) {
                if let Ok(data) = std::fs::read(path) {
                    if let Ok(font) =
                        fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
                    {
                        tracing::info!(
                            "Loaded fallback font '{}' from: {}",
                            fallback,
                            path
                        );
                        return Some(font);
                    }
                }
            }
        }

        tracing::warn!(
            "Could not load font '{}' or any fallback, using placeholder glyphs",
            font_family
        );
        None
    }

    /// Get common font file paths based on font name (platform-specific)
    ///
    /// Generates paths using multiple naming conventions (with spaces, without,
    /// with -Regular suffix, lowercase) to maximize the chance of finding the font.
    fn get_font_paths(font_family: &str) -> Vec<String> {
        let mut paths = Vec::new();

        // Generate name variants: "JetBrains Mono" -> "JetBrainsMono", "jetbrainsmono"
        let no_spaces: String = font_family.split_whitespace().collect();
        let lower = font_family.to_lowercase();
        let lower_no_spaces: String = lower.split_whitespace().collect();
        let hyphenated = font_family.replace(' ', "-");

        // Collect all name variants (deduplicated via order)
        let variants = [
            font_family.to_string(),
            no_spaces.clone(),
            lower.clone(),
            lower_no_spaces.clone(),
            hyphenated.clone(),
        ];

        // Suffixes to try for each variant
        let suffixes = ["", "-Regular", "-regular"];

        #[cfg(windows)]
        {
            for variant in &variants {
                for suffix in &suffixes {
                    paths.push(format!("C:\\Windows\\Fonts\\{}{}.ttf", variant, suffix));
                    paths.push(format!("C:\\Windows\\Fonts\\{}{}.otf", variant, suffix));
                }
            }

            // Common monospace fonts on Windows
            paths.push("C:\\Windows\\Fonts\\consola.ttf".to_string());
            paths.push("C:\\Windows\\Fonts\\cour.ttf".to_string());
            paths.push("C:\\Windows\\Fonts\\lucon.ttf".to_string());

            // User fonts directory on Windows
            if let Some(home) = dirs::home_dir() {
                let local_fonts = home
                    .join("AppData")
                    .join("Local")
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts");
                for variant in &variants {
                    for suffix in &suffixes {
                        paths.push(format!(
                            "{}\\{}{}.ttf",
                            local_fonts.display(),
                            variant,
                            suffix
                        ));
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Common Linux font directories
            let font_dirs = [
                "/usr/share/fonts/truetype",
                "/usr/share/fonts/TTF",
                "/usr/share/fonts/opentype",
                "/usr/share/fonts/OTF",
                "/usr/share/fonts",
                "/usr/local/share/fonts",
            ];

            for dir in &font_dirs {
                for variant in &variants {
                    for suffix in &suffixes {
                        // Direct in directory
                        paths.push(format!("{}/{}{}.ttf", dir, variant, suffix));
                        paths.push(format!("{}/{}{}.otf", dir, variant, suffix));
                        // In subdirectory named after the font
                        paths.push(format!("{}/{}/{}{}.ttf", dir, lower_no_spaces, variant, suffix));
                        paths.push(format!("{}/{}/{}{}.ttf", dir, lower, variant, suffix));
                        paths.push(format!("{}/{}/{}{}.ttf", dir, hyphenated.to_lowercase(), variant, suffix));
                    }
                }
            }

            // Debian/Ubuntu specific paths for common fonts
            paths.push("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf".to_string());
            paths.push("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf".to_string());
            paths.push("/usr/share/fonts/truetype/noto/NotoSansMono-Regular.ttf".to_string());
            paths.push("/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf".to_string());
            paths.push("/usr/share/fonts/truetype/freefont/FreeMono.ttf".to_string());
            paths.push("/usr/share/fonts/truetype/droid/DroidSansMono.ttf".to_string());

            // Arch/Fedora/openSUSE paths
            paths.push("/usr/share/fonts/TTF/DejaVuSansMono.ttf".to_string());
            paths.push("/usr/share/fonts/liberation-mono/LiberationMono-Regular.ttf".to_string());
            paths.push("/usr/share/fonts/google-noto/NotoSansMono-Regular.ttf".to_string());
            paths.push("/usr/share/fonts/noto/NotoSansMono-Regular.ttf".to_string());

            // User fonts directory on Linux
            if let Some(home) = dirs::home_dir() {
                for variant in &variants {
                    for suffix in &suffixes {
                        paths.push(format!(
                            "{}/.local/share/fonts/{}{}.ttf",
                            home.display(),
                            variant,
                            suffix
                        ));
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let mac_dirs = [
                "/System/Library/Fonts",
                "/Library/Fonts",
                "/System/Library/Fonts/Supplemental",
            ];

            for dir in &mac_dirs {
                for variant in &variants {
                    for suffix in &suffixes {
                        paths.push(format!("{}/{}{}.ttf", dir, variant, suffix));
                        paths.push(format!("{}/{}{}.otf", dir, variant, suffix));
                        paths.push(format!("{}/{}{}.ttc", dir, variant, suffix));
                    }
                }
            }

            // macOS built-in monospace fonts
            paths.push("/System/Library/Fonts/Menlo.ttc".to_string());
            paths.push("/System/Library/Fonts/Monaco.ttf".to_string());
            paths.push("/System/Library/Fonts/Courier.ttc".to_string());
            paths.push("/System/Library/Fonts/SFMono-Regular.otf".to_string());
            paths.push("/Library/Fonts/Courier New.ttf".to_string());

            // User fonts directory on macOS
            if let Some(home) = dirs::home_dir() {
                for variant in &variants {
                    for suffix in &suffixes {
                        paths.push(format!(
                            "{}/Library/Fonts/{}{}.ttf",
                            home.display(),
                            variant,
                            suffix
                        ));
                        paths.push(format!(
                            "{}/Library/Fonts/{}{}.otf",
                            home.display(),
                            variant,
                            suffix
                        ));
                    }
                }
            }
        }

        paths
    }

    /// Pre-cache ASCII characters for faster rendering
    ///
    /// BUG FIX #4: Actually rasterize glyphs and upload to atlas
    fn precache_ascii(&mut self) {
        {
            if self.font.is_some() {
                // Cache printable ASCII (32-126)
                for code in 32u32..=126 {
                    if let Some(c) = char::from_u32(code) {
                        // Borrow font separately to avoid borrow conflict
                        self.cache_glyph_rendered(c, code);
                    }
                }
                return;
            }
        }

        // Fallback: use placeholder rects if no font available
        self.precache_ascii_placeholders();
    }

    /// Cache a single glyph by rendering it
    ///
    /// BUG FIX #4: Helper method to avoid borrow conflicts
    fn cache_glyph_rendered(&mut self, c: char, code: u32) {
        // Get font reference
        let Some(ref font) = self.font else {
            return;
        };

        // Rasterize the glyph
        let (metrics, bitmap) = font.rasterize(c, self.font_size);

        let width = metrics.width as u32;
        let height = metrics.height as u32;

        // For zero-size glyphs (e.g., space), register them with an empty UV region
        // so they are still recognized as cached and render as blank.
        if width == 0 || height == 0 {
            self.glyph_map.insert(
                code,
                GlyphInfo {
                    uv: [0.0, 0.0, 0.0, 0.0],
                    advance: metrics.advance_width,
                    bearing: [metrics.xmin as f32, metrics.ymin as f32],
                    size: [0.0, 0.0],
                },
            );
            return;
        }

        // Skip if glyph is too large
        if width > 256 || height > 256 {
            return;
        }

        // Find space in atlas
        if self.cursor_x + width > self.atlas_size {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 2;
            self.row_height = 0;
        }

        // Check if we have vertical space
        if self.cursor_y + height > self.atlas_size {
            tracing::warn!("Atlas full, cannot cache more glyphs");
            return;
        }

        // Copy bitmap data to atlas
        for y in 0..height {
            for x in 0..width {
                let src_idx = (y * width + x) as usize;
                let dst_x = self.cursor_x + x;
                let dst_y = self.cursor_y + y;
                let dst_idx = (dst_y * self.atlas_size + dst_x) as usize;

                if src_idx < bitmap.len() && dst_idx < self.atlas_data.len() {
                    self.atlas_data[dst_idx] = bitmap[src_idx];
                }
            }
        }

        // Calculate UV coordinates (normalized 0-1)
        let atlas_size = self.atlas_size as f32;
        let uv = [
            self.cursor_x as f32 / atlas_size,
            self.cursor_y as f32 / atlas_size,
            width as f32 / atlas_size,
            height as f32 / atlas_size,
        ];

        // Store glyph info
        self.glyph_map.insert(
            code,
            GlyphInfo {
                uv,
                advance: metrics.advance_width,
                bearing: [metrics.xmin as f32, metrics.ymin as f32],
                size: [width as f32, height as f32],
            },
        );

        self.cursor_x += width + 2; // 2-pixel padding
        self.row_height = self.row_height.max(height);
    }

    /// Fallback: pre-cache ASCII with solid placeholder rectangles
    ///
    /// When no font can be loaded, fill atlas regions with solid white pixels
    /// so that text is visible as filled blocks rather than invisible.
    fn precache_ascii_placeholders(&mut self) {
        // BUG FIX #8: Use consistent font metric ratios
        const CELL_WIDTH_RATIO: f32 = 0.6;
        const CELL_HEIGHT_RATIO: f32 = 1.2;
        let glyph_width = (self.font_size * CELL_WIDTH_RATIO) as u32;
        let glyph_height = (self.font_size * CELL_HEIGHT_RATIO) as u32;
        let atlas_size_f = self.atlas_size as f32;

        // Cache printable ASCII (32-126)
        for code in 32u32..=126 {
            // Calculate position in atlas
            if self.cursor_x + glyph_width > self.atlas_size {
                self.cursor_x = 0;
                self.cursor_y += self.row_height + 2;
                self.row_height = 0;
            }

            // Check if we have vertical space
            if self.cursor_y + glyph_height > self.atlas_size {
                break;
            }

            // Write solid white pixels into the atlas region so the glyph is visible.
            // Space (code 32) is left empty so backgrounds render correctly.
            if code != 32 {
                for y in 0..glyph_height {
                    for x in 0..glyph_width {
                        let dst_x = self.cursor_x + x;
                        let dst_y = self.cursor_y + y;
                        let dst_idx = (dst_y * self.atlas_size + dst_x) as usize;
                        if dst_idx < self.atlas_data.len() {
                            self.atlas_data[dst_idx] = 255;
                        }
                    }
                }
            }

            let uv = [
                self.cursor_x as f32 / atlas_size_f,
                self.cursor_y as f32 / atlas_size_f,
                glyph_width as f32 / atlas_size_f,
                glyph_height as f32 / atlas_size_f,
            ];

            self.glyph_map.insert(
                code,
                GlyphInfo {
                    uv,
                    advance: glyph_width as f32,
                    bearing: [0.0, glyph_height as f32 * 0.8],
                    size: [glyph_width as f32, glyph_height as f32],
                },
            );

            self.cursor_x += glyph_width + 2;
            self.row_height = self.row_height.max(glyph_height);
        }
    }

    /// Get UV coordinates for a glyph
    pub fn get_glyph_uv(&self, char_code: u32) -> Option<[f32; 4]> {
        self.glyph_map.get(&char_code).map(|info| info.uv)
    }

    /// Get glyph info
    pub fn get_glyph(&self, char_code: u32) -> Option<&GlyphInfo> {
        self.glyph_map.get(&char_code)
    }

    /// Cache a new glyph (returns UV coordinates)
    ///
    /// BUG FIX #4: Support dynamic glyph caching at runtime
    pub fn cache_glyph(
        &mut self,
        char_code: u32,
        bitmap: &[u8],
        width: u32,
        height: u32,
    ) -> [f32; 4] {
        // Check if already cached
        if let Some(info) = self.glyph_map.get(&char_code) {
            return info.uv;
        }

        let atlas_size = self.atlas_size as f32;

        // Find space in atlas
        if self.cursor_x + width > self.atlas_size {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 2;
            self.row_height = 0;
        }

        // Copy bitmap to atlas if provided
        if !bitmap.is_empty() {
            for y in 0..height {
                for x in 0..width {
                    let src_idx = (y * width + x) as usize;
                    let dst_x = self.cursor_x + x;
                    let dst_y = self.cursor_y + y;
                    let dst_idx = (dst_y * self.atlas_size + dst_x) as usize;

                    if src_idx < bitmap.len() && dst_idx < self.atlas_data.len() {
                        self.atlas_data[dst_idx] = bitmap[src_idx];
                    }
                }
            }
        }

        let uv = [
            self.cursor_x as f32 / atlas_size,
            self.cursor_y as f32 / atlas_size,
            width as f32 / atlas_size,
            height as f32 / atlas_size,
        ];

        self.glyph_map.insert(
            char_code,
            GlyphInfo {
                uv,
                advance: width as f32,
                bearing: [0.0, height as f32 * 0.8],
                size: [width as f32, height as f32],
            },
        );

        self.cursor_x += width + 2;
        self.row_height = self.row_height.max(height);

        uv
    }

    /// Get atlas data for uploading to GPU
    ///
    /// BUG FIX #4: Provide access to atlas bitmap for GPU upload
    pub fn atlas_data(&self) -> &[u8] {
        &self.atlas_data
    }

    /// Get atlas dimensions
    pub fn atlas_size(&self) -> u32 {
        self.atlas_size
    }

    /// Get number of cached glyphs
    pub fn len(&self) -> usize {
        self.glyph_map.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.glyph_map.is_empty()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.glyph_map.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
        self.atlas_data.fill(0);
    }

    /// Get the font family name
    ///
    /// Returns the name of the font family currently in use.
    ///
    /// # Production Use Cases
    /// - Displaying current font in settings UI
    /// - Logging font information for debugging
    /// - Saving font preference to configuration
    /// - Implementing font selection dialog
    pub fn font_family(&self) -> &str {
        &self.font_family
    }

    /// Reload the font with a new family
    ///
    /// Changes the font family and reloads glyphs with the new font.
    /// The atlas is cleared and all glyphs are re-cached with the new font.
    ///
    /// **Note**: This method is only available with the `gpu` feature enabled.
    ///
    /// # Production Use Cases
    /// - Implementing font change in settings
    /// - Font hot-reloading during development
    /// - A/B testing different fonts for readability
    /// - User preference customization
    ///
    /// # Arguments
    /// * `font_family` - Name of the new font family to load
    ///
    /// # Returns
    /// Returns `true` if the font was successfully loaded, `false` if fallback was used
    pub fn reload_font(&mut self, font_family: &str) -> bool {
        self.font_family = font_family.to_string();
        self.font = Self::load_font(font_family);

        // Clear and rebuild cache with new font
        self.clear();

        // Re-cache ASCII characters with new font
        let has_font = self.font.is_some();
        self.precache_ascii();
        has_font
    }

    /// Get font metrics information
    ///
    /// Returns detailed metrics about the current font for layout calculations
    /// and debugging.
    ///
    /// # Production Use Cases
    /// - Calculating precise cell dimensions
    /// - Implementing pixel-perfect layout
    /// - Debugging font rendering issues
    /// - Optimizing atlas size based on font characteristics
    ///
    /// # Returns
    /// A tuple of (font_size, font_family, has_real_font, cached_glyph_count)
    pub fn font_metrics(&self) -> (f32, &str, bool, usize) {
        let has_font = self.font.is_some();

        (
            self.font_size,
            &self.font_family,
            has_font,
            self.glyph_map.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_cache_creation() {
        let cache = GlyphCache::new(14.0, "JetBrains Mono");
        // ASCII characters should be pre-cached
        assert!(cache.len() >= 95); // 126 - 32 + 1
    }

    #[test]
    fn test_get_ascii_glyph() {
        let cache = GlyphCache::new(14.0, "Monospace");

        // Test getting ASCII characters
        assert!(cache.get_glyph('A' as u32).is_some());
        assert!(cache.get_glyph('z' as u32).is_some());
        assert!(cache.get_glyph(' ' as u32).is_some());
    }

    #[test]
    fn test_cache_new_glyph() {
        let mut cache = GlyphCache::new(14.0, "Monospace");

        // Cache a non-ASCII glyph
        let uv = cache.cache_glyph(0x1F600, &[], 20, 20); // Emoji

        // Should be retrievable
        assert_eq!(cache.get_glyph_uv(0x1F600), Some(uv));
    }

    #[test]
    fn test_atlas_data_access() {
        let cache = GlyphCache::new(14.0, "Monospace");
        let data = cache.atlas_data();
        assert_eq!(data.len(), (2048 * 2048) as usize);
    }

    #[test]
    fn test_font_family_getter() {
        let cache = GlyphCache::new(14.0, "JetBrains Mono");
        assert_eq!(cache.font_family(), "JetBrains Mono");
    }

    #[test]
    fn test_font_metrics() {
        let cache = GlyphCache::new(14.0, "Monospace");
        let (size, family, _has_font, count) = cache.font_metrics();

        assert_eq!(size, 14.0);
        assert_eq!(family, "Monospace");
        assert!(count >= 95); // At least ASCII characters
                              // _has_font depends on whether the font was successfully loaded
    }

    #[test]
    fn test_reload_font() {
        let mut cache = GlyphCache::new(14.0, "Monospace");

        // Reload with different font
        cache.reload_font("Courier");
        assert_eq!(cache.font_family(), "Courier");

        // Cache should still have ASCII characters
        assert!(cache.len() >= 95);
    }

    #[test]
    fn test_placeholder_atlas_has_pixel_data() {
        // Use a font name that definitely won't exist to force placeholders
        let cache = GlyphCache::new(14.0, "NonExistentFontXYZ123");

        // ASCII characters should be cached
        assert!(cache.len() >= 95);

        // The atlas should have non-zero pixel data for visible characters
        let atlas = cache.atlas_data();
        let has_nonzero = atlas.iter().any(|&b| b != 0);
        assert!(
            has_nonzero,
            "Placeholder glyph atlas must have non-zero pixel data for text to be visible"
        );

        // Space (32) should have a glyph entry
        assert!(cache.get_glyph(' ' as u32).is_some());

        // 'A' (65) should have a glyph entry with non-zero UV dimensions
        let a_glyph = cache.get_glyph('A' as u32).expect("'A' should be cached");
        assert!(a_glyph.uv[2] > 0.0, "Glyph width should be non-zero");
        assert!(a_glyph.uv[3] > 0.0, "Glyph height should be non-zero");
    }
}
