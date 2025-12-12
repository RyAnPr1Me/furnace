use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced keybinding system with shell integration
#[derive(Debug, Clone)]
pub struct KeybindingManager {
    bindings: HashMap<KeyBinding, Action>,
    shell_integration: ShellIntegration,
}

/// Key binding definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub modifiers: Vec<String>,
}

/// Actions that can be triggered by keybindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // Terminal actions
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    SplitHorizontal,
    SplitVertical,

    // Navigation
    FocusNextPane,
    FocusPrevPane,

    // Editing
    Copy,
    Paste,
    SelectAll,
    Clear,

    // Search
    Search,
    SearchNext,
    SearchPrev,

    // Command palette & features
    ToggleAutocomplete,
    NextTheme,
    PrevTheme,

    // Resource monitor
    ToggleResourceMonitor,

    // Session management
    SaveSession,
    LoadSession,
    ListSessions,

    // Shell integration
    SendToShell(String),
    ExecuteCommand(String),

    // Custom
    Custom(String),

    // Lua execution
    ExecuteLua(String),
}

/// Shell integration features (infrastructure for future OSC 7/133 support)
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ShellIntegration {
    /// OSC sequences support
    pub osc_sequences: bool,

    /// Shell prompt detection
    pub prompt_detection: bool,

    /// Directory tracking
    pub directory_tracking: bool,

    /// Command tracking
    pub command_tracking: bool,

    /// Current working directory
    pub current_dir: Option<String>,

    /// Last command
    pub last_command: Option<String>,
}

impl KeybindingManager {
    /// Create new keybinding manager with defaults
    #[must_use]
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            shell_integration: ShellIntegration::default(),
        };

        manager.load_defaults();
        manager
    }

    /// Load default keybindings
    fn load_defaults(&mut self) {
        // Tab management
        self.add_binding("t", &["Ctrl"], Action::NewTab);
        self.add_binding("w", &["Ctrl"], Action::CloseTab);

        // BUG FIX #7: Ctrl+Tab is not reliably supported by crossterm on all terminals
        // Most terminals intercept Ctrl+Tab before it reaches the application.
        // Using Ctrl+PageDown/PageUp or Alt+number is more reliable, but we keep
        // these bindings as they work in some terminals (e.g., Windows Terminal).
        // Users can remap these in their config if needed.
        self.add_binding("Tab", &["Ctrl"], Action::NextTab);
        self.add_binding("Tab", &["Ctrl", "Shift"], Action::PrevTab);

        // Pane management
        self.add_binding("h", &["Ctrl", "Shift"], Action::SplitHorizontal);
        self.add_binding("v", &["Ctrl", "Shift"], Action::SplitVertical);
        self.add_binding("o", &["Ctrl"], Action::FocusNextPane);

        // Editing
        self.add_binding("c", &["Ctrl", "Shift"], Action::Copy);
        self.add_binding("v", &["Ctrl", "Shift"], Action::Paste);
        self.add_binding("a", &["Ctrl", "Shift"], Action::SelectAll);
        self.add_binding("l", &["Ctrl"], Action::Clear);

        // Search
        self.add_binding("f", &["Ctrl"], Action::Search);
        self.add_binding("n", &["Ctrl"], Action::SearchNext);
        self.add_binding("N", &["Ctrl", "Shift"], Action::SearchPrev);

        // Features
        self.add_binding("r", &["Ctrl"], Action::ToggleResourceMonitor);
        self.add_binding("Tab", &["Alt"], Action::ToggleAutocomplete);
        self.add_binding("]", &["Ctrl"], Action::NextTheme);
        self.add_binding("[", &["Ctrl"], Action::PrevTheme);

        // Session management
        // BUG FIX #16: Removed duplicate Ctrl+O binding
        // Ctrl+O is used for FocusNextPane above
        self.add_binding("s", &["Ctrl"], Action::SaveSession);
        self.add_binding("l", &["Ctrl", "Shift"], Action::LoadSession);
    }

    /// Add a keybinding
    pub fn add_binding(&mut self, key: &str, modifiers: &[&str], action: Action) {
        let binding = KeyBinding {
            key: key.to_string(),
            modifiers: modifiers
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        };
        self.bindings.insert(binding, action);
    }

    /// Parse and add a keybinding from a config string like "Ctrl+T" or "Ctrl+Shift+C"
    ///
    /// # Arguments
    /// * `combo` - Key combination string (e.g., "Ctrl+T", "Ctrl+Shift+V", "Alt+F")
    /// * `action` - Action to bind to this combination
    ///
    /// # Returns
    /// Ok(()) if binding was added successfully, Err if combo string is invalid
    ///
    /// # Examples
    /// ```ignore
    /// manager.add_binding_from_string("Ctrl+T", Action::NewTab)?;
    /// manager.add_binding_from_string("Ctrl+Shift+C", Action::Copy)?;
    /// ```
    pub fn add_binding_from_string(&mut self, combo: &str, action: Action) -> Result<(), String> {
        if combo.is_empty() {
            return Err("Empty key combination".to_string());
        }

        let parts: Vec<&str> = combo.split('+').map(str::trim).collect();
        if parts.is_empty() {
            return Err("Invalid key combination format".to_string());
        }

        // Last part is the key, everything before is modifiers
        let key = match parts.last() {
            Some(k) => *k,
            None => return Err("Invalid key combination format".to_string()),
        };
        let modifiers: Vec<&str> = parts[..parts.len().saturating_sub(1)].to_vec();

        // Validate and normalize modifiers
        let normalized_mods: Vec<&str> = modifiers
            .iter()
            .filter_map(|m| {
                match m.to_lowercase().as_str() {
                    "ctrl" | "control" => Some("Ctrl"),
                    "shift" => Some("Shift"),
                    "alt" => Some("Alt"),
                    _ => None, // Ignore unknown modifiers
                }
            })
            .collect();

        // Normalize key name
        let key_lower = key.to_lowercase();
        let normalized_key = match key_lower.as_str() {
            "tab" => "Tab",
            "enter" | "return" => "Enter",
            "esc" | "escape" => "Esc",
            "up" => "Up",
            "down" => "Down",
            "left" => "Left",
            "right" => "Right",
            "space" => " ",
            // Single character - use character count for UTF-8 safety
            k if k.chars().count() == 1 => {
                if let Some(c) = k.chars().next() {
                    // For single characters, convert to lowercase for consistency
                    let char_str = c.to_lowercase().to_string();
                    self.add_binding(&char_str, &normalized_mods, action);
                    return Ok(());
                }
                k
            }
            k => k,
        };

        self.add_binding(normalized_key, &normalized_mods, action);
        Ok(())
    }

    /// Get action for key event
    ///
    /// BUG FIX #6: Normalize character keys to lowercase for consistent matching.
    /// When Shift is pressed with Ctrl (e.g., Ctrl+Shift+C), crossterm provides
    /// an uppercase 'C', but our bindings use lowercase 'c'. This function normalizes
    /// the key to lowercase for character keys while preserving Shift in modifiers.
    #[must_use]
    pub fn get_action(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        let key_str = match code {
            // BUG FIX #6: Normalize character keys to lowercase for case-insensitive matching
            // This allows Ctrl+Shift+C to match a binding defined as ctrl+shift+c
            KeyCode::Char(c) => c.to_lowercase().to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            _ => return None,
        };

        let mut mod_vec = Vec::new();
        if modifiers.contains(KeyModifiers::CONTROL) {
            mod_vec.push("Ctrl".to_string());
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            mod_vec.push("Shift".to_string());
        }
        if modifiers.contains(KeyModifiers::ALT) {
            mod_vec.push("Alt".to_string());
        }

        let binding = KeyBinding {
            key: key_str,
            modifiers: mod_vec,
        };

        self.bindings.get(&binding).cloned()
    }

    /// Enable shell integration features (future OSC parsing support)
    pub fn enable_shell_integration(&mut self, feature: ShellIntegrationFeature, enabled: bool) {
        match feature {
            ShellIntegrationFeature::OscSequences => self.shell_integration.osc_sequences = enabled,
            ShellIntegrationFeature::PromptDetection => {
                self.shell_integration.prompt_detection = enabled;
            }
            ShellIntegrationFeature::DirectoryTracking => {
                self.shell_integration.directory_tracking = enabled;
            }
            ShellIntegrationFeature::CommandTracking => {
                self.shell_integration.command_tracking = enabled;
            }
        }
    }

    /// Update current directory from shell (future OSC 7 support)
    pub fn update_directory(&mut self, dir: String) {
        self.shell_integration.current_dir = Some(dir);
    }

    /// Update last command from shell (future OSC 133 support)
    pub fn update_last_command(&mut self, command: String) {
        self.shell_integration.last_command = Some(command);
    }

    /// Get shell integration status
    #[must_use]
    pub fn shell_integration(&self) -> &ShellIntegration {
        &self.shell_integration
    }
}

/// Shell integration features (future API for OSC parsing)
#[derive(Debug, Clone, Copy)]
pub enum ShellIntegrationFeature {
    OscSequences,
    PromptDetection,
    DirectoryTracking,
    CommandTracking,
}

impl Default for ShellIntegration {
    fn default() -> Self {
        Self {
            osc_sequences: true,
            prompt_detection: true,
            directory_tracking: true,
            command_tracking: true,
            current_dir: None,
            last_command: None,
        }
    }
}

impl Default for KeybindingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybinding_manager() {
        let manager = KeybindingManager::new();

        let action = manager.get_action(KeyCode::Char('t'), KeyModifiers::CONTROL);

        assert!(matches!(action, Some(Action::NewTab)));
    }

    #[test]
    fn test_shell_integration() {
        let mut manager = KeybindingManager::new();
        manager.update_directory("/home/user".to_string());

        assert_eq!(
            manager.shell_integration().current_dir,
            Some("/home/user".to_string())
        );
    }
}
