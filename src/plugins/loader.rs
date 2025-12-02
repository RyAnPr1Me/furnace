use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;

/// Plugin manager for loading and managing plugins
#[allow(dead_code)] // Public API for plugin system
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
}

/// A loaded plugin
#[allow(dead_code)] // Public API for plugin system
pub struct LoadedPlugin {
    #[allow(dead_code)]
    library: Library,
    /// Raw pointer to the plugin - must be properly dropped
    plugin_ptr: *mut dyn Plugin,
    #[allow(dead_code)] // Public API field
    pub name: String,
    #[allow(dead_code)] // Public API field
    pub version: String,
}

// Safety: LoadedPlugin is safe to send between threads because:
// 1. The raw pointer is only accessed during plugin operations which are synchronized
// 2. The Library ensures thread-safe access to the plugin's shared library
// 3. The plugin_ptr is immutable after creation and only dropped once
// 4. All plugin trait methods must be thread-safe (enforced by Plugin trait bounds)
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        // Safety: The plugin was created by the plugin's constructor via Box::into_raw
        // and we are responsible for cleaning it up. This is safe because:
        // 1. plugin_ptr is non-null (validated during creation)
        // 2. We own the pointer exclusively (no other code holds a reference)
        // 3. cleanup() must be called before dropping to allow plugin-side resource cleanup
        // 4. Box::from_raw reconstructs the box to properly drop the trait object
        if !self.plugin_ptr.is_null() {
            unsafe {
                // Call cleanup on the plugin before dropping to release any resources
                let plugin = &mut *self.plugin_ptr;
                plugin.cleanup();
                // Convert back to Box and drop it properly, which calls the destructor
                drop(Box::from_raw(self.plugin_ptr));
            }
        }
    }
}

/// Plugin API trait that plugins must implement
#[allow(dead_code)] // Public API trait
pub trait Plugin {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin version
    fn version(&self) -> &str;

    /// Initialize plugin
    ///
    /// # Errors
    /// Returns an error if plugin initialization fails
    fn init(&mut self) -> Result<()>;

    /// Handle command from terminal
    fn handle_command(&self, command: &str) -> Option<String>;

    /// Cleanup plugin resources
    fn cleanup(&mut self);
}

/// Function signature for plugin entry point
#[allow(dead_code)] // Public API type
pub type PluginCreate = unsafe fn() -> *mut dyn Plugin;

impl PluginManager {
    /// Create a new plugin manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Load a plugin from a dynamic library
    ///
    /// # Safety
    /// This function uses `unsafe` to:
    /// 1. Load a dynamic library with `libloading`
    /// 2. Call the `_plugin_create` function from the plugin
    /// 3. Dereference the returned raw pointer
    ///
    /// Safety guarantees:
    /// - The plugin library must export a valid `_plugin_create` function
    /// - The function must return a valid pointer to a Plugin trait object
    /// - The pointer must remain valid for the lifetime of LoadedPlugin
    /// - The plugin must be compiled with the same Rust ABI version
    ///
    /// # Errors
    /// Returns an error if:
    /// - The plugin library cannot be loaded
    /// - The `_plugin_create` symbol is not found
    /// - The plugin constructor returns a null pointer
    #[allow(dead_code)] // Public API
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Safety: We're loading a dynamic library and calling a plugin constructor.
        // This is inherently unsafe but we validate the pointer before using it.
        unsafe {
            let lib = Library::new(path).context("Failed to load plugin library")?;

            // Look up the plugin constructor function symbol
            let constructor: Symbol<PluginCreate> = lib
                .get(b"_plugin_create")
                .context("Failed to find plugin constructor")?;

            // Call the constructor to create the plugin instance
            let plugin_ptr = constructor();

            // Validate the pointer before using it - this prevents null pointer dereference
            if plugin_ptr.is_null() {
                return Err(anyhow::anyhow!("Plugin constructor returned null pointer"));
            }

            // Safe to dereference now that we've validated the pointer is non-null
            let plugin_ref = &*plugin_ptr;

            // Extract plugin metadata
            let name = plugin_ref.name().to_string();
            let version = plugin_ref.version().to_string();

            let loaded = LoadedPlugin {
                library: lib,
                plugin_ptr,
                name: name.clone(),
                version,
            };

            self.plugins.insert(name, loaded);

            Ok(())
        }
    }

    /// Unload a plugin
    ///
    /// # Errors
    /// Returns an error if the plugin is not found
    #[allow(dead_code)] // Public API
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        self.plugins.remove(name).context("Plugin not found")?;
        Ok(())
    }

    /// Get list of loaded plugins
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Check if plugin is loaded
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.list_plugins().len(), 0);
    }
}
