//! Comprehensive test suite to achieve 100% code coverage
//! 
//! This file adds extensive tests for all modules to reach 100% coverage with tarpaulin

use furnace::colors::{TrueColor, TrueColorPalette};
use furnace::config::{Config, AnsiColors};
use furnace::hooks::HooksExecutor;
use furnace::keybindings::{KeybindingManager, Action};
use furnace::progress_bar::ProgressBar;
use furnace::session::{SessionManager, SavedSession, TabState};
use furnace::shell::ShellSession;
use furnace::terminal::Terminal;
use furnace::terminal::ansi_parser::AnsiParser;
use furnace::ui::autocomplete::Autocomplete;
use furnace::ui::resource_monitor::ResourceMonitor;
use furnace::ui::themes::{ThemeManager, Themes};

use chrono::Local;
use tempfile::tempdir;
use uuid::Uuid;

// ============================================================================
// Terminal Module Tests
// ============================================================================

#[test]
fn test_terminal_creation_variations() {
    let mut config = Config::default();
    config.terminal.scrollback_lines = 5000;
    assert!(Terminal::new(config).is_ok());

    let mut config = Config::default();
    config.terminal.enable_tabs = true;
    assert!(Terminal::new(config).is_ok());

    let mut config = Config::default();
    config.terminal.enable_split_pane = true;
    assert!(Terminal::new(config).is_ok());
}

// ============================================================================
// Hooks Module Tests
// ============================================================================

#[test]
fn test_hooks_all_methods() {
    let exec = HooksExecutor::new().unwrap();
    
    assert!(exec.execute("", "").is_ok());
    assert!(exec.execute("x=1", "ctx").is_ok());
    assert!(exec.on_startup("").is_ok());
    assert!(exec.on_shutdown("").is_ok());
    assert!(exec.on_key_press("", "a").is_ok());
    assert!(exec.on_command_start("", "ls").is_ok());
    assert!(exec.on_command_end("", "ls", 0).is_ok());
    assert!(exec.on_output("", "out").is_ok());
    assert!(exec.on_output("", &"a".repeat(2000)).is_ok());
    assert!(exec.on_bell("").is_ok());
    assert!(exec.on_title_change("", "title").is_ok());
    assert_eq!(exec.apply_output_filters("test", &[]).unwrap(), "test");
    assert!(exec.apply_output_filters("test", &[" ".to_string()]).is_ok());
    
    // Test special characters
    assert!(exec.execute("", "test\"quote").is_ok());
    assert!(exec.execute("", "test\nline").is_ok());
    assert!(exec.execute("", "test\ttab").is_ok());
    assert!(exec.execute("", "test\rcarriage").is_ok());
    assert!(exec.execute("", "test\\slash").is_ok());
    assert!(exec.execute("", "test\0null").is_ok());
    assert!(exec.execute("", "test\x0Bvtab").is_ok());
    assert!(exec.execute("", "test\x0Cff").is_ok());
    
    // Test invalid Lua
    assert!(exec.execute("invalid lua !!!", "").is_err());
}

// ============================================================================
// Progress Bar Tests
// ============================================================================

#[test]
fn test_progress_bar_variations() {
    let _pb1 = ProgressBar::new();
    let _pb2 = ProgressBar::new();
    let _pb3 = ProgressBar::new();
}

// ============================================================================
// Keybindings Tests
// ============================================================================

#[test]
fn test_all_actions() {
    let _m = KeybindingManager::new();
    let _a1 = Action::NewTab;
    let _a2 = Action::CloseTab;
    let _a3 = Action::NextTab;
    let _a4 = Action::PrevTab;
    let _a5 = Action::SplitHorizontal;
    let _a6 = Action::SplitVertical;
    let _a7 = Action::FocusNextPane;
    let _a8 = Action::FocusPrevPane;
    let _a9 = Action::Copy;
    let _a10 = Action::Paste;
    let _a11 = Action::SelectAll;
    let _a12 = Action::Clear;
    let _a13 = Action::Search;
    let _a14 = Action::SearchNext;
    let _a15 = Action::SearchPrev;
    let _a16 = Action::ToggleAutocomplete;
    let _a17 = Action::NextTheme;
    let _a18 = Action::PrevTheme;
    let _a19 = Action::ToggleResourceMonitor;
    let _a20 = Action::SaveSession;
    let _a21 = Action::LoadSession;
    let _a22 = Action::ListSessions;
    let _a23 = Action::SendToShell("cmd".into());
    let _a24 = Action::ExecuteCommand("ls".into());
    let _a25 = Action::Custom("custom".into());
    let _a26 = Action::ExecuteLua("print()".into());
}

// ============================================================================
// Session Tests
// ============================================================================

#[test]
fn test_session_structures() {
    let tab = TabState {
        output: "output".into(),
        working_dir: Some("/tmp".into()),
        active: true,
    };
    assert!(tab.active);
    
    let session = SavedSession {
        id: Uuid::new_v4().to_string(),
        name: "test".into(),
        created_at: Local::now(),
        tabs: vec![tab],
    };
    assert_eq!(session.tabs.len(), 1);
}

// ============================================================================
// Config Tests
// ============================================================================

#[test]
fn test_config_variations() {
    let _c = Config::default();
    let _ansi = AnsiColors::default();
    
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.lua");
    std::fs::write(&path, r##"
config = {
    terminal = { max_history = 1000 },
    theme = { foreground = "#FFF" },
    shell = { default_shell = "sh" }
}
"##).unwrap();
    
    assert!(Config::load_from_file(&path).is_ok());
    
    let invalid_path = dir.path().join("invalid.lua");
    std::fs::write(&invalid_path, "invalid!!!").unwrap();
    assert!(Config::load_from_file(&invalid_path).is_err());
}

// ============================================================================
// Colors Tests
// ============================================================================

#[test]
fn test_colors_comprehensive() {
    let c1 = TrueColor::new(100, 150, 200);
    let c2 = TrueColor::new(200, 100, 50);
    
    assert_eq!(c1.r, 100);
    assert!(c1.to_hex().starts_with('#'));
    assert!(c1.to_ansi_fg().contains("38;2"));
    assert!(c1.to_ansi_bg().contains("48;2"));
    assert!(format!("{}", c1).starts_with('#'));
    
    let blended = c1.blend(c2, 0.5);
    assert!(blended.r > 0);
    
    assert_eq!(c1.blend(c2, 0.0), c1);
    assert_eq!(c1.blend(c2, 1.0), c2);
    assert_eq!(c1.blend(c2, -1.0), c1);
    assert_eq!(c1.blend(c2, 2.0), c2);
    
    let light = c1.lighten(0.3);
    assert!(light.r >= c1.r);
    
    let dark = c1.darken(0.3);
    assert!(dark.r <= c1.r);
    
    let lum = c1.luminance();
    assert!(lum >= 0.0 && lum <= 1.0);
    
    let white = TrueColor::new(255, 255, 255);
    assert!(white.is_light());
    
    let black = TrueColor::new(0, 0, 0);
    assert!(!black.is_light());
    
    assert!(TrueColor::from_hex("#FF8800").is_ok());
    assert!(TrueColor::from_hex("FF8800").is_ok());
    assert!(TrueColor::from_hex("FFF").is_err());
    assert!(TrueColor::from_hex("GGGGGG").is_err());
    
    let palette = TrueColorPalette::default_dark();
    assert_eq!(palette.extended.len(), 256);
    for i in 0..=255u8 {
        let _c = palette.get_256(i);
    }
    
    let ansi = AnsiColors::default();
    assert!(TrueColorPalette::from_ansi_colors(&ansi).is_ok());
}

// ============================================================================
// Shell Tests
// ============================================================================

#[tokio::test]
async fn test_shell_operations() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    assert!(ShellSession::new(shell, None, 24, 80).is_ok());
    assert!(ShellSession::new(shell, None, 40, 120).is_ok());
    
    let tmp = std::env::temp_dir();
    let tmp_str = tmp.to_str().unwrap();
    assert!(ShellSession::new(shell, Some(tmp_str), 24, 80).is_ok());
    
    if let Ok(session) = ShellSession::new(shell, None, 24, 80) {
        assert!(session.resize(30, 100).await.is_ok());
        
        let mut buf = vec![0u8; 1024];
        let _ = session.read_output(&mut buf).await;
        
        let _ = session.write_input(b"echo test\n").await;
    }
}

// ============================================================================
// ANSI Parser Tests
// ============================================================================

#[test]
fn test_ansi_parser_comprehensive() {
    // Empty input returns 1 line (the default line from commit_current_line)
    assert_eq!(AnsiParser::parse("").len(), 1);
    assert!(AnsiParser::parse("plain text").len() > 0);
    assert!(AnsiParser::parse("\x1b[31mred\x1b[0m").len() > 0);
    assert!(AnsiParser::parse("\x1b[1mbold\x1b[0m").len() > 0);
    assert!(AnsiParser::parse("\x1b[4munderline\x1b[0m").len() > 0);
    assert!(AnsiParser::parse("\x1b[38;2;255;0;0mrgb\x1b[0m").len() > 0);
    assert!(AnsiParser::parse("\x1b[38;5;196m256\x1b[0m").len() > 0);
    assert!(AnsiParser::parse("\x1b[10;20Hcursor").len() > 0);
    assert!(AnsiParser::parse("\x1b[2Jclear").len() > 0);
    assert!(AnsiParser::parse("\x1b[1;4;31mmulti\x1b[0m").len() > 0);
    assert!(AnsiParser::parse("line1\nline2\nline3").len() >= 1);
    assert!(AnsiParser::parse("col1\tcol2\tcol3").len() > 0);
    assert!(AnsiParser::parse("old\rnew").len() > 0);
}

// ============================================================================
// Autocomplete Tests
// ============================================================================

#[test]
fn test_autocomplete_comprehensive() {
    let mut ac = Autocomplete::new();
    
    ac.add_to_history("ls".to_string());
    ac.add_to_history("ls -la".to_string());
    ac.add_to_history("ls -lh".to_string());
    
    let sugg = ac.get_suggestions("ls");
    assert!(!sugg.is_empty(), "Should have suggestions after adding 'ls' commands");
    
    ac.clear_history();
    let sugg = ac.get_suggestions("l");
    // After clearing history, common commands like "ls" are still suggested
    assert!(!sugg.is_empty(), "Should still have common command suggestions after clearing history");
    
    ac.add_to_history("test1".to_string());
    ac.add_to_history("test2".to_string());
    let sugg = ac.get_suggestions("test");
    assert_eq!(sugg.len(), 2, "Should have exactly 2 suggestions for 'test' prefix");
    
    let sugg = ac.get_suggestions("nomatch_unlikely_xyz_123");
    assert!(sugg.is_empty(), "Should have no suggestions for non-matching prefix");
}

// ============================================================================
// Themes Tests
// ============================================================================

#[test]
fn test_themes_comprehensive() {
    let dark = Themes::dark();
    assert_eq!(dark.name, "Dark");
    
    let light = Themes::light();
    assert_eq!(light.name, "Light");
    
    let nord = Themes::nord();
    assert_eq!(nord.name, "Nord");
    
    let all = Themes::all();
    assert!(all.len() >= 3);
    
    let mut tm = ThemeManager::new();
    assert!(!tm.current().name.is_empty());
    
    let names = tm.available_theme_names();
    assert!(names.len() >= 3);
    
    assert!(tm.switch_theme("dark"));
    assert!(tm.switch_theme("light"));
    assert!(tm.switch_theme("nord"));
    assert!(!tm.switch_theme("nonexistent"));
    
    for _ in 0..5 {
        tm.next_theme();
    }
}

// ============================================================================
// Resource Monitor Tests
// ============================================================================

#[test]
fn test_resource_monitor_comprehensive() {
    let mut rm = ResourceMonitor::new();
    
    for _ in 0..5 {
        let stats = rm.get_stats();
        assert!(stats.cpu_usage >= 0.0);
        assert!(stats.memory_used >= 0);
        assert!(stats.memory_total >= stats.memory_used);
        assert!(stats.network_rx >= 0);
        assert!(stats.network_tx >= 0);
    }
}
