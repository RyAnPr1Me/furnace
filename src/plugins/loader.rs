use libloading::{Library, Symbol};
use anyhow::{Context, Result};
use std::path::Path;
use std::collections::HashMap;

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
    #[allow(dead_code)] // Public API field
    pub name: String,
    #[allow(dead_code)] // Public API field
    pub version: String,
}

/// Plugin API trait that plugins must implement
#[allow(dead_code)] // Public API trait
pub trait Plugin {
    /// Get plugin name
    fn name(&self) -> &str;
    
    /// Get plugin version
    fn version(&self) -> &str;
    
    /// Initialize plugin
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
    #[allow(dead_code)] // Public API
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        unsafe {
            let lib = Library::new(path)
                .context("Failed to load plugin library")?;
            
            let constructor: Symbol<PluginCreate> = lib
                .get(b"_plugin_create")
                .context("Failed to find plugin constructor")?;
            
            let plugin = constructor();
            let plugin_ref = &*plugin;
            
            let name = plugin_ref.name().to_string();
            let version = plugin_ref.version().to_string();
            
            let loaded = LoadedPlugin {
                library: lib,
                name: name.clone(),
                version,
            };
            
            self.plugins.insert(name, loaded);
            
            Ok(())
        }
    }

    /// Unload a plugin
    #[allow(dead_code)] // Public API
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        self.plugins.remove(name)
            .context("Plugin not found")?;
        Ok(())
    }

    /// Get list of loaded plugins
    #[allow(dead_code)] // Public API
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Check if plugin is loaded
    #[allow(dead_code)] // Public API
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
