use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::VecDeque;
use std::sync::OnceLock;

/// Maximum number of commands to keep in history
const MAX_HISTORY: usize = 100;

/// Lazy-initialized command list for better startup performance
static DEFAULT_COMMANDS: OnceLock<Vec<Command>> = OnceLock::new();

/// Command palette for quick command execution and search
pub struct CommandPalette {
    pub visible: bool,
    pub input: String,
    pub suggestions: Vec<CommandSuggestion>,
    pub selected_index: usize,
    matcher: SkimMatcherV2,
    history: VecDeque<String>,
    // Commands reference the static data instead of duplicating
}

#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    pub command: String,
    pub description: String,
    pub score: i64,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
}

impl CommandPalette {
    #[must_use]
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::with_capacity(64), // Pre-allocate for typical command length
            suggestions: Vec::with_capacity(10), // Pre-allocate for typical suggestion count
            selected_index: 0,
            matcher: SkimMatcherV2::default(),
            history: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    /// Load default built-in commands (lazy initialization)
    fn get_commands() -> &'static [Command] {
        DEFAULT_COMMANDS.get_or_init(|| {
            vec![
                Command {
                    name: "new-tab".to_string(),
                    description: "Create a new tab".to_string(),
                    aliases: vec!["tab".to_string(), "nt".to_string()],
                },
                Command {
                    name: "close-tab".to_string(),
                    description: "Close current tab".to_string(),
                    aliases: vec!["ct".to_string()],
                },
                Command {
                    name: "split-horizontal".to_string(),
                    description: "Split pane horizontally".to_string(),
                    aliases: vec!["sh".to_string(), "hsplit".to_string()],
                },
                Command {
                    name: "split-vertical".to_string(),
                    description: "Split pane vertically".to_string(),
                    aliases: vec!["sv".to_string(), "vsplit".to_string()],
                },
                Command {
                    name: "theme".to_string(),
                    description: "Cycle to next theme".to_string(),
                    aliases: vec!["t".to_string(), "next-theme".to_string()],
                },
                Command {
                    name: "theme dark".to_string(),
                    description: "Switch to dark theme".to_string(),
                    aliases: vec!["dark".to_string()],
                },
                Command {
                    name: "theme light".to_string(),
                    description: "Switch to light theme".to_string(),
                    aliases: vec!["light".to_string()],
                },
                Command {
                    name: "theme nord".to_string(),
                    description: "Switch to Nord theme".to_string(),
                    aliases: vec!["nord".to_string()],
                },
                Command {
                    name: "clear".to_string(),
                    description: "Clear terminal".to_string(),
                    aliases: vec!["cls".to_string()],
                },
                Command {
                    name: "config".to_string(),
                    description: "Open configuration".to_string(),
                    aliases: vec!["settings".to_string()],
                },
                Command {
                    name: "help".to_string(),
                    description: "Show help".to_string(),
                    aliases: vec!["?".to_string()],
                },
                Command {
                    name: "quit".to_string(),
                    description: "Quit application".to_string(),
                    aliases: vec!["exit".to_string(), "q".to_string()],
                },
            ]
        })
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.input.clear();
            self.suggestions.clear();
            self.selected_index = 0;
        }
    }

    /// Update input and refresh suggestions (optimized)
    #[allow(dead_code)] // Used in tests
    pub fn update_input(&mut self, input: &str) {
        self.input = input.to_string();
        self.refresh_suggestions();
    }

    /// Refresh suggestions based on current input (optimized with early returns)
    pub fn refresh_suggestions(&mut self) {
        self.suggestions.clear(); // Reuse existing vector capacity

        if self.input.is_empty() {
            // Show recent history when no input (limit to 10 for performance)
            self.suggestions
                .extend(self.history.iter().take(10).map(|cmd| CommandSuggestion {
                    command: cmd.clone(),
                    description: "Recent command".to_string(),
                    score: 100,
                }));
        } else {
            // Fuzzy search through commands (optimized with early scoring)
            let commands = Self::get_commands();
            let input_lower = self.input.to_lowercase(); // Cache lowercase for faster comparison

            for cmd in commands {
                // Try exact prefix match first (faster than fuzzy)
                if cmd.name.starts_with(&input_lower) {
                    self.suggestions.push(CommandSuggestion {
                        command: cmd.name.clone(),
                        description: cmd.description.clone(),
                        score: 1000, // High score for exact prefix match
                    });
                    continue;
                }

                // Try matching command name with fuzzy matcher
                if let Some(score) = self.matcher.fuzzy_match(&cmd.name, &self.input) {
                    self.suggestions.push(CommandSuggestion {
                        command: cmd.name.clone(),
                        description: cmd.description.clone(),
                        score,
                    });
                    continue;
                }

                // Try matching aliases
                for alias in &cmd.aliases {
                    if let Some(score) = self.matcher.fuzzy_match(alias, &self.input) {
                        self.suggestions.push(CommandSuggestion {
                            command: cmd.name.clone(),
                            description: cmd.description.clone(),
                            score,
                        });
                        break; // Only add once per command
                    }
                }
            }

            // Sort by score (descending) - use unstable sort for better performance
            self.suggestions
                .sort_unstable_by(|a, b| b.score.cmp(&a.score));

            // Keep only top 10 suggestions for UI performance
            self.suggestions.truncate(10);
        }

        // Reset selection
        if !self.suggestions.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index < self.suggestions.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Get currently selected suggestion
    #[must_use]
    pub fn get_selected(&self) -> Option<&CommandSuggestion> {
        self.suggestions.get(self.selected_index)
    }

    /// Execute selected command
    pub fn execute_selected(&mut self) -> Option<String> {
        if let Some(suggestion) = self.get_selected() {
            let command = suggestion.command.clone();
            self.add_to_history(command.clone());
            self.visible = false;
            Some(command)
        } else {
            None
        }
    }

    /// Add command to history
    fn add_to_history(&mut self, command: String) {
        // Remove if already exists
        if let Some(pos) = self.history.iter().position(|x| x == &command) {
            self.history.remove(pos);
        }

        // Add to front
        self.history.push_front(command);

        // Limit size
        if self.history.len() > MAX_HISTORY {
            self.history.pop_back();
        }
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_creation() {
        let palette = CommandPalette::new();
        assert!(!palette.visible);
        assert_eq!(palette.input, "");
    }

    #[test]
    fn test_toggle() {
        let mut palette = CommandPalette::new();
        palette.toggle();
        assert!(palette.visible);
        palette.toggle();
        assert!(!palette.visible);
    }

    #[test]
    fn test_fuzzy_search() {
        let mut palette = CommandPalette::new();
        palette.update_input("nt");

        assert!(!palette.suggestions.is_empty());
        assert!(palette.suggestions[0].command.contains("new-tab"));
    }

    #[test]
    fn test_navigation() {
        let mut palette = CommandPalette::new();
        palette.update_input("t");

        assert_eq!(palette.selected_index, 0);
        palette.select_next();
        assert_eq!(palette.selected_index, 1);
        palette.select_previous();
        assert_eq!(palette.selected_index, 0);
    }
}
