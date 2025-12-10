# Furnace 🔥

Furnace is a cross-platform terminal emulator written in Rust. It combines Crossterm + Ratatui for text rendering, portable-pty for shell sessions, and Tokio for async I/O. Configuration is driven by Lua so you can script behaviour, keybindings, and UI tweaks without rebuilding.

## Features

- Cross-platform PTY shell sessions (Windows, Linux, macOS) with async read/write.
- Lua configuration (`~/.furnace/config.lua` by default or `--config`) with lifecycle hooks (`on_startup`, `on_shutdown`, `on_key_press`, `on_command_start`, `on_command_end`, `on_output`, `on_bell`, `on_title_change`), output filters, custom keybindings, and custom widgets.
- 24-bit color pipeline with ANSI parsing and themeable palettes.
- Tabs for multiple sessions and optional split panes when `terminal.enable_split_pane` is enabled.
- Optional modules (disabled by default):
  - Resource monitor (Ctrl+R) powered by `sysinfo`.
  - Autocomplete suggestions sourced from history and common commands.
  - Progress bar for long-running commands.
  - Session manager to save/restore sessions.
  - Theme manager to cycle bundled themes.
- Clipboard copy/paste, search mode, configurable cursor styles and font sizing metadata, and scrollback/history limits.
- Background color overlays and cursor trail effects from the theme configuration.

### Not yet implemented

- GPU rendering (the `hardware_acceleration` flag is reserved for future work)
- Command palette (keybinding placeholders are intentionally omitted)

## Installation

### Prerequisites
- Rust 1.70+ (install via [rustup.rs](https://rustup.rs))
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

## Configuration

Furnace looks for `~/.furnace/config.lua` by default. All optional UI modules are disabled until you opt in.

### Basic example

```lua
config = {
    shell = {
        default_shell = "/bin/bash", -- on Windows use the PowerShell path in your PATH (e.g. "pwsh" / "pwsh.exe" or "powershell.exe")
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
        hardware_acceleration = false -- reserved for future GPU rendering
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
| Split Vertical | `Ctrl+Shift+V` | Requires `terminal.enable_split_pane = true`; overlaps with Paste by default—rebind if you need vertical splits |
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
