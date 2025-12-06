use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

/// Common commands - cached as &'static str (Bug #26: avoid re-allocation)
static COMMON_COMMANDS: &[&str] = &[
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
    // Git shortcuts
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
];

/// Bug #28: Use Arc<str> for shared strings to avoid cloning
type SharedString = Arc<str>;

/// Advanced autocomplete system for shell commands
/// Bug #6: Optimized for performance - O(1) dedup, minimal allocations
#[allow(dead_code)]
pub struct Autocomplete {
    /// Bug #28: History uses `Arc<str>` for efficient sharing
    history: VecDeque<SharedString>,
    /// Bug #22: `HashSet` for O(1) duplicate detection
    history_set: HashSet<SharedString>,
    /// Current suggestions (references to history or static commands)
    current_suggestions: Vec<SharedString>,
    /// Current index in suggestions
    current_index: usize,
    /// Cached prefix for incremental filtering
    prefix: String,
    /// Bug #26: Cached filtered common commands (reused across calls)
    cached_common_filtered: Vec<&'static str>,
}

impl Autocomplete {
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(1000),
            history_set: HashSet::with_capacity(1000),
            current_suggestions: Vec::with_capacity(20),
            current_index: 0,
            prefix: String::new(),
            cached_common_filtered: Vec::with_capacity(10),
        }
    }

    /// Add command to history (Bug #22: O(1) duplicate detection)
    #[allow(dead_code)]
    pub fn add_to_history(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }

        let shared: SharedString = command.into();

        // Bug #22: O(1) duplicate check instead of linear scan
        if self.history_set.contains(&shared) {
            // Move existing entry to front (remove and re-add)
            if let Some(pos) = self.history.iter().position(|x| *x == shared) {
                self.history.remove(pos);
            }
        } else {
            self.history_set.insert(shared.clone());
        }

        // Add to front
        self.history.push_front(shared);

        // Limit size
        if self.history.len() > 1000 {
            if let Some(removed) = self.history.pop_back() {
                self.history_set.remove(&removed);
            }
        }
    }

    /// Get suggestions for prefix (Bug #6: optimized, minimal allocations)
    #[allow(dead_code)]
    #[must_use]
    pub fn get_suggestions(&mut self, prefix: &str) -> Vec<String> {
        self.prefix.clear();
        self.prefix.push_str(prefix);
        self.current_index = 0;
        self.current_suggestions.clear();

        // Bug #6: Use HashSet to deduplicate without sort
        let mut seen = HashSet::with_capacity(20);

        // Add matching history entries (already deduplicated)
        for cmd in self.history.iter().take(10) {
            if cmd.starts_with(prefix) && seen.insert(cmd.clone()) {
                self.current_suggestions.push(cmd.clone());
            }
        }

        // Bug #26: Filter common commands without allocation
        self.cached_common_filtered.clear();
        for cmd in COMMON_COMMANDS.iter().copied() {
            if cmd.starts_with(prefix) {
                self.cached_common_filtered.push(cmd);
            }
        }

        // Add common commands that aren't already in suggestions
        // Note: For static strings, we create Arc<str> which is efficient since
        // it's just a pointer + reference count for the static data
        for &cmd in &self.cached_common_filtered {
            let shared: SharedString = Arc::from(cmd);
            if seen.insert(shared.clone()) {
                self.current_suggestions.push(shared);
                if self.current_suggestions.len() >= 15 {
                    break;
                }
            }
        }

        // Return cloned strings (required by API)
        self.current_suggestions
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Get next suggestion (Bug #27: return reference, avoid clone)
    #[allow(dead_code)]
    pub fn next_suggestion(&mut self) -> Option<&str> {
        if self.current_suggestions.is_empty() {
            return None;
        }

        let suggestion = &self.current_suggestions[self.current_index];
        self.current_index = (self.current_index + 1) % self.current_suggestions.len();
        Some(suggestion)
    }

    /// Get next suggestion as owned String (legacy API)
    #[allow(dead_code)]
    pub fn next_suggestion_owned(&mut self) -> Option<String> {
        self.next_suggestion().map(std::string::ToString::to_string)
    }

    /// Get previous suggestion (Bug #27: return reference, avoid clone)
    #[allow(dead_code)]
    pub fn previous_suggestion(&mut self) -> Option<&str> {
        if self.current_suggestions.is_empty() {
            return None;
        }

        if self.current_index == 0 {
            self.current_index = self.current_suggestions.len() - 1;
        } else {
            self.current_index -= 1;
        }

        Some(&self.current_suggestions[self.current_index])
    }

    /// Get previous suggestion as owned String (legacy API)
    #[allow(dead_code)]
    pub fn previous_suggestion_owned(&mut self) -> Option<String> {
        self.previous_suggestion().map(std::string::ToString::to_string)
    }

    /// Get history (for up/down arrow navigation)
    #[allow(dead_code)]
    pub fn get_history(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(std::convert::AsRef::as_ref)
    }

    /// Get history length
    #[allow(dead_code)]
    #[must_use]
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Clear history
    #[allow(dead_code)]
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_set.clear();
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
        assert_eq!(autocomplete.history_len(), 0);
    }

    #[test]
    fn test_add_to_history() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("ls -la".to_string());
        autocomplete.add_to_history("cd /home".to_string());

        assert_eq!(autocomplete.history_len(), 2);
        let history: Vec<_> = autocomplete.get_history().collect();
        assert_eq!(history[0], "cd /home");
    }

    #[test]
    fn test_add_to_history_dedup() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("ls -la".to_string());
        autocomplete.add_to_history("cd /home".to_string());
        autocomplete.add_to_history("ls -la".to_string()); // Duplicate

        // Should still have 2 entries, with "ls -la" moved to front
        assert_eq!(autocomplete.history_len(), 2);
        let history: Vec<_> = autocomplete.get_history().collect();
        assert_eq!(history[0], "ls -la");
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

        let suggestions = autocomplete.get_suggestions("cmd");
        assert!(!suggestions.is_empty());

        let first = autocomplete.next_suggestion().map(std::string::ToString::to_string);
        assert!(first.is_some());

        let second = autocomplete.next_suggestion().map(std::string::ToString::to_string);
        assert!(second.is_some());
        assert_ne!(first, second);
    }

    #[test]
    fn test_previous_navigation() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("cmd1".to_string());
        autocomplete.add_to_history("cmd2".to_string());

        let suggestions = autocomplete.get_suggestions("cmd");
        assert!(!suggestions.is_empty());

        // Go forward twice
        let _ = autocomplete.next_suggestion();
        let _ = autocomplete.next_suggestion();

        // Go back
        let prev = autocomplete.previous_suggestion();
        assert!(prev.is_some());
    }

    #[test]
    fn test_common_commands_cached() {
        let mut autocomplete = Autocomplete::new();

        // First call
        let suggestions1 = autocomplete.get_suggestions("git");
        assert!(suggestions1.iter().any(|s| s.starts_with("git")));

        // Second call - should use cached common commands
        let suggestions2 = autocomplete.get_suggestions("git");
        assert!(suggestions2.iter().any(|s| s.starts_with("git")));
    }
}
