# Furnace v1.0 - Major Simplification and Lua Configuration

## Summary
This update transforms Furnace into a more focused, highly extensible terminal emulator by removing complex integrations and introducing Lua-based configuration for extreme customization.

## Removed Features
- ❌ **Plugin System**: Removed all plugin infrastructure (loader, API, examples)
- ❌ **SSH Manager**: Removed built-in SSH connection management
- ❌ **Command Translator**: Removed automatic command translation between platforms
- ❌ **URL Handler**: Removed clickable URL functionality

## Changed Defaults
- 🔴 **Tabs**: Disabled by default (enable in config if needed)
- 🔴 **Split Panes**: Disabled by default (enable in config if needed)
- ✅ **GPU Acceleration**: Enabled by default (recommended for performance)

## New Features

### Lua Configuration System
Replaced YAML with Lua for extreme extensibility:
- Full Lua scripting support with standard library access
- Dynamic runtime configuration logic
- Conditional settings based on environment/time/OS
- Default location: `~/.furnace/config.lua`

### Background Customization
- 🖼️ Image backgrounds (PNG, JPEG support)
- 🎨 Configurable opacity (0.0-1.0)
- 🌫️ Blur effects for subtle backgrounds
- 📐 Display modes: fill, fit, stretch, tile, center
- ⏰ Dynamic switching based on custom logic

### Cursor Trail Effects
- ✨ Smooth visual feedback with animated trails
- 📏 Adjustable trail length (number of positions)
- 🎨 Custom colors with alpha channel support
- 📉 Multiple fade modes: linear, exponential, smooth
- ⚡ Performance-aware animation speed

## Migration Guide

### From YAML to Lua
**Old (config.yaml):**
```yaml
terminal:
  enable_tabs: true
  scrollback_lines: 10000
```

**New (config.lua):**
```lua
config = {
    terminal = {
        enable_tabs = true,
        scrollback_lines = 10000
    }
}
```

### Enabling Previously Default Features
To restore tabs and split panes:
```lua
config.terminal.enable_tabs = true
config.terminal.enable_split_pane = true
```

## Configuration Examples

### Time-Based Theme Switching
```lua
local hour = tonumber(os.date("%H"))
if hour >= 6 and hour < 18 then
    config.theme.background = "#FFFFFF"
else
    config.theme.background = "#1E1E1E"
end
```

### Custom Background
```lua
config.theme.background_image = {
    image_path = "~/.furnace/backgrounds/wallpaper.png",
    opacity = 0.3,
    mode = "fill",
    blur = 5.0
}
```

### Cursor Trail
```lua
config.theme.cursor_trail = {
    enabled = true,
    length = 15,
    color = "#00FFFF80",
    fade_mode = "smooth",
    animation_speed = 16
}
```

## Security Considerations
⚠️ **Warning**: Lua configuration files have full access to the Lua standard library, including file I/O and OS operations. Only load trusted configuration files.

## Performance
- GPU acceleration remains enabled by default
- All core terminal functionality maintained
- Simplified codebase reduces memory footprint
- Lua configuration adds minimal runtime overhead

## Documentation
- See `config.example.lua` for 9+ advanced configuration examples
- Updated README.md with all new features
- Comprehensive Lua scripting examples included

## Testing
All 71 tests pass:
- 46 library tests
- 18 functionality tests
- 7 integration tests
