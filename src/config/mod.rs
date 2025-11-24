use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure with zero-copy design for performance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub plugins: Vec<String>,
    #[serde(default)]
    pub command_translation: CommandTranslationConfig,
    #[serde(default)]
    pub ssh_manager: SshManagerConfig,
    #[serde(default)]
    pub url_handler: UrlHandlerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub default_shell: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Maximum command history entries (memory-efficient circular buffer)
    #[serde(default = "default_max_history")]
    pub max_history: usize,
    
    /// Enable tabs for multiple sessions
    #[serde(default = "default_true")]
    pub enable_tabs: bool,
    
    /// Enable split panes
    #[serde(default = "default_true")]
    pub enable_split_pane: bool,
    
    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: u16,
    
    /// Cursor style: block, underline, bar
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    
    /// Number of scrollback lines (memory-mapped for large buffers)
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: usize,
    
    /// Hardware acceleration for rendering
    #[serde(default = "default_true")]
    pub hardware_acceleration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub selection: String,
    pub colors: AnsiColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandTranslationConfig {
    /// Enable automatic command translation between Linux and Windows
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Show visual notification when commands are translated
    #[serde(default = "default_true")]
    pub show_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshManagerConfig {
    /// Enable SSH connection manager
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Auto-show SSH manager when typing ssh command
    #[serde(default = "default_true")]
    pub auto_show: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlHandlerConfig {
    /// Enable clickable URLs with Ctrl+Click
    #[serde(default = "default_true")]
    pub enabled: bool,
}

// Default value functions
fn default_max_history() -> usize {
    10000
}

fn default_true() -> bool {
    true
}

fn default_font_size() -> u16 {
    12
}

fn default_cursor_style() -> String {
    "block".to_string()
}

fn default_scrollback() -> usize {
    10000
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
            enable_tabs: true,
            enable_split_pane: true,
            font_size: 12,
            cursor_style: "block".to_string(),
            scrollback_lines: 10000,
            hardware_acceleration: true,
        }
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
        }
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

impl Default for CommandTranslationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_notifications: true,
        }
    }
}

impl Default for SshManagerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_show: true,
        }
    }
}

impl Default for UrlHandlerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
        }
    }
}

impl Config {
    /// Load configuration from default location
    pub fn load_default() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        
        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;
        
        let config: Config = serde_yaml::from_str(&contents)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = serde_yaml::to_string(self)
            .context("Failed to serialize config")?;
        
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        fs::write(path.as_ref(), contents)
            .context("Failed to write config file")?;
        
        Ok(())
    }

    /// Get default configuration path
    pub fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;
        
        Ok(home.join(".furnace").join("config.yaml"))
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
    fn test_default_command_translation_config() {
        let config = CommandTranslationConfig::default();
        assert!(config.enabled);
        assert!(config.show_notifications);
    }
    
    #[test]
    fn test_config_with_command_translation() {
        let config = Config::default();
        assert!(config.command_translation.enabled);
        assert!(config.command_translation.show_notifications);
    }
    
    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
command_translation:
  enabled: false
  show_notifications: false
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.command_translation.enabled);
        assert!(!config.command_translation.show_notifications);
    }
}
