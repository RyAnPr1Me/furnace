# Furnace - Summary

## Overview

Furnace is a high-performance, cross-platform terminal emulator written in Rust. It prioritizes native performance, memory safety, and modern features while maintaining compatibility with standard terminal protocols.

## Quick Reference

### Key Features

| Category | Features |
|----------|----------|
| **Performance** | 170 FPS rendering, <100ms startup, <5% idle CPU |
| **Memory** | 10-20MB base, zero leaks (Rust guarantees) |
| **Colors** | 24-bit true color, 256-color palette, themes |
| **Windows** | Tabs, split panes, dynamic resizing |
| **Shell** | PTY sessions, directory tracking, history |
| **Config** | Lua scripting, YAML themes, custom keybindings |
| **Optional** | GPU acceleration, resource monitor, autocomplete |

### Quick Start

```bash
# Install
git clone https://github.com/RyAnPr1Me/furnace.git
cd furnace
cargo build --release --all-features

# Run
./target/release/furnace

# With options
furnace --config ~/.furnace/config.lua
furnace --shell /bin/zsh
furnace --debug
```

### Default Keybindings

| Action | Keys |
|--------|------|
| New Tab | `Ctrl+T` |
| Close Tab | `Ctrl+W` |
| Next Tab | `Ctrl+Tab` |
| Previous Tab | `Ctrl+Shift+Tab` |
| Split Horizontal | `Ctrl+Shift+H` |
| Split Vertical | `Ctrl+Shift+V` |
| Focus Next Pane | `Ctrl+O` |
| Copy | `Ctrl+Shift+C` |
| Paste | `Ctrl+Shift+V` |
| Search | `Ctrl+F` |
| Clear | `Ctrl+L` |
| Resource Monitor | `Ctrl+R` |
| Save Session | `Ctrl+S` |
| Load Session | `Ctrl+Shift+L` |
| Next Theme | `Ctrl+]` |
| Previous Theme | `Ctrl+[` |

### Configuration

Default config location: `~/.furnace/config.lua`

```lua
config = {
    shell = {
        default_shell = "/bin/bash",
    },
    terminal = {
        enable_tabs = true,
        enable_split_pane = false,
        scrollback_lines = 10000,
        hardware_acceleration = true,
    },
    features = {
        resource_monitor = true,
        autocomplete = true,
        session_manager = true,
    },
    theme = {
        name = "default",
        foreground = "#FFFFFF",
        background = "#1E1E1E",
    },
}
```

### Build Options

```bash
# Standard build
cargo build --release

# With GPU acceleration
cargo build --release --features gpu

# With all features
cargo build --release --all-features

# Development build
cargo build
```

### Project Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library exports
├── config/           # Configuration (Lua, YAML)
├── terminal/         # Core terminal logic
├── shell/            # PTY management
├── ui/               # UI components
├── gpu/              # GPU rendering (optional)
├── session.rs        # Session save/restore
├── keybindings.rs    # Keybinding system
├── colors.rs         # True color support
└── progress_bar.rs   # Progress indicators
```

### Documentation Index

| Document | Description |
|----------|-------------|
| [README.md](README.md) | Main documentation and usage |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Technical design details |
| [FEATURES.md](FEATURES.md) | Complete feature list |
| [PERFORMANCE.md](PERFORMANCE.md) | Benchmarks and comparisons |
| [OPTIMIZATIONS.md](OPTIMIZATIONS.md) | Performance optimization techniques |
| [CONFIGURATION.md](CONFIGURATION.md) | Configuration reference |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development guide |
| [SECURITY.md](SECURITY.md) | Security analysis |
| [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md) | Plugin API guide |

### Requirements

- **Rust**: 1.70+ (MSRV)
- **Platform**: Windows, Linux, macOS
- **Optional**: GPU with Vulkan/Metal/DX12 support (for GPU feature)

### License

MIT License - See [LICENSE](LICENSE) for details.

---

**Furnace** - A high-performance terminal emulator for the modern developer. 🔥
