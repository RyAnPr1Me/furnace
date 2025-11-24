// Plugin system for extensibility without memory leaks
pub mod loader;
pub mod api;

#[allow(unused_imports)] // Public API exports
pub use loader::{PluginManager, LoadedPlugin};
#[allow(unused_imports)] // Public API exports
pub use api::{PluginApi, PluginMetadata, PluginCommand, ScriptingApi, ExamplePlugin};

// Safe plugin system using Rust's type safety:
// - Dynamic loading with libloading
// - Safe FFI boundaries
// - Plugin lifecycle management
// - Zero-overhead when no plugins loaded
