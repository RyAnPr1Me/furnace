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
        lua.load(
            r#"
            -- Disable dangerous functions
            os.execute = nil
            os.exit = nil
            io.popen = nil
            loadfile = nil
            dofile = nil
        "#,
        )
        .exec()?;

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

        // Escape special characters for Lua string literal safety
        // This prevents script injection and ensures proper parsing
        // Handle all characters that have special meaning in Lua strings
        let escaped_context = context
            .replace('\\', r"\\") // Escape backslashes first (must be first)
            .replace('"', r#"\""#) // Escape double quotes
            .replace('\n', r"\n") // Escape newlines
            .replace('\r', r"\r") // Escape carriage returns
            .replace('\t', r"\t") // Escape tabs
            .replace('\x0B', r"\v") // Escape vertical tab
            .replace('\x0C', r"\f") // Escape form feed
            .replace('\0', r"\0"); // Escape null bytes

        // Create a table with context
        self.lua
            .load(format!(
                r#"
            local context = "{}"
            {}
            "#,
                escaped_context, script
            ))
            .exec()
            .map_err(|e| {
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

    /// Apply output filters to transform output text
    ///
    /// Filters are Lua functions that transform string input to string output.
    /// Each filter in the pipeline receives the output of the previous filter.
    ///
    /// # Arguments
    /// * `output` - Original output text
    /// * `filters` - Vector of Lua code strings, each should set `output` global
    ///
    /// # Returns
    /// Transformed output string, or original on error
    ///
    /// # Example Lua Filter
    /// ```lua
    /// -- Filter that converts to uppercase
    /// output = string.upper(input)
    /// ```
    pub fn apply_output_filters(&self, output: &str, filters: &[String]) -> Result<String> {
        if filters.is_empty() {
            return Ok(output.to_string());
        }

        let mut result = output.to_string();

        for (idx, filter) in filters.iter().enumerate() {
            if filter.trim().is_empty() {
                continue;
            }

            // Set input in Lua globals
            let globals = self.lua.globals();
            globals.set("input", result.clone())?;
            globals.set("output", result.clone())?; // Default: output = input

            // Execute the filter
            match self.lua.load(filter).exec() {
                Ok(()) => {
                    // Get the transformed output
                    match globals.get::<_, String>("output") {
                        Ok(transformed) => {
                            result = transformed;
                            debug!("Output filter {} applied successfully", idx);
                        }
                        Err(e) => {
                            warn!("Output filter {} didn't set output variable: {}", idx, e);
                            // Keep previous result
                        }
                    }
                }
                Err(e) => {
                    warn!("Output filter {} execution failed: {}", idx, e);
                    // Continue with current result, don't break the chain
                }
            }
        }

        Ok(result)
    }

    /// Execute custom keybinding Lua function
    ///
    /// # Arguments
    /// * `lua_code` - Lua function code to execute
    /// * `context` - Context data (cwd, last_command, etc.)
    pub fn execute_custom_keybinding(
        &self,
        lua_code: &str,
        cwd: &str,
        last_command: &str,
    ) -> Result<()> {
        if lua_code.trim().is_empty() {
            return Ok(());
        }

        // Set up context
        let globals = self.lua.globals();
        let ctx_table = self.lua.create_table()?;
        ctx_table.set("cwd", cwd)?;
        ctx_table.set("last_command", last_command)?;
        globals.set("context", ctx_table)?;

        // Execute Lua code
        self.lua.load(lua_code).exec().map_err(|e| {
            warn!("Custom keybinding execution failed: {}", e);
            anyhow::anyhow!("Keybinding error: {}", e)
        })?;

        debug!("Custom keybinding executed successfully");
        Ok(())
    }

    /// Execute custom widget Lua code and return widget specification
    ///
    /// Widget Lua code should set a global `widget` table with:
    /// - x, y: position
    /// - width, height: dimensions
    /// - content: array of strings (lines)
    /// - style: optional style (fg_color, bg_color, bold, etc.)
    ///
    /// # Example Lua Widget
    /// ```lua
    /// widget = {
    ///     x = 0,
    ///     y = 0,
    ///     width = 20,
    ///     height = 3,
    ///     content = {"Line 1", "Line 2", "Line 3"},
    ///     fg_color = "#00FF00",
    ///     bg_color = "#000000"
    /// }
    /// ```
    pub fn execute_widget(&self, lua_code: &str) -> Result<LuaWidget> {
        if lua_code.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty widget code"));
        }

        // Execute Lua code
        self.lua.load(lua_code).exec().map_err(|e| {
            warn!("Widget execution failed: {}", e);
            anyhow::anyhow!("Widget error: {}", e)
        })?;

        // Extract widget definition from globals
        let globals = self.lua.globals();
        let widget_table: mlua::Table = globals
            .get("widget")
            .map_err(|_| anyhow::anyhow!("Widget code must set 'widget' global table"))?;

        // Extract position and dimensions
        let x = widget_table.get::<_, u16>("x")?;
        let y = widget_table.get::<_, u16>("y")?;
        let width = widget_table.get::<_, u16>("width")?;
        let height = widget_table.get::<_, u16>("height")?;

        // Extract content
        let content_table: mlua::Table = widget_table.get("content")?;
        let mut content = Vec::new();
        for value in content_table.sequence_values::<String>() {
            content.push(value?);
        }

        // Extract optional style
        let fg_color = widget_table.get::<_, Option<String>>("fg_color")?;
        let bg_color = widget_table.get::<_, Option<String>>("bg_color")?;
        let bold = widget_table
            .get::<_, Option<bool>>("bold")?
            .unwrap_or(false);

        Ok(LuaWidget {
            x,
            y,
            width,
            height,
            content,
            fg_color,
            bg_color,
            bold,
        })
    }
}

/// Widget specification from Lua
#[derive(Debug, Clone)]
pub struct LuaWidget {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub content: Vec<String>,
    pub fg_color: Option<String>,
    pub bg_color: Option<String>,
    pub bold: bool,
}

impl Default for HooksExecutor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!("Failed to create Lua hooks executor: {}", e);
            // Create a dummy executor that will fail gracefully
            Self { lua: Lua::new() }
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

    #[test]
    fn test_output_filter_single() {
        let executor = HooksExecutor::new().unwrap();
        let filters = vec!["output = string.upper(input)".to_string()];
        let result = executor.apply_output_filters("hello world", &filters);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO WORLD");
    }

    #[test]
    fn test_output_filter_pipeline() {
        let executor = HooksExecutor::new().unwrap();
        let filters = vec![
            "output = string.upper(input)".to_string(),
            "output = string.gsub(output, 'WORLD', 'LUA')".to_string(),
        ];
        let result = executor.apply_output_filters("hello world", &filters);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO LUA");
    }

    #[test]
    fn test_output_filter_error_handling() {
        let executor = HooksExecutor::new().unwrap();
        let filters = vec![
            "output = string.upper(input)".to_string(),
            "this is invalid lua code!!!".to_string(), // This will fail
            "output = output .. '!'".to_string(), // This should still work with previous output
        ];
        let result = executor.apply_output_filters("hello", &filters);
        assert!(result.is_ok());
        // Should have uppercased from first filter, ignored invalid filter, added ! from third
        let output = result.unwrap();
        assert!(output.contains("HELLO"));
    }
}
