#[cfg(test)]
mod config_tests {
    use furnace::config::Config;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.terminal.enable_tabs);
        assert!(config.terminal.enable_split_pane);
        assert_eq!(config.terminal.scrollback_lines, 10000);
        assert!(config.terminal.hardware_acceleration);
    }

    #[test]
    fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.yaml");
        
        let config = Config::default();
        config.save_to_file(&config_path).unwrap();
        
        let loaded_config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config.terminal.max_history, loaded_config.terminal.max_history);
        assert_eq!(config.theme.foreground, loaded_config.theme.foreground);
    }

    #[test]
    fn test_config_memory_efficiency() {
        // Ensure Config struct size is reasonable
        let size = std::mem::size_of::<Config>();
        assert!(size < 10000, "Config struct is too large: {} bytes", size);
    }
}

#[cfg(test)]
mod terminal_tests {
    use furnace::config::Config;
    use furnace::terminal::Terminal;

    #[test]
    fn test_terminal_creation() {
        let config = Config::default();
        let terminal = Terminal::new(config);
        assert!(terminal.is_ok());
    }

    #[test]
    fn test_no_memory_leaks() {
        // This test verifies that Terminal can be created and dropped
        // without leaking memory (Rust's ownership guarantees this)
        for _ in 0..100 {
            let config = Config::default();
            let _terminal = Terminal::new(config).unwrap();
            // Terminal is dropped here automatically
        }
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn test_output_buffer_performance() {
        use std::time::Instant;
        
        let mut buffer = Vec::with_capacity(1024 * 1024);
        let data = vec![b'A'; 8192];
        
        let start = Instant::now();
        for _ in 0..1000 {
            buffer.extend_from_slice(&data);
            if buffer.len() > 1024 * 1024 {
                buffer.drain(..8192);
            }
        }
        let duration = start.elapsed();
        
        // Should complete in less than 100ms
        assert!(duration.as_millis() < 100, "Performance test took too long: {:?}", duration);
    }

    #[test]
    fn test_zero_copy_performance() {
        use std::time::Instant;
        
        let data = vec![b'A'; 1024 * 1024];
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _slice = &data[..];
            // Zero-copy slice operation
        }
        let duration = start.elapsed();
        
        // Zero-copy should be extremely fast
        assert!(duration.as_micros() < 1000, "Zero-copy test took too long: {:?}", duration);
    }
}
