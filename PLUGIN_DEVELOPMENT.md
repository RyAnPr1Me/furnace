# Plugin Development Guide

## Overview

Furnace provides a safe and powerful plugin system that allows you to extend the terminal's functionality using Rust's FFI with guaranteed memory safety.

## Plugin Architecture

Plugins in Furnace are:
- **Dynamic libraries** (.so, .dll, .dylib) loaded at runtime
- **Type-safe** with Rust's safety guarantees
- **Sandboxed** with clear API boundaries
- **Hot-reloadable** (can be loaded/unloaded without restarting)

## Creating a Plugin

### 1. Setup Your Plugin Project

```bash
cargo new --lib my_furnace_plugin
cd my_furnace_plugin
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my_furnace_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Important: creates dynamic library

[dependencies]
# Add any dependencies your plugin needs
```

### 3. Implement the Plugin API

```rust
use std::os::raw::c_char;
use std::ffi::CString;

#[repr(C)]
pub struct Plugin {
    name: *const c_char,
    version: *const c_char,
}

impl Plugin {
    fn new() -> Self {
        Plugin {
            name: CString::new("MyPlugin").unwrap().into_raw(),
            version: CString::new("0.1.0").unwrap().into_raw(),
        }
    }
    
    fn name(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(self.name)
                .to_str()
                .unwrap()
        }
    }
    
    fn version(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(self.version)
                .to_str()
                .unwrap()
        }
    }
    
    fn init(&mut self) {
        println!("Plugin {} initialized!", self.name());
    }
    
    fn handle_command(&self, command: &str) -> Option<String> {
        match command {
            "hello" => Some("Hello from plugin!".to_string()),
            _ => None,
        }
    }
    
    fn cleanup(&mut self) {
        println!("Plugin {} cleaned up!", self.name());
    }
}

// Export the plugin creation function
#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut Plugin {
    Box::into_raw(Box::new(Plugin::new()))
}
```

### 4. Build the Plugin

```bash
cargo build --release
```

The plugin will be in `target/release/libmy_furnace_plugin.so` (Linux), 
`.dll` (Windows), or `.dylib` (macOS).

## Plugin API Reference

### Core Traits

#### `PluginApi` Trait

```rust
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
```

#### `ScriptingApi` Trait

```rust
pub trait ScriptingApi {
    /// Evaluate a script
    fn eval(&self, script: &str) -> Result<String>;
    
    /// Load a script file
    fn load_script(&self, path: &str) -> Result<()>;
    
    /// Get available script functions
    fn functions(&self) -> Vec<String>;
}
```

### Data Structures

#### `PluginMetadata`

```rust
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}
```

#### `PluginCommand`

```rust
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
}
```

## Example: Git Integration Plugin

```rust
use anyhow::Result;

pub struct GitPlugin {
    initialized: bool,
}

impl GitPlugin {
    pub fn new() -> Self {
        Self { initialized: false }
    }
    
    fn git_status(&self) -> Result<String> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(&["status", "--short"])
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    fn git_branch(&self) -> Result<String> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl PluginApi for GitPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "git-integration".to_string(),
            version: "1.0.0".to_string(),
            author: "Your Name".to_string(),
            description: "Git integration for Furnace".to_string(),
        }
    }
    
    fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[&str]) -> Result<String> {
        match command {
            "git-status" => self.git_status(),
            "git-branch" => self.git_branch(),
            _ => Ok("Unknown command".to_string()),
        }
    }
    
    fn commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "git-status".to_string(),
                description: "Show git status".to_string(),
                usage: "git-status".to_string(),
            },
            PluginCommand {
                name: "git-branch".to_string(),
                description: "Show current branch".to_string(),
                usage: "git-branch".to_string(),
            },
        ]
    }
    
    fn shutdown(&mut self) {
        self.initialized = false;
    }
}
```

## Loading Plugins

### From Configuration

Add plugins to your `config.yaml`:

```yaml
plugins:
  - "~/.furnace/plugins/git-integration.so"
  - "~/.furnace/plugins/custom-commands.so"
```

### Programmatically

```rust
let mut plugin_manager = PluginManager::new();
plugin_manager.load_plugin("path/to/plugin.so")?;
```

### From Command Palette

Use the command palette (Ctrl+P) and type:
- `load-plugin /path/to/plugin.so`
- `unload-plugin plugin_name`
- `list-plugins`

## Scripting Support

Plugins can also provide scripting capabilities:

```rust
impl ScriptingApi for MyPlugin {
    fn eval(&self, script: &str) -> Result<String> {
        // Evaluate script in your scripting engine
        // Could be Lua, JavaScript, Python, etc.
        Ok("Script result".to_string())
    }
    
    fn load_script(&self, path: &str) -> Result<()> {
        // Load and execute script file
        Ok(())
    }
    
    fn functions(&self) -> Vec<String> {
        vec!["my_function".to_string(), "another_function".to_string()]
    }
}
```

## Best Practices

### 1. Memory Safety

- Always use Rust's safety features
- Avoid `unsafe` blocks unless absolutely necessary
- Properly clean up resources in `shutdown()`

### 2. Error Handling

- Return `Result` types for fallible operations
- Provide meaningful error messages
- Don't panic in plugin code

### 3. Performance

- Avoid blocking operations in command handlers
- Use async operations for I/O
- Cache expensive computations

### 4. API Stability

- Version your plugin API
- Document breaking changes
- Maintain backward compatibility when possible

## Testing Plugins

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.metadata().name, "my-plugin");
    }

    #[test]
    fn test_command_execution() {
        let plugin = MyPlugin::new();
        let result = plugin.execute("test", &[]).unwrap();
        assert!(!result.is_empty());
    }
}
```

## Debugging Plugins

Enable debug logging:

```bash
RUST_LOG=debug furnace
```

Plugin loading errors will be logged to help diagnose issues.

## Example Plugins

See the `examples/plugins/` directory for:
- `hello_world` - Basic plugin example
- `git_integration` - Git commands integration
- `weather` - Fetch weather information
- `custom_commands` - Add custom shell commands

## Security Considerations

1. **Trust**: Only load plugins from trusted sources
2. **Sandboxing**: Plugins run in the same process (future: WASM sandboxing)
3. **Permissions**: Plugins have full process access (be cautious)
4. **Code Review**: Review plugin source code before loading

## Future Enhancements

- WebAssembly plugin support for better sandboxing
- Plugin marketplace
- Hot-reload support
- Plugin dependency management
- Scripting language integration (Lua, JavaScript)

## Support

For plugin development questions:
- Open an issue on GitHub
- Check the API documentation
- See example plugins
