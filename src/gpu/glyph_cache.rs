//! Glyph cache for GPU text rendering
//!
//! Caches rasterized glyphs in a texture atlas for efficient GPU rendering.

use std::collections::HashMap;

/// Glyph cache for efficient text rendering
#[allow(dead_code)] // Some fields are for future use in complete GPU implementation
pub struct GlyphCache {
    /// Map from character code to UV coordinates in atlas
    glyph_map: HashMap<u32, GlyphInfo>,
    /// Font size
    font_size: f32,
    /// Font family name
    font_family: String,
    /// Atlas dimensions
    atlas_size: u32,
    /// Current x position in atlas
    cursor_x: u32,
    /// Current y position in atlas  
    cursor_y: u32,
    /// Maximum height in current row
    row_height: u32,
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
    /// Create a new glyph cache
    pub fn new(font_size: f32, font_family: &str) -> Self {
        let mut cache = Self {
            glyph_map: HashMap::with_capacity(256),
            font_size,
            font_family: font_family.to_string(),
            atlas_size: 2048,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
        };

        // Pre-cache ASCII characters
        cache.precache_ascii();

        cache
    }

    /// Pre-cache ASCII characters for faster rendering
    fn precache_ascii(&mut self) {
        // Calculate approximate glyph dimensions
        let glyph_width = (self.font_size * 0.6) as u32;
        let glyph_height = (self.font_size * 1.2) as u32;
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
    pub fn cache_glyph(&mut self, char_code: u32, _bitmap: &[u8], width: u32, height: u32) -> [f32; 4] {
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
}
