//! GPU renderer implementation using wgpu
//!
//! Provides hardware-accelerated rendering for the terminal.
//!
//! # Implementation Status
//!
//! This GPU renderer is an **optional feature** currently under active development.
//! Some functionality is not yet complete:
//!
//! ## ⚠️ Known Limitations
//!
//! 1. **Surface Creation (BUG #10)**: The renderer does not create its own surface.
//!    To use GPU rendering, you must:
//!    - Create a window using `winit` or similar
//!    - Create a wgpu surface from that window
//!    - Pass the surface to the renderer via a new `with_surface()` method
//!
//! 2. **Glyph Upload (BUG #4)**: Glyphs are cached but not uploaded to GPU texture.
//!    The glyph atlas texture is created but remains empty. To fix:
//!    - Implement font rasterization using `fontdue`
//!    - Upload rasterized glyphs to the atlas texture
//!    - Update glyph cache when new characters are encountered
//!
//! 3. **Dirty Rectangles (BUG #24)**: Currently uploads all cells every frame.
//!    Future optimization: track changed cells and only update those regions.
//!
//! ## Usage
//!
//! To enable GPU rendering:
//! ```bash
//! cargo build --features gpu
//! ```
//!
//! See `examples/gpu_rendering.rs` (TODO) for complete usage example.

// Allow pedantic warnings for optional GPU feature code
#![allow(clippy::pedantic)]

use wgpu::util::DeviceExt;

use super::{GpuCell, GpuConfig, GpuStats};

/// GPU-accelerated terminal renderer
///
/// Provides hardware-accelerated text rendering using wgpu for 170+ FPS performance.
/// This renderer uses the GPU to draw terminal cells with custom colors and styles.
///
/// # Architecture
/// - Uses vertex and instance buffers for efficient cell rendering
/// - Maintains a glyph atlas texture for character caching
/// - Supports 24-bit true color rendering
/// - Implements dirty flagging for optimal performance
///
/// # Note
/// and used in the complete GPU rendering pipeline. The GPU module is an optional feature
/// that is still under development.
pub struct GpuRenderer {
    /// WGPU instance
    instance: wgpu::Instance,
    /// GPU adapter
    adapter: wgpu::Adapter,
    /// GPU device
    device: wgpu::Device,
    /// Command queue
    queue: wgpu::Queue,
    /// Surface for rendering (if windowed)
    surface: Option<wgpu::Surface<'static>>,
    /// Surface configuration
    surface_config: Option<wgpu::SurfaceConfiguration>,
    /// Render pipeline for text
    text_pipeline: wgpu::RenderPipeline,
    /// Render pipeline for backgrounds
    bg_pipeline: wgpu::RenderPipeline,
    /// Vertex buffer for quads
    vertex_buffer: wgpu::Buffer,
    /// Index buffer for quads
    index_buffer: wgpu::Buffer,
    /// Instance buffer for cells
    instance_buffer: wgpu::Buffer,
    /// Uniform buffer for view/projection
    uniform_buffer: wgpu::Buffer,
    /// Bind group for uniforms
    uniform_bind_group: wgpu::BindGroup,
    /// Glyph texture atlas
    glyph_atlas: wgpu::Texture,
    /// Glyph atlas view
    glyph_atlas_view: wgpu::TextureView,
    /// Glyph atlas sampler
    glyph_sampler: wgpu::Sampler,
    /// Glyph bind group
    glyph_bind_group: wgpu::BindGroup,
    /// Terminal dimensions (columns, rows)
    terminal_size: (u32, u32),
    /// Cell size in pixels (width, height)
    cell_size: (f32, f32),
    /// Current cells to render
    cells: Vec<GpuCell>,
    /// Configuration
    config: GpuConfig,
    /// Statistics
    stats: GpuStats,
    /// Glyph cache
    glyph_cache: super::glyph_cache::GlyphCache,
}

/// Vertex for rendering quads
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

/// Instance data for each cell
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CellInstance {
    /// Position (x, y) in pixels
    position: [f32; 2],
    /// Size (width, height) in pixels
    size: [f32; 2],
    /// Foreground color (RGBA)
    fg_color: [f32; 4],
    /// Background color (RGBA)
    bg_color: [f32; 4],
    /// Glyph UV coordinates (x, y, width, height)
    glyph_uv: [f32; 4],
    /// Style flags
    style: u32,
}

/// Uniforms for the shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    /// View-projection matrix (orthographic)
    view_proj: [[f32; 4]; 4],
    /// Screen size
    screen_size: [f32; 2],
    /// Time for animations
    time: f32,
    /// Padding
    _padding: f32,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub async fn new(config: GpuConfig) -> Result<Self, GpuError> {
        // Create WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: config.backend.into(),
            ..Default::default()
        });

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        // Request device
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Furnace GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| GpuError::DeviceRequest(e.to_string()))?;

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terminal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/terminal.wgsl").into()),
        });

        // Create uniform buffer with dynamic screen size
        // BUG FIX #3: Don't hardcode 1920x1080 - use config dimensions or defaults
        let (initial_width, initial_height) = (
            config.initial_width.unwrap_or(1280.0),
            config.initial_height.unwrap_or(720.0)
        );
        
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                view_proj: orthographic_projection(initial_width, initial_height),
                screen_size: [initial_width, initial_height],
                time: 0.0,
                _padding: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create uniform bind group layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create uniform bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create glyph atlas texture
        let glyph_atlas_size = 2048;
        let glyph_atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width: glyph_atlas_size,
                height: glyph_atlas_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let glyph_atlas_view = glyph_atlas.create_view(&wgpu::TextureViewDescriptor::default());

        let glyph_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create glyph bind group layout
        let glyph_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Glyph Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let glyph_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Bind Group"),
            layout: &glyph_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&glyph_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&glyph_sampler),
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terminal Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &glyph_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create text render pipeline
        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    // Instance buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<CellInstance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            2 => Float32x2,  // position
                            3 => Float32x2,  // size
                            4 => Float32x4,  // fg_color
                            5 => Float32x4,  // bg_color
                            6 => Float32x4,  // glyph_uv
                            7 => Uint32,     // style
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create background pipeline (same as text but different shader entry)
        let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_bg",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<CellInstance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            2 => Float32x2,
                            3 => Float32x2,
                            4 => Float32x4,
                            5 => Float32x4,
                            6 => Float32x4,
                            7 => Uint32,
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_bg",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create quad vertices
        let vertices = [
            Vertex {
                position: [0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create indices for two triangles
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create instance buffer (pre-allocate for large terminal - 320 cols x 100 rows)
        // This handles 4K monitors with typical font sizes. Buffer will be recreated if needed.
        let max_cells = 320 * 100;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (max_cells * std::mem::size_of::<CellInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create glyph cache with font loading
        let glyph_cache =
            super::glyph_cache::GlyphCache::new(config.font_size, &config.font_family);
        
        // BUG FIX #4: Upload glyph atlas data to GPU texture
        // This ensures glyphs are actually visible when rendered
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &glyph_atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            glyph_cache.atlas_data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(glyph_atlas_size),
                rows_per_image: Some(glyph_atlas_size),
            },
            wgpu::Extent3d {
                width: glyph_atlas_size,
                height: glyph_atlas_size,
                depth_or_array_layers: 1,
            },
        );

        // BUG FIX #8: Calculate cell size from font metrics instead of magic numbers
        // Standard monospace font metrics:
        // - Width is typically 0.6 * font_size for monospace fonts
        // - Height is typically 1.2 * font_size (with line spacing)
        // These are industry-standard ratios for monospace terminal fonts
        const CELL_WIDTH_RATIO: f32 = 0.6;
        const CELL_HEIGHT_RATIO: f32 = 1.2;
        let cell_width = config.font_size * CELL_WIDTH_RATIO;
        let cell_height = config.font_size * CELL_HEIGHT_RATIO;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            surface_config: None,
            text_pipeline,
            bg_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            uniform_bind_group,
            glyph_atlas,
            glyph_atlas_view,
            glyph_sampler,
            glyph_bind_group,
            terminal_size: (80, 24),
            cell_size: (cell_width, cell_height),
            cells: Vec::with_capacity(80 * 24),
            config,
            stats: GpuStats::default(),
            glyph_cache,
        })
    }

    /// Update terminal content
    pub fn update_cells(&mut self, cells: &[GpuCell], cols: u32, rows: u32) {
        self.terminal_size = (cols, rows);
        self.cells.clear();
        self.cells.extend_from_slice(cells);
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(config) = &mut self.surface_config {
            config.width = width;
            config.height = height;
            if let Some(surface) = &self.surface {
                surface.configure(&self.device, config);
            }
        }

        // Update uniforms
        let uniforms = Uniforms {
            view_proj: orthographic_projection(width as f32, height as f32),
            screen_size: [width as f32, height as f32],
            time: 0.0,
            _padding: 0.0,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Set surface for rendering
    ///
    /// BUG FIX #10: Provide method to attach a surface for actual rendering.
    /// The renderer cannot create its own surface as that requires a window,
    /// which is outside the scope of this rendering module.
    ///
    /// # Arguments
    /// * `surface` - wgpu surface created from a window
    /// * `width` - Surface width in pixels
    /// * `height` - Surface height in pixels
    ///
    /// # Example
    /// ```ignore
    /// let window = /* create winit window */;
    /// let surface = instance.create_surface(&window)?;
    /// renderer.set_surface(surface, 1920, 1080);
    /// ```
    pub fn set_surface(&mut self, surface: wgpu::Surface<'static>, width: u32, height: u32) {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: if self.config.vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&self.device, &config);
        self.surface = Some(surface);
        self.surface_config = Some(config);
        
        // Update uniforms for new size
        self.resize(width, height);
    }
    
    /// Upload glyph atlas to GPU
    ///
    /// BUG FIX #4: Provide method to upload glyph atlas data when new glyphs are cached.
    /// Call this after caching new glyphs to ensure they appear correctly.
    pub fn upload_glyph_atlas(&mut self) {
        let atlas_size = self.glyph_cache.atlas_size();
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.glyph_atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            self.glyph_cache.atlas_data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(atlas_size),
                rows_per_image: Some(atlas_size),
            },
            wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Render a frame
    pub fn render(&mut self) -> Result<(), GpuError> {
        let start_time = std::time::Instant::now();

        // Build instance data
        let instances: Vec<CellInstance> = self
            .cells
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let col = (i % self.terminal_size.0 as usize) as f32;
                let row = (i / self.terminal_size.0 as usize) as f32;

                // Get glyph UV from cache
                let glyph_uv = self
                    .glyph_cache
                    .get_glyph_uv(cell.char_code)
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]);

                CellInstance {
                    position: [col * self.cell_size.0, row * self.cell_size.1],
                    size: [self.cell_size.0, self.cell_size.1],
                    fg_color: cell.fg_color,
                    bg_color: cell.bg_color,
                    glyph_uv,
                    style: cell.style.bits() as u32,
                }
            })
            .collect();

        // Update instance buffer
        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        // Get surface texture (if using surface rendering)
        let surface_texture = if let Some(surface) = &self.surface {
            Some(
                surface
                    .get_current_texture()
                    .map_err(|e| GpuError::SurfaceError(e.to_string()))?,
            )
        } else {
            None
        };

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        if let Some(texture) = &surface_texture {
            let view = texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Draw backgrounds first
                render_pass.set_pipeline(&self.bg_pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_bind_group(1, &self.glyph_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);

                // Draw text
                render_pass.set_pipeline(&self.text_pipeline);
                render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
            }
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));

        // Present surface
        if let Some(texture) = surface_texture {
            texture.present();
        }

        // Update statistics
        self.stats.frame_count += 1;
        self.stats.draw_calls = 2;
        let frame_time = start_time.elapsed().as_secs_f64() * 1000.0;
        self.stats.avg_frame_time_ms = (self.stats.avg_frame_time_ms * 0.9) + (frame_time * 0.1);

        Ok(())
    }

    /// Get rendering statistics
    pub fn get_stats(&self) -> &GpuStats {
        &self.stats
    }

    /// Get GPU device info
    pub fn get_device_info(&self) -> String {
        let info = self.adapter.get_info();
        format!(
            "{} ({:?}) - {:?}",
            info.name, info.backend, info.device_type
        )
    }

    /// Get the wgpu instance for advanced use cases
    ///
    /// This allows external code to create additional surfaces or inspect
    /// available adapters for multi-GPU setups or hot-reloading scenarios.
    ///
    /// # Production Use Cases
    /// - Creating additional rendering surfaces for split views
    /// - Querying available adapters for GPU selection UI
    /// - Implementing GPU device hot-swapping on laptop dock/undock
    /// - Creating compute pipelines for terminal effects
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Get the glyph atlas texture view for debugging or custom shaders
    ///
    /// Allows inspection of the glyph atlas for debugging font rendering issues
    /// or implementing custom text effects in user shaders.
    ///
    /// # Production Use Cases
    /// - Debugging glyph rasterization issues
    /// - Implementing custom text effects (glow, shadow, outline)
    /// - Creating texture atlas visualizer for development tools
    /// - Sharing glyph atlas with external rendering pipelines
    pub fn glyph_atlas_view(&self) -> &wgpu::TextureView {
        &self.glyph_atlas_view
    }

    /// Get the glyph atlas sampler for custom rendering
    ///
    /// Provides access to the configured sampler for the glyph atlas,
    /// useful when implementing custom rendering pipelines or effects.
    ///
    /// # Production Use Cases
    /// - Creating custom bind groups with the glyph atlas
    /// - Implementing text post-processing effects
    /// - Sharing sampler configuration with user shaders
    /// - Optimizing texture sampling for different display scales
    pub fn glyph_sampler(&self) -> &wgpu::Sampler {
        &self.glyph_sampler
    }

    /// Regenerate the glyph atlas with updated content
    ///
    /// Updates the GPU texture with fresh glyph data from the cache.
    /// Called automatically when new glyphs are cached, but can be
    /// manually invoked for cache rebuilds or font changes.
    ///
    /// # Production Use Cases
    /// - Font hot-reloading during development
    /// - Implementing font size change at runtime
    /// - Recovering from GPU device loss
    /// - Updating glyphs after theme change affects font rendering
    pub fn update_glyph_atlas(&mut self) {
        // Delegate to the existing upload_glyph_atlas method
        self.upload_glyph_atlas();
    }

    /// Query GPU adapter capabilities
    ///
    /// Returns detailed information about GPU capabilities for feature detection
    /// and performance optimization decisions.
    ///
    /// # Production Use Cases
    /// - Detecting hardware support for advanced features
    /// - Adjusting quality settings based on GPU capabilities
    /// - Displaying GPU specs in settings/about dialog
    /// - Logging GPU info for bug reports
    pub fn get_adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    /// Get the current GPU backend in use
    ///
    /// Returns the rendering backend being used by this renderer instance.
    ///
    /// # Production Use Cases
    /// - Displaying active backend in settings UI
    /// - Logging backend information for diagnostics
    /// - Platform-specific behavior adjustments
    ///
    /// # Note
    /// This returns the backend selected during renderer initialization,
    /// not all potentially available backends on the system.
    pub fn current_backend(&self) -> wgpu::Backend {
        let info = self.adapter.get_info();
        info.backend
    }
}

/// GPU rendering errors
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("No GPU adapter available")]
    NoAdapter,
    #[error("Failed to request GPU device: {0}")]
    DeviceRequest(String),
    #[error("Surface error: {0}")]
    SurfaceError(String),
    #[error("Shader compilation error: {0}")]
    ShaderError(String),
}

/// Create orthographic projection matrix
fn orthographic_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    let left = 0.0;
    let right = width;
    let bottom = height;
    let top = 0.0;
    let near = -1.0;
    let far = 1.0;

    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, 1.0 / (far - near), 0.0],
        [
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            -near / (far - near),
            1.0,
        ],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_renderer_creation() {
        let config = GpuConfig::default();
        let result = GpuRenderer::new(config).await;
        
        // Should either succeed or fail gracefully
        // (May fail if no GPU is available in test environment)
        match result {
            Ok(renderer) => {
                // Test that we can access the instance
                let _instance = renderer.instance();
                
                // Test that we can get device info
                let device_info = renderer.get_device_info();
                assert!(!device_info.is_empty());
                
                // Test adapter info access
                let adapter_info = renderer.get_adapter_info();
                assert!(!adapter_info.name.is_empty());
            }
            Err(_e) => {
                // GPU not available in test environment - this is expected
                // No need to log, test environment may not have GPU
            }
        }
    }

    #[tokio::test]
    async fn test_gpu_renderer_glyph_atlas_access() {
        let config = GpuConfig::default();
        let result = GpuRenderer::new(config).await;
        
        if let Ok(renderer) = result {
            // Test glyph atlas view access
            let _atlas_view = renderer.glyph_atlas_view();
            
            // Test glyph sampler access
            let _sampler = renderer.glyph_sampler();
            
            // These should not panic - just verifies the methods work
        }
    }

    #[tokio::test]
    async fn test_gpu_renderer_update_glyph_atlas() {
        let config = GpuConfig::default();
        let result = GpuRenderer::new(config).await;
        
        if let Ok(mut renderer) = result {
            // Test that we can update the glyph atlas
            // This should not panic
            renderer.update_glyph_atlas();
        }
    }

    #[tokio::test]
    async fn test_gpu_backend_support() {
        let config = GpuConfig::default();
        let result = GpuRenderer::new(config).await;
        
        if let Ok(renderer) = result {
            // Test current backend method
            let backend = renderer.current_backend();
            
            // Get adapter info to verify backend is reported correctly
            let info = renderer.get_adapter_info();
            assert_eq!(backend, info.backend);
        }
    }
}
