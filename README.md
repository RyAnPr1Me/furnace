# Furnace 🔥

Furnace is a cross-platform terminal emulator written in Rust. It combines Crossterm + Ratatui for text rendering, portable-pty for shell sessions, and Tokio for async I/O. Configuration is driven by Lua so you can script behaviour, keybindings, and UI tweaks without rebuilding.

## Features

- Cross-platform PTY shell sessions (Windows, Linux, macOS) with async read/write.
- Lua configuration (`~/.furnace/config.lua` by default or `--config`) with lifecycle hooks (`on_startup`, `on_shutdown`, `on_key_press`, `on_command_start`, `on_command_end`, `on_output`, `on_bell`, `on_title_change`), output filters, custom keybindings, and custom widgets.
- 24-bit color pipeline with ANSI parsing and themeable palettes.
- Tabs for multiple sessions and optional split panes when `terminal.enable_split_pane` is enabled.
- Optional GPU rendering via `wgpu` when built with `--features gpu` and `terminal.hardware_acceleration` enabled (falls back to CPU if unavailable at runtime).
- Optional modules (disabled by default; enable via `features.*` in config):
  - Resource monitor (Ctrl+R) powered by `sysinfo`.
  - Autocomplete suggestions sourced from history and common commands.
  - Progress bar for long-running commands.
  - Session manager to save/restore sessions.
  - Theme manager to cycle bundled themes.
- Clipboard copy/paste, search mode, configurable cursor styles and font sizing metadata, and scrollback/history limits.
- Background color overlays and cursor trail effects from the theme configuration.

### Current defaults

- Tabs: disabled by default (`terminal.enable_tabs = false`)
- Split panes: disabled by default (`terminal.enable_split_pane = false`)
- Hardware acceleration: enabled by default; automatically falls back to CPU when GPU support is unavailable at build time or runtime
- Optional UI modules: disabled until explicitly enabled in config (`features.*`)

### Not yet implemented

- Command palette (keybinding placeholders are intentionally omitted)

## Installation

### Prerequisites
- Rust 1.70+ MSRV (install via [rustup.rs](https://rustup.rs))
- A terminal on Windows, Linux, or macOS

### Build from source

```bash
git clone https://github.com/RyAnPr1Me/furnace.git
cd furnace
cargo build --release
./target/release/furnace
```

## Quick start

```bash
furnace                     # Run with default config
furnace --config /path/to/config.lua
furnace --debug             # Enable debug logging to stderr
furnace --shell /bin/bash   # Override the detected shell
```

## Hardware acceleration

- Build with GPU support: `cargo build --release --features gpu`
- Runtime toggle: `terminal.hardware_acceleration = true` (default)
- Fallback: If the binary is built without `--features gpu` or no compatible GPU is detected, Furnace automatically uses CPU rendering and logs a warning when hardware acceleration is requested.

## Configuration

Furnace looks for `~/.furnace/config.lua` by default. The loader executes a Lua file that defines a global `config` table; any fields you omit fall back to built-in defaults. The YAML example is for reference only—the runtime only loads Lua. All optional UI modules are disabled until you opt in.

### Basic example

```lua
config = {
    shell = {
        -- On Windows set this to a PowerShell executable in your PATH (e.g. "pwsh", "pwsh.exe", or "powershell.exe")
        default_shell = "/bin/bash",
        working_dir = nil,
        env = {}
    },

    terminal = {
        max_history = 10000,
        enable_tabs = true,
        enable_split_pane = false,
        font_size = 12,
        cursor_style = "block",
        scrollback_lines = 10000,
        hardware_acceleration = true -- defaults to GPU when built with `--features gpu`, automatically falls back to CPU otherwise
    },

    features = {
        resource_monitor = true,
        autocomplete = true,
        progress_bar = true,
        session_manager = true,
        theme_manager = true
    },

    theme = {
        name = "default",
        foreground = "#FFFFFF",
        background = "#1E1E1E",
        cursor = "#00FF00",
        selection = "#264F78"
    },

    keybindings = {
        new_tab = "Ctrl+T",
        close_tab = "Ctrl+W",
        copy = "Ctrl+Shift+C",
        paste = "Ctrl+Shift+V",
        search = "Ctrl+F",
        clear = "Ctrl+L"
    },

    hooks = {
        on_startup = "print('Starting up!')",
        output_filters = {"function(text) return text:gsub('ERROR', '🔴 ERROR') end"},
        custom_keybindings = {
            ["Ctrl+Shift+G"] = "function() print(os.date()) end"
        }
    }
}
```

The example above enables most optional modules for demonstration. See the reference below for the exact defaults applied when you omit fields.

### Hooks and scripting

Lua hooks let you extend Furnace without plugins. Example output filter and notification inside your config table:

```lua
config = {
    hooks = {
        on_command_end = [[
            if exit_code ~= 0 then
                print("Command failed: " .. command)
            end
        ]],
        output_filters = {
            "function(text) return text:gsub('todo', 'TODO') end"
        }
    }
}
```

See `config.example.lua` for more options, including additional hook ideas, theme settings, and feature toggles.

### Configuration reference

- **File location**: `~/.furnace/config.lua` by default (override with `--config`).
- **Format**: Lua script that sets a global `config` table. The file is executed, so only load trusted configs.
- **Fallback behavior**: Missing sections or keys use the defaults listed below.

#### `shell` (defaults)
- `default_shell`: auto-detected. On Windows prefers `pwsh.exe`, then `powershell.exe`, else `cmd.exe`; on Unix uses `$SHELL` or `/bin/bash`.
- `working_dir`: `nil` (home directory).
- `env`: `{}` (no extra environment variables).

#### `terminal` (defaults)
- `max_history`: `10000`
- `enable_tabs`: `false`
- `enable_split_pane`: `false`
- `font_size`: `12`
- `cursor_style`: `"block"`
- `scrollback_lines`: `10000`
- `hardware_acceleration`: `true`

#### `theme` (defaults)
- `name`: `"default"`, `foreground`: `#FFFFFF`, `background`: `#1E1E1E`, `cursor`: `#00FF00`, `selection`: `#264F78`.
- `colors` (ANSI palette):
  - Normal: `black #000000`, `red #FF0000`, `green #00FF00`, `yellow #FFFF00`, `blue #0000FF`, `magenta #FF00FF`, `cyan #00FFFF`, `white #FFFFFF`
  - Bright: `bright_black #808080`, `bright_red #FF8080`, `bright_green #80FF80`, `bright_yellow #FFFF80`, `bright_blue #8080FF`, `bright_magenta #FF80FF`, `bright_cyan #80FFFF`, `bright_white #FFFFFF`
- Optional `background_image`: set `image_path` for an image and optionally `color` as the fallback if the image is missing or fails to load (setting only `color` uses a solid background; omitting both skips the section). Defaults when present: `opacity 1.0`, `mode "fill"` (`fill`/`fit`/`stretch`/`tile`/`center`), `blur 0.0`.
- Optional `cursor_trail`: enables a cursor effect (`enabled`/`length`/`color`/`fade_mode` of `linear`|`exponential`|`smooth`/`width`/`animation_speed`). Defaults when provided: `enabled false`, `length 10`, `color "#00FF0080"`, `fade_mode "exponential"`, `width 1.0`, `animation_speed 16`.

#### `keybindings` (defaults)
- `new_tab` `Ctrl+T`, `close_tab` `Ctrl+W`, `next_tab` `Ctrl+Tab`, `prev_tab` `Ctrl+Shift+Tab`
- `split_vertical` `Ctrl+Shift+V`, `split_horizontal` `Ctrl+Shift+H`
- `copy` `Ctrl+Shift+C`, `paste` `Ctrl+Shift+V`, `search` `Ctrl+F`, `clear` `Ctrl+L`
- Note: `split_vertical` conflicts with the default `paste` binding; rebind `split_vertical` (for example to `Ctrl+Shift+|` or `Ctrl+Alt+V`) if you enable split panes.

#### `features` (all default to `false`)
- `resource_monitor`, `autocomplete`, `progress_bar`, `session_manager`, `theme_manager`, `command_palette`

#### `hooks` (all optional)
- Lifecycle hooks: `on_startup`, `on_shutdown`, `on_key_press`, `on_command_start`, `on_command_end`, `on_output`, `on_bell`, `on_title_change`
- `custom_keybindings`: map of key -> Lua function string
- `output_filters`: list of Lua functions that transform output text
- `custom_widgets`: list of Lua snippets for extra UI elements

## Key bindings

| Action | Default Key | Notes |
|--------|-------------|-------|
| Resource Monitor | `Ctrl+R` | Requires `features.resource_monitor = true` |
| Toggle Autocomplete | `Alt+Tab` | Requires `features.autocomplete = true`; many desktops reserve Alt+Tab, so consider remapping in config |
| Next Theme | `Ctrl+]` | Requires `features.theme_manager = true` |
| Previous Theme | `Ctrl+[` | Requires `features.theme_manager = true` |
| Save Session | `Ctrl+S` | Requires `features.session_manager = true` |
| Load Session | `Ctrl+Shift+L` | Requires `features.session_manager = true` |
| New Tab | `Ctrl+T` | Requires `terminal.enable_tabs = true` |
| Close Tab | `Ctrl+W` | Requires `terminal.enable_tabs = true` |
| Next Tab | `Ctrl+Tab` | Requires `terminal.enable_tabs = true` |
| Previous Tab | `Ctrl+Shift+Tab` | Requires `terminal.enable_tabs = true` |
| Split Vertical | `Ctrl+Shift+V` | Requires `terminal.enable_split_pane = true`; overlaps with Paste by default, so rebind (e.g. `Ctrl+|`) if you need vertical splits |
| Split Horizontal | `Ctrl+Shift+H` | Requires `terminal.enable_split_pane = true` |
| Focus Next Pane | `Ctrl+O` | Requires split panes |
| Copy | `Ctrl+Shift+C` | |
| Paste | `Ctrl+Shift+V` | |
| Select All | `Ctrl+Shift+A` | |
| Search | `Ctrl+F` | |
| Search Next | `Ctrl+N` | |
| Search Previous | `Ctrl+Shift+N` | |
| Clear | `Ctrl+L` | |
| Quit | `Ctrl+C` or `Ctrl+D` | |

To resolve the default split-vertical/paste conflict or any other shortcut, override keybindings in your config:

```lua
keybindings = {
    split_vertical = "Ctrl+|",
    paste = "Ctrl+Shift+V"
}
```

## Architecture

```
src/
├── main.rs           # CLI entry (config path, debug flag, shell override)
├── config/           # Lua config parsing, defaults, theme and feature structs
├── terminal/         # Event loop, tabs, splits, rendering, search
├── shell/            # PTY management with portable-pty
├── ui/               # Autocomplete, resource monitor, theme manager
├── session.rs        # Session save/restore support
├── keybindings.rs    # Keybinding manager and default bindings
├── progress_bar.rs   # Long-running command indicator
└── colors.rs         # True color helpers and ANSI parsing
```

## Development

```bash
cargo build           # Debug build
cargo build --release # Release build
cargo test            # Run all tests
cargo check           # Type-check quickly
cargo fmt             # Format code
cargo clippy -- -D warnings # Lint
```

## Contributing

Contributions are welcome! Please ensure:
1. Code passes `cargo fmt` and `cargo clippy -- -D warnings`
2. All tests pass: `cargo test`
3. Add tests for new features
4. Update documentation when behavior changes

## License

MIT License - see LICENSE file for details.
