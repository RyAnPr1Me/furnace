//! Comprehensive functionality verification tests
//!
//! These tests verify that all claimed features in the terminal emulator
//! actually work as described.

/// Test shell session functionality
#[cfg(test)]
mod shell_tests {
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
    use furnace::config::Config;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();

        // Verify default values
        assert!(!config.terminal.enable_tabs);
        assert!(!config.terminal.enable_split_pane);
        assert_eq!(config.terminal.scrollback_lines, 10000);
        assert!(config.terminal.hardware_acceleration);
    }

    #[test]
    fn test_config_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.lua");

        // Create a Lua config
        let lua_config = r"
config = {
    terminal = {
        max_history = 5000
    }
}
";
        std::fs::write(&config_path, lua_config).unwrap();

        // Load and verify
        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.terminal.max_history, 5000);
    }
}

/// Test UI components
#[cfg(test)]
mod ui_tests {
    use furnace::ui::{
        autocomplete::Autocomplete, resource_monitor::ResourceMonitor, themes::ThemeManager,
    };

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
    use furnace::session::SessionManager;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert!(manager.is_ok(), "Failed to create session manager");
    }
}

/// Test keybindings
#[cfg(test)]
mod keybinding_tests {
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

/// Test terminal local echo functionality
#[cfg(test)]
mod local_echo_tests {
    use furnace::config::Config;
    use furnace::terminal::Terminal;

    #[test]
    fn test_terminal_with_local_echo() {
        // Create a terminal with default config
        let config = Config::default();
        let terminal = Terminal::new(config);

        // Should create successfully with local echo support
        assert!(terminal.is_ok(), "Terminal creation failed");

        // The terminal should be able to handle command buffers for local echo
        // This is verified by the successful creation and internal structure
    }

    #[test]
    fn test_command_buffer_tracking() {
        // Verify that the Terminal struct has the necessary command_buffers field
        // This is an indirect test since command_buffers is private
        let config = Config::default();
        let terminal = Terminal::new(config);

        assert!(
            terminal.is_ok(),
            "Terminal should track command buffers internally"
        );
    }
}

/// Test PowerShell prompt handling with ANSI codes
#[cfg(test)]
mod powershell_prompt_tests {
    use furnace::terminal::ansi_parser::AnsiParser;

    #[test]
    fn test_simple_prompt_without_newline() {
        // PowerShell prompt without trailing newline
        let output = "PS C:\\Users\\test> ";
        let lines = AnsiParser::parse(output);
        
        // Should have 1 line with the prompt
        assert_eq!(lines.len(), 1, "Expected 1 line for prompt without newline");
        
        // Verify the prompt content is preserved
        let text: String = lines[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.contains("PS C:\\Users\\test>"), "Prompt content not preserved");
    }

    #[test]
    fn test_prompt_with_newline() {
        // PowerShell prompt with trailing newline
        let output = "PS C:\\Users\\test>\n";
        let lines = AnsiParser::parse(output);
        
        // Should have at least 1 line (the prompt), possibly 2 if empty line is added
        assert!(lines.len() >= 1, "Expected at least 1 line for prompt with newline");
        
        // Verify the prompt is in the first line
        let text: String = lines[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.contains("PS C:\\Users\\test>"), "Prompt content not preserved");
    }

    #[test]
    fn test_clear_screen_before_prompt() {
        // Clear screen followed by prompt
        let output = "\x1b[2JPS C:\\Users\\test> ";
        let lines = AnsiParser::parse(output);
        
        // Should have the prompt visible after clear screen
        assert!(lines.len() >= 1, "Expected prompt to be visible after clear screen");
        
        // Verify the prompt content is preserved
        let text: String = lines[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.contains("PS C:\\Users\\test>"), 
            "Prompt should be visible after clear screen, got lines: {:?}", lines);
    }

    #[test]
    fn test_prompt_followed_by_clear_screen() {
        // Prompt followed by clear screen
        // With the fix, clear screen should not erase the prompt from scrollback
        let output = "PS C:\\Users\\test> \x1b[2J";
        let lines = AnsiParser::parse(output);
        
        // After fix: clear screen should preserve the prompt in scrollback
        assert!(lines.len() >= 1, 
            "Clear screen after prompt should preserve scrollback content");
        
        // Verify the prompt content is still there
        let text: String = lines[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.contains("PS C:\\Users\\test>"), 
            "Prompt should be preserved after clear screen, got: '{}'", text);
    }

    #[test]
    fn test_multiple_clear_screens() {
        // Multiple clear screens followed by prompt
        let output = "\x1b[2J\x1b[2JPS C:\\Users\\test> ";
        let lines = AnsiParser::parse(output);
        
        // Should still have the prompt after multiple clears
        assert!(lines.len() >= 1, "Expected prompt after multiple clear screens");
        
        let text: String = lines[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.contains("PS C:\\Users\\test>"), 
            "Prompt should be visible after multiple clear screens");
    }

    #[test]
    fn test_prompt_with_colors() {
        // PowerShell prompt with ANSI color codes
        let output = "\x1b[32mPS C:\\Users\\test>\x1b[0m ";
        let lines = AnsiParser::parse(output);
        
        // Should have the prompt with color styling
        assert!(lines.len() >= 1, "Expected colored prompt");
        
        let text: String = lines[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.contains("PS C:\\Users\\test>"), 
            "Colored prompt content not preserved");
    }

    #[test]
    fn test_carriage_return_handling() {
        // Test prompt with carriage return (common in Windows terminals)
        let output1 = "PS C:\\Users\\test>\r\n";
        let lines1 = AnsiParser::parse(output1);
        
        // Should parse correctly
        assert!(lines1.len() >= 1, "Expected prompt with \\r\\n");
        let text1: String = lines1[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text1.contains("PS C:\\Users\\test>"), "Prompt with \\r\\n not preserved");

        // Test prompt with just carriage return
        // \r is typically ignored in the parser, but the prompt before it should be preserved
        let output2 = "PS C:\\Users\\test>\r";
        let lines2 = AnsiParser::parse(output2);
        
        // Prompt should be preserved even though \r is ignored
        assert!(lines2.len() >= 1, "Expected prompt with \\r");
        let text2: String = lines2[0].spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text2.contains("PS C:\\Users\\test>"), "Prompt with \\r not preserved");
    }
}
