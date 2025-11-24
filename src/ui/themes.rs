use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Dark theme (default)
    #[allow(dead_code)] // Public API
    pub fn dark() -> Theme {
        Theme {
            name: "Dark".to_string(),
            colors: ColorPalette {
                black: "#000000".to_string(),
                red: "#FF5555".to_string(),
                green: "#50FA7B".to_string(),
                yellow: "#F1FA8C".to_string(),
                blue: "#BD93F9".to_string(),
                magenta: "#FF79C6".to_string(),
                cyan: "#8BE9FD".to_string(),
                white: "#BFBFBF".to_string(),
                bright_black: "#4D4D4D".to_string(),
                bright_red: "#FF6E67".to_string(),
                bright_green: "#5AF78E".to_string(),
                bright_yellow: "#F4F99D".to_string(),
                bright_blue: "#CAA9FA".to_string(),
                bright_magenta: "#FF92D0".to_string(),
                bright_cyan: "#9AEDFE".to_string(),
                bright_white: "#E6E6E6".to_string(),
            },
            ui: UiColors {
                foreground: "#F8F8F2".to_string(),
                background: "#1E1E1E".to_string(),
                cursor: "#50FA7B".to_string(),
                selection: "#44475A".to_string(),
                border: "#6272A4".to_string(),
                tab_active: "#BD93F9".to_string(),
                tab_inactive: "#44475A".to_string(),
                status_bar: "#282A36".to_string(),
                command_palette: "#282A36".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#FF79C6".to_string(),
                string: "#F1FA8C".to_string(),
                comment: "#6272A4".to_string(),
                function: "#8BE9FD".to_string(),
                variable: "#F8F8F2".to_string(),
                error: "#FF5555".to_string(),
                warning: "#FFB86C".to_string(),
            },
        }
    }

    /// Light theme
    pub fn light() -> Theme {
    #[allow(dead_code)] // Public API
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
    pub fn nord() -> Theme {
        Theme {
    #[allow(dead_code)] // Public API
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
    pub fn all() -> HashMap<String, Theme> {
        let mut themes = HashMap::new();
        themes.insert("dark".to_string(), Self::dark());
    #[allow(dead_code)] // Public API
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
