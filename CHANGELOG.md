# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CODE_OF_CONDUCT.md for community guidelines
- CHANGELOG.md for version tracking
- .editorconfig for consistent editor settings
- CI/CD workflow for automated testing
- Dependabot configuration for dependency updates
- Issue and PR templates

### Changed
- Fixed code formatting issues
- Improved inline documentation

## [1.0.0] - 2024-12-05

### Added
- Initial release of Furnace terminal emulator
- Native performance with Rust implementation
- GPU-accelerated rendering at 170 FPS
- 24-bit true color support
- Lua configuration system with runtime hooks
- Optional UI features (tabs, split panes, command palette, resource monitor)
- Session management for save/restore functionality
- Shell integration with directory tracking
- Enhanced keybindings system
- Custom themes support
- Progress bar for long-running commands
- Autocomplete with history
- Memory-safe architecture with zero unsafe code
- Async I/O with Tokio
- Cross-platform PTY support
- Comprehensive test suite (78+ tests)

### Performance
- 170 FPS rendering with dirty-flag optimization
- Idle CPU < 5%
- Memory usage: 10-18MB base
- Startup time: < 100ms
- 80% reduction in memory allocations through buffer reuse
- Zero-cost abstractions throughout

### Security
- 100% safe Rust code (no unsafe blocks)
- Memory safety guarantees from Rust compiler
- No data races through compile-time checks
- Comprehensive error handling with Result types

[Unreleased]: https://github.com/RyAnPr1Me/furnace/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/RyAnPr1Me/furnace/releases/tag/v1.0.0
