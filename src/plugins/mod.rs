// Plugin system for extensibility without memory leaks
pub mod api;
pub mod loader;

#[allow(unused_imports)] // Public API exports
pub use api::{ExamplePlugin, PluginApi, PluginCommand, PluginMetadata, ScriptingApi};
#[allow(unused_imports)] // Public API exports
pub use loader::{LoadedPlugin, PluginManager};

// Safe plugin system using Rust's type safety:
// - Dynamic loading with libloading
// - Safe FFI boundaries
// - Plugin lifecycle management
// - Zero-overhead when no plugins loaded
