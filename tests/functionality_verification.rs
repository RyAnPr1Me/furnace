//! Comprehensive functionality verification tests
//!
//! These tests verify that all claimed features in the terminal emulator
//! actually work as described.

use furnace::*;
use tempfile::tempdir;

/// Test shell session functionality
#[cfg(test)]
mod shell_tests {
    use super::*;
    use furnace::shell::ShellSession;

    #[tokio::test]
    async fn test_shell_creation() {
        // Test that shell sessions can be created
        let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };

        let session = ShellSession::new(shell, None, 24, 80);
        assert!(session.is_ok(), "Failed to create shell session");
    }

    #[tokio::test]
    async fn test_shell_write_and_read() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };

        let session = ShellSession::new(shell, None, 24, 80).unwrap();

        // Write a simple command
        let command = if cfg!(windows) {
            "echo test\r\n"
        } else {
            "echo test\n"
        };

        let write_result = session.write_input(command.as_bytes()).await;
        assert!(write_result.is_ok(), "Failed to write to shell");
        assert_eq!(write_result.unwrap(), command.len());

        // Give shell time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Read output
        let mut buffer = vec![0u8; 1024];
        let read_result = session.read_output(&mut buffer).await;
        assert!(read_result.is_ok(), "Failed to read from shell");

        // Verify we got some output
        let bytes_read = read_result.unwrap();
        assert!(bytes_read > 0, "No output from shell");
    }

    #[tokio::test]
    async fn test_shell_resize() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };

        let session = ShellSession::new(shell, None, 24, 80).unwrap();

        // Test resizing
        let resize_result = session.resize(30, 100).await;
        assert!(resize_result.is_ok(), "Failed to resize PTY");
    }
}

/// Test ANSI parser functionality
#[cfg(test)]
mod ansi_parser_tests {
    use super::*;
    use furnace::terminal::ansi_parser::AnsiParser;

    #[test]
    fn test_ansi_parser_basic_colors() {
        let result = AnsiParser::parse("\x1b[31mRed Text\x1b[0m");

        assert!(!result.is_empty(), "No lines produced");
    }

    #[test]
    fn test_ansi_parser_rgb_colors() {
        // RGB color: ESC[38;2;R;G;Bm
        let result = AnsiParser::parse("\x1b[38;2;255;100;50mRGB Text\x1b[0m");

        assert!(!result.is_empty(), "No lines produced for RGB");
    }

    #[test]
    fn test_ansi_parser_multiple_attributes() {
        // Bold + Red + Underline
        let result = AnsiParser::parse("\x1b[1;31;4mBold Red Underlined\x1b[0m");

        assert!(
            !result.is_empty(),
            "No lines produced for multiple attributes"
        );
    }
}

/// Test color functionality
#[cfg(test)]
mod color_tests {
    use super::*;
    use furnace::colors::TrueColor;

    #[test]
    fn test_true_color_from_hex() {
        let color = TrueColor::from_hex("#FF0000");
        assert!(color.is_ok(), "Failed to parse hex color");
        let c = color.unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_true_color_blending() {
        let color1 = TrueColor::new(255, 0, 0); // Red
        let color2 = TrueColor::new(0, 0, 255); // Blue

        let blended = color1.blend(color2, 0.5);
        // Should be purple-ish (127, 0, 127)
        assert!(blended.r > 100 && blended.r < 150);
        assert!(blended.b > 100 && blended.b < 150);
    }

    #[test]
    fn test_true_color_luminance() {
        let white = TrueColor::new(255, 255, 255);
        let black = TrueColor::new(0, 0, 0);

        assert!(white.luminance() > black.luminance());
        assert!(white.luminance() > 0.9);
        assert!(black.luminance() < 0.1);
    }
}

/// Test configuration functionality
#[cfg(test)]
mod config_tests {
    use super::*;
    use furnace::config::Config;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();

        // Verify default values
        assert!(config.terminal.enable_tabs);
        assert!(config.terminal.enable_split_pane);
        assert_eq!(config.terminal.scrollback_lines, 10000);
        assert!(config.terminal.hardware_acceleration);
        assert!(config.command_translation.enabled);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.yaml");

        // Create and save config
        let mut config = Config::default();
        config.terminal.max_history = 5000;
        config.save_to_file(&config_path).unwrap();

        // Load and verify
        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.terminal.max_history, 5000);
    }
}

/// Test translator functionality
#[cfg(test)]
mod translator_tests {
    use super::*;
    use furnace::translator::CommandTranslator;

    #[test]
    fn test_translator_enabled() {
        let translator = CommandTranslator::new(true);
        assert!(translator.is_enabled());
    }

    #[test]
    fn test_translator_disabled() {
        let translator = CommandTranslator::new(false);
        assert!(!translator.is_enabled());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_command_translation() {
        let translator = CommandTranslator::new(true);

        // Test Windows to Linux translation
        let result = translator.translate("dir");
        assert!(result.translated);
        assert_eq!(result.final_command, "ls");

        let result = translator.translate("type file.txt");
        assert!(result.translated);
        assert_eq!(result.final_command, "cat file.txt");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_command_translation() {
        let translator = CommandTranslator::new(true);

        // Test Linux to Windows translation
        let result = translator.translate("ls");
        assert!(result.translated);
        assert_eq!(result.final_command, "dir");

        let result = translator.translate("cat file.txt");
        assert!(result.translated);
        assert_eq!(result.final_command, "type file.txt");
    }
}

/// Test SSH manager functionality
#[cfg(test)]
mod ssh_manager_tests {
    use super::*;
    use furnace::ssh_manager::SshManager;

    #[test]
    fn test_ssh_manager_creation() {
        let manager = SshManager::new();
        assert!(manager.is_ok(), "Failed to create SSH manager");
    }

    #[test]
    fn test_ssh_manager_visibility() {
        let mut manager = SshManager::new().unwrap();
        assert!(!manager.visible);

        manager.toggle();
        assert!(manager.visible);

        manager.toggle();
        assert!(!manager.visible);
    }
}

/// Test URL handler functionality
#[cfg(test)]
mod url_handler_tests {
    use super::*;
    use furnace::url_handler::UrlHandler;

    #[test]
    fn test_url_handler_detection() {
        let text = "Check out https://github.com/RyAnPr1Me/furnace for more info!";
        let urls = UrlHandler::detect_urls(text);

        assert!(!urls.is_empty(), "Failed to detect URLs");
        assert_eq!(urls.len(), 1);
        assert!(urls[0].url.contains("github.com"));
    }

    #[test]
    fn test_url_handler_multiple_urls() {
        let text = "Visit https://github.com and http://example.com";
        let urls = UrlHandler::detect_urls(text);

        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_url_handler_enabled() {
        let handler = UrlHandler::new(true);
        assert!(handler.is_enabled());

        let handler = UrlHandler::new(false);
        assert!(!handler.is_enabled());
    }
}

/// Test UI components
#[cfg(test)]
mod ui_tests {
    use super::*;
    use furnace::ui::{
        autocomplete::Autocomplete, command_palette::CommandPalette,
        resource_monitor::ResourceMonitor, themes::ThemeManager,
    };

    #[test]
    fn test_command_palette_creation() {
        let palette = CommandPalette::new();
        assert!(!palette.visible);
    }

    #[test]
    fn test_resource_monitor_creation() {
        let mut monitor = ResourceMonitor::new();
        // Should be able to get stats
        let stats = monitor.get_stats();
        // Stats is a struct with CPU/memory info
        assert!(stats.cpu_usage >= 0.0);
        assert!(stats.memory_used > 0);
    }

    #[test]
    fn test_autocomplete_creation() {
        let mut autocomplete = Autocomplete::new();
        // Should have common commands cached
        let suggestions = autocomplete.get_suggestions("gi");
        assert!(!suggestions.is_empty(), "No autocomplete suggestions");
    }

    #[test]
    fn test_theme_manager() {
        let manager = ThemeManager::new();
        let themes = manager.available_theme_names();

        // Should have at least 3 built-in themes
        assert!(themes.len() >= 3, "Not enough themes available");
        assert!(themes.contains(&"dark".to_string()));
        assert!(themes.contains(&"light".to_string()));
        assert!(themes.contains(&"nord".to_string()));
    }
}

/// Test session management
#[cfg(test)]
mod session_tests {
    use super::*;
    use furnace::session::SessionManager;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert!(manager.is_ok(), "Failed to create session manager");
    }
}

/// Test plugin system
#[cfg(test)]
mod plugin_tests {
    use super::*;
    use furnace::plugins::PluginManager;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        // Should create without errors - no public methods to check count
        // Just verify it can be created
        drop(manager);
    }
}

/// Test keybindings
#[cfg(test)]
mod keybinding_tests {
    use super::*;
    use furnace::keybindings::KeybindingManager;

    #[test]
    fn test_keybinding_manager_creation() {
        let _manager = KeybindingManager::new();
        // Should create without errors
    }
}

/// Test progress bar
#[cfg(test)]
mod progress_bar_tests {
    use super::*;
    use furnace::progress_bar::ProgressBar;

    #[test]
    fn test_progress_bar_start_stop() {
        let mut bar = ProgressBar::new();
        assert!(!bar.visible);

        bar.start("test command".to_string());
        assert!(bar.visible);

        bar.stop();
        assert!(!bar.visible);
    }
}
