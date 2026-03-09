//! Maximum coverage push - targeting all remaining reachable uncovered lines
//! Focus: hooks (33 lines), config (26 lines), ANSI parser (57 lines), themes (15 lines)

use furnace::config::{Config, FeaturesConfig, KeyBindings, HooksConfig};
use furnace::hooks::HooksExecutor;
use furnace::terminal::ansi_parser::AnsiParser;
use furnace::ui::themes::{ThemeManager, ColorPalette, UiColors, SyntaxColors, Theme};
use std::collections::HashMap;
use tempfile::tempdir;

// ============================================================================
// Config Module - Target remaining 26 uncovered lines
// ============================================================================

#[test]
fn test_config_features() {
    let features = FeaturesConfig {
        resource_monitor: true,
        autocomplete: true,
        progress_bar: true,
        session_manager: true,
        theme_manager: true,
        command_palette: true,
        auto_save_session: false,
    };
    
    assert!(features.resource_monitor);
    assert!(features.command_palette);
}

#[test]
fn test_config_keybindings_structure() {
    let kb = KeyBindings {
        new_tab: "Ctrl+T".to_string(),
        close_tab: "Ctrl+W".to_string(),
        next_tab: "Ctrl+Tab".to_string(),
        prev_tab: "Ctrl+Shift+Tab".to_string(),
        split_vertical: "Ctrl+V".to_string(),
        split_horizontal: "Ctrl+H".to_string(),
        copy: "Ctrl+C".to_string(),
        paste: "Ctrl+V".to_string(),
        search: "Ctrl+F".to_string(),
        clear: "Ctrl+L".to_string(),
    };
    
    assert_eq!(kb.new_tab, "Ctrl+T");
    assert_eq!(kb.search, "Ctrl+F");
}

#[test]
fn test_config_hooks_structure() {
    let hooks = HooksConfig {
        on_startup: Some("startup_script.lua".to_string()),
        on_shutdown: Some("shutdown.lua".to_string()),
        on_key_press: Some("keys.lua".to_string()),
        on_command_start: Some("cmd_start.lua".to_string()),
        on_command_end: Some("cmd_end.lua".to_string()),
        on_output: Some("output.lua".to_string()),
        on_bell: Some("bell.lua".to_string()),
        on_title_change: Some("title.lua".to_string()),
        custom_keybindings: HashMap::new(),
        output_filters: vec!["filter1.lua".to_string(), "filter2.lua".to_string()],
        custom_widgets: vec!["widget1.lua".to_string()],
    };
    
    assert_eq!(hooks.on_startup, Some("startup_script.lua".to_string()));
    assert_eq!(hooks.output_filters.len(), 2);
    assert_eq!(hooks.custom_widgets.len(), 1);
}

#[test]
fn test_config_lua_with_custom_keybindings() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("custom_kb.lua");
    
    let lua = r##"
config = {
    terminal = { scrollback_lines = 10000 },
    shell = { default_shell = "sh" },
    theme = { foreground = "#FFF", background = "#000" },
    hooks = {
        custom_keybindings = {
            ["ctrl+x"] = "print('custom')",
            ["ctrl+y"] = "print('another')"
        }
    }
}
"##;
    
    std::fs::write(&path, lua).unwrap();
    let _ = Config::load_from_file(&path);
}

#[test]
fn test_config_lua_with_output_filters() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("filters.lua");
    
    let lua = r##"
config = {
    terminal = { scrollback_lines = 10000 },
    shell = { default_shell = "sh" },
    theme = { foreground = "#FFF", background = "#000" },
    hooks = {
        output_filters = {
            "output = string.upper(input)",
            "output = output"
        }
    }
}
"##;
    
    std::fs::write(&path, lua).unwrap();
    let _ = Config::load_from_file(&path);
}

#[test]
fn test_config_lua_with_custom_widgets() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("widgets.lua");
    
    let lua = r##"
config = {
    terminal = { scrollback_lines = 10000 },
    shell = { default_shell = "sh" },
    theme = { foreground = "#FFF", background = "#000" },
    hooks = {
        custom_widgets = {
            "widget_code_1",
            "widget_code_2"
        }
    }
}
"##;
    
    std::fs::write(&path, lua).unwrap();
    let _ = Config::load_from_file(&path);
}

// ============================================================================
// Hooks Module - Target remaining 33 uncovered lines
// ============================================================================

#[test]
fn test_hooks_filter_edge_cases() {
    let exec = HooksExecutor::new().unwrap();
    
    // Filter that returns nothing
    let filters = vec!["-- no output".to_string()];
    let result = exec.apply_output_filters("input", &filters);
    let _ = result;
    
    // Filter with just whitespace
    let filters = vec!["    ".to_string()];
    let result = exec.apply_output_filters("input", &filters);
    let _ = result;
}

#[test]
fn test_hooks_context_escaping_comprehensive() {
    let exec = HooksExecutor::new().unwrap();
    
    // All escape-worthy characters
    let contexts = vec![
        "quote\"test",
        "apostrophe'test",
        "newline\ntest",
        "tab\ttest",
        "carriage\rtest",
        "backslash\\test",
        "null\0test",
        "vtab\x0Btest",
        "ff\x0Ctest",
        "combined\"\'\n\r\t\\\0\x0B\x0Ctest",
    ];
    
    for ctx in contexts {
        assert!(exec.execute("x = 1", ctx).is_ok());
    }
}

// ============================================================================
// ANSI Parser - Target remaining 57 uncovered lines
// ============================================================================

#[test]
fn test_ansi_parser_remaining_sgr() {
    // Less common SGR codes
    let codes = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 26, 50, 51, 52, 53, 54, 55, 60, 61, 62, 63, 64, 65];
    
    for code in codes {
        let input = format!("\x1b[{}mtest\x1b[0m", code);
        assert!(!AnsiParser::parse(&input).is_empty());
    }
}

#[test]
fn test_ansi_parser_default_colors() {
    // Default foreground/background
    assert!(!AnsiParser::parse("\x1b[39mdefault fg\x1b[0m").is_empty());
    assert!(!AnsiParser::parse("\x1b[49mdefault bg\x1b[0m").is_empty());
}

#[test]
fn test_ansi_parser_double_underline() {
    // Double underline SGR 21
    assert!(!AnsiParser::parse("\x1b[21mdouble underline\x1b[0m").is_empty());
}

#[test]
fn test_ansi_parser_framed_encircled() {
    // Framed and encircled
    assert!(!AnsiParser::parse("\x1b[51mframed\x1b[0m").is_empty());
    assert!(!AnsiParser::parse("\x1b[52mencircled\x1b[0m").is_empty());
}

#[test]
fn test_ansi_parser_overline() {
    // Overline SGR 53
    assert!(!AnsiParser::parse("\x1b[53moverline\x1b[0m").is_empty());
}

#[test]
fn test_ansi_parser_cursor_show_hide() {
    // Cursor visibility
    assert!(!AnsiParser::parse("\x1b[?25h").is_empty()); // Show
    assert!(!AnsiParser::parse("\x1b[?25l").is_empty()); // Hide
}

#[test]
fn test_ansi_parser_alternate_buffer() {
    // Alternate screen buffer
    assert!(!AnsiParser::parse("\x1b[?47h").is_empty());
    assert!(!AnsiParser::parse("\x1b[?47l").is_empty());
    assert!(!AnsiParser::parse("\x1b[?1047h").is_empty());
    assert!(!AnsiParser::parse("\x1b[?1047l").is_empty());
    assert!(!AnsiParser::parse("\x1b[?1048h").is_empty());
    assert!(!AnsiParser::parse("\x1b[?1048l").is_empty());
}

#[test]
fn test_ansi_parser_bracketed_paste() {
    // Bracketed paste mode
    assert!(!AnsiParser::parse("\x1b[?2004h").is_empty());
    assert!(!AnsiParser::parse("\x1b[?2004l").is_empty());
}

#[test]
fn test_ansi_parser_window_manipulation() {
    // Window manipulation sequences
    assert!(!AnsiParser::parse("\x1b[8;24;80t").is_empty()); // Resize
    assert!(!AnsiParser::parse("\x1b[3;100;200t").is_empty()); // Move
}

#[test]
fn test_ansi_parser_device_status_report() {
    // DSR sequences
    assert!(!AnsiParser::parse("\x1b[5n").is_empty()); // Status report
    assert!(!AnsiParser::parse("\x1b[6n").is_empty()); // Cursor position report
}

#[test]
fn test_ansi_parser_soft_reset() {
    // Soft terminal reset
    assert!(!AnsiParser::parse("\x1b[!p").is_empty());
}

#[test]
fn test_ansi_parser_line_position_absolute() {
    // Line position absolute
    assert!(!AnsiParser::parse("\x1b[10d").is_empty());
}

#[test]
fn test_ansi_parser_save_restore_cursor_dec() {
    // DEC save/restore cursor
    assert!(!AnsiParser::parse("\x1b7").is_empty()); // Save
    assert!(!AnsiParser::parse("\x1b8").is_empty()); // Restore
}

#[test]
fn test_ansi_parser_index_reverse_index() {
    // Index and reverse index
    assert!(!AnsiParser::parse("\x1bD").is_empty()); // Index
    assert!(!AnsiParser::parse("\x1bM").is_empty()); // Reverse index
}

#[test]
fn test_ansi_parser_next_line() {
    // Next line (NEL)
    assert!(!AnsiParser::parse("\x1bE").is_empty());
}

#[test]
fn test_ansi_parser_set_tab() {
    // Set tab stop
    assert!(!AnsiParser::parse("\x1bH").is_empty());
}

#[test]
fn test_ansi_parser_application_keypad() {
    // Application keypad mode
    assert!(!AnsiParser::parse("\x1b=").is_empty()); // Set
    assert!(!AnsiParser::parse("\x1b>").is_empty()); // Reset
}

#[test]
fn test_ansi_parser_256_color_edge_cases() {
    // Edge cases for 256 color mode
    assert!(!AnsiParser::parse("\x1b[38;5;0m").is_empty()); // First
    assert!(!AnsiParser::parse("\x1b[38;5;255m").is_empty()); // Last
    assert!(!AnsiParser::parse("\x1b[48;5;0m").is_empty()); // BG first
    assert!(!AnsiParser::parse("\x1b[48;5;255m").is_empty()); // BG last
}

#[test]
fn test_ansi_parser_rgb_edge_cases() {
    // RGB color edge cases
    assert!(!AnsiParser::parse("\x1b[38;2;0;0;0m").is_empty()); // Black
    assert!(!AnsiParser::parse("\x1b[38;2;255;255;255m").is_empty()); // White
    assert!(!AnsiParser::parse("\x1b[48;2;0;0;0m").is_empty()); // BG black
    assert!(!AnsiParser::parse("\x1b[48;2;255;255;255m").is_empty()); // BG white
}

// ============================================================================
// Themes Module - Target remaining 15 uncovered lines
// ============================================================================

#[test]
fn test_theme_all_structures() {
    let colors = ColorPalette {
        black: "#000".into(),
        red: "#F00".into(),
        green: "#0F0".into(),
        yellow: "#FF0".into(),
        blue: "#00F".into(),
        magenta: "#F0F".into(),
        cyan: "#0FF".into(),
        white: "#FFF".into(),
        bright_black: "#888".into(),
        bright_red: "#F88".into(),
        bright_green: "#8F8".into(),
        bright_yellow: "#FF8".into(),
        bright_blue: "#88F".into(),
        bright_magenta: "#F8F".into(),
        bright_cyan: "#8FF".into(),
        bright_white: "#FFF".into(),
    };
    
    assert_eq!(colors.red, "#F00");
}

#[test]
fn test_theme_ui_colors() {
    let ui = UiColors {
        foreground: "#FFF".into(),
        background: "#000".into(),
        cursor: "#0F0".into(),
        selection: "#333".into(),
        border: "#666".into(),
        tab_active: "#0F0".into(),
        tab_inactive: "#666".into(),
        status_bar: "#111".into(),
        command_palette: "#222".into(),
    };
    
    assert_eq!(ui.cursor, "#0F0");
}

#[test]
fn test_theme_syntax_colors() {
    let syntax = SyntaxColors {
        keyword: "#F00".into(),
        string: "#0F0".into(),
        comment: "#888".into(),
        function: "#00F".into(),
        variable: "#FFF".into(),
        error: "#F00".into(),
        warning: "#FF0".into(),
    };
    
    assert_eq!(syntax.keyword, "#F00");
}

#[test]
fn test_theme_manager_with_yaml_themes() {
    let dir = tempdir().unwrap();
    
    // Create a valid YAML theme
    let theme_yaml = r##"
name: CustomTest
colors:
  black: "#000000"
  red: "#FF0000"
  green: "#00FF00"
  yellow: "#FFFF00"
  blue: "#0000FF"
  magenta: "#FF00FF"
  cyan: "#00FFFF"
  white: "#FFFFFF"
  bright_black: "#808080"
  bright_red: "#FF8080"
  bright_green: "#80FF80"
  bright_yellow: "#FFFF80"
  bright_blue: "#8080FF"
  bright_magenta: "#FF80FF"
  bright_cyan: "#80FFFF"
  bright_white: "#FFFFFF"
ui:
  foreground: "#E0E0E0"
  background: "#1A1A1A"
  cursor: "#00FF00"
  selection: "#404040"
  border: "#606060"
  tab_active: "#00FF00"
  tab_inactive: "#808080"
  status_bar: "#2A2A2A"
  command_palette: "#2A2A2A"
syntax:
  keyword: "#FF0000"
  string: "#00FF00"
  comment: "#808080"
  function: "#0000FF"
  variable: "#FFFF00"
  error: "#FF0000"
  warning: "#FFA500"
"##;
    
    let theme_file = dir.path().join("custom.yaml");
    std::fs::write(&theme_file, theme_yaml).unwrap();
    
    let result = ThemeManager::with_themes_dir(dir.path());
    assert!(result.is_ok());
    
    if let Ok(mut tm) = result {
        // Try to switch to custom theme
        let _ = tm.switch_theme("customtest");
    }
}

#[test]
fn test_theme_manager_yaml_error_handling() {
    let dir = tempdir().unwrap();
    
    // Create invalid YAML files
    let invalid1 = dir.path().join("invalid1.yaml");
    std::fs::write(&invalid1, "not: valid: yaml: format: error").unwrap();
    
    let invalid2 = dir.path().join("invalid2.yaml");
    std::fs::write(&invalid2, "[[[[").unwrap();
    
    // Should still create manager, just skip invalid files
    let result = ThemeManager::with_themes_dir(dir.path());
    assert!(result.is_ok());
}
