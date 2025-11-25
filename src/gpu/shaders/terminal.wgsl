// Terminal rendering shader for Furnace
// Supports text rendering with glyph atlas and background colors

struct Uniforms {
    view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,
    time: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var glyph_atlas: texture_2d<f32>;

@group(1) @binding(1)
var glyph_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) inst_position: vec2<f32>,
    @location(3) inst_size: vec2<f32>,
    @location(4) fg_color: vec4<f32>,
    @location(5) bg_color: vec4<f32>,
    @location(6) glyph_uv: vec4<f32>,
    @location(7) style: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) fg_color: vec4<f32>,
    @location(2) bg_color: vec4<f32>,
    @location(3) glyph_uv: vec4<f32>,
    @location(4) style: u32,
}

// Style bit flags
const STYLE_BOLD: u32 = 1u;
const STYLE_ITALIC: u32 = 2u;
const STYLE_UNDERLINE: u32 = 4u;
const STYLE_STRIKETHROUGH: u32 = 8u;
const STYLE_BLINK: u32 = 16u;
const STYLE_REVERSE: u32 = 32u;
const STYLE_DIM: u32 = 64u;
const STYLE_HIDDEN: u32 = 128u;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Calculate world position
    let world_pos = instance.inst_position + vertex.position * instance.inst_size;
    
    // Apply view-projection matrix
    output.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    
    // Pass through texture coordinates adjusted for glyph UV
    output.tex_coords = instance.glyph_uv.xy + vertex.tex_coords * instance.glyph_uv.zw;
    
    // Pass through colors and style
    output.fg_color = instance.fg_color;
    output.bg_color = instance.bg_color;
    output.glyph_uv = instance.glyph_uv;
    output.style = instance.style;
    
    return output;
}

@vertex
fn vs_bg(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var output: VertexOutput;
    
    let world_pos = instance.inst_position + vertex.position * instance.inst_size;
    output.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    output.tex_coords = vertex.tex_coords;
    output.fg_color = instance.fg_color;
    output.bg_color = instance.bg_color;
    output.glyph_uv = instance.glyph_uv;
    output.style = instance.style;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample glyph from atlas
    let glyph_alpha = textureSample(glyph_atlas, glyph_sampler, input.tex_coords).r;
    
    var fg_color = input.fg_color;
    var bg_color = input.bg_color;
    
    // Apply style modifications
    let style = input.style;
    
    // Reverse video
    if (style & STYLE_REVERSE) != 0u {
        let temp = fg_color;
        fg_color = bg_color;
        bg_color = temp;
    }
    
    // Dim
    if (style & STYLE_DIM) != 0u {
        fg_color = fg_color * 0.5;
    }
    
    // Bold (brighten)
    if (style & STYLE_BOLD) != 0u {
        fg_color = min(fg_color * 1.3, vec4<f32>(1.0));
    }
    
    // Hidden
    if (style & STYLE_HIDDEN) != 0u {
        fg_color = bg_color;
    }
    
    // Blink effect (using time)
    if (style & STYLE_BLINK) != 0u {
        let blink = sin(uniforms.time * 3.14159) * 0.5 + 0.5;
        fg_color.a = fg_color.a * blink;
    }
    
    // Mix foreground and background based on glyph alpha
    var color = mix(bg_color, fg_color, glyph_alpha);
    
    // Underline
    if (style & STYLE_UNDERLINE) != 0u {
        // Draw underline at bottom of cell
        let local_y = fract(input.tex_coords.y * 10.0);
        if local_y > 0.9 {
            color = fg_color;
        }
    }
    
    // Strikethrough
    if (style & STYLE_STRIKETHROUGH) != 0u {
        let local_y = fract(input.tex_coords.y * 10.0);
        if local_y > 0.45 && local_y < 0.55 {
            color = fg_color;
        }
    }
    
    return color;
}

@fragment
fn fs_bg(input: VertexOutput) -> @location(0) vec4<f32> {
    var bg_color = input.bg_color;
    
    // Apply reverse if needed
    if (input.style & STYLE_REVERSE) != 0u {
        bg_color = input.fg_color;
    }
    
    return bg_color;
}
