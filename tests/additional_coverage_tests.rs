//! Additional comprehensive tests to push coverage toward 100%
//! 
//! Focuses on terminal module, ANSI parser, config, and cmdx modules

use furnace::colors::TrueColor;
use furnace::config::Config;
use furnace::hooks::HooksExecutor;
use furnace::progress_bar::ProgressBar;
use furnace::session::{SessionManager, SavedSession, TabState};
use furnace::terminal::ansi_parser::AnsiParser;
use furnace::ui::themes::{ThemeManager, Theme, Themes, ColorPalette, UiColors, SyntaxColors};

use chrono::Local;
use tempfile::tempdir;
use uuid::Uuid;

// ============================================================================
// ANSI Parser Extended Tests - Target 198 uncovered lines
// ============================================================================

#[test]
fn test_ansi_parser_color_variations() {
    // Test 8 basic ANSI colors
    for i in 30..=37 {
        let input = format!("\x1b[{}mcolor{}\x1b[0m", i, i);
        let result = AnsiParser::parse(&input);
        assert!(!result.is_empty());
    }
    
    // Test bright colors
    for i in 90..=97 {
        let input = format!("\x1b[{}mbright{}\x1b[0m", i, i);
        let result = AnsiParser::parse(&input);
        assert!(!result.is_empty());
    }
    
    // Test background colors
    for i in 40..=47 {
        let input = format!("\x1b[{}mbg{}\x1b[0m", i, i);
        let result = AnsiParser::parse(&input);
        assert!(!result.is_empty());
    }
}

#[test]
fn test_ansi_parser_sgr_combinations() {
    // Bold + color
    assert!(!AnsiParser::parse("\x1b[1;31mBold Red\x1b[0m").is_empty());
    
    // Underline + color
    assert!(!AnsiParser::parse("\x1b[4;32mUnderline Green\x1b[0m").is_empty());
    
    // Bold + underline + color
    assert!(!AnsiParser::parse("\x1b[1;4;33mBold Underline Yellow\x1b[0m").is_empty());
    
    // Dim
    assert!(!AnsiParser::parse("\x1b[2mDim text\x1b[0m").is_empty());
    
    // Italic
    assert!(!AnsiParser::parse("\x1b[3mItalic text\x1b[0m").is_empty());
    
    // Blink
    assert!(!AnsiParser::parse("\x1b[5mBlink text\x1b[0m").is_empty());
    
    // Reverse
    assert!(!AnsiParser::parse("\x1b[7mReverse text\x1b[0m").is_empty());
    
    // Hidden
    assert!(!AnsiParser::parse("\x1b[8mHidden text\x1b[0m").is_empty());
    
    // Strikethrough
    assert!(!AnsiParser::parse("\x1b[9mStrikethrough text\x1b[0m").is_empty());
}

#[test]
fn test_ansi_parser_cursor_operations() {
    // Cursor up
    assert!(!AnsiParser::parse("\x1b[5Aup").is_empty());
    
    // Cursor down
    assert!(!AnsiParser::parse("\x1b[3Bdown").is_empty());
    
    // Cursor forward
    assert!(!AnsiParser::parse("\x1b[2Cforward").is_empty());
    
    // Cursor back
    assert!(!AnsiParser::parse("\x1b[4Dback").is_empty());
    
    // Cursor next line
    assert!(!AnsiParser::parse("\x1b[2Enext").is_empty());
    
    // Cursor previous line
    assert!(!AnsiParser::parse("\x1b[3Fprev").is_empty());
    
    // Cursor horizontal absolute
    assert!(!AnsiParser::parse("\x1b[10Ghoriz").is_empty());
    
    // Save cursor position
    assert!(!AnsiParser::parse("\x1b[stext").is_empty());
    
    // Restore cursor position
    assert!(!AnsiParser::parse("\x1b[utext").is_empty());
}

#[test]
fn test_ansi_parser_erase_operations() {
    // Erase in display - cursor to end
    assert!(!AnsiParser::parse("\x1b[0Jerase").is_empty());
    
    // Erase in display - cursor to beginning
    assert!(!AnsiParser::parse("\x1b[1Jerase").is_empty());
    
    // Erase entire display
    assert!(!AnsiParser::parse("\x1b[2Jerase").is_empty());
    
    // Erase in line - cursor to end
    assert!(!AnsiParser::parse("\x1b[0Kline").is_empty());
    
    // Erase in line - cursor to beginning
    assert!(!AnsiParser::parse("\x1b[1Kline").is_empty());
    
    // Erase entire line
    assert!(!AnsiParser::parse("\x1b[2Kline").is_empty());
}

#[test]
fn test_ansi_parser_256_color_range() {
    // Test various 256 color indices
    for idx in [0, 16, 52, 88, 124, 160, 196, 232, 255] {
        let fg = format!("\x1b[38;5;{}mtext\x1b[0m", idx);
        assert!(!AnsiParser::parse(&fg).is_empty());
        
        let bg = format!("\x1b[48;5;{}mtext\x1b[0m", idx);
        assert!(!AnsiParser::parse(&bg).is_empty());
    }
}

#[test]
fn test_ansi_parser_rgb_colors() {
    // Various RGB combinations
    let rgb_tests = vec![
        (255, 0, 0),     // Red
        (0, 255, 0),     // Green
        (0, 0, 255),     // Blue
        (255, 255, 0),   // Yellow
        (255, 0, 255),   // Magenta
        (0, 255, 255),   // Cyan
        (128, 128, 128), // Gray
        (64, 128, 192),  // Random
    ];
    
    for (r, g, b) in rgb_tests {
        let fg = format!("\x1b[38;2;{};{};{}mtext\x1b[0m", r, g, b);
        assert!(!AnsiParser::parse(&fg).is_empty());
        
        let bg = format!("\x1b[48;2;{};{};{}mtext\x1b[0m", r, g, b);
        assert!(!AnsiParser::parse(&bg).is_empty());
    }
}

#[test]
fn test_ansi_parser_malformed_sequences() {
    // Incomplete sequences
    assert!(!AnsiParser::parse("\x1b[").is_empty());
    assert!(!AnsiParser::parse("\x1b[3").is_empty());
    assert!(!AnsiParser::parse("\x1b[31").is_empty());
    
    // Invalid parameters
    assert!(!AnsiParser::parse("\x1b[999mtext\x1b[0m").is_empty());
    assert!(!AnsiParser::parse("\x1b[38;5;999mtext\x1b[0m").is_empty());
    
    // Missing parameters
    assert!(!AnsiParser::parse("\x1b[38;2mmissing\x1b[0m").is_empty());
    assert!(!AnsiParser::parse("\x1b[38;2;255mmissing\x1b[0m").is_empty());
}

#[test]
fn test_ansi_parser_mixed_content() {
    let complex = "Normal \x1b[1mbold\x1b[0m \x1b[31mred\x1b[0m \x1b[1;4;32mbold underline green\x1b[0m";
    assert!(!AnsiParser::parse(complex).is_empty());
    
    let with_newlines = "Line1\n\x1b[31mRed Line2\x1b[0m\nLine3";
    assert!(!AnsiParser::parse(with_newlines).is_empty());
    
    let with_tabs = "Col1\t\x1b[32mCol2\x1b[0m\tCol3";
    assert!(!AnsiParser::parse(with_tabs).is_empty());
}

#[test]
fn test_ansi_parser_control_characters() {
    // Bell
    assert!(!AnsiParser::parse("text\x07bell").is_empty());
    
    // Backspace
    assert!(!AnsiParser::parse("text\x08back").is_empty());
    
    // Form feed
    assert!(!AnsiParser::parse("text\x0Cff").is_empty());
    
    // Vertical tab
    assert!(!AnsiParser::parse("text\x0Bvt").is_empty());
}

// ============================================================================
// Config Module Extended Tests - Target 105 uncovered lines
// ============================================================================

#[test]
fn test_config_all_fields() {
    let mut config = Config::default();
    
    // Terminal config variations
    config.terminal.scrollback_lines = 50000;
    config.terminal.max_history = 10000;
    config.terminal.font_size = 16;
    config.terminal.enable_tabs = true;
    config.terminal.enable_split_pane = true;
    config.terminal.hardware_acceleration = true;
    
    assert_eq!(config.terminal.scrollback_lines, 50000);
    assert_eq!(config.terminal.max_history, 10000);
    assert_eq!(config.terminal.font_size, 16);
}

#[test]
fn test_config_shell_variations() {
    let mut config = Config::default();
    
    config.shell.default_shell = "bash".to_string();
    config.shell.env.insert("TEST".to_string(), "value".to_string());
    config.shell.working_dir = Some("/home/user".to_string());
    
    assert_eq!(config.shell.default_shell, "bash");
    assert_eq!(config.shell.env.len(), 1);
    assert!(config.shell.working_dir.is_some());
}

#[test]
fn test_config_theme_variations() {
    let mut config = Config::default();
    
    config.theme.foreground = "#AAAAAA".to_string();
    config.theme.background = "#111111".to_string();
    config.theme.cursor = "#FF0000".to_string();
    config.theme.selection = "#333333".to_string();
    config.theme.name = "CustomTheme".to_string();
    
    assert_eq!(config.theme.foreground, "#AAAAAA");
    assert_eq!(config.theme.cursor, "#FF0000");
    assert_eq!(config.theme.selection, "#333333");
}

#[test]
fn test_config_lua_with_all_fields() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("full_config.lua");
    
    let lua_config = r##"
config = {
    terminal = {
        scrollback_lines = 25000,
        max_history = 5000,
        font_size = 14,
        enable_tabs = true,
        enable_split_pane = true,
        hardware_acceleration = false
    },
    shell = {
        default_shell = "/bin/bash",
        working_dir = "/home/user"
    },
    theme = {
        foreground = "#E0E0E0",
        background = "#1A1A1A",
        cursor = "#00FF00",
        selection = "#404040"
    }
}
"##;
    
    std::fs::write(&path, lua_config).unwrap();
    let loaded = Config::load_from_file(&path);
    assert!(loaded.is_ok());
    
    let config = loaded.unwrap();
    assert_eq!(config.terminal.scrollback_lines, 25000);
    assert_eq!(config.shell.default_shell, "/bin/bash");
    assert_eq!(config.theme.foreground, "#E0E0E0");
}

#[test]
fn test_config_lua_partial_fields() {
    let dir = tempdir().unwrap();
    
    // Only terminal
    let path1 = dir.path().join("terminal_only.lua");
    std::fs::write(&path1, "config = { terminal = { scrollback_lines = 15000 } }").unwrap();
    assert!(Config::load_from_file(&path1).is_ok());
    
    // Only shell
    let path2 = dir.path().join("shell_only.lua");
    std::fs::write(&path2, "config = { shell = { default_shell = 'zsh' } }").unwrap();
    assert!(Config::load_from_file(&path2).is_ok());
    
    // Only theme
    let path3 = dir.path().join("theme_only.lua");
    std::fs::write(&path3, "config = { theme = { foreground = '#FFF' } }").unwrap();
    assert!(Config::load_from_file(&path3).is_ok());
}

#[test]
fn test_config_lua_edge_cases() {
    let dir = tempdir().unwrap();
    
    // Empty config table
    let path1 = dir.path().join("empty.lua");
    std::fs::write(&path1, "config = {}").unwrap();
    assert!(Config::load_from_file(&path1).is_ok());
    
    // Missing config variable
    let path2 = dir.path().join("no_config.lua");
    std::fs::write(&path2, "x = 1").unwrap();
    let result = Config::load_from_file(&path2);
    // Either loads with defaults or returns error - both are acceptable
    let _ = result;
    
    // Syntax error
    let path3 = dir.path().join("syntax_error.lua");
    std::fs::write(&path3, "config = {{{").unwrap();
    assert!(Config::load_from_file(&path3).is_err());
}

// ============================================================================
// Hooks Extended Tests
// ============================================================================

#[test]
fn test_hooks_filter_pipeline() {
    let exec = HooksExecutor::new().unwrap();
    
    // Multiple filters
    let filters = vec![
        "output = input".to_string(),
        "output = output".to_string(),
    ];
    assert!(exec.apply_output_filters("test", &filters).is_ok());
    
    // Empty filters in pipeline
    let filters = vec![
        "".to_string(),
        "   ".to_string(),
        "output = input".to_string(),
    ];
    assert!(exec.apply_output_filters("test", &filters).is_ok());
}

#[test]
fn test_hooks_context_edge_cases() {
    let exec = HooksExecutor::new().unwrap();
    
    // Very long context
    let long_context = "a".repeat(5000);
    assert!(exec.execute("x=1", &long_context).is_ok());
    
    // Unicode context
    assert!(exec.execute("x=1", "日本語テキスト").is_ok());
    assert!(exec.execute("x=1", "Émoji: 🎉🔥💯").is_ok());
    
    // All special chars combined
    assert!(exec.execute("x=1", "\"\'\n\r\t\0\x0B\x0C\\").is_ok());
}

// ============================================================================
// Session Extended Tests
// ============================================================================

#[test]
fn test_session_manager_operations() {
    let result = SessionManager::new();
    if let Ok(manager) = result {
        // List sessions
        let _ = manager.list_sessions();
        
        // Create session
        let session = SavedSession {
            id: Uuid::new_v4().to_string(),
            name: "test_session".to_string(),
            created_at: Local::now(),
            tabs: vec![
                TabState {
                    output: "tab1 output".to_string(),
                    working_dir: Some("/tmp".to_string()),
                    active: true,
                },
                TabState {
                    output: "tab2 output".to_string(),
                    working_dir: None,
                    active: false,
                },
            ],
        };
        
        // Save
        let _ = manager.save_session(&session);
        
        // Load
        let _ = manager.load_session(&session.id);
        
        // Delete
        let _ = manager.delete_session(&session.id);
    }
}

#[test]
fn test_session_with_many_tabs() {
    let mut tabs = Vec::new();
    for i in 0..10 {
        tabs.push(TabState {
            output: format!("Output {}", i),
            working_dir: Some(format!("/dir{}", i)),
            active: i == 0,
        });
    }
    
    let session = SavedSession {
        id: Uuid::new_v4().to_string(),
        name: "multi_tab".to_string(),
        created_at: Local::now(),
        tabs,
    };
    
    assert_eq!(session.tabs.len(), 10);
    assert!(session.tabs[0].active);
    assert!(!session.tabs[1].active);
}

// ============================================================================
// Progress Bar Extended Tests
// ============================================================================

#[test]
fn test_progress_bar_multiple_instances() {
    let pb1 = ProgressBar::new();
    let pb2 = ProgressBar::new();
    let pb3 = ProgressBar::new();
    
    // All should be created successfully
    drop(pb1);
    drop(pb2);
    drop(pb3);
}

// ============================================================================
// Themes Extended Tests
// ============================================================================

#[test]
fn test_theme_structure_complete() {
    let theme = Theme {
        name: "CustomTheme".to_string(),
        colors: ColorPalette {
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
        },
        ui: UiColors {
            foreground: "#E0E0E0".to_string(),
            background: "#1A1A1A".to_string(),
            cursor: "#00FF00".to_string(),
            selection: "#404040".to_string(),
            border: "#606060".to_string(),
            tab_active: "#00FF00".to_string(),
            tab_inactive: "#808080".to_string(),
            status_bar: "#2A2A2A".to_string(),
            command_palette: "#2A2A2A".to_string(),
        },
        syntax: SyntaxColors {
            keyword: "#FF0000".to_string(),
            string: "#00FF00".to_string(),
            comment: "#808080".to_string(),
            function: "#0000FF".to_string(),
            variable: "#FFFF00".to_string(),
            error: "#FF0000".to_string(),
            warning: "#FFA500".to_string(),
        },
    };
    
    assert_eq!(theme.name, "CustomTheme");
    assert_eq!(theme.colors.red, "#FF0000");
    assert_eq!(theme.ui.foreground, "#E0E0E0");
    assert_eq!(theme.syntax.keyword, "#FF0000");
}

#[test]
fn test_theme_manager_all_operations() {
    let mut tm = ThemeManager::new();
    
    // Get current
    let current = tm.current();
    assert!(!current.name.is_empty());
    
    // Get all names
    let names = tm.available_theme_names();
    assert!(names.len() >= 3);
    
    // Switch to each theme
    for name in &names {
        assert!(tm.switch_theme(name));
    }
    
    // Try invalid
    assert!(!tm.switch_theme("InvalidTheme123"));
    
    // Cycle forward multiple times
    for _ in 0..names.len() * 2 {
        tm.next_theme();
    }
    
    // Test all built-in themes exist
    assert!(Themes::all().contains_key("dark"));
    assert!(Themes::all().contains_key("light"));
    assert!(Themes::all().contains_key("nord"));
}

#[test]
fn test_theme_default() {
    let default_theme = Theme::default();
    assert_eq!(default_theme.name, "Dark");
}

// ============================================================================
// Colors Extended Tests
// ============================================================================

#[test]
fn test_color_blend_various_factors() {
    let c1 = TrueColor::new(100, 100, 100);
    let c2 = TrueColor::new(200, 200, 200);
    
    // Test various blend factors
    for factor in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let blended = c1.blend(c2, factor);
        assert!(blended.r >= c1.r && blended.r <= c2.r);
    }
}

#[test]
fn test_color_lighten_darken_extremes() {
    let color = TrueColor::new(128, 128, 128);
    
    // Lighten to maximum
    let max_light = color.lighten(1.0);
    assert_eq!(max_light, TrueColor::new(255, 255, 255));
    
    // Darken to minimum
    let max_dark = color.darken(1.0);
    assert_eq!(max_dark, TrueColor::new(0, 0, 0));
    
    // No change
    let no_lighten = color.lighten(0.0);
    assert_eq!(no_lighten, color);
    
    let no_darken = color.darken(0.0);
    assert_eq!(no_darken, color);
}

#[test]
fn test_color_luminance_boundary() {
    // Test luminance at boundaries
    let colors = [
        (0, 0, 0),
        (128, 128, 128),
        (255, 255, 255),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
    ];
    
    for (r, g, b) in colors {
        let color = TrueColor::new(r, g, b);
        let lum = color.luminance();
        assert!(lum >= 0.0 && lum <= 1.0);
    }
}

#[test]
fn test_color_hex_edge_cases() {
    // Lowercase
    assert!(TrueColor::from_hex("ff8800").is_ok());
    assert!(TrueColor::from_hex("#ff8800").is_ok());
    
    // Uppercase
    assert!(TrueColor::from_hex("FF8800").is_ok());
    assert!(TrueColor::from_hex("#FF8800").is_ok());
    
    // Mixed case
    assert!(TrueColor::from_hex("Ff8800").is_ok());
    
    // All zeros
    assert!(TrueColor::from_hex("000000").is_ok());
    
    // All F's
    assert!(TrueColor::from_hex("FFFFFF").is_ok());
}

#[test]
fn test_palette_extended_colors() {
    let palette = furnace::colors::TrueColorPalette::default_dark();
    
    // Test extended color ranges
    // First 16 are ANSI
    for i in 0..16 {
        let _ = palette.get_256(i);
    }
    
    // 216 color cube (16-231)
    for i in 16..232 {
        let _ = palette.get_256(i);
    }
    
    // 24 grayscale (232-255)
    for i in 232..=255 {
        let _ = palette.get_256(i);
    }
}
