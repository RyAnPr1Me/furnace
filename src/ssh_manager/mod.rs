//! SSH Connection Manager
//!
//! Manages SSH connections with persistent storage and filtering capabilities.
//!
//! # Features
//! - Store and retrieve SSH connection configurations
//! - Parse SSH commands automatically
//! - Filter connections by name, host, or username
//! - Persistent JSON storage
//! - Connection management (add/remove/select)
//!
//! # Storage
//! Connections are stored in `~/.furnace/ssh_connections.json`

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// SSH connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub identity_file: Option<String>,
    pub last_used: Option<String>,
}

impl SshConnection {
    /// Format as SSH command string
    #[must_use]
    pub fn to_command(&self) -> String {
        use std::fmt::Write;
        let mut cmd = format!("ssh {}@{}", self.username, self.host);

        if self.port != 22 {
            let _ = write!(cmd, " -p {}", self.port);
        }

        if let Some(ref key) = self.identity_file {
            let _ = write!(cmd, " -i {key}");
        }

        cmd
    }
}

/// SSH Manager for storing and managing SSH connections
#[derive(Debug)]
pub struct SshManager {
    connections: HashMap<String, SshConnection>,
    config_path: PathBuf,
    pub visible: bool,
    pub selected_index: usize,
    pub filtered_connections: Vec<String>,
    pub filter_input: String,
}

impl SshManager {
    /// Create a new SSH manager
    pub fn new() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        let connections = Self::load_connections(&config_path)?;
        let filtered_connections: Vec<String> = connections.keys().cloned().collect();

        Ok(Self {
            connections,
            config_path,
            visible: false,
            selected_index: 0,
            filtered_connections,
            filter_input: String::new(),
        })
    }

    /// Get default config path
    fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".furnace").join("ssh_connections.json"))
    }

    /// Load connections from file
    fn load_connections(path: &PathBuf) -> Result<HashMap<String, SshConnection>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let contents = fs::read_to_string(path).context("Failed to read SSH connections file")?;

        let connections: HashMap<String, SshConnection> =
            serde_json::from_str(&contents).context("Failed to parse SSH connections")?;

        Ok(connections)
    }

    /// Save connections to file
    pub fn save_connections(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let contents = serde_json::to_string_pretty(&self.connections)
            .context("Failed to serialize SSH connections")?;

        fs::write(&self.config_path, contents).context("Failed to write SSH connections file")?;

        Ok(())
    }

    /// Toggle SSH manager visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.update_filter();
        }
    }

    /// Add or update a connection
    pub fn add_connection(&mut self, name: String, conn: SshConnection) {
        self.connections.insert(name, conn);
        self.update_filter();
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, name: &str) {
        self.connections.remove(name);
        self.update_filter();
    }

    /// Get a connection by name
    pub fn get_connection(&self, name: &str) -> Option<&SshConnection> {
        self.connections.get(name)
    }

    /// Get all connection names
    #[allow(dead_code)] // Public API
    pub fn connection_names(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }

    /// Update filter based on input
    pub fn update_filter(&mut self) {
        self.filtered_connections.clear();

        if self.filter_input.is_empty() {
            // Fast path: no filtering needed
            self.filtered_connections
                .extend(self.connections.keys().cloned());
        } else {
            let filter_lower = self.filter_input.to_lowercase();
            self.filtered_connections.extend(
                self.connections
                    .iter()
                    .filter(|(name, conn)| {
                        name.to_lowercase().contains(&filter_lower)
                            || conn.host.to_lowercase().contains(&filter_lower)
                            || conn.username.to_lowercase().contains(&filter_lower)
                    })
                    .map(|(name, _)| name.clone()),
            );
        }

        self.filtered_connections.sort_unstable(); // Faster for types that don't need stability

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_connections.len()
            && !self.filtered_connections.is_empty()
        {
            self.selected_index = self.filtered_connections.len() - 1;
        }
    }

    /// Select next connection
    pub fn select_next(&mut self) {
        if !self.filtered_connections.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_connections.len();
        }
    }

    /// Select previous connection
    pub fn select_previous(&mut self) {
        if !self.filtered_connections.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_connections.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Get currently selected connection
    #[must_use]
    pub fn get_selected(&self) -> Option<&SshConnection> {
        if self.selected_index < self.filtered_connections.len() {
            let name = &self.filtered_connections[self.selected_index];
            self.connections.get(name)
        } else {
            None
        }
    }

    /// Parse SSH command and create connection
    #[must_use]
    pub fn parse_ssh_command(command: &str) -> Option<SshConnection> {
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() || parts[0] != "ssh" {
            return None;
        }

        let mut username = String::new();
        let mut host = String::new();
        let mut ssh_port = 22;
        let mut identity_file = None;

        let mut i = 1;
        while i < parts.len() {
            match parts[i] {
                "-p" => {
                    if i + 1 < parts.len() {
                        ssh_port = parts[i + 1].parse().unwrap_or(22);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-i" => {
                    if i + 1 < parts.len() {
                        identity_file = Some(parts[i + 1].to_string());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                arg => {
                    if arg.contains('@') {
                        let conn_parts: Vec<&str> = arg.split('@').collect();
                        if conn_parts.len() == 2 {
                            username = conn_parts[0].to_string();
                            host = conn_parts[1].to_string();
                        }
                    } else if host.is_empty() {
                        host = arg.to_string();
                    }
                    i += 1;
                }
            }
        }

        if host.is_empty() {
            return None;
        }
        
        let name = if username.is_empty() {
            host.clone()
        } else {
            format!("{username}@{host}")
        };

        // Use current system user if no username specified
        let default_username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string());

        Some(SshConnection {
            name,
            host,
            port: ssh_port,
            username: if username.is_empty() {
                default_username
            } else {
                username
            },
            identity_file,
            last_used: Some(chrono::Utc::now().to_rfc3339()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_connection_to_command() {
        let conn = SshConnection {
            name: "test".to_string(),
            host: "example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            identity_file: None,
            last_used: None,
        };

        assert_eq!(conn.to_command(), "ssh user@example.com");
    }

    #[test]
    fn test_ssh_connection_with_port() {
        let conn = SshConnection {
            name: "test".to_string(),
            host: "example.com".to_string(),
            port: 2222,
            username: "user".to_string(),
            identity_file: None,
            last_used: None,
        };

        assert!(conn.to_command().contains("-p 2222"));
    }

    #[test]
    fn test_ssh_connection_with_key() {
        let conn = SshConnection {
            name: "test".to_string(),
            host: "example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            identity_file: Some("/path/to/key".to_string()),
            last_used: None,
        };

        assert!(conn.to_command().contains("-i /path/to/key"));
    }

    #[test]
    fn test_parse_ssh_command_basic() {
        let conn = SshManager::parse_ssh_command("ssh user@example.com");
        assert!(conn.is_some());

        let conn = conn.unwrap();
        assert_eq!(conn.username, "user");
        assert_eq!(conn.host, "example.com");
        assert_eq!(conn.port, 22);
    }

    #[test]
    fn test_parse_ssh_command_with_port() {
        let conn = SshManager::parse_ssh_command("ssh -p 2222 user@example.com");
        assert!(conn.is_some());

        let conn = conn.unwrap();
        assert_eq!(conn.port, 2222);
    }

    #[test]
    fn test_parse_ssh_command_with_key() {
        let conn = SshManager::parse_ssh_command("ssh -i ~/.ssh/id_rsa user@example.com");
        assert!(conn.is_some());

        let conn = conn.unwrap();
        assert_eq!(conn.identity_file, Some("~/.ssh/id_rsa".to_string()));
    }

    #[test]
    fn test_parse_ssh_command_invalid() {
        let conn = SshManager::parse_ssh_command("not_ssh user@example.com");
        assert!(conn.is_none());
    }
}
