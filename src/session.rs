use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Session manager for saving and restoring terminal sessions
#[allow(dead_code)] // Public API for session management
pub struct SessionManager {
    sessions_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Public API
pub struct SavedSession {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub working_dir: String,
    pub shell: String,
    pub env: HashMap<String, String>,
    pub tabs: Vec<TabState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Public API
pub struct TabState {
    pub name: String,
    pub working_dir: String,
    pub command_history: Vec<String>,
}

impl SessionManager {
    /// Create a new session manager
    ///
    /// # Errors
    /// Returns an error if the home directory cannot be determined or the sessions directory cannot be created
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to get home directory")?;

        let sessions_dir = home.join(".furnace").join("sessions");
        fs::create_dir_all(&sessions_dir).context("Failed to create sessions directory")?;

        Ok(Self { sessions_dir })
    }

    /// Save a session
    ///
    /// # Errors
    /// Returns an error if:
    /// - JSON serialization fails
    /// - The session file cannot be written
    #[allow(dead_code)] // Public API
    pub fn save_session(&self, session: &SavedSession) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session).context("Failed to serialize session")?;

        fs::write(&session_file, json).context("Failed to write session file")?;

        Ok(())
    }

    /// Load a session by ID
    ///
    /// # Errors
    /// Returns an error if:
    /// - The session file doesn't exist
    /// - The file cannot be read
    /// - JSON deserialization fails
    #[allow(dead_code)] // Public API
    pub fn load_session(&self, id: &str) -> Result<SavedSession> {
        let session_file = self.sessions_dir.join(format!("{id}.json"));
        let json = fs::read_to_string(&session_file).context("Failed to read session file")?;

        let session: SavedSession =
            serde_json::from_str(&json).context("Failed to parse session file")?;

        Ok(session)
    }

    /// List all saved sessions
    ///
    /// # Errors
    /// Returns an error if the sessions directory cannot be read
    #[allow(dead_code)] // Public API
    pub fn list_sessions(&self) -> Result<Vec<SavedSession>> {
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<SavedSession>(&json) {
                        sessions.push(session);
                    }
                }
            }
        }

        // Sort by creation date (most recent first)
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(sessions)
    }

    /// Delete a session
    ///
    /// # Errors
    /// Returns an error if the session file cannot be deleted
    #[allow(dead_code)] // Public API
    pub fn delete_session(&self, id: &str) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{id}.json"));
        fs::remove_file(&session_file).context("Failed to delete session file")?;

        Ok(())
    }

    /// Get sessions directory path
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }
}

impl Default for SessionManager {
    /// Create a default session manager
    ///
    /// If the home directory cannot be determined, falls back to using
    /// the system's temporary directory.
    fn default() -> Self {
        // Try to create with graceful fallback
        if let Ok(manager) = Self::new() {
            manager
        } else {
            // Fallback: use temp directory if home is unavailable
            let sessions_dir = std::env::temp_dir().join("furnace_sessions");
            let _ = std::fs::create_dir_all(&sessions_dir);
            Self { sessions_dir }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_save_and_load_session() {
        let manager = SessionManager::new().unwrap();

        let session = SavedSession {
            id: "test-session".to_string(),
            name: "Test Session".to_string(),
            created_at: Utc::now(),
            working_dir: "/home/user".to_string(),
            shell: "bash".to_string(),
            env: HashMap::new(),
            tabs: vec![],
        };

        manager.save_session(&session).unwrap();
        let loaded = manager.load_session("test-session").unwrap();

        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.name, session.name);

        // Cleanup
        manager.delete_session("test-session").ok();
    }
}
