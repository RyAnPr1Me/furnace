use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::collections::VecDeque;

/// Maximum number of commands to keep in history
const MAX_HISTORY: usize = 100;

/// Command palette for quick command execution and search
pub struct CommandPalette {
    pub visible: bool,
    pub input: String,
    pub suggestions: Vec<CommandSuggestion>,
    pub selected_index: usize,
    matcher: SkimMatcherV2,
    history: VecDeque<String>,
    commands: Vec<Command>,
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
    pub fn new() -> Self {
        let mut palette = Self {
            visible: false,
            input: String::new(),
            suggestions: Vec::new(),
            selected_index: 0,
            matcher: SkimMatcherV2::default(),
            history: VecDeque::with_capacity(MAX_HISTORY),
            commands: Vec::new(),
        };
        
        palette.load_default_commands();
        palette
    }

    /// Load default built-in commands
    fn load_default_commands(&mut self) {
        self.commands = vec![
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
                description: "Change theme".to_string(),
                aliases: vec!["t".to_string()],
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
        ];
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

    /// Update input and refresh suggestions
    pub fn update_input(&mut self, input: String) {
        self.input = input;
        self.refresh_suggestions();
    }

    /// Refresh suggestions based on current input
    fn refresh_suggestions(&mut self) {
        if self.input.is_empty() {
            // Show recent history when no input
            self.suggestions = self.history
                .iter()
                .take(10)
                .map(|cmd| CommandSuggestion {
                    command: cmd.clone(),
                    description: "Recent command".to_string(),
                    score: 100,
                })
                .collect();
        } else {
            // Fuzzy search through commands
            let mut suggestions: Vec<CommandSuggestion> = self.commands
                .iter()
                .filter_map(|cmd| {
                    // Try matching command name
                    let name_score = self.matcher.fuzzy_match(&cmd.name, &self.input);
                    
                    // Try matching aliases
                    let alias_score = cmd.aliases
                        .iter()
                        .filter_map(|alias| self.matcher.fuzzy_match(alias, &self.input))
                        .max();
                    
                    // Use best score
                    let score = name_score.or(alias_score)?;
                    
                    Some(CommandSuggestion {
                        command: cmd.name.clone(),
                        description: cmd.description.clone(),
                        score,
                    })
                })
                .collect();

            // Sort by score (descending)
            suggestions.sort_by(|a, b| b.score.cmp(&a.score));
            
            self.suggestions = suggestions.into_iter().take(10).collect();
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
        palette.update_input("nt".to_string());
        
        assert!(!palette.suggestions.is_empty());
        assert!(palette.suggestions[0].command.contains("new-tab"));
    }

    #[test]
    fn test_navigation() {
        let mut palette = CommandPalette::new();
        palette.update_input("t".to_string());
        
        assert_eq!(palette.selected_index, 0);
        palette.select_next();
        assert_eq!(palette.selected_index, 1);
        palette.select_previous();
        assert_eq!(palette.selected_index, 0);
    }
}
