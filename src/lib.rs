//! Furnace - A high-performance terminal emulator
//!
//! This library provides the core functionality for the Furnace terminal emulator,
//! including terminal rendering, shell integration, and various UI components.
//!
//! # Modules
//!
//! - [`config`]: Configuration management and serialization
//! - [`terminal`]: Main terminal logic and event loop
//! - [`shell`]: PTY and shell session management
//! - [`ui`]: UI components (command palette, resource monitor, themes)
//! - [`session`]: Session save/restore functionality
//! - [`keybindings`]: Keyboard shortcut handling
//! - [`colors`]: 24-bit true color support
//! - [`progress_bar`]: Command execution progress tracking
//! - [`gpu`]: GPU-accelerated rendering (optional, requires `gpu` feature)

pub mod colors;
pub mod config;
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
