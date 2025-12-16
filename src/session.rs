use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Session manager for saving and restoring terminal sessions
pub struct SessionManager {
    sessions_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Local>,
    pub tabs: Vec<TabState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub output: String,
    pub working_dir: Option<String>,
    pub active: bool,
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
    pub fn delete_session(&self, id: &str) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{id}.json"));
        fs::remove_file(&session_file).context("Failed to delete session file")?;

        Ok(())
    }

    /// Get sessions directory path
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
            created_at: Local::now(),
            tabs: vec![TabState {
                output: "test output".to_string(),
                working_dir: Some("/home/user".to_string()),
                active: true,
            }],
        };

        manager.save_session(&session).unwrap();
        let loaded = manager.load_session("test-session").unwrap();

        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.name, session.name);
        assert_eq!(loaded.tabs.len(), 1);

        // Cleanup
        manager.delete_session("test-session").ok();
    }

    #[test]
    fn test_list_sessions() {
        let manager = SessionManager::new().unwrap();

        // Create a test session
        let session = SavedSession {
            id: "list-test-session".to_string(),
            name: "List Test".to_string(),
            created_at: Local::now(),
            tabs: vec![TabState {
                output: "test".to_string(),
                working_dir: None,
                active: true,
            }],
        };

        manager.save_session(&session).unwrap();

        // List should include our session
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.iter().any(|s| s.id == "list-test-session"));

        // Cleanup
        manager.delete_session("list-test-session").ok();
    }

    #[test]
    fn test_delete_session() {
        let manager = SessionManager::new().unwrap();

        let session = SavedSession {
            id: "delete-test".to_string(),
            name: "Delete Test".to_string(),
            created_at: Local::now(),
            tabs: vec![],
        };

        manager.save_session(&session).unwrap();
        assert!(manager.load_session("delete-test").is_ok());

        manager.delete_session("delete-test").unwrap();
        assert!(manager.load_session("delete-test").is_err());
    }

    #[test]
    fn test_sessions_dir() {
        let manager = SessionManager::new().unwrap();
        let dir = manager.sessions_dir();
        assert!(dir.exists());
    }

    #[test]
    fn test_default_implementation() {
        let manager = SessionManager::default();
        assert!(manager.sessions_dir().exists());
    }

    #[test]
    fn test_session_with_multiple_tabs() {
        let manager = SessionManager::new().unwrap();

        let session = SavedSession {
            id: "multi-tab-test".to_string(),
            name: "Multi Tab Test".to_string(),
            created_at: Local::now(),
            tabs: vec![
                TabState {
                    output: "tab1 output".to_string(),
                    working_dir: Some("/home/user".to_string()),
                    active: false,
                },
                TabState {
                    output: "tab2 output".to_string(),
                    working_dir: Some("/tmp".to_string()),
                    active: true,
                },
                TabState {
                    output: "tab3 output".to_string(),
                    working_dir: None,
                    active: false,
                },
            ],
        };

        manager.save_session(&session).unwrap();
        let loaded = manager.load_session("multi-tab-test").unwrap();

        assert_eq!(loaded.tabs.len(), 3);
        assert!(!loaded.tabs[0].active);
        assert!(loaded.tabs[1].active);
        assert_eq!(loaded.tabs[1].working_dir, Some("/tmp".to_string()));

        // Cleanup
        manager.delete_session("multi-tab-test").ok();
    }

    #[test]
    fn test_load_nonexistent_session() {
        let manager = SessionManager::new().unwrap();
        let result = manager.load_session("nonexistent-session-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_with_special_characters() {
        let manager = SessionManager::new().unwrap();

        let session = SavedSession {
            id: "special-chars-test".to_string(),
            name: "Test with \"quotes\" and \\ backslash".to_string(),
            created_at: Local::now(),
            tabs: vec![TabState {
                output: "output with\nnewlines\tand\ttabs".to_string(),
                working_dir: Some("/path/with spaces/and'quotes".to_string()),
                active: true,
            }],
        };

        manager.save_session(&session).unwrap();
        let loaded = manager.load_session("special-chars-test").unwrap();

        assert_eq!(loaded.name, session.name);
        assert!(loaded.tabs[0].output.contains('\n'));
        assert!(loaded.tabs[0].output.contains('\t'));

        // Cleanup
        manager.delete_session("special-chars-test").ok();
    }
}
