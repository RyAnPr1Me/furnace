use std::collections::VecDeque;

/// Advanced autocomplete system for shell commands
#[allow(dead_code)] // Public API for autocomplete feature
pub struct Autocomplete {
    history: VecDeque<String>,
    current_suggestions: Vec<String>,
    current_index: usize,
    prefix: String,
}

impl Autocomplete {
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(1000),
            current_suggestions: Vec::new(),
            current_index: 0,
            prefix: String::new(),
        }
    }

    /// Add command to history
    #[allow(dead_code)] // Public API
    pub fn add_to_history(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }

        // Remove duplicates
        if let Some(pos) = self.history.iter().position(|x| x == &command) {
            self.history.remove(pos);
        }

        // Add to front
        self.history.push_front(command);

        // Limit size
        if self.history.len() > 1000 {
            self.history.pop_back();
        }
    }

    /// Get suggestions for prefix
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn get_suggestions(&mut self, prefix: &str) -> Vec<String> {
        self.prefix = prefix.to_string();
        self.current_index = 0;

        // Get commands from history that start with prefix
        let mut suggestions: Vec<String> = self
            .history
            .iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .take(10)
            .cloned()
            .collect();

        // Add common commands if prefix matches
        suggestions.extend(
            Self::common_commands()
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .take(5)
                .map(|s| s.to_string()),
        );

        // Remove duplicates
        suggestions.sort();
        suggestions.dedup();

        self.current_suggestions = suggestions.clone();
        suggestions
    }

    /// Get next suggestion (for Tab completion)
    #[allow(dead_code)] // Public API
    pub fn next_suggestion(&mut self) -> Option<String> {
        if self.current_suggestions.is_empty() {
            return None;
        }

        let suggestion = self.current_suggestions[self.current_index].clone();
        self.current_index = (self.current_index + 1) % self.current_suggestions.len();
        Some(suggestion)
    }

    /// Get previous suggestion (for Shift+Tab)
    #[allow(dead_code)] // Public API
    pub fn previous_suggestion(&mut self) -> Option<String> {
        if self.current_suggestions.is_empty() {
            return None;
        }

        if self.current_index == 0 {
            self.current_index = self.current_suggestions.len() - 1;
        } else {
            self.current_index -= 1;
        }

        Some(self.current_suggestions[self.current_index].clone())
    }

    /// Common commands for different platforms
    #[allow(dead_code)] // Used by get_suggestions
    fn common_commands() -> Vec<&'static str> {
        vec![
            // Unix/Linux/Mac
            "ls",
            "cd",
            "pwd",
            "mkdir",
            "rm",
            "cp",
            "mv",
            "cat",
            "grep",
            "find",
            "chmod",
            "chown",
            "ps",
            "kill",
            "top",
            "df",
            "du",
            "tar",
            "zip",
            "unzip",
            "git",
            "ssh",
            "curl",
            "wget",
            "vim",
            "nano",
            "echo",
            "export",
            // Windows
            "dir",
            "cls",
            "type",
            "copy",
            "move",
            "del",
            "xcopy",
            "attrib",
            "tasklist",
            "taskkill",
            "ipconfig",
            "ping",
            "netstat",
            // PowerShell
            "Get-Command",
            "Get-Help",
            "Get-Process",
            "Get-Service",
            "Set-Location",
            "Get-ChildItem",
            "Remove-Item",
            "Copy-Item",
            "Move-Item",
            // Git
            "git status",
            "git add",
            "git commit",
            "git push",
            "git pull",
            "git clone",
            "git branch",
            "git checkout",
            "git merge",
            "git log",
            // Docker
            "docker ps",
            "docker images",
            "docker run",
            "docker build",
            "docker exec",
            "docker-compose up",
            "docker-compose down",
            // NPM/Node
            "npm install",
            "npm start",
            "npm run",
            "npm test",
            "npx",
            // Cargo/Rust
            "cargo build",
            "cargo run",
            "cargo test",
            "cargo bench",
            "cargo check",
        ]
    }

    /// Get history (for up/down arrow navigation)
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn get_history(&self) -> &VecDeque<String> {
        &self.history
    }

    /// Clear history
    #[allow(dead_code)] // Public API
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.current_suggestions.clear();
        self.current_index = 0;
    }
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_creation() {
        let autocomplete = Autocomplete::new();
        assert_eq!(autocomplete.history.len(), 0);
    }

    #[test]
    fn test_add_to_history() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("ls -la".to_string());
        autocomplete.add_to_history("cd /home".to_string());

        assert_eq!(autocomplete.history.len(), 2);
        assert_eq!(autocomplete.history[0], "cd /home");
    }

    #[test]
    fn test_get_suggestions() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("git status".to_string());
        autocomplete.add_to_history("git commit".to_string());

        let suggestions = autocomplete.get_suggestions("git");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("git")));
    }

    #[test]
    fn test_navigation() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("cmd1".to_string());
        autocomplete.add_to_history("cmd2".to_string());

        autocomplete.get_suggestions("cmd");

        let first = autocomplete.next_suggestion();
        assert!(first.is_some());

        let second = autocomplete.next_suggestion();
        assert!(second.is_some());
        assert_ne!(first, second);
    }
}
