# Copilot Instructions for Furnace

## Overview

Furnace is a high-performance, memory-safe terminal emulator written in Rust. The project prioritizes native performance, memory safety, and async I/O through Tokio.

## Tech Stack

- **Language**: Rust 1.70+
- **Async Runtime**: Tokio
- **Terminal UI**: Ratatui + Crossterm
- **Configuration**: YAML (serde_yaml)
- **CLI Parsing**: Clap

## Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build (optimized)

# Test
cargo test               # Run all tests
cargo test test_name     # Run specific test
cargo test -- --nocapture # Run with output

# Lint and Format
cargo fmt                # Format code
cargo clippy -- -D warnings  # Run linter (treat warnings as errors)
cargo check              # Quick syntax check

# Benchmarks
cargo bench              # Run performance benchmarks
```

## Code Style

- Follow the official Rust style guide
- Use `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` and fix all warnings
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and structs
- Add doc comments (`///`) for all public APIs

## Architecture

The project follows a modular architecture:

```
src/
├── main.rs           # Entry point with CLI parsing
├── lib.rs            # Library exports
├── config/           # Configuration management (YAML parsing)
├── terminal/         # Main terminal logic (async event loop)
├── shell/            # PTY and shell session management
├── ui/               # UI rendering (Ratatui-based)
├── plugins/          # Plugin system (safe FFI)
├── translator/       # Cross-platform command translation
├── ssh_manager/      # SSH connection manager
├── url_handler/      # URL detection and opening
├── session.rs        # Session save/restore
├── keybindings.rs    # Keybinding system
└── colors.rs         # 24-bit true color support
```

## Performance Guidelines

1. **Avoid allocations in hot paths**: Use stack allocation or pre-allocated buffers
2. **Prefer borrowing over cloning**: Use `&str` over `String`, `&[T]` over `Vec<T>`
3. **Use zero-cost abstractions**: Leverage Rust's type system
4. **Profile before optimizing**: Use `cargo bench` to measure

Example:
```rust
// ❌ Bad - unnecessary allocation
fn process_data(data: String) -> String {
    data.to_uppercase()
}

// ✅ Good - zero-copy
fn process_data(data: &str) -> String {
    data.to_uppercase()
}
```

## Memory Safety

1. Minimize `unsafe` code - only use when absolutely necessary
2. Document safety invariants with comments explaining why unsafe code is safe
3. Test thoroughly with edge cases
4. Use safe abstractions over raw pointers

## Testing Requirements

- Add unit tests for all public functions
- Add integration tests in the `tests/` directory for features
- Add benchmarks in `benches/` for performance-critical code
- Ensure all tests pass before submitting changes

## Commit Message Format

Follow conventional commits:

```
<type>(<scope>): <subject>

<body>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Example:
```
feat(terminal): add split pane support

Implement horizontal and vertical split panes with configurable ratios.
```

## Key Documentation

- [README.md](../README.md) - Project overview and usage
- [ARCHITECTURE.md](../ARCHITECTURE.md) - Detailed architecture documentation
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Full contribution guidelines
- [PLUGIN_DEVELOPMENT.md](../PLUGIN_DEVELOPMENT.md) - Plugin development guide

## Common Patterns

### Error Handling
Use `anyhow` for application errors and `thiserror` for library errors:
```rust
use anyhow::{Context, Result};

fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read config file")?;
    // ...
}
```

### Async Code
Use Tokio for async operations:
```rust
use tokio::select;

async fn event_loop() {
    loop {
        select! {
            input = read_input() => handle_input(input),
            output = read_shell() => handle_output(output),
        }
    }
}
```

### Configuration Structs
Use serde for serialization:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub shell: ShellConfig,
    pub terminal: TerminalConfig,
}
```
