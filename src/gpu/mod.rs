//! GPU-accelerated rendering module for Furnace terminal
//!
//! This module provides true hardware-accelerated rendering using wgpu,
//! enabling 170+ FPS rendering with minimal CPU usage.
//!
//! # Features
//! - Hardware-accelerated text rendering via glyphon
//! - True 24-bit color support with HDR capability
//! - Sub-pixel font rendering for crisp text
//! - Efficient glyph caching to minimize GPU uploads
//! - Background blur and transparency effects
//! - Smooth cursor animation

#[cfg(feature = "gpu")]
pub mod renderer;

#[cfg(feature = "gpu")]
pub mod text;

#[cfg(feature = "gpu")]
pub mod glyph_cache;

#[cfg(feature = "gpu")]
#[allow(unused_imports)] // Re-export may not be used internally but is part of public API
pub use renderer::GpuRenderer;

/// GPU rendering configuration
///
/// Configuration options for GPU-accelerated terminal rendering.
/// These fields are read by the GPU renderer when initializing.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
#[allow(dead_code)] // Public API - fields used by GPU renderer consumers
pub struct GpuConfig {
    /// Enable GPU acceleration (falls back to CPU if unavailable)
    pub enabled: bool,
    /// Preferred GPU backend (Vulkan, Metal, DX12, etc.)
    pub backend: GpuBackend,
    /// Enable `VSync` (limits to monitor refresh rate)
    pub vsync: bool,
    /// Font size in points
    pub font_size: f32,
    /// Font family name
    pub font_family: String,
    /// Enable sub-pixel rendering for sharper text
    pub subpixel_rendering: bool,
    /// Background opacity (0.0 = transparent, 1.0 = opaque)
    pub background_opacity: f32,
    /// Enable background blur effect
    pub background_blur: bool,
    /// Cell padding in pixels
    pub cell_padding: u32,
    /// Initial window width (None = use defaults)
    /// BUG FIX #3: Make screen dimensions configurable instead of hardcoding
    pub initial_width: Option<f32>,
    /// Initial window height (None = use defaults)
    pub initial_height: Option<f32>,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: GpuBackend::Auto,
            vsync: true,
            font_size: 14.0,
            font_family: String::from("JetBrains Mono"),
            subpixel_rendering: true,
            background_opacity: 1.0,
            background_blur: false,
            cell_padding: 2,
            initial_width: None,  // Will use 1280.0 by default in renderer
            initial_height: None, // Will use 720.0 by default in renderer
        }
    }
}

/// GPU backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuBackend {
    /// Automatically select best available backend
    #[default]
    Auto,
    /// Vulkan (Linux, Windows, Android)
    Vulkan,
    /// Metal (macOS, iOS)
    Metal,
    /// DirectX 12 (Windows)
    Dx12,
    /// DirectX 11 (Windows, fallback)
    Dx11,
    /// OpenGL (fallback, all platforms)
    OpenGl,
    /// WebGPU (browser)
    WebGpu,
}

#[cfg(feature = "gpu")]
impl From<GpuBackend> for wgpu::Backends {
    fn from(backend: GpuBackend) -> Self {
        match backend {
            GpuBackend::Auto => wgpu::Backends::all(),
            GpuBackend::Vulkan => wgpu::Backends::VULKAN,
            GpuBackend::Metal => wgpu::Backends::METAL,
            GpuBackend::Dx12 => wgpu::Backends::DX12,
            // NOTE: DX11 is not available in wgpu 0.19, falling back to DX12
            // This maintains compatibility with the API while using the best available backend
            GpuBackend::Dx11 => {
                tracing::warn!("DX11 backend not available in wgpu 0.19, using DX12 instead");
                wgpu::Backends::DX12
            }
            GpuBackend::OpenGl => wgpu::Backends::GL,
            GpuBackend::WebGpu => wgpu::Backends::BROWSER_WEBGPU,
        }
    }
}

/// Terminal cell for GPU rendering
///
/// Represents a single cell in the terminal grid for GPU rendering.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Public API - used by GPU renderer consumers
pub struct GpuCell {
    /// Character to render (as u32 for Unicode support)
    pub char_code: u32,
    /// Foreground color (RGBA)
    pub fg_color: [f32; 4],
    /// Background color (RGBA)
    pub bg_color: [f32; 4],
    /// Style flags (bold, italic, underline, etc.)
    pub style: CellStyle,
}

impl Default for GpuCell {
    fn default() -> Self {
        Self {
            char_code: ' ' as u32,
            fg_color: [1.0, 1.0, 1.0, 1.0], // White
            bg_color: [0.0, 0.0, 0.0, 1.0], // Black
            style: CellStyle::empty(),
        }
    }
}

bitflags::bitflags! {
    /// Cell style flags for GPU rendering
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CellStyle: u8 {
        const BOLD = 0b0000_0001;
        const ITALIC = 0b0000_0010;
        const UNDERLINE = 0b0000_0100;
        const STRIKETHROUGH = 0b0000_1000;
        const BLINK = 0b0001_0000;
        const REVERSE = 0b0010_0000;
        const DIM = 0b0100_0000;
        const HIDDEN = 0b1000_0000;
    }
}

/// GPU rendering statistics
///
/// Statistics about GPU rendering performance for monitoring and debugging.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Public API - fields used for performance monitoring
pub struct GpuStats {
    /// Frames rendered
    pub frame_count: u64,
    /// Average frame time in milliseconds
    pub avg_frame_time_ms: f64,
    /// GPU memory used in bytes
    pub gpu_memory_bytes: u64,
    /// Number of cached glyphs
    pub cached_glyphs: usize,
    /// Draw calls per frame
    pub draw_calls: u32,
}

/// Check if GPU rendering is available
#[cfg(feature = "gpu")]
#[must_use]
pub fn is_gpu_available() -> bool {
    // Try to create an instance to check availability
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Check if any adapters are available
    pollster::block_on(async {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .is_some()
    })
}

#[cfg(not(feature = "gpu"))]
#[must_use]
pub fn is_gpu_available() -> bool {
    false
}

/// Get GPU device information
///
/// Returns information about the available GPU device for debugging.
#[cfg(feature = "gpu")]
#[must_use]
#[allow(dead_code)] // Public API - used for debugging and diagnostics
pub fn get_gpu_info() -> Option<String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    pollster::block_on(async {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map(|adapter| {
                let info = adapter.get_info();
                format!(
                    "{} ({:?}) - {:?}",
                    info.name, info.backend, info.device_type
                )
            })
    })
}

#[cfg(not(feature = "gpu"))]
#[must_use]
pub fn get_gpu_info() -> Option<String> {
    None
}
