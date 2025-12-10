# Feature Implementation Specification

This document provides detailed technical specifications for implementing all remaining features to eliminate dead code warnings.

## Overview

All features listed below MUST be fully implemented per user requirement. No shortcuts, no `#[allow(dead_code)]`, no stub implementations.

## Current Status

**Completed:**
- ✅ Color palette system
- ✅ All 4 hooks (on_output, on_bell, on_title_change, on_command_end)
- ✅ Theme ANSI colors integration
- ✅ Removed all `#[allow(dead_code)]` attributes

**Must Implement:** 7 major features

---

## Feature 1: Text Selection (`ThemeConfig.selection`)

### Status: 🔴 NOT IMPLEMENTED

### Requirements:
- Mouse-based text selection (click and drag)
- Use `ThemeConfig.selection` color for highlight
- Copy to clipboard support
- Selection state tracking

### Implementation Plan:

**1. Add Selection State to Terminal:**
```rust
// In Terminal struct
selection_start: Option<(u16, u16)>,  // (col, row)
selection_end: Option<(u16, u16)>,
selection_active: bool,
```

**2. Handle Mouse Events:**
```rust
// In handle_mouse_event()
MouseEventKind::Down(MouseButton::Left) => {
    self.selection_start = Some((event.column, event.row));
    self.selection_active = true;
}
MouseEventKind::Drag(MouseButton::Left) => {
    self.selection_end = Some((event.column, event.row));
}
MouseEventKind::Up(MouseButton::Left) => {
    self.copy_selection_to_clipboard();
}
```

**3. Render Selection:**
```rust
// In render() method, apply selection background color
if self.is_position_selected(col, row) {
    let selection_color = parse_color(&self.config.theme.selection)?;
    style = style.bg(selection_color);
}
```

**4. Clipboard Integration:**
```rust
// Use arboard crate (already in Cargo.toml)
fn copy_selection_to_clipboard(&mut self) {
    let text = self.get_selected_text();
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        clipboard.set_text(text).ok();
    }
}
```

### Files to Modify:
- `src/terminal/mod.rs`: Add selection fields, mouse handler, rendering
- `Cargo.toml`: Ensure arboard is listed (already present)

### Tests Required:
```rust
#[test]
fn test_text_selection() {
    // Test selection start/end tracking
    // Test selection rendering
    // Test clipboard copy
}
```

### Estimated Complexity: HIGH (6-8 hours)

---

## Feature 2: Lua Custom Keybindings (`custom_keybindings`)

### Status: 🔴 NOT IMPLEMENTED

### Requirements:
- Execute Lua functions on custom key combinations
- Pass context (cwd, last_command, etc.) to Lua
- Register with KeybindingManager

### Implementation Plan:

**1. Execute Lua Functions:**
```rust
// In HooksExecutor
pub fn execute_custom_keybinding(&self, lua_code: &str, context: &KeyContext) -> Result<()> {
    // Set up context table
    let globals = self.lua.globals();
    let ctx_table = self.lua.create_table()?;
    ctx_table.set("cwd", context.cwd)?;
    ctx_table.set("last_command", context.last_command)?;
    globals.set("context", ctx_table)?;
    
    // Execute Lua code
    self.lua.load(lua_code).exec()?;
    Ok(())
}
```

**2. Register Custom Bindings:**
```rust
// In Terminal::new()
for (key_combo, lua_func) in &config.hooks.custom_keybindings {
    let action = Action::ExecuteLua(lua_func.clone());
    keybindings.register_custom(key_combo, action)?;
}
```

**3. Handle in Event Loop:**
```rust
// When Action::ExecuteLua is triggered
Action::ExecuteLua(code) => {
    if let Some(ref executor) = self.hooks_executor {
        let ctx = KeyContext {
            cwd: self.keybindings.shell_integration().current_directory.clone(),
            last_command: self.keybindings.shell_integration().last_command.clone(),
        };
        executor.execute_custom_keybinding(&code, &ctx)?;
    }
}
```

### Files to Modify:
- `src/hooks.rs`: Add `execute_custom_keybinding()` method
- `src/keybindings.rs`: Add `Action::ExecuteLua` variant, `register_custom()` method
- `src/terminal/mod.rs`: Register bindings in `new()`, handle in event loop

### Tests Required:
```rust
#[test]
fn test_custom_keybinding_execution() {
    // Test Lua function execution
    // Test context passing
}
```

### Estimated Complexity: MEDIUM-HIGH (4-6 hours)

---

## Feature 3: Lua Output Filters (`output_filters`)

### Status: 🔴 NOT IMPLEMENTED

### Requirements:
- Apply Lua transformation functions to shell output
- Pipeline multiple filters in sequence
- Handle errors gracefully

### Implementation Plan:

**1. Add Filter Application Method:**
```rust
// In HooksExecutor
pub fn apply_output_filters(&self, output: &str, filters: &[String]) -> Result<String> {
    let mut result = output.to_string();
    
    for filter in filters {
        // Set up globals
        let globals = self.lua.globals();
        globals.set("input", result.clone())?;
        
        // Execute filter
        self.lua.load(filter).exec()?;
        
        // Get result
        result = globals.get::<_, String>("output")?;
    }
    
    Ok(result)
}
```

**2. Apply in Shell Output Processing:**
```rust
// In terminal mod.rs, shell output reading
let mut output_str = String::from_utf8_lossy(&self.read_buffer[..n]).into_owned();

// Apply output filters if configured
if !self.config.hooks.output_filters.is_empty() {
    if let Some(ref executor) = self.hooks_executor {
        output_str = executor.apply_output_filters(&output_str, &self.config.hooks.output_filters)
            .unwrap_or_else(|e| {
                warn!("Output filter failed: {}", e);
                output_str  // Use unfiltered on error
            });
    }
}
```

### Files to Modify:
- `src/hooks.rs`: Add `apply_output_filters()` method
- `src/terminal/mod.rs`: Apply filters in shell output processing

### Tests Required:
```rust
#[test]
fn test_output_filter_pipeline() {
    // Test single filter
    // Test multiple filters in sequence
    // Test error handling
}
```

### Estimated Complexity: MEDIUM (3-4 hours)

---

## Feature 4: Lua Custom Widgets (`custom_widgets`)

### Status: 🔴 NOT IMPLEMENTED

### Requirements:
- Execute Lua widget code each frame
- Render returned widget specifications
- Reserve screen space for widgets

### Implementation Plan:

**1. Define Widget API:**
```rust
pub struct LuaWidget {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub content: Vec<String>,
    pub style: WidgetStyle,
}
```

**2. Execute Widget Code:**
```rust
// In HooksExecutor
pub fn execute_widget(&self, lua_code: &str) -> Result<LuaWidget> {
    // Execute Lua code
    self.lua.load(lua_code).exec()?;
    
    // Extract widget definition from globals
    let globals = self.lua.globals();
    let widget_table: mlua::Table = globals.get("widget")?;
    
    Ok(LuaWidget {
        x: widget_table.get("x")?,
        y: widget_table.get("y")?,
        width: widget_table.get("width")?,
        height: widget_table.get("height")?,
        content: widget_table.get("content")?,
        style: parse_style(widget_table.get("style")?)?,
    })
}
```

**3. Render Widgets:**
```rust
// In render() method
for widget_code in &self.config.hooks.custom_widgets {
    if let Some(ref executor) = self.hooks_executor {
        if let Ok(widget) = executor.execute_widget(widget_code) {
            self.render_lua_widget(f, widget)?;
        }
    }
}
```

### Files to Modify:
- `src/hooks.rs`: Add `execute_widget()` method
- `src/terminal/mod.rs`: Execute and render widgets in `render()`

### Tests Required:
```rust
#[test]
fn test_lua_widget_rendering() {
    // Test widget execution
    // Test widget rendering
}
```

### Estimated Complexity: VERY HIGH (8-10 hours)

---

## Feature 5: Background Images (`BackgroundConfig`)

### Status: 🔴 NOT IMPLEMENTED

### Requirements:
- Load images (PNG, JPEG) from `image_path`
- Implement 5 rendering modes: fill, fit, stretch, tile, center
- Apply `opacity` and `blur` effects
- Use `color` as fallback

### Implementation Plan:

**1. Load Image:**
```rust
// Add to Terminal struct
background_image: Option<DynamicImage>,

// In Terminal::new()
let background_image = if let Some(ref bg_config) = config.theme.background_image {
    if let Some(ref path) = bg_config.image_path {
        image::open(path).ok()
    } else {
        None
    }
} else {
    None
};
```

**2. Implement Rendering Modes:**
```rust
fn render_background(&self, f: &mut Frame, area: Rect, config: &BackgroundConfig) -> Result<()> {
    match config.mode.as_str() {
        "fill" => self.render_background_fill(f, area, config),
        "fit" => self.render_background_fit(f, area, config),
        "stretch" => self.render_background_stretch(f, area, config),
        "tile" => self.render_background_tile(f, area, config),
        "center" => self.render_background_center(f, area, config),
        _ => Ok(()),
    }
}
```

**3. Apply Opacity and Blur:**
```rust
fn apply_effects(img: &mut DynamicImage, opacity: f32, blur: f32) {
    // Apply blur
    if blur > 0.0 {
        *img = img.blur(blur);
    }
    
    // Apply opacity by adjusting alpha channel
    if opacity < 1.0 {
        for pixel in img.as_mut_rgba8().unwrap().pixels_mut() {
            pixel[3] = (pixel[3] as f32 * opacity) as u8;
        }
    }
}
```

**Challenge:** Ratatui doesn't support background images natively. Solutions:
- Option A: Render to terminal using Unicode block characters (hacky)
- Option B: Make this GPU-only feature (clean)
- Option C: Use custom backend

**Recommendation:** Implement as GPU-only feature with clear documentation.

### Files to Modify:
- `src/terminal/mod.rs` OR `src/gpu/renderer.rs` (if GPU-only)
- Add `image` crate usage

### Tests Required:
```rust
#[test]
fn test_background_image_loading() {
    // Test image loading
    // Test each render mode
    // Test opacity and blur
}
```

### Estimated Complexity: VERY HIGH (8-12 hours for full implementation, 2-3 hours for GPU-only stub)

---

## Feature 6: Cursor Trails (`CursorTrailConfig`)

### Status: 🔴 NOT IMPLEMENTED

### Requirements:
- Track cursor position history (circular buffer of `length` positions)
- Render trail with fading based on `fade_mode` (linear, exponential, smooth)
- Use `color` with alpha channel
- Apply `width` multiplier
- Update every `animation_speed` ms

### Implementation Plan:

**1. Add Trail State:**
```rust
// In Terminal struct
cursor_trail: VecDeque<(u16, u16, Instant)>,  // (col, row, timestamp)
last_trail_update: Instant,
```

**2. Track Cursor:**
```rust
// Update cursor position tracking
fn update_cursor_position(&mut self, col: u16, row: u16) {
    let config = &self.config.theme.cursor_trail;
    if let Some(trail_config) = config {
        if trail_config.enabled {
            self.cursor_trail.push_back((col, row, Instant::now()));
            
            // Limit trail length
            while self.cursor_trail.len() > trail_config.length {
                self.cursor_trail.pop_front();
            }
        }
    }
}
```

**3. Render Trail:**
```rust
fn render_cursor_trail(&self, f: &mut Frame, config: &CursorTrailConfig) {
    let now = Instant::now();
    
    for (i, (col, row, timestamp)) in self.cursor_trail.iter().enumerate() {
        let age = now.duration_since(*timestamp).as_millis() as f32;
        let alpha = self.calculate_trail_alpha(age, i, config);
        
        let color = parse_color_with_alpha(&config.color, alpha);
        // Render trail position with color
    }
}

fn calculate_trail_alpha(&self, age: f32, position: usize, config: &CursorTrailConfig) -> f32 {
    let ratio = position as f32 / config.length as f32;
    match config.fade_mode.as_str() {
        "linear" => ratio,
        "exponential" => ratio.powf(2.0),
        "smooth" => 1.0 - (1.0 - ratio).powf(3.0),  // ease-out cubic
        _ => ratio,
    }
}
```

**Challenge:** Similar to backgrounds, cursor trails require overlay rendering not supported by ratatui.

**Recommendation:** Implement as GPU-only feature.

### Files to Modify:
- `src/terminal/mod.rs` OR `src/gpu/renderer.rs` (if GPU-only)

### Tests Required:
```rust
#[test]
fn test_cursor_trail() {
    // Test trail tracking
    // Test fade calculations
    // Test rendering
}
```

### Estimated Complexity: HIGH (6-8 hours for full implementation, 2 hours for GPU-only stub)

---

## Feature 7: KeyBindings Integration

### Status: 🔴 NOT IMPLEMENTED

### Requirements:
- Parse key combinations from config strings ("Ctrl+T", "Ctrl+Shift+V", etc.)
- Register with KeybindingManager
- Override defaults with config values

### Implementation Plan:

**1. Parse Key Combinations:**
```rust
fn parse_key_combination(combo: &str) -> Result<(Vec<KeyModifiers>, KeyCode)> {
    let parts: Vec<&str> = combo.split('+').collect();
    let mut modifiers = Vec::new();
    let key = parts.last().unwrap();
    
    for part in &parts[..parts.len()-1] {
        match part.to_lowercase().as_str() {
            "ctrl" => modifiers.push(KeyModifiers::CONTROL),
            "shift" => modifiers.push(KeyModifiers::SHIFT),
            "alt" => modifiers.push(KeyModifiers::ALT),
            _ => {}
        }
    }
    
    let keycode = parse_keycode(key)?;
    Ok((modifiers, keycode))
}
```

**2. Register Config Bindings:**
```rust
// In Terminal::new()
if !config.keybindings.new_tab.is_empty() {
    let (mods, key) = parse_key_combination(&config.keybindings.new_tab)?;
    keybindings.add_binding_from_parts(&key, &mods, Action::NewTab);
}
// Repeat for all bindings...
```

### Files to Modify:
- `src/keybindings.rs`: Add `parse_key_combination()` and `add_binding_from_parts()`
- `src/terminal/mod.rs`: Register config bindings in `new()`

### Tests Required:
```rust
#[test]
fn test_key_combination_parsing() {
    // Test various combinations
    // Test invalid input handling
}
```

### Estimated Complexity: MEDIUM (3-4 hours)

---

## Implementation Priority

Given constraints, implement in this order:

1. **KeyBindings Integration** (3-4 hours) - Quick win, enables config customization
2. **Lua Output Filters** (3-4 hours) - Useful feature, moderate complexity
3. **Lua Custom Keybindings** (4-6 hours) - Extends keybinding system
4. **Text Selection** (6-8 hours) - Important UX feature
5. **GPU-only stubs for Background/Trails** (2-3 hours) - Document as GPU-only
6. **Lua Custom Widgets** (8-10 hours) - Most complex, lowest priority

## Total Estimated Time: 30-45 hours

This is a substantial engineering effort requiring multiple full work days. Each feature should be implemented, tested, and committed separately.

## Decision: GPU vs Ratatui

For visual features (background images, cursor trails), recommend:
- Implement as GPU-only features
- Add clear documentation
- Provide stub implementations that check `cfg(feature = "gpu")`
- Update README to explain GPU features

This maintains code quality while acknowledging ratatui's limitations as a TUI framework.
