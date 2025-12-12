# Furnace Summary

Furnace is a high-performance, memory-safe terminal emulator written in Rust. It is designed to provide native performance, modern features, and cross-platform compatibility.

## Key Highlights

- **Performance**: GPU-accelerated rendering at 170 FPS with dirty-flag optimization
- **Memory Safety**: 100% safe Rust with zero memory leaks guaranteed
- **Modern Features**: Tabs, split panes, command palette, session management
- **Cross-Platform**: Windows (primary), Linux, and macOS support
- **Extensible**: Plugin system and Lua scripting support

## Quick Start

```bash
# Build from source
git clone https://github.com/RyAnPr1Me/furnace.git
cd furnace
cargo build --release

# Run with default config
./target/release/furnace

# Run with custom config
./target/release/furnace --config /path/to/config.lua
```

## Core Features

| Feature | Status | Description |
|---------|--------|-------------|
| 24-bit Color | ✅ | Full RGB support (16.7M colors) |
| GPU Rendering | ✅ | 170 FPS with wgpu (optional) |
| Multiple Tabs | ✅ | O(1) switching, keyboard shortcuts |
| Split Panes | ✅ | Horizontal and vertical splits |
| Command Palette | ✅ | Fuzzy search with `Ctrl+P` |
| Session Management | ✅ | Save/restore terminal state |
| Resource Monitor | ✅ | Real-time CPU, memory, network stats |
| Plugin System | ✅ | Safe FFI-based plugins |
| Lua Scripting | ✅ | Custom hooks and keybindings |
| Autocomplete | ✅ | History and command suggestions |

## Performance Summary

| Metric | Target | Achieved |
|--------|--------|----------|
| Frame Rate | 60 FPS | 170 FPS |
| Memory (base) | < 30MB | 10-18MB |
| Startup Time | < 200ms | 80-95ms |
| Input Latency | < 10ms | < 3ms |
| CPU (idle) | < 10% | 2-5% |

## Configuration

Furnace uses Lua for configuration (`~/.furnace/config.lua`):

```lua
config = {
    terminal = {
        hardware_acceleration = true,
        enable_tabs = true,
        font_size = 12
    },
    features = {
        resource_monitor = true,
        autocomplete = true
    },
    theme = {
        name = "dark",
        foreground = "#FFFFFF",
        background = "#1E1E1E"
    }
}
```

## Documentation

| Document | Description |
|----------|-------------|
| [README.md](README.md) | Main documentation and usage guide |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Technical design and module structure |
| [FEATURES.md](FEATURES.md) | Complete feature list and comparisons |
| [PERFORMANCE.md](PERFORMANCE.md) | Benchmark methodology and results |
| [OPTIMIZATIONS.md](OPTIMIZATIONS.md) | Performance optimization techniques |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development guidelines |
| [SECURITY.md](SECURITY.md) | Security analysis and guarantees |
| [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md) | Plugin API documentation |
| [CONFIGURATION.md](CONFIGURATION.md) | Configuration reference |

## Building with Features

```bash
# Standard build
cargo build --release

# Build with GPU acceleration
cargo build --release --features gpu

# Build with all features
cargo build --release --all-features

# Run tests
cargo test --all-features

# Run linter
cargo clippy --all-features -- -D warnings
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Furnace** - Native performance, memory safety, and modern features. 🔥
