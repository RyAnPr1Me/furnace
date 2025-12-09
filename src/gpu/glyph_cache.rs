//! Glyph cache for GPU text rendering
//!
//! Caches rasterized glyphs in a texture atlas for efficient GPU rendering.
//! Uses fontdue for font rasterization to provide actual glyph bitmaps.

use std::collections::HashMap;

/// Glyph cache for efficient text rendering with fontdue font rasterization
pub struct GlyphCache {
    /// Map from character code to UV coordinates in atlas
    glyph_map: HashMap<u32, GlyphInfo>,
    /// Font for rasterization
    #[cfg(feature = "gpu")]
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
#[derive(Debug, Clone, Copy)]
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

impl GlyphCache {
    /// Create a new glyph cache with font loading
    ///
    /// BUG FIX #4: Implement actual font loading and rasterization
    pub fn new(font_size: f32, font_family: &str) -> Self {
        #[cfg(feature = "gpu")]
        let font = Self::load_font(font_family);
        
        let atlas_size = 2048;
        let mut cache = Self {
            glyph_map: HashMap::with_capacity(256),
            #[cfg(feature = "gpu")]
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
    /// BUG FIX #4: Actually load fonts for rendering
    #[cfg(feature = "gpu")]
    fn load_font(font_family: &str) -> Option<fontdue::Font> {
        // Try to load from common system font paths
        let font_paths = Self::get_font_paths(font_family);
        
        for path in font_paths {
            if let Ok(data) = std::fs::read(&path) {
                if let Ok(font) = fontdue::Font::from_bytes(
                    data,
                    fontdue::FontSettings::default()
                ) {
                    tracing::info!("Loaded font from: {}", path);
                    return Some(font);
                }
            }
        }
        
        // Fallback to embedded font data (a minimal monospace font)
        // In production, include a default monospace font like JetBrains Mono
        tracing::warn!("Could not load font '{}', using fallback", font_family);
        None
    }
    
    /// Get common font file paths based on font name
    #[cfg(feature = "gpu")]
    fn get_font_paths(font_family: &str) -> Vec<String> {
        let mut paths = Vec::new();
        
        // Windows
        paths.push(format!("C:\\Windows\\Fonts\\{}.ttf", font_family));
        paths.push(format!("C:\\Windows\\Fonts\\{}.otf", font_family));
        
        // Common monospace fonts on Windows
        if font_family.contains("Mono") || font_family.contains("Consolas") {
            paths.push("C:\\Windows\\Fonts\\consola.ttf".to_string());
            paths.push("C:\\Windows\\Fonts\\cour.ttf".to_string());
        }
        
        // Linux
        paths.push(format!("/usr/share/fonts/truetype/{}/{}.ttf", 
            font_family.to_lowercase(), font_family));
        paths.push(format!("/usr/share/fonts/TTF/{}.ttf", font_family));
        
        // macOS
        paths.push(format!("/System/Library/Fonts/{}.ttf", font_family));
        paths.push(format!("/Library/Fonts/{}.ttf", font_family));
        
        paths
    }

    /// Pre-cache ASCII characters for faster rendering
    ///
    /// BUG FIX #4: Actually rasterize glyphs and upload to atlas
    fn precache_ascii(&mut self) {
        #[cfg(feature = "gpu")]
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
    #[cfg(feature = "gpu")]
    fn cache_glyph_rendered(&mut self, c: char, code: u32) {
        // Get font reference
        let Some(ref font) = self.font else { return; };
        
        // Rasterize the glyph
        let (metrics, bitmap) = font.rasterize(c, self.font_size);
        
        let width = metrics.width as u32;
        let height = metrics.height as u32;
        
        // Skip if glyph is too large or empty
        if width == 0 || height == 0 || width > 256 || height > 256 {
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
    
    /// Fallback: pre-cache ASCII with placeholder rectangles
    fn precache_ascii_placeholders(&mut self) {
        // BUG FIX #8: Use consistent font metric ratios
        const CELL_WIDTH_RATIO: f32 = 0.6;
        const CELL_HEIGHT_RATIO: f32 = 1.2;
        let glyph_width = (self.font_size * CELL_WIDTH_RATIO) as u32;
        let glyph_height = (self.font_size * CELL_HEIGHT_RATIO) as u32;
        let atlas_size = self.atlas_size as f32;

        // Cache printable ASCII (32-126)
        for code in 32u32..=126 {
            // Calculate position in atlas
            if self.cursor_x + glyph_width > self.atlas_size {
                self.cursor_x = 0;
                self.cursor_y += self.row_height + 2;
                self.row_height = 0;
            }

            let uv = [
                self.cursor_x as f32 / atlas_size,
                self.cursor_y as f32 / atlas_size,
                glyph_width as f32 / atlas_size,
                glyph_height as f32 / atlas_size,
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
    #[cfg(feature = "gpu")]
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
        #[cfg(feature = "gpu")]
        let has_font = self.font.is_some();
        #[cfg(not(feature = "gpu"))]
        let has_font = false;
        
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
    #[cfg(feature = "gpu")]
    fn test_reload_font() {
        let mut cache = GlyphCache::new(14.0, "Monospace");
        
        // Reload with different font
        cache.reload_font("Courier");
        assert_eq!(cache.font_family(), "Courier");
        
        // Cache should still have ASCII characters
        assert!(cache.len() >= 95);
    }
}
