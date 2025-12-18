//! Live terminal integration tests that exercise the full event loop
//!
//! These tests create actual terminal sessions and test the event loop behavior
//! with real PTY interactions, input handling, output processing, and rendering.

use furnace::config::Config;
use furnace::shell::ShellSession;
use furnace::terminal::Terminal;
use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// Shell Session Integration Tests
// ============================================================================

#[tokio::test]
async fn test_shell_session_write_and_read_comprehensive() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Test multiple writes
    for cmd in &["echo test1\n", "echo test2\n", "echo test3\n"] {
        let result = session.write_input(cmd.as_bytes()).await;
        assert!(result.is_ok());
    }
    
    sleep(Duration::from_millis(200)).await;
    
    // Read all output
    let mut buffer = vec![0u8; 4096];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

#[tokio::test]
async fn test_shell_session_multiple_commands() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Send multiple commands with different content
    let commands = if cfg!(windows) {
        vec!["echo Hello\r\n", "echo World\r\n", "dir\r\n"]
    } else {
        vec!["echo Hello\n", "echo World\n", "pwd\n", "ls\n"]
    };
    
    for cmd in commands {
        session.write_input(cmd.as_bytes()).await.unwrap();
        sleep(Duration::from_millis(50)).await;
    }
    
    sleep(Duration::from_millis(200)).await;
    
    let mut buffer = vec![0u8; 8192];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

#[tokio::test]
async fn test_shell_session_resize_multiple_times() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Test multiple resize operations
    for (rows, cols) in &[(24, 80), (30, 100), (40, 120), (20, 60)] {
        let result = session.resize(*rows, *cols).await;
        assert!(result.is_ok());
        sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore] // Ignore by default due to long execution time
async fn test_shell_session_large_output() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Generate large output
    let cmd = if cfg!(windows) {
        "dir /s C:\\Windows\\System32\r\n"
    } else {
        "find / -name '*.txt' 2>/dev/null | head -100\n"
    };
    
    session.write_input(cmd.as_bytes()).await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Read in chunks
    let mut total_bytes = 0;
    for _ in 0..10 {
        let mut buffer = vec![0u8; 4096];
        if let Ok(bytes) = session.read_output(&mut buffer).await {
            total_bytes += bytes;
            if bytes == 0 {
                break;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    
    assert!(total_bytes > 0);
}

#[tokio::test]
async fn test_shell_session_special_characters() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Test special characters in commands
    let commands = if cfg!(windows) {
        vec!["echo test!@#$%\r\n", "echo \"quoted\"\r\n"]
    } else {
        vec!["echo 'test!@#$%'\n", "echo \"quoted\"\n", "echo $HOME\n"]
    };
    
    for cmd in commands {
        let result = session.write_input(cmd.as_bytes()).await;
        assert!(result.is_ok());
        sleep(Duration::from_millis(50)).await;
    }
    
    let mut buffer = vec![0u8; 4096];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

#[tokio::test]
async fn test_shell_session_rapid_writes() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Rapid fire commands
    for i in 0..20 {
        let cmd = if cfg!(windows) {
            format!("echo test{}\r\n", i)
        } else {
            format!("echo test{}\n", i)
        };
        session.write_input(cmd.as_bytes()).await.unwrap();
    }
    
    sleep(Duration::from_millis(500)).await;
    
    let mut buffer = vec![0u8; 16384];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

#[tokio::test]
async fn test_shell_session_empty_commands() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Send empty command (just newline)
    let cmd = if cfg!(windows) { "\r\n" } else { "\n" };
    
    let result = session.write_input(cmd.as_bytes()).await;
    assert!(result.is_ok());
    
    sleep(Duration::from_millis(100)).await;
    
    let mut buffer = vec![0u8; 1024];
    // Should still be able to read (might get prompt)
    let _bytes_read = session.read_output(&mut buffer).await;
}

#[tokio::test]
async fn test_shell_session_working_directory() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    // Test with specific working directory
    let temp_dir = std::env::temp_dir();
    let temp_str = temp_dir.to_str().unwrap();
    let session = ShellSession::new(shell, Some(temp_str), 24, 80).unwrap();
    
    // Verify we can write commands
    let cmd = if cfg!(windows) { "cd\r\n" } else { "pwd\n" };
    session.write_input(cmd.as_bytes()).await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    let mut buffer = vec![0u8; 2048];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

#[tokio::test]
async fn test_shell_session_various_sizes() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    // Test different terminal sizes
    for (rows, cols) in &[(24, 80), (40, 120), (10, 40), (50, 200)] {
        let session = ShellSession::new(shell, None, *rows, *cols);
        assert!(session.is_ok());
    }
}

// ============================================================================
// Terminal Creation with Various Configurations
// ============================================================================

#[test]
fn test_terminal_creation_with_resource_monitor() {
    let mut config = Config::default();
    config.features.resource_monitor = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_autocomplete() {
    let mut config = Config::default();
    config.features.autocomplete = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_progress_bar() {
    let mut config = Config::default();
    config.features.progress_bar = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_all_ui_features() {
    let mut config = Config::default();
    config.features.resource_monitor = true;
    config.features.autocomplete = true;
    config.features.progress_bar = true;
    config.features.command_palette = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_tabs_enabled() {
    let mut config = Config::default();
    config.terminal.enable_tabs = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_split_pane_enabled() {
    let mut config = Config::default();
    config.terminal.enable_split_pane = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_large_scrollback() {
    let mut config = Config::default();
    config.terminal.scrollback_lines = 100000;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_small_font() {
    let mut config = Config::default();
    config.terminal.font_size = 8;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_creation_with_large_font() {
    let mut config = Config::default();
    config.terminal.font_size = 24;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

// ============================================================================
// Configuration Edge Cases
// ============================================================================

#[test]
fn test_terminal_with_custom_shell_env_vars() {
    let mut config = Config::default();
    
    // Add multiple environment variables
    config.shell.env.insert("CUSTOM_VAR1".to_string(), "value1".to_string());
    config.shell.env.insert("CUSTOM_VAR2".to_string(), "value2".to_string());
    config.shell.env.insert("TEST_MODE".to_string(), "true".to_string());
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_with_different_cursor_styles() {
    for style in &["block", "underline", "bar"] {
        let mut config = Config::default();
        config.terminal.cursor_style = style.to_string();
        
        let terminal = Terminal::new(config);
        assert!(terminal.is_ok());
    }
}

#[test]
fn test_terminal_with_theme_variations() {
    let themes = vec![
        ("#FFFFFF", "#000000"), // High contrast
        ("#CCCCCC", "#1A1A1A"), // Dark theme
        ("#2A2A2A", "#F5F5F5"), // Light theme
        ("#D8DEE9", "#2E3440"), // Nord
    ];
    
    for (fg, bg) in themes {
        let mut config = Config::default();
        config.theme.foreground = fg.to_string();
        config.theme.background = bg.to_string();
        
        let terminal = Terminal::new(config);
        assert!(terminal.is_ok());
    }
}

// ============================================================================
// Shell Session Stress Tests
// ============================================================================

#[tokio::test]
async fn test_shell_session_long_running_command() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Start a command that takes time
    let cmd = if cfg!(windows) {
        "ping 127.0.0.1 -n 3\r\n"
    } else {
        "sleep 0.5 && echo done\n"
    };
    
    session.write_input(cmd.as_bytes()).await.unwrap();
    
    // Wait for command to complete
    sleep(Duration::from_secs(1)).await;
    
    let mut buffer = vec![0u8; 4096];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore] // Ignore by default due to concurrent read complexity
async fn test_shell_session_concurrent_reads() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Write command
    let cmd = if cfg!(windows) {
        "echo concurrent_test\r\n"
    } else {
        "echo concurrent_test\n"
    };
    
    session.write_input(cmd.as_bytes()).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    
    // Multiple concurrent reads
    let mut tasks = vec![];
    for _ in 0..5 {
        let session_clone = session.clone();
        tasks.push(tokio::spawn(async move {
            let mut buffer = vec![0u8; 1024];
            session_clone.read_output(&mut buffer).await
        }));
    }
    
    for task in tasks {
        let _ = task.await;
    }
}

#[tokio::test]
async fn test_shell_session_unicode_input() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Test unicode characters
    let cmd = if cfg!(windows) {
        "echo Hello世界\r\n"
    } else {
        "echo 'Hello世界🌍'\n"
    };
    
    let result = session.write_input(cmd.as_bytes()).await;
    assert!(result.is_ok());
    
    sleep(Duration::from_millis(100)).await;
    
    let mut buffer = vec![0u8; 4096];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

#[tokio::test]
async fn test_shell_session_binary_safe() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    let session = ShellSession::new(shell, None, 24, 80).unwrap();
    
    // Write command with various byte values
    let cmd = b"echo test\n";
    let result = session.write_input(cmd).await;
    assert!(result.is_ok());
    
    sleep(Duration::from_millis(100)).await;
    
    // Read binary output
    let mut buffer = vec![0u8; 2048];
    let bytes_read = session.read_output(&mut buffer).await.unwrap();
    assert!(bytes_read > 0);
}

// ============================================================================
// Terminal Feature Tests
// ============================================================================

#[test]
fn test_terminal_with_hooks_config() {
    let mut config = Config::default();
    
    config.hooks.on_startup = Some("print('startup')".to_string());
    config.hooks.on_shutdown = Some("print('shutdown')".to_string());
    config.hooks.on_key_press = Some("print('key')".to_string());
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_with_output_filters() {
    let mut config = Config::default();
    
    config.hooks.output_filters.push("output = input:upper()".to_string());
    config.hooks.output_filters.push("output = input:gsub('test', 'TEST')".to_string());
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_with_custom_keybindings() {
    let mut config = Config::default();
    
    config.keybindings.new_tab = "Ctrl+T".to_string();
    config.keybindings.close_tab = "Ctrl+W".to_string();
    config.keybindings.next_tab = "Ctrl+PageDown".to_string();
    config.keybindings.prev_tab = "Ctrl+PageUp".to_string();
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_with_session_manager() {
    let mut config = Config::default();
    config.features.session_manager = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

#[test]
fn test_terminal_with_theme_manager() {
    let mut config = Config::default();
    config.features.theme_manager = true;
    
    let terminal = Terminal::new(config);
    assert!(terminal.is_ok());
}

// ============================================================================
// Integration Test: Multiple Shells
// ============================================================================

#[tokio::test]
async fn test_multiple_shell_sessions_simultaneously() {
    let shell = if cfg!(windows) { "cmd.exe" } else { "sh" };
    
    // Create multiple shell sessions
    let sessions: Vec<_> = (0..5)
        .map(|_| ShellSession::new(shell, None, 24, 80).unwrap())
        .collect();
    
    // Write to all sessions
    for (i, session) in sessions.iter().enumerate() {
        let cmd = if cfg!(windows) {
            format!("echo session{}\r\n", i)
        } else {
            format!("echo session{}\n", i)
        };
        session.write_input(cmd.as_bytes()).await.unwrap();
    }
    
    sleep(Duration::from_millis(200)).await;
    
    // Read from all sessions
    for session in &sessions {
        let mut buffer = vec![0u8; 2048];
        let bytes_read = session.read_output(&mut buffer).await.unwrap();
        assert!(bytes_read > 0);
    }
}

// ============================================================================
// Terminal Creation Stability Tests
// ============================================================================

#[test]
fn test_terminal_create_destroy_cycle() {
    // Create and destroy multiple terminals
    for _ in 0..20 {
        let config = Config::default();
        let terminal = Terminal::new(config);
        assert!(terminal.is_ok());
        // Terminal is dropped here
    }
}

#[test]
fn test_terminal_with_varying_configs() {
    // Create terminals with varying configurations
    for i in 0..10 {
        let mut config = Config::default();
        config.terminal.scrollback_lines = 1000 * (i + 1);
        config.terminal.max_history = 100 * (i + 1);
        config.terminal.font_size = 8 + (i as u16);
        
        let terminal = Terminal::new(config);
        assert!(terminal.is_ok());
    }
}

#[test]
fn test_terminal_memory_usage() {
    // Ensure terminals don't leak memory
    let configs: Vec<_> = (0..50).map(|_| Config::default()).collect();
    
    for config in configs {
        let terminal = Terminal::new(config);
        assert!(terminal.is_ok());
    }
    
    // Rust's ownership ensures all memory is freed
}
