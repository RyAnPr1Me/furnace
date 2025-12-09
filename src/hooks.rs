//! Lua hooks system for custom functionality
//!
//! Executes user-defined Lua scripts at various points in the terminal lifecycle.

use anyhow::Result;
use mlua::Lua;
use tracing::{debug, warn};

/// Lua hooks executor
pub struct HooksExecutor {
    lua: Lua,
}

impl HooksExecutor {
    /// Create a new hooks executor
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        
        // Set up a safe Lua environment
        // Disable potentially dangerous functions
        lua.load(r#"
            -- Disable dangerous functions
            os.execute = nil
            os.exit = nil
            io.popen = nil
            loadfile = nil
            dofile = nil
        "#).exec()?;
        
        Ok(Self { lua })
    }

    /// Execute a Lua hook script
    ///
    /// # Arguments
    /// * `script` - Lua code to execute
    /// * `context` - Context data to pass to the script
    pub fn execute(&self, script: &str, context: &str) -> Result<()> {
        if script.is_empty() {
            return Ok(());
        }

        // Create a table with context
        self.lua.load(format!(
            r#"
            local context = "{}"
            {}
            "#,
            context.replace('"', r#"\""#),
            script
        )).exec().map_err(|e| {
            warn!("Lua hook execution failed: {}", e);
            anyhow::anyhow!("Lua hook error: {}", e)
        })?;

        debug!("Executed Lua hook successfully");
        Ok(())
    }

    /// Execute startup hook
    pub fn on_startup(&self, script: &str) -> Result<()> {
        self.execute(script, "startup")
    }

    /// Execute shutdown hook
    pub fn on_shutdown(&self, script: &str) -> Result<()> {
        self.execute(script, "shutdown")
    }

    /// Execute key press hook
    pub fn on_key_press(&self, script: &str, key: &str) -> Result<()> {
        self.execute(script, &format!("key_press:{}", key))
    }

    /// Execute command start hook
    pub fn on_command_start(&self, script: &str, command: &str) -> Result<()> {
        self.execute(script, &format!("command_start:{}", command))
    }

    /// Execute command end hook
    pub fn on_command_end(&self, script: &str, command: &str, exit_code: i32) -> Result<()> {
        self.execute(script, &format!("command_end:{}:{}", command, exit_code))
    }

    /// Execute output hook
    pub fn on_output(&self, script: &str, output: &str) -> Result<()> {
        // Limit output to prevent performance issues
        let limited_output = if output.len() > 1000 {
            &output[..1000]
        } else {
            output
        };
        self.execute(script, &format!("output:{}", limited_output))
    }

    /// Execute bell hook
    pub fn on_bell(&self, script: &str) -> Result<()> {
        self.execute(script, "bell")
    }

    /// Execute title change hook
    pub fn on_title_change(&self, script: &str, title: &str) -> Result<()> {
        self.execute(script, &format!("title_change:{}", title))
    }
}

impl Default for HooksExecutor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!("Failed to create Lua hooks executor: {}", e);
            // Create a dummy executor that will fail gracefully
            Self {
                lua: Lua::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_executor_creation() {
        let executor = HooksExecutor::new();
        assert!(executor.is_ok());
    }

    #[test]
    fn test_simple_lua_execution() {
        let executor = HooksExecutor::new().unwrap();
        let result = executor.execute("local x = 1 + 1", "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dangerous_functions_disabled() {
        let executor = HooksExecutor::new().unwrap();
        // This should fail because os.execute is disabled
        let result = executor.execute("os.execute('ls')", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_startup_hook() {
        let executor = HooksExecutor::new().unwrap();
        let script = "print('Starting up!')";
        let result = executor.on_startup(script);
        assert!(result.is_ok());
    }
}
