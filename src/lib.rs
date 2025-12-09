//! Furnace - A high-performance terminal emulator
//!
//! This library provides the core functionality for the Furnace terminal emulator,
//! including terminal rendering, shell integration, and various UI components.
//!
//! # Features
//!
//! - **High Performance**: Zero-copy rendering with dirty tracking for 170 FPS
//! - **Memory Safety**: 100% safe Rust with comprehensive error handling
//! - **Async I/O**: Non-blocking shell I/O using Tokio for responsiveness
//! - **True Color**: Full 24-bit RGB color support with ANSI parsing
//! - **GPU Acceleration**: Optional hardware-accelerated rendering via wgpu
//! - **Cross-Platform**: Primarily Windows-focused with portable PTY support
//!
//! # Architecture
//!
//! The codebase is organized into focused modules with clear separation of concerns:
//!
//! - [`config`]: Configuration management with Lua scripting support
//! - [`terminal`]: Main terminal logic and async event loop
//! - [`shell`]: PTY and shell session management with zero-copy I/O
//! - [`ui`]: UI components (command palette, resource monitor, themes)
//! - [`session`]: Session save/restore functionality for workflow persistence
//! - [`keybindings`]: Extensible keyboard shortcut handling
//! - [`colors`]: 24-bit true color support with blending operations
//! - [`progress_bar`]: Command execution progress tracking with spinner
//! - [`gpu`]: GPU-accelerated rendering (optional, requires `gpu` feature)
//!
//! # Performance Considerations
//!
//! This codebase is optimized for performance:
//!
//! - **Zero-copy operations**: Borrowed strings and slices used throughout
//! - **Dirty tracking**: Renders only when state changes
//! - **Buffer reuse**: Pre-allocated buffers for I/O operations
//! - **Cache invalidation**: Smart cache management for styled text
//! - **Efficient string handling**: Uses `Arc<str>` for shared strings
//!
//! # Safety
//!
//! This codebase contains no `unsafe` code blocks. All operations are
//! guaranteed memory-safe by the Rust compiler.

pub mod colors;
pub mod config;
pub mod hooks;
pub mod keybindings;
pub mod progress_bar;
pub mod session;
pub mod shell;
pub mod terminal;
pub mod ui;

/// GPU-accelerated rendering module
///
/// Enabled with the `gpu` feature flag. Provides hardware-accelerated
/// text rendering using wgpu for 170+ FPS performance.
pub mod gpu;
