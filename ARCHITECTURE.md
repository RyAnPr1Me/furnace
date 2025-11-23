# Furnace Architecture

## Overview

Furnace is designed as a high-performance, memory-safe terminal emulator written in Rust. The architecture prioritizes:

1. **Native Performance**: Zero-cost abstractions and compile-time optimizations
2. **Memory Safety**: Rust's ownership system prevents leaks and data races
3. **Async I/O**: Non-blocking shell interaction for responsive UI
4. **Modularity**: Clean separation of concerns

## Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                         Main (CLI)                          │
│  - Argument parsing (clap)                                  │
│  - Configuration loading                                    │
│  - Application initialization                               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Terminal (Core)                          │
│  - Event loop management                                    │
│  - Session management                                       │
│  - UI rendering coordination                                │
│  - Input handling                                           │
└──────┬──────────────────────┬───────────────────┬───────────┘
       │                      │                   │
       ▼                      ▼                   ▼
┌─────────────┐      ┌─────────────┐    ┌─────────────┐
│   Shell     │      │     UI      │    │   Config    │
│  - PTY mgmt │      │  - Rendering│    │  - Settings │
│  - I/O      │      │  - Layouts  │    │  - Themes   │
│  - Process  │      │  - Panes    │    │  - Keys     │
└─────────────┘      └─────────────┘    └─────────────┘
```

## Module Breakdown

### 1. Main (`src/main.rs`)
- Entry point for the application
- CLI argument parsing using `clap`
- Configuration loading and validation
- Tokio runtime initialization
- Error handling and logging setup

### 2. Terminal (`src/terminal/mod.rs`)
**Responsibilities:**
- Main event loop using `tokio::select!`
- Multiple session management (tabs)
- Keyboard input processing
- Shell output buffering
- UI rendering coordination

**Performance Optimizations:**
- Async I/O for non-blocking shell interaction
- Circular buffers for output management
- 60 FPS rendering with vsync
- Zero-copy operations where possible

### 3. Shell (`src/shell/mod.rs`)
**Responsibilities:**
- PTY (Pseudo-Terminal) management
- Shell process spawning and lifecycle
- Non-blocking I/O with shell processes
- PTY resizing for terminal dimensions

**Key Features:**
- Cross-platform PTY abstraction using `portable-pty`
- Async read/write with optimized buffer sizes
- Automatic resource cleanup via RAII

### 4. Config (`src/config/mod.rs`)
**Responsibilities:**
- Configuration file parsing (YAML)
- Default configuration generation
- Theme management
- Keybinding configuration

**Design Decisions:**
- Serde for zero-copy deserialization
- Type-safe configuration structures
- Platform-specific defaults

### 5. UI (`src/ui/`)
**Responsibilities:**
- Terminal rendering using `ratatui`
- Split pane layout management
- Theme application
- Custom widgets

**Components:**
- `panes.rs`: Split pane layout calculation
- Future: `themes.rs`, `widgets.rs`

### 6. Plugins (`src/plugins/`)
**Responsibilities:**
- Safe plugin loading using `libloading`
- Plugin API definition
- Plugin lifecycle management

**Safety Guarantees:**
- FFI boundary validation
- Plugin isolation
- No unsafe memory access from plugins

## Data Flow

### Input Path
```
User Keystroke → Crossterm Event → Terminal Handler → Shell Write → PTY
```

### Output Path
```
Shell Process → PTY → Shell Read → Output Buffer → UI Render → Screen
```

## Memory Management

### Zero Leaks Guarantee
Rust's ownership system ensures:
- All heap allocations have a clear owner
- Resources are freed when owner goes out of scope (RAII)
- No manual memory management needed
- Compile-time prevention of use-after-free

### Buffer Management
- **Output Buffers**: Circular buffer with fixed max size
- **History**: Ring buffer for command history
- **Scrollback**: Memory-mapped for large buffers (future)

### Performance Profile
- **Startup**: < 100ms cold start
- **Memory**: ~10-20MB base + scrollback
- **CPU**: < 5% during idle, < 10% during rendering
- **Latency**: < 5ms input-to-shell

## Async Architecture

### Tokio Runtime
- Multi-threaded work-stealing scheduler
- Efficient task spawning
- Non-blocking I/O primitives

### Event Loop
```rust
tokio::select! {
    // User input (blocking task)
    _ = spawn_blocking(poll_input) => handle_input(),
    
    // Shell output (async I/O)
    _ = read_shell_output() => update_buffer(),
    
    // Rendering (interval timer)
    _ = render_tick() => draw_ui(),
}
```

## Build Profile

### Release Optimizations
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen for better optimization
strip = true            # Strip debug symbols
panic = "abort"         # Smaller binary, faster panics
```

## Testing Strategy

### Unit Tests
- Individual module functionality
- Configuration parsing
- Layout calculations

### Integration Tests
- End-to-end terminal creation
- Memory leak detection
- Performance benchmarks

### Benchmarks
- Output buffer throughput
- Scrollback management
- Memory allocation patterns

## Security Considerations

### Memory Safety
- No unsafe code in core (only in FFI boundaries)
- Bounds checking on all array access
- Type-safe configuration

### Shell Execution
- No shell command injection
- Environment isolation
- Proper signal handling

## Future Enhancements

### Planned Features
1. **GPU Acceleration**: Wgpu for rendering
2. **Advanced Plugins**: WebAssembly plugin runtime
3. **Session Persistence**: Save/restore sessions
4. **Remote Shells**: SSH integration
5. **Multiplexing**: Detachable sessions like tmux

### Performance Improvements
1. **Zero-copy rendering**: Direct buffer mapping
2. **Parallel rendering**: Multi-threaded UI updates
3. **Smart invalidation**: Only redraw changed regions
4. **Font caching**: Glyph atlas for faster text rendering

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## References

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Ratatui Docs](https://ratatui.rs/)
- [PTY Documentation](https://github.com/wez/wezterm/tree/main/pty)
