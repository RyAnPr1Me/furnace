use anyhow::{Context, Result};
use mlua::{Lua, Table};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure with zero-copy design for performance
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct Config {
    pub shell: ShellConfig,
    pub terminal: TerminalConfig,
    pub theme: ThemeConfig,
    pub keybindings: KeyBindings,
    pub features: FeaturesConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct HooksConfig {
    /// Lua script paths for various hooks
    pub on_startup: Option<String>,
    pub on_shutdown: Option<String>,
    pub on_key_press: Option<String>,
    pub on_command_start: Option<String>,
    pub on_command_end: Option<String>,
    pub on_output: Option<String>,
    pub on_bell: Option<String>,
    pub on_title_change: Option<String>,

    /// Custom keybinding handlers (key -> lua function string)
    pub custom_keybindings: HashMap<String, String>,

    /// Output filters (Lua functions that transform output)
    pub output_filters: Vec<String>,

    /// Custom widgets (Lua code for rendering custom UI)
    pub custom_widgets: Vec<String>,
}

impl HooksConfig {
    fn from_lua_table(table: &Table) -> Result<Self> {
        let on_startup = table.get::<_, Option<String>>("on_startup")?;
        let on_shutdown = table.get::<_, Option<String>>("on_shutdown")?;
        let on_key_press = table.get::<_, Option<String>>("on_key_press")?;
        let on_command_start = table.get::<_, Option<String>>("on_command_start")?;
        let on_command_end = table.get::<_, Option<String>>("on_command_end")?;
        let on_output = table.get::<_, Option<String>>("on_output")?;
        let on_bell = table.get::<_, Option<String>>("on_bell")?;
        let on_title_change = table.get::<_, Option<String>>("on_title_change")?;

        let custom_keybindings = if let Ok(kb_table) = table.get::<_, Table>("custom_keybindings") {
            let mut map = HashMap::new();
            for pair in kb_table.pairs::<String, String>() {
                let (key, func) = pair?;
                map.insert(key, func);
            }
            map
        } else {
            HashMap::new()
        };

        let output_filters = if let Ok(filters_table) = table.get::<_, Table>("output_filters") {
            let mut filters = Vec::new();
            for pair in filters_table.sequence_values::<String>() {
                filters.push(pair?);
            }
            filters
        } else {
            Vec::new()
        };

        let custom_widgets = if let Ok(widgets_table) = table.get::<_, Table>("custom_widgets") {
            let mut widgets = Vec::new();
            for pair in widgets_table.sequence_values::<String>() {
                widgets.push(pair?);
            }
            widgets
        } else {
            Vec::new()
        };

        Ok(Self {
            on_startup,
            on_shutdown,
            on_key_press,
            on_command_start,
            on_command_end,
            on_output,
            on_bell,
            on_title_change,
            custom_keybindings,
            output_filters,
            custom_widgets,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct ShellConfig {
    pub default_shell: String,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct TerminalConfig {
    /// Maximum command history entries (memory-efficient circular buffer)
    pub max_history: usize,

    /// Enable tabs for multiple sessions
    pub enable_tabs: bool,

    /// Enable split panes
    pub enable_split_pane: bool,

    /// Font size
    pub font_size: u16,

    /// Cursor style: block, underline, bar
    pub cursor_style: String,

    /// Number of scrollback lines (memory-mapped for large buffers)
    pub scrollback_lines: usize,

    /// Hardware acceleration for rendering
    pub hardware_acceleration: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct ThemeConfig {
    pub name: String,
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub selection: String,
    pub colors: AnsiColors,
    pub background_image: Option<BackgroundConfig>,
    pub cursor_trail: Option<CursorTrailConfig>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct BackgroundConfig {
    /// Path to background image file (supports PNG, JPEG, etc.)
    pub image_path: Option<String>,
    /// Solid color as fallback or alternative
    pub color: Option<String>,
    /// Opacity/transparency (0.0 = fully transparent, 1.0 = fully opaque)
    pub opacity: f32,
    /// How the image should be displayed: "fill", "fit", "stretch", "tile", "center"
    pub mode: String,
    /// Blur effect strength (0.0 = no blur, higher = more blur)
    pub blur: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct CursorTrailConfig {
    /// Enable cursor trail effect
    pub enabled: bool,
    /// Length of the trail (number of past positions to show)
    pub length: usize,
    /// Trail color (with alpha channel support like "#RRGGBBAA")
    pub color: String,
    /// Fade mode: "linear", "exponential", "smooth"
    pub fade_mode: String,
    /// Trail width (multiplier of cursor size)
    pub width: f32,
    /// Animation speed (milliseconds per trail update)
    pub animation_speed: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct AnsiColors {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields accessed by users via config files
pub struct KeyBindings {
    pub new_tab: String,
    pub close_tab: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub split_vertical: String,
    pub split_horizontal: String,
    pub copy: String,
    pub paste: String,
    pub search: String,
    pub clear: String,
}

#[derive(Debug, Clone, Default)]
pub struct FeaturesConfig {
    /// Enable command palette (Ctrl+P)
    pub command_palette: bool,
    /// Enable resource monitor (Ctrl+R)
    pub resource_monitor: bool,
    /// Enable autocomplete suggestions
    pub autocomplete: bool,
    /// Enable progress bar for long-running commands
    pub progress_bar: bool,
    /// Enable session save/restore
    pub session_manager: bool,
    /// Enable theme manager for theme switching
    pub theme_manager: bool,
}

impl FeaturesConfig {
    fn from_lua_table(table: &Table) -> Result<Self> {
        Ok(Self {
            command_palette: table
                .get::<_, Option<bool>>("command_palette")?
                .unwrap_or(false),
            resource_monitor: table
                .get::<_, Option<bool>>("resource_monitor")?
                .unwrap_or(false),
            autocomplete: table
                .get::<_, Option<bool>>("autocomplete")?
                .unwrap_or(false),
            progress_bar: table
                .get::<_, Option<bool>>("progress_bar")?
                .unwrap_or(false),
            session_manager: table
                .get::<_, Option<bool>>("session_manager")?
                .unwrap_or(false),
            theme_manager: table
                .get::<_, Option<bool>>("theme_manager")?
                .unwrap_or(false),
        })
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            default_shell: detect_default_shell(),
            env: HashMap::new(),
            working_dir: None,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            max_history: 10000,
            enable_tabs: false,
            enable_split_pane: false,
            font_size: 12,
            cursor_style: "block".to_string(),
            scrollback_lines: 10000,
            hardware_acceleration: true,
        }
    }
}

impl ShellConfig {
    fn from_lua_table(table: &Table) -> Result<Self> {
        let default_shell = table
            .get::<_, Option<String>>("default_shell")?
            .unwrap_or_else(detect_default_shell);

        let env = if let Ok(env_table) = table.get::<_, Table>("env") {
            let mut map = HashMap::new();
            for pair in env_table.pairs::<String, String>() {
                let (key, value) = pair?;
                map.insert(key, value);
            }
            map
        } else {
            HashMap::new()
        };

        let working_dir = table.get::<_, Option<String>>("working_dir")?;

        Ok(Self {
            default_shell,
            env,
            working_dir,
        })
    }
}

impl TerminalConfig {
    fn from_lua_table(table: &Table) -> Result<Self> {
        Ok(Self {
            max_history: table
                .get::<_, Option<usize>>("max_history")?
                .unwrap_or(10000),
            enable_tabs: table
                .get::<_, Option<bool>>("enable_tabs")?
                .unwrap_or(false),
            enable_split_pane: table
                .get::<_, Option<bool>>("enable_split_pane")?
                .unwrap_or(false),
            font_size: table.get::<_, Option<u16>>("font_size")?.unwrap_or(12),
            cursor_style: table
                .get::<_, Option<String>>("cursor_style")?
                .unwrap_or_else(|| "block".to_string()),
            scrollback_lines: table
                .get::<_, Option<usize>>("scrollback_lines")?
                .unwrap_or(10000),
            hardware_acceleration: table
                .get::<_, Option<bool>>("hardware_acceleration")?
                .unwrap_or(true),
        })
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            foreground: "#FFFFFF".to_string(),
            background: "#1E1E1E".to_string(),
            cursor: "#00FF00".to_string(),
            selection: "#264F78".to_string(),
            colors: AnsiColors::default(),
            background_image: None,
            cursor_trail: None,
        }
    }
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            image_path: None,
            color: None,
            opacity: 1.0,
            mode: "fill".to_string(),
            blur: 0.0,
        }
    }
}

impl Default for CursorTrailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            length: 10,
            color: "#00FF0080".to_string(), // Green with 50% alpha
            fade_mode: "exponential".to_string(),
            width: 1.0,
            animation_speed: 16, // ~60 FPS
        }
    }
}

impl BackgroundConfig {
    fn from_lua_table(table: &Table) -> Result<Self> {
        Ok(Self {
            image_path: table.get::<_, Option<String>>("image_path")?,
            color: table.get::<_, Option<String>>("color")?,
            opacity: table.get::<_, Option<f32>>("opacity")?.unwrap_or(1.0),
            mode: table
                .get::<_, Option<String>>("mode")?
                .unwrap_or_else(|| "fill".to_string()),
            blur: table.get::<_, Option<f32>>("blur")?.unwrap_or(0.0),
        })
    }
}

impl CursorTrailConfig {
    fn from_lua_table(table: &Table) -> Result<Self> {
        Ok(Self {
            enabled: table.get::<_, Option<bool>>("enabled")?.unwrap_or(false),
            length: table.get::<_, Option<usize>>("length")?.unwrap_or(10),
            color: table
                .get::<_, Option<String>>("color")?
                .unwrap_or_else(|| "#00FF0080".to_string()),
            fade_mode: table
                .get::<_, Option<String>>("fade_mode")?
                .unwrap_or_else(|| "exponential".to_string()),
            width: table.get::<_, Option<f32>>("width")?.unwrap_or(1.0),
            animation_speed: table
                .get::<_, Option<u64>>("animation_speed")?
                .unwrap_or(16),
        })
    }
}

impl ThemeConfig {
    fn from_lua_table(table: &Table) -> Result<Self> {
        let name = table
            .get::<_, Option<String>>("name")?
            .unwrap_or_else(|| "default".to_string());
        let foreground = table
            .get::<_, Option<String>>("foreground")?
            .unwrap_or_else(|| "#FFFFFF".to_string());
        let background = table
            .get::<_, Option<String>>("background")?
            .unwrap_or_else(|| "#1E1E1E".to_string());
        let cursor = table
            .get::<_, Option<String>>("cursor")?
            .unwrap_or_else(|| "#00FF00".to_string());
        let selection = table
            .get::<_, Option<String>>("selection")?
            .unwrap_or_else(|| "#264F78".to_string());

        let colors = if let Ok(colors_table) = table.get::<_, Table>("colors") {
            AnsiColors::from_lua_table(&colors_table)?
        } else {
            AnsiColors::default()
        };

        let background_image = if let Ok(bg_table) = table.get::<_, Table>("background_image") {
            Some(BackgroundConfig::from_lua_table(&bg_table)?)
        } else {
            None
        };

        let cursor_trail = if let Ok(trail_table) = table.get::<_, Table>("cursor_trail") {
            Some(CursorTrailConfig::from_lua_table(&trail_table)?)
        } else {
            None
        };

        Ok(Self {
            name,
            foreground,
            background,
            cursor,
            selection,
            colors,
            background_image,
            cursor_trail,
        })
    }
}

impl Default for AnsiColors {
    fn default() -> Self {
        Self {
            black: "#000000".to_string(),
            red: "#FF0000".to_string(),
            green: "#00FF00".to_string(),
            yellow: "#FFFF00".to_string(),
            blue: "#0000FF".to_string(),
            magenta: "#FF00FF".to_string(),
            cyan: "#00FFFF".to_string(),
            white: "#FFFFFF".to_string(),
            bright_black: "#808080".to_string(),
            bright_red: "#FF8080".to_string(),
            bright_green: "#80FF80".to_string(),
            bright_yellow: "#FFFF80".to_string(),
            bright_blue: "#8080FF".to_string(),
            bright_magenta: "#FF80FF".to_string(),
            bright_cyan: "#80FFFF".to_string(),
            bright_white: "#FFFFFF".to_string(),
        }
    }
}

impl AnsiColors {
    fn from_lua_table(table: &Table) -> Result<Self> {
        Ok(Self {
            black: table
                .get::<_, Option<String>>("black")?
                .unwrap_or_else(|| "#000000".to_string()),
            red: table
                .get::<_, Option<String>>("red")?
                .unwrap_or_else(|| "#FF0000".to_string()),
            green: table
                .get::<_, Option<String>>("green")?
                .unwrap_or_else(|| "#00FF00".to_string()),
            yellow: table
                .get::<_, Option<String>>("yellow")?
                .unwrap_or_else(|| "#FFFF00".to_string()),
            blue: table
                .get::<_, Option<String>>("blue")?
                .unwrap_or_else(|| "#0000FF".to_string()),
            magenta: table
                .get::<_, Option<String>>("magenta")?
                .unwrap_or_else(|| "#FF00FF".to_string()),
            cyan: table
                .get::<_, Option<String>>("cyan")?
                .unwrap_or_else(|| "#00FFFF".to_string()),
            white: table
                .get::<_, Option<String>>("white")?
                .unwrap_or_else(|| "#FFFFFF".to_string()),
            bright_black: table
                .get::<_, Option<String>>("bright_black")?
                .unwrap_or_else(|| "#808080".to_string()),
            bright_red: table
                .get::<_, Option<String>>("bright_red")?
                .unwrap_or_else(|| "#FF8080".to_string()),
            bright_green: table
                .get::<_, Option<String>>("bright_green")?
                .unwrap_or_else(|| "#80FF80".to_string()),
            bright_yellow: table
                .get::<_, Option<String>>("bright_yellow")?
                .unwrap_or_else(|| "#FFFF80".to_string()),
            bright_blue: table
                .get::<_, Option<String>>("bright_blue")?
                .unwrap_or_else(|| "#8080FF".to_string()),
            bright_magenta: table
                .get::<_, Option<String>>("bright_magenta")?
                .unwrap_or_else(|| "#FF80FF".to_string()),
            bright_cyan: table
                .get::<_, Option<String>>("bright_cyan")?
                .unwrap_or_else(|| "#80FFFF".to_string()),
            bright_white: table
                .get::<_, Option<String>>("bright_white")?
                .unwrap_or_else(|| "#FFFFFF".to_string()),
        })
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            new_tab: "Ctrl+T".to_string(),
            close_tab: "Ctrl+W".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            prev_tab: "Ctrl+Shift+Tab".to_string(),
            split_vertical: "Ctrl+Shift+V".to_string(),
            split_horizontal: "Ctrl+Shift+H".to_string(),
            copy: "Ctrl+Shift+C".to_string(),
            paste: "Ctrl+Shift+V".to_string(),
            search: "Ctrl+F".to_string(),
            clear: "Ctrl+L".to_string(),
        }
    }
}

impl KeyBindings {
    fn from_lua_table(table: &Table) -> Result<Self> {
        Ok(Self {
            new_tab: table
                .get::<_, Option<String>>("new_tab")?
                .unwrap_or_else(|| "Ctrl+T".to_string()),
            close_tab: table
                .get::<_, Option<String>>("close_tab")?
                .unwrap_or_else(|| "Ctrl+W".to_string()),
            next_tab: table
                .get::<_, Option<String>>("next_tab")?
                .unwrap_or_else(|| "Ctrl+Tab".to_string()),
            prev_tab: table
                .get::<_, Option<String>>("prev_tab")?
                .unwrap_or_else(|| "Ctrl+Shift+Tab".to_string()),
            split_vertical: table
                .get::<_, Option<String>>("split_vertical")?
                .unwrap_or_else(|| "Ctrl+Shift+V".to_string()),
            split_horizontal: table
                .get::<_, Option<String>>("split_horizontal")?
                .unwrap_or_else(|| "Ctrl+Shift+H".to_string()),
            copy: table
                .get::<_, Option<String>>("copy")?
                .unwrap_or_else(|| "Ctrl+Shift+C".to_string()),
            paste: table
                .get::<_, Option<String>>("paste")?
                .unwrap_or_else(|| "Ctrl+Shift+V".to_string()),
            search: table
                .get::<_, Option<String>>("search")?
                .unwrap_or_else(|| "Ctrl+F".to_string()),
            clear: table
                .get::<_, Option<String>>("clear")?
                .unwrap_or_else(|| "Ctrl+L".to_string()),
        })
    }
}

impl Config {
    /// Load configuration from default location
    ///
    /// # Errors
    /// Returns an error if the config file exists but cannot be read or parsed
    pub fn load_default() -> Result<Self> {
        let config_path = Self::default_config_path()?;

        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a Lua file
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be read
    /// - The Lua code is invalid or has syntax errors
    /// - The Lua code does not define a 'config' table
    /// - The config table has invalid structure or data types
    ///
    /// # Security
    /// This executes Lua code from the configuration file. Only load trusted
    /// configuration files. The Lua environment has access to the full Lua standard
    /// library, including file I/O and OS operations.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref()).context("Failed to read config file")?;

        let lua = Lua::new();
        lua.load(&contents)
            .exec()
            .context("Failed to execute Lua config")?;

        let globals = lua.globals();
        let config_table: Table = globals
            .get("config")
            .context("Config table not found in Lua file")?;

        Self::from_lua_table(&config_table)
    }

    /// Parse configuration from a Lua table
    fn from_lua_table(table: &Table) -> Result<Self> {
        let shell = if let Ok(shell_table) = table.get::<_, Table>("shell") {
            ShellConfig::from_lua_table(&shell_table)?
        } else {
            ShellConfig::default()
        };

        let terminal = if let Ok(terminal_table) = table.get::<_, Table>("terminal") {
            TerminalConfig::from_lua_table(&terminal_table)?
        } else {
            TerminalConfig::default()
        };

        let theme = if let Ok(theme_table) = table.get::<_, Table>("theme") {
            ThemeConfig::from_lua_table(&theme_table)?
        } else {
            ThemeConfig::default()
        };

        let keybindings = if let Ok(kb_table) = table.get::<_, Table>("keybindings") {
            KeyBindings::from_lua_table(&kb_table)?
        } else {
            KeyBindings::default()
        };

        let features = if let Ok(features_table) = table.get::<_, Table>("features") {
            FeaturesConfig::from_lua_table(&features_table)?
        } else {
            FeaturesConfig::default()
        };

        let hooks = if let Ok(hooks_table) = table.get::<_, Table>("hooks") {
            HooksConfig::from_lua_table(&hooks_table)?
        } else {
            HooksConfig::default()
        };

        Ok(Self {
            shell,
            terminal,
            theme,
            keybindings,
            features,
            hooks,
        })
    }

    /// Get default configuration path
    ///
    /// # Errors
    /// Returns an error if the home directory cannot be determined
    pub fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;

        Ok(home.join(".furnace").join("config.lua"))
    }
}

/// Detect the default shell for the current platform
fn detect_default_shell() -> String {
    #[cfg(windows)]
    {
        // Try PowerShell 7+ first (pwsh.exe)
        if which::which("pwsh").is_ok() {
            return "pwsh.exe".to_string();
        }

        // Try PowerShell 5.1
        if which::which("powershell").is_ok() {
            return "powershell.exe".to_string();
        }

        // Fallback to cmd
        "cmd.exe".to_string()
    }

    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();
        assert!(!config.terminal.enable_tabs);
        assert!(!config.terminal.enable_split_pane);
        assert!(config.terminal.hardware_acceleration);
    }

    #[test]
    fn test_lua_config_deserialization() {
        let lua_config = r"
config = {
    terminal = {
        enable_tabs = true,
        enable_split_pane = true
    }
}
";
        let lua = Lua::new();
        lua.load(lua_config).exec().unwrap();
        let globals = lua.globals();
        let config_table: Table = globals.get("config").unwrap();
        let config = Config::from_lua_table(&config_table).unwrap();
        assert!(config.terminal.enable_tabs);
        assert!(config.terminal.enable_split_pane);
    }
}
