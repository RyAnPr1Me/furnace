use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Advanced theme system supporting multiple color schemes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Public API for theme system
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
    pub ui: UiColors,
    pub syntax: SyntaxColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Public API for theme system
pub struct ColorPalette {
    // ANSI colors
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,

    // Bright colors
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
pub struct UiColors {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub selection: String,
    pub border: String,
    pub tab_active: String,
    pub tab_inactive: String,
    pub status_bar: String,
    pub command_palette: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: String,
    pub string: String,
    pub comment: String,
    pub function: String,
    pub variable: String,
    pub error: String,
    pub warning: String,
}

/// Built-in themes
pub struct Themes;

impl Themes {
    /// Dark theme (default) with cool red/black color scheme
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn dark() -> Theme {
        Theme {
            name: "Dark".to_string(),
            colors: ColorPalette {
                black: "#000000".to_string(),
                red: "#CC5555".to_string(),        // Darker, cooler red
                green: "#5A8A6A".to_string(),      // Muted green
                yellow: "#B89860".to_string(),     // Darker yellow
                blue: "#6A7A9A".to_string(),       // Cool blue-gray
                magenta: "#B05A7A".to_string(),    // Dark magenta-red
                cyan: "#5A8A8A".to_string(),       // Dark teal
                white: "#C0B0B0".to_string(),      // Slightly reddish gray
                bright_black: "#3A2A2A".to_string(),   // Dark reddish-gray
                bright_red: "#DD6666".to_string(),     // Medium cool red
                bright_green: "#6A9A7A".to_string(),   // Muted bright green
                bright_yellow: "#C8A870".to_string(),  // Muted gold
                bright_blue: "#7A8AAA".to_string(),    // Cool light blue
                bright_magenta: "#C06A8A".to_string(), // Bright magenta-red
                bright_cyan: "#6A9A9A".to_string(),    // Muted cyan
                bright_white: "#D0C0C0".to_string(),   // Light reddish-gray
            },
            ui: UiColors {
                foreground: "#D0C0C0".to_string(),      // Light reddish-gray text
                background: "#000000".to_string(),      // Pure black background
                cursor: "#DD6666".to_string(),          // Cool red cursor
                selection: "#2A1A1A".to_string(),       // Very dark red selection
                border: "#4A3A3A".to_string(),          // Dark reddish border
                tab_active: "#DD6666".to_string(),      // Cool red active tab
                tab_inactive: "#2A1A1A".to_string(),    // Dark inactive tab
                status_bar: "#1A0A0A".to_string(),      // Almost black status bar
                command_palette: "#1A0A0A".to_string(), // Almost black palette
            },
            syntax: SyntaxColors {
                keyword: "#DD6666".to_string(),         // Cool red keywords
                string: "#B89860".to_string(),          // Muted gold strings
                comment: "#5A4A4A".to_string(),         // Dark gray comments
                function: "#B05A7A".to_string(),        // Magenta-red functions
                variable: "#C0B0B0".to_string(),        // Reddish-gray variables
                error: "#EE5555".to_string(),           // Brighter red for errors
                warning: "#C8A870".to_string(),         // Gold warnings
            },
        }
    }

    /// Light theme
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn light() -> Theme {
        Theme {
            name: "Light".to_string(),
            colors: ColorPalette {
                black: "#000000".to_string(),
                red: "#D70000".to_string(),
                green: "#008700".to_string(),
                yellow: "#AF8700".to_string(),
                blue: "#0087FF".to_string(),
                magenta: "#AF00DB".to_string(),
                cyan: "#00AFAF".to_string(),
                white: "#5F5F5F".to_string(),
                bright_black: "#767676".to_string(),
                bright_red: "#D75F00".to_string(),
                bright_green: "#00AF00".to_string(),
                bright_yellow: "#FFAF00".to_string(),
                bright_blue: "#5FAFFF".to_string(),
                bright_magenta: "#D787D7".to_string(),
                bright_cyan: "#00DFFF".to_string(),
                bright_white: "#FFFFFF".to_string(),
            },
            ui: UiColors {
                foreground: "#000000".to_string(),
                background: "#FFFFFF".to_string(),
                cursor: "#008700".to_string(),
                selection: "#B4D5FE".to_string(),
                border: "#D0D0D0".to_string(),
                tab_active: "#0087FF".to_string(),
                tab_inactive: "#E0E0E0".to_string(),
                status_bar: "#F0F0F0".to_string(),
                command_palette: "#F8F8F8".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#AF00DB".to_string(),
                string: "#AF8700".to_string(),
                comment: "#767676".to_string(),
                function: "#0087FF".to_string(),
                variable: "#000000".to_string(),
                error: "#D70000".to_string(),
                warning: "#D75F00".to_string(),
            },
        }
    }

    /// Nord theme
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn nord() -> Theme {
        Theme {
            name: "Nord".to_string(),
            colors: ColorPalette {
                black: "#3B4252".to_string(),
                red: "#BF616A".to_string(),
                green: "#A3BE8C".to_string(),
                yellow: "#EBCB8B".to_string(),
                blue: "#81A1C1".to_string(),
                magenta: "#B48EAD".to_string(),
                cyan: "#88C0D0".to_string(),
                white: "#E5E9F0".to_string(),
                bright_black: "#4C566A".to_string(),
                bright_red: "#BF616A".to_string(),
                bright_green: "#A3BE8C".to_string(),
                bright_yellow: "#EBCB8B".to_string(),
                bright_blue: "#81A1C1".to_string(),
                bright_magenta: "#B48EAD".to_string(),
                bright_cyan: "#8FBCBB".to_string(),
                bright_white: "#ECEFF4".to_string(),
            },
            ui: UiColors {
                foreground: "#D8DEE9".to_string(),
                background: "#2E3440".to_string(),
                cursor: "#88C0D0".to_string(),
                selection: "#434C5E".to_string(),
                border: "#4C566A".to_string(),
                tab_active: "#88C0D0".to_string(),
                tab_inactive: "#3B4252".to_string(),
                status_bar: "#3B4252".to_string(),
                command_palette: "#3B4252".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#81A1C1".to_string(),
                string: "#A3BE8C".to_string(),
                comment: "#616E88".to_string(),
                function: "#88C0D0".to_string(),
                variable: "#D8DEE9".to_string(),
                error: "#BF616A".to_string(),
                warning: "#EBCB8B".to_string(),
            },
        }
    }

    /// Get all built-in themes
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn all() -> HashMap<String, Theme> {
        let mut themes = HashMap::new();
        themes.insert("dark".to_string(), Self::dark());
        themes.insert("light".to_string(), Self::light());
        themes.insert("nord".to_string(), Self::nord());
        themes
    }
}

impl Default for Theme {
    fn default() -> Self {
        Themes::dark()
    }
}

/// Theme manager for dynamic theme loading and switching at runtime
#[derive(Debug)]
pub struct ThemeManager {
    /// Currently active theme
    current_theme: Theme,
    /// All available themes (built-in + custom)
    available_themes: HashMap<String, Theme>,
    /// Path to custom themes directory
    themes_dir: Option<PathBuf>,
}

impl ThemeManager {
    /// Create a new theme manager with built-in themes
    #[must_use]
    pub fn new() -> Self {
        let available_themes = Themes::all();
        let current_theme = Themes::dark();

        Self {
            current_theme,
            available_themes,
            themes_dir: None,
        }
    }

    /// Create a theme manager with custom themes directory
    ///
    /// # Errors
    /// Returns an error if the themes directory cannot be created or themes cannot be loaded
    pub fn with_themes_dir<P: AsRef<Path>>(themes_dir: P) -> Result<Self> {
        let themes_dir = themes_dir.as_ref().to_path_buf();

        // Create themes directory if it doesn't exist
        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir).context("Failed to create themes directory")?;
        }

        let mut manager = Self::new();
        manager.themes_dir = Some(themes_dir);
        manager.load_custom_themes()?;

        Ok(manager)
    }

    /// Load custom themes from the themes directory
    fn load_custom_themes(&mut self) -> Result<()> {
        let Some(ref themes_dir) = self.themes_dir else {
            return Ok(());
        };

        if !themes_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(themes_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .yaml or .yml files
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    match Self::load_theme_from_file(&path) {
                        Ok(theme) => {
                            let name = theme.name.to_lowercase();
                            self.available_themes.insert(name, theme);
                        }
                        Err(e) => {
                            // Log warning but continue loading other themes
                            eprintln!("Warning: Failed to load theme from {path:?}: {e}");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a theme from a YAML file
    fn load_theme_from_file<P: AsRef<Path>>(path: P) -> Result<Theme> {
        let contents = fs::read_to_string(path.as_ref()).context("Failed to read theme file")?;
        let theme: Theme = serde_yaml::from_str(&contents).context("Failed to parse theme file")?;
        Ok(theme)
    }

    /// Get the current theme
    #[must_use]
    pub fn current(&self) -> &Theme {
        &self.current_theme
    }

    /// Get a list of all available theme names
    #[must_use]
    pub fn available_theme_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.available_themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// Switch to a different theme by name
    ///
    /// Returns true if the theme was switched successfully, false if the theme was not found
    pub fn switch_theme(&mut self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        if let Some(theme) = self.available_themes.get(&name_lower) {
            self.current_theme = theme.clone();
            true
        } else {
            false
        }
    }

    /// Cycle to the next theme in alphabetical order
    pub fn next_theme(&mut self) {
        let names = self.available_theme_names();
        if names.is_empty() {
            return;
        }

        let current_name = self.current_theme.name.to_lowercase();
        // If current theme isn't in the list (e.g., custom theme was removed),
        // start from the first theme
        let current_idx = names
            .iter()
            .position(|n| n == &current_name)
            .unwrap_or(names.len().saturating_sub(1));
        let next_idx = (current_idx + 1) % names.len();

        if let Some(theme) = self.available_themes.get(&names[next_idx]) {
            self.current_theme = theme.clone();
        }
    }

    /// Cycle to the previous theme in alphabetical order
    #[allow(dead_code)] // Public API
    pub fn prev_theme(&mut self) {
        let names = self.available_theme_names();
        if names.is_empty() {
            return;
        }

        let current_name = self.current_theme.name.to_lowercase();
        // If current theme isn't in the list, start from the first theme
        let current_idx = names.iter().position(|n| n == &current_name).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            names.len() - 1
        } else {
            current_idx - 1
        };

        if let Some(theme) = self.available_themes.get(&names[prev_idx]) {
            self.current_theme = theme.clone();
        }
    }

    /// Add a custom theme
    #[allow(dead_code)] // Public API
    pub fn add_theme(&mut self, theme: Theme) {
        let name = theme.name.to_lowercase();
        self.available_themes.insert(name, theme);
    }

    /// Save a theme to the custom themes directory
    ///
    /// # Errors
    /// Returns an error if the themes directory is not set or the file cannot be written
    #[allow(dead_code)] // Public API
    pub fn save_theme(&self, theme: &Theme) -> Result<()> {
        let themes_dir = self
            .themes_dir
            .as_ref()
            .context("Themes directory not configured")?;

        let filename = format!("{}.yaml", theme.name.to_lowercase().replace(' ', "_"));
        let path = themes_dir.join(filename);

        let contents = serde_yaml::to_string(theme).context("Failed to serialize theme")?;
        fs::write(&path, contents).context("Failed to write theme file")?;

        Ok(())
    }

    /// Get the default themes directory path
    ///
    /// # Errors
    /// Returns an error if the home directory cannot be determined
    pub fn default_themes_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".furnace").join("themes"))
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current().name, "Dark");
        assert!(!manager.available_theme_names().is_empty());
    }

    #[test]
    fn test_theme_switching() {
        let mut manager = ThemeManager::new();

        assert!(manager.switch_theme("light"));
        assert_eq!(manager.current().name, "Light");

        assert!(manager.switch_theme("nord"));
        assert_eq!(manager.current().name, "Nord");

        assert!(!manager.switch_theme("nonexistent"));
    }

    #[test]
    fn test_theme_cycling() {
        let mut manager = ThemeManager::new();
        let initial_name = manager.current().name.clone();

        manager.next_theme();
        let next_name = manager.current().name.clone();

        // Should have changed to a different theme
        assert_ne!(initial_name, next_name);

        manager.prev_theme();
        // Should be back to initial
        assert_eq!(manager.current().name, initial_name);
    }

    #[test]
    fn test_available_themes() {
        let manager = ThemeManager::new();
        let names = manager.available_theme_names();

        assert!(names.contains(&"dark".to_string()));
        assert!(names.contains(&"light".to_string()));
        assert!(names.contains(&"nord".to_string()));
    }
}
