/// Plugin API definitions
use anyhow::Result;

/// Plugin API trait that all plugins must implement
pub trait PluginApi {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize the plugin
    fn initialize(&mut self) -> Result<()>;
    
    /// Execute a command
    fn execute(&self, command: &str, args: &[&str]) -> Result<String>;
    
    /// Get available commands
    fn commands(&self) -> Vec<PluginCommand>;
    
    /// Shutdown the plugin
    fn shutdown(&mut self);
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

/// Plugin command definition
#[derive(Debug, Clone)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
}

/// Scripting API for plugins
pub trait ScriptingApi {
    /// Evaluate a script
    fn eval(&self, script: &str) -> Result<String>;
    
    /// Load a script file
    fn load_script(&self, path: &str) -> Result<()>;
    
    /// Get available script functions
    fn functions(&self) -> Vec<String>;
}

/// Example plugin implementation
pub struct ExamplePlugin {
    initialized: bool,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl PluginApi for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "example".to_string(),
            version: "1.0.0".to_string(),
            author: "Furnace".to_string(),
            description: "Example plugin for demonstration".to_string(),
        }
    }
    
    fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[&str]) -> Result<String> {
        match command {
            "hello" => Ok(format!("Hello, {}!", args.get(0).unwrap_or(&"World"))),
            _ => Ok("Unknown command".to_string()),
        }
    }
    
    fn commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "hello".to_string(),
                description: "Say hello".to_string(),
                usage: "hello [name]".to_string(),
            }
        ]
    }
    
    fn shutdown(&mut self) {
        self.initialized = false;
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}
