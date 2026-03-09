use std::collections::{HashSet, VecDeque};
use std::path::Path;
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
    /// Maximum history entries (configurable from terminal config)
    max_history: usize,
}

impl Autocomplete {
    #[must_use]
    pub fn new() -> Self {
        Self::with_max_history(1000)
    }

    /// Create autocomplete with custom max history limit
    #[must_use]
    pub fn with_max_history(max_history: usize) -> Self {
        let capacity = max_history.min(10000); // Cap at 10k for safety
        Self {
            history: VecDeque::with_capacity(capacity),
            history_set: HashSet::with_capacity(capacity),
            current_suggestions: Vec::with_capacity(20),
            current_index: 0,
            prefix: String::new(),
            cached_common_filtered: Vec::with_capacity(10),
            max_history: capacity,
        }
    }

    /// Add command to history (Bug #22: O(1) duplicate detection)
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

        // Limit size based on configured max_history
        if self.history.len() > self.max_history {
            if let Some(removed) = self.history.pop_back() {
                self.history_set.remove(&removed);
            }
        }
    }

    /// Get suggestions for prefix (Bug #6: optimized, minimal allocations)
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

        // Add file path suggestions if the prefix looks like a path or follows a path-taking command
        let path_suggestions = Self::get_path_suggestions(prefix);
        for path_str in &path_suggestions {
            let shared: SharedString = Arc::from(path_str.as_str());
            if seen.insert(shared.clone()) && self.current_suggestions.len() < 15 {
                self.current_suggestions.push(shared);
            }
        }

        // Return cloned strings (required by API)
        self.current_suggestions
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Get file path suggestions based on the current input prefix
    /// Supports: "cd dir", "cat file", "vim path", bare paths starting with / or ./ or ~/
    fn get_path_suggestions(prefix: &str) -> Vec<String> {
        // Commands that commonly take file/directory arguments
        const PATH_COMMANDS: &[&str] = &[
            "cd ", "ls ", "cat ", "vim ", "nano ", "less ", "more ", "head ", "tail ",
            "mkdir ", "rmdir ", "rm ", "cp ", "mv ", "chmod ", "chown ",
            "source ", ".", "code ", "open ",
        ];

        // Extract the path portion from the prefix
        let path_part = if let Some(stripped) = PATH_COMMANDS
            .iter()
            .find_map(|cmd| prefix.strip_prefix(cmd))
        {
            stripped
        } else if prefix.starts_with('/')
            || prefix.starts_with("./")
            || prefix.starts_with("~/")
            || prefix.starts_with("..")
        {
            prefix
        } else {
            return Vec::new();
        };

        // Expand ~ to home directory
        let expanded = if let Some(rest) = path_part.strip_prefix('~') {
            if let Some(home) = dirs::home_dir() {
                format!("{}{rest}", home.display())
            } else {
                return Vec::new();
            }
        } else {
            path_part.to_string()
        };

        // Split into directory and file prefix
        let (dir_path, file_prefix) = if expanded.ends_with('/') {
            (expanded.as_str(), "")
        } else {
            let path = Path::new(&expanded);
            let parent = path.parent().map_or(".", |p| {
                let s = p.to_str().unwrap_or(".");
                if s.is_empty() { "." } else { s }
            });
            let prefix_part = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            (parent, prefix_part)
        };

        let mut results = Vec::with_capacity(10);

        // Read directory entries
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.take(50).flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip hidden files unless the prefix explicitly starts with .
                    if name.starts_with('.') && !file_prefix.starts_with('.') {
                        continue;
                    }
                    if name.starts_with(file_prefix) {
                        let full_path = if dir_path == "." {
                            name.to_string()
                        } else {
                            format!("{dir_path}/{name}")
                        };
                        // Add trailing / for directories
                        let suggestion = if entry.path().is_dir() {
                            format!("{full_path}/")
                        } else {
                            full_path
                        };
                        // Reconstruct the full command prefix + suggestion
                        let cmd_prefix = &prefix[..prefix.len() - path_part.len()];
                        results.push(format!("{cmd_prefix}{suggestion}"));
                        if results.len() >= 10 {
                            break;
                        }
                    }
                }
            }
        }

        results
    }

    /// Get next suggestion (Bug #27: return reference, avoid clone)
    pub fn next_suggestion(&mut self) -> Option<&str> {
        if self.current_suggestions.is_empty() {
            return None;
        }

        let suggestion = &self.current_suggestions[self.current_index];
        self.current_index = (self.current_index + 1) % self.current_suggestions.len();
        Some(suggestion)
    }

    /// Get next suggestion as owned String (legacy API)
    pub fn next_suggestion_owned(&mut self) -> Option<String> {
        self.next_suggestion().map(std::string::ToString::to_string)
    }

    /// Get previous suggestion (Bug #27: return reference, avoid clone)
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
    pub fn previous_suggestion_owned(&mut self) -> Option<String> {
        self.previous_suggestion()
            .map(std::string::ToString::to_string)
    }

    /// Get history (for up/down arrow navigation)
    pub fn get_history(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(std::convert::AsRef::as_ref)
    }

    /// Get history length
    #[must_use]
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Clear history
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

        let first = autocomplete
            .next_suggestion()
            .map(std::string::ToString::to_string);
        assert!(first.is_some());

        let second = autocomplete
            .next_suggestion()
            .map(std::string::ToString::to_string);
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

    #[test]
    fn test_with_max_history() {
        let mut autocomplete = Autocomplete::with_max_history(5);

        // Add 10 commands
        for i in 0..10 {
            autocomplete.add_to_history(format!("cmd{}", i));
        }

        // Should only keep last 5
        assert_eq!(autocomplete.history_len(), 5);
    }

    #[test]
    fn test_max_history_limit() {
        // Test with small limit
        let mut autocomplete = Autocomplete::with_max_history(3);

        autocomplete.add_to_history("cmd1".to_string());
        autocomplete.add_to_history("cmd2".to_string());
        autocomplete.add_to_history("cmd3".to_string());
        autocomplete.add_to_history("cmd4".to_string());

        // Should only have 3 entries (oldest removed)
        assert_eq!(autocomplete.history_len(), 3);

        let history: Vec<_> = autocomplete.get_history().collect();
        // Most recent first
        assert_eq!(history[0], "cmd4");
        assert_eq!(history[1], "cmd3");
        assert_eq!(history[2], "cmd2");
        // cmd1 should be removed
    }

    #[test]
    fn test_clear_history_resets_state() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("one".to_string());
        autocomplete.add_to_history("two".to_string());

        autocomplete.clear_history();

        assert_eq!(autocomplete.history_len(), 0);
        assert!(autocomplete.get_history().next().is_none());
        assert!(autocomplete.next_suggestion().is_none());

        // After clearing, suggestions should still come from common commands
        let suggestions = autocomplete.get_suggestions("git");
        assert!(suggestions.iter().any(|s| s.starts_with("git")));
        assert!(autocomplete.next_suggestion().is_some());
        assert!(autocomplete.previous_suggestion().is_some());
    }

    #[test]
    fn test_ignores_empty_or_whitespace_commands() {
        let mut autocomplete = Autocomplete::new();
        autocomplete.add_to_history("   ".to_string());
        autocomplete.add_to_history(String::new());

        assert_eq!(autocomplete.history_len(), 0);
    }

    #[test]
    fn test_suggestions_cap_at_limit() {
        let mut autocomplete = Autocomplete::new();

        // Prefix "c" matches many built-in commands; should cap at 15
        let suggestions = autocomplete.get_suggestions("c");
        assert!(suggestions.len() <= 15);
    }
}
