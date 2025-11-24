use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Command translator for cross-platform command compatibility
/// Translates Linux commands to Windows equivalents and vice versa
#[derive(Debug, Clone)]
pub struct CommandTranslator {
    enabled: bool,
    current_os: OsType,
    // Use references to static maps instead of cloning
    _phantom: std::marker::PhantomData<()>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OsType {
    Linux,
    Windows,
    MacOs,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CommandMapping {
    pub target_cmd: &'static str,
    pub description: &'static str,
    pub arg_translator: fn(&str) -> String,
}

#[derive(Debug)]
pub struct TranslationResult {
    pub translated: bool,
    pub original_command: String,
    pub final_command: String,
    pub description: String,
}

impl Default for CommandMapping {
    fn default() -> Self {
        Self {
            target_cmd: "",
            description: "",
            arg_translator: |args| args.to_string(),
        }
    }
}

// Argument translators for different command types
fn identity_args(args: &str) -> String {
    args.to_string()
}

fn ls_to_dir_args(args: &str) -> String {
    let args = args.trim();
    
    if args.is_empty() {
        return String::new();
    }
    
    // Pre-allocate capacity based on input length
    let mut result = String::with_capacity(args.len() + 10);
    
    // Common ls flags to dir equivalents
    // Note: dir without flags shows multiple columns (like ls), /W is not needed
    if args.contains("-a") {
        result.push_str(" /A");
    }
    
    // Extract paths (non-flag arguments) - single pass
    for part in args.split_whitespace() {
        if !part.starts_with('-') {
            result.push(' ');
            result.push_str(part);
        }
    }
    
    result
}

fn dir_to_ls_args(args: &str) -> String {
    let args = args.trim();
    
    if args.is_empty() {
        return String::new();
    }
    
    // Pre-allocate capacity
    let mut result = String::with_capacity(args.len() + 10);
    
    // Common dir flags to ls equivalents
    if args.contains("/W") || args.contains("/w") {
        result.push_str(" -l");
    }
    if args.contains("/A") || args.contains("/a") {
        result.push_str(" -a");
    }
    
    // Extract paths (non-flag arguments)
    for part in args.split_whitespace() {
        if !part.starts_with('/') && !part.starts_with('-') {
            result.push(' ');
            result.push_str(part);
        }
    }
    
    result
}

fn rm_to_del_args(args: &str) -> String {
    let args = args.trim();
    
    if args.is_empty() {
        return String::new();
    }
    
    // Pre-allocate capacity
    let mut result = String::with_capacity(args.len() + 15);
    
    // Handle combined flags like -rf
    let has_recursive = args.contains("-r");
    let has_force = args.contains("-f");
    
    if has_recursive {
        result.push_str(" /S");
    }
    if has_force {
        result.push_str(" /F /Q");
    }
    
    // Extract paths
    for part in args.split_whitespace() {
        if !part.starts_with('-') {
            result.push(' ');
            result.push_str(part);
        }
    }
    
    result
}

fn del_to_rm_args(args: &str) -> String {
    let args = args.trim();
    
    if args.is_empty() {
        return String::new();
    }
    
    // Pre-allocate capacity
    let mut result = String::with_capacity(args.len() + 10);
    
    // Common del flags to rm equivalents
    if args.contains("/S") || args.contains("/s") {
        result.push_str(" -r");
    }
    if args.contains("/F") || args.contains("/f") || args.contains("/Q") || args.contains("/q") {
        result.push_str(" -f");
    }
    
    // Extract paths
    for part in args.split_whitespace() {
        if !part.starts_with('/') && !part.starts_with('-') {
            result.push(' ');
            result.push_str(part);
        }
    }
    
    result
}

fn cp_to_copy_args(args: &str) -> String {
    // cp handles arguments differently than copy
    // For simplicity, pass through as-is
    args.to_string()
}

fn copy_to_cp_args(args: &str) -> String {
    // copy handles arguments differently than cp
    // For simplicity, pass through as-is  
    args.to_string()
}

fn cat_to_type_args(args: &str) -> String {
    // type is simpler, just pass filenames
    args.to_string()
}

fn type_to_cat_args(args: &str) -> String {
    // cat can handle multiple files, pass through
    args.to_string()
}

// Static command mappings
static LINUX_TO_WINDOWS_MAP: Lazy<HashMap<&'static str, CommandMapping>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    m.insert("ls", CommandMapping {
        target_cmd: "dir",
        description: "List directory contents",
        arg_translator: ls_to_dir_args,
    });
    
    m.insert("pwd", CommandMapping {
        target_cmd: "cd",
        description: "Print working directory",
        arg_translator: identity_args,
    });
    
    m.insert("cat", CommandMapping {
        target_cmd: "type",
        description: "Display file contents",
        arg_translator: cat_to_type_args,
    });
    
    m.insert("rm", CommandMapping {
        target_cmd: "del",
        description: "Remove files",
        arg_translator: rm_to_del_args,
    });
    
    m.insert("cp", CommandMapping {
        target_cmd: "copy",
        description: "Copy files",
        arg_translator: cp_to_copy_args,
    });
    
    m.insert("mv", CommandMapping {
        target_cmd: "move",
        description: "Move/rename files",
        arg_translator: identity_args,
    });
    
    m.insert("clear", CommandMapping {
        target_cmd: "cls",
        description: "Clear screen",
        arg_translator: identity_args,
    });
    
    m.insert("touch", CommandMapping {
        target_cmd: "type nul",
        description: "Create empty file",
        arg_translator: |args| format!(" > {}", args.trim()),
    });
    
    m.insert("grep", CommandMapping {
        target_cmd: "findstr",
        description: "Search text patterns",
        arg_translator: identity_args,
    });
    
    m.insert("which", CommandMapping {
        target_cmd: "where",
        description: "Locate command",
        arg_translator: identity_args,
    });
    
    m.insert("ps", CommandMapping {
        target_cmd: "tasklist",
        description: "List processes",
        arg_translator: identity_args,
    });
    
    m.insert("kill", CommandMapping {
        target_cmd: "taskkill",
        description: "Terminate process",
        arg_translator: |args| {
            // Convert kill -9 PID to taskkill /F /PID PID
            let parts: Vec<&str> = args.split_whitespace().collect();
            if args.contains("-9") {
                if let Some(pid) = parts.last() {
                    if !pid.is_empty() && *pid != "-9" {
                        return format!(" /F /PID {}", pid);
                    }
                }
            } else if let Some(pid) = parts.last() {
                if !pid.is_empty() {
                    return format!(" /PID {}", pid);
                }
            }
            String::new()
        },
    });
    
    m.insert("df", CommandMapping {
        target_cmd: "powershell Get-PSDrive -PSProvider FileSystem",
        description: "Display disk space",
        arg_translator: |_| String::new(),
    });
    
    m.insert("du", CommandMapping {
        target_cmd: "dir",
        description: "Display disk usage",
        arg_translator: |args| format!(" /S {}", args.trim()),
    });
    
    m.insert("head", CommandMapping {
        target_cmd: "powershell Get-Content",
        description: "Display first lines of file",
        arg_translator: |args| format!("{} -Head 10", args.trim()),
    });
    
    m.insert("tail", CommandMapping {
        target_cmd: "powershell Get-Content",
        description: "Display last lines of file",
        arg_translator: |args| format!("{} -Tail 10", args.trim()),
    });
    
    m.insert("chmod", CommandMapping {
        target_cmd: "icacls",
        description: "Change file permissions",
        arg_translator: identity_args,
    });
    
    m.insert("chown", CommandMapping {
        target_cmd: "icacls",
        description: "Change file owner",
        arg_translator: identity_args,
    });
    
    m
});

static WINDOWS_TO_LINUX_MAP: Lazy<HashMap<&'static str, CommandMapping>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    m.insert("dir", CommandMapping {
        target_cmd: "ls",
        description: "List directory contents",
        arg_translator: dir_to_ls_args,
    });
    
    m.insert("cd", CommandMapping {
        target_cmd: "pwd",
        description: "Print working directory (when used alone)",
        arg_translator: identity_args,
    });
    
    m.insert("type", CommandMapping {
        target_cmd: "cat",
        description: "Display file contents",
        arg_translator: type_to_cat_args,
    });
    
    m.insert("del", CommandMapping {
        target_cmd: "rm",
        description: "Remove files",
        arg_translator: del_to_rm_args,
    });
    
    m.insert("erase", CommandMapping {
        target_cmd: "rm",
        description: "Remove files",
        arg_translator: del_to_rm_args,
    });
    
    m.insert("copy", CommandMapping {
        target_cmd: "cp",
        description: "Copy files",
        arg_translator: copy_to_cp_args,
    });
    
    m.insert("move", CommandMapping {
        target_cmd: "mv",
        description: "Move/rename files",
        arg_translator: identity_args,
    });
    
    m.insert("cls", CommandMapping {
        target_cmd: "clear",
        description: "Clear screen",
        arg_translator: identity_args,
    });
    
    m.insert("findstr", CommandMapping {
        target_cmd: "grep",
        description: "Search text patterns",
        arg_translator: identity_args,
    });
    
    m.insert("where", CommandMapping {
        target_cmd: "which",
        description: "Locate command",
        arg_translator: identity_args,
    });
    
    m.insert("tasklist", CommandMapping {
        target_cmd: "ps",
        description: "List processes",
        arg_translator: identity_args,
    });
    
    m.insert("taskkill", CommandMapping {
        target_cmd: "kill",
        description: "Terminate process",
        arg_translator: |args| {
            // Convert taskkill /F /PID PID to kill -9 PID
            if args.contains("/F") || args.contains("/f") {
                let pid = args.split_whitespace()
                    .skip_while(|&s| s.to_lowercase() != "/pid")
                    .nth(1)
                    .unwrap_or("");
                format!(" -9 {}", pid)
            } else {
                args.split_whitespace()
                    .skip_while(|&s| s.to_lowercase() != "/pid")
                    .nth(1)
                    .map(|pid| format!(" {}", pid))
                    .unwrap_or_default()
            }
        },
    });
    
    m
});

impl CommandTranslator {
    /// Create a new command translator
    pub fn new(enabled: bool) -> Self {
        let current_os = Self::detect_os();
        
        Self {
            enabled,
            current_os,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Detect the current operating system
    fn detect_os() -> OsType {
        if cfg!(target_os = "windows") {
            OsType::Windows
        } else if cfg!(target_os = "linux") {
            OsType::Linux
        } else if cfg!(target_os = "macos") {
            OsType::MacOs
        } else {
            OsType::Unknown
        }
    }
    
    /// Translate a command if translation is enabled and applicable
    pub fn translate(&self, command: &str) -> TranslationResult {
        let command = command.trim();
        
        // Fast path: disabled or empty command
        if !self.enabled || command.is_empty() {
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
            };
        }
        
        // Parse command into parts - avoid collecting into Vec
        let mut parts = command.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => {
                return TranslationResult {
                    translated: false,
                    original_command: command.to_string(),
                    final_command: command.to_string(),
                    description: String::new(),
                };
            }
        };
        
        let args = command.strip_prefix(cmd).unwrap_or("").trim();
        
        // Determine which direction to translate
        let (mapping, should_translate) = match self.current_os {
            OsType::Windows => {
                // On Windows, translate Linux commands to Windows
                (LINUX_TO_WINDOWS_MAP.get(cmd), true)
            },
            OsType::Linux | OsType::MacOs => {
                // On Linux/Mac, translate Windows commands to Linux
                (WINDOWS_TO_LINUX_MAP.get(cmd), true)
            },
            OsType::Unknown => (None, false),
        };
        
        if !should_translate {
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
            };
        }
        
        // Don't translate cd command with arguments on any platform
        if cmd == "cd" && !args.is_empty() {
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
            };
        }
        
        // Special case: translate bare "cd" to "pwd" on Windows (shows current directory)
        if cmd == "cd" && self.current_os == OsType::Windows && args.is_empty() {
            // On Windows, bare "cd" shows current directory like pwd
            // Let it through for translation
        } else if cmd == "cd" && args.is_empty() {
            // On Linux/Mac, bare "cd" changes to home directory, not pwd
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
            };
        }
        
        if let Some(mapping) = mapping {
            let translated_args = (mapping.arg_translator)(args);
            let final_cmd = format!("{}{}", mapping.target_cmd, translated_args);
            
            TranslationResult {
                translated: true,
                original_command: command.to_string(),
                final_command: final_cmd.trim().to_string(),
                description: mapping.description.to_string(),
            }
        } else {
            TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
            }
        }
    }
    
    /// Enable or disable command translation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if translation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get current OS type
    pub fn current_os(&self) -> OsType {
        self.current_os
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_translator_creation() {
        let translator = CommandTranslator::new(true);
        assert!(translator.is_enabled());
    }
    
    #[test]
    fn test_os_detection() {
        let translator = CommandTranslator::new(true);
        let os = translator.current_os();
        
        #[cfg(target_os = "windows")]
        assert_eq!(os, OsType::Windows);
        
        #[cfg(target_os = "linux")]
        assert_eq!(os, OsType::Linux);
        
        #[cfg(target_os = "macos")]
        assert_eq!(os, OsType::MacOs);
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_basic() {
        let translator = CommandTranslator::new(true);
        
        let result = translator.translate("ls");
        assert!(result.translated);
        assert_eq!(result.final_command, "dir");
        
        let result = translator.translate("pwd");
        assert!(result.translated);
        assert_eq!(result.final_command, "cd");
        
        let result = translator.translate("clear");
        assert!(result.translated);
        assert_eq!(result.final_command, "cls");
    }
    
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_basic() {
        let translator = CommandTranslator::new(true);
        
        let result = translator.translate("dir");
        assert!(result.translated);
        assert_eq!(result.final_command, "ls");
        
        let result = translator.translate("cls");
        assert!(result.translated);
        assert_eq!(result.final_command, "clear");
    }
    
    #[test]
    fn test_disabled_translator() {
        let translator = CommandTranslator::new(false);
        
        let result = translator.translate("ls");
        assert!(!result.translated);
        assert_eq!(result.original_command, "ls");
        assert_eq!(result.final_command, "ls");
    }
    
    #[test]
    fn test_unknown_command() {
        let translator = CommandTranslator::new(true);
        
        let result = translator.translate("unknown_command");
        assert!(!result.translated);
        assert_eq!(result.original_command, "unknown_command");
        assert_eq!(result.final_command, "unknown_command");
    }
    
    #[test]
    fn test_empty_command() {
        let translator = CommandTranslator::new(true);
        
        let result = translator.translate("");
        assert!(!result.translated);
        assert_eq!(result.final_command, "");
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_ls_with_args() {
        let translator = CommandTranslator::new(true);
        
        let result = translator.translate("ls -la");
        assert!(result.translated);
        assert!(result.final_command.contains("dir"));
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_cat_with_file() {
        let translator = CommandTranslator::new(true);
        
        let result = translator.translate("cat file.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("type"));
        assert!(result.final_command.contains("file.txt"));
    }
}
