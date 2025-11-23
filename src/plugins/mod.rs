// Plugin system for extensibility without memory leaks
pub mod loader;
pub mod api;

pub use loader::{PluginManager, LoadedPlugin};
pub use api::{PluginApi, PluginMetadata, PluginCommand, ScriptingApi, ExamplePlugin};

// Safe plugin system using Rust's type safety:
// - Dynamic loading with libloading
// - Safe FFI boundaries
// - Plugin lifecycle management
// - Zero-overhead when no plugins loaded
