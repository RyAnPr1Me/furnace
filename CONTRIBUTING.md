# Contributing to Furnace

Thank you for your interest in contributing to Furnace! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful and professional. We're all here to build something great together.

## Getting Started

### Prerequisites
- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs))
- Git
- A terminal emulator for testing

### Development Setup

```bash
# Clone the repository
git clone https://github.com/RyAnPr1Me/furnace.git
cd furnace

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

## Development Guidelines

### Rust Style Guide

We follow the official Rust style guide. Key points:

1. **Formatting**: Use `cargo fmt` before committing
2. **Linting**: Run `cargo clippy` and fix all warnings
3. **Naming**: Use `snake_case` for functions/variables, `PascalCase` for types
4. **Documentation**: Add doc comments for public APIs

### Code Quality

#### Required Before Committing
```bash
# Format code
cargo fmt

# Check for errors
cargo check

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```

#### Performance Considerations

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

#### Memory Safety

1. **Minimize `unsafe` code**: Only use when absolutely necessary
2. **Document safety invariants**: Explain why unsafe code is safe
3. **Test thoroughly**: Add tests for edge cases
4. **Use safe abstractions**: Prefer safe wrappers

### Testing

#### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let result = function_to_test();
        assert_eq!(result, expected_value);
    }
}
```

#### Test Coverage
- Add unit tests for all public functions
- Add integration tests for features
- Add benchmarks for performance-critical code

#### Running Tests
```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Benchmarks
cargo bench
```

### Documentation

#### Code Documentation
```rust
/// Brief description of the function.
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// let result = my_function(arg1, arg2);
/// assert_eq!(result, expected);
/// ```
pub fn my_function(arg1: Type1, arg2: Type2) -> ReturnType {
    // Implementation
}
```

#### Documentation Generation
```bash
cargo doc --open
```

## Pull Request Process

### Before Submitting

1. **Create an issue**: Discuss major changes before implementing
2. **Branch naming**: Use descriptive names like `feature/tab-support` or `fix/memory-leak`
3. **Commits**: Write clear commit messages
4. **Tests**: Ensure all tests pass
5. **Documentation**: Update docs for API changes

### Commit Messages

Follow conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Adding tests
- `chore`: Maintenance

Example:
```
feat(terminal): add split pane support

Implement horizontal and vertical split panes with configurable ratios.
Add keybindings for splitting and navigating between panes.

Closes #123
```

### Pull Request Template

```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- List of changes
- Another change

## Testing
How was this tested?

## Checklist
- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] Documentation updated
- [ ] No compiler warnings
- [ ] Benchmarks run (if performance-related)
```

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## Performance

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench bench_name

# With profiling
cargo bench --features "profiling"
```

### Profiling

For CPU profiling:
```bash
cargo build --release
perf record ./target/release/furnace
perf report
```

For memory profiling:
```bash
cargo build --release
valgrind --tool=massif ./target/release/furnace
```

## Debugging

### Debug Build
```bash
cargo build
cargo run
```

### With Logging
```bash
RUST_LOG=debug cargo run
```

### With GDB
```bash
cargo build
gdb ./target/debug/furnace
```

## Common Issues

### Build Failures

**Issue**: Missing dependencies
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

**Issue**: Clippy warnings
```bash
# Fix automatically (when possible)
cargo clippy --fix

# Allow specific warnings (use sparingly)
#[allow(clippy::warning_name)]
```

### Test Failures

**Issue**: Tests fail on your machine
```bash
# Check Rust version
rustc --version

# Update Rust
rustup update

# Clean test artifacts
cargo clean
cargo test
```

## Resources

### Rust Learning
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

### Terminal Development
- [Ratatui](https://ratatui.rs/)
- [Crossterm](https://github.com/crossterm-rs/crossterm)
- [PTY Docs](https://github.com/wez/wezterm/tree/main/pty)

### Performance
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
