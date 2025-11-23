use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process::Command;

/// Git Integration Plugin - Provides git commands in Furnace
#[repr(C)]
pub struct GitPlugin {
    name: *const c_char,
    version: *const c_char,
    initialized: bool,
}

impl GitPlugin {
    fn new() -> Self {
        Self {
            name: CString::new("Git Integration").unwrap().into_raw(),
            version: CString::new("0.1.0").unwrap().into_raw(),
            initialized: false,
        }
    }

    fn init(&mut self) {
        self.initialized = true;
        println!("[Git Plugin] Initialized!");
    }

    fn handle_command(&self, command: &str) -> Option<String> {
        if !self.initialized {
            return Some("Plugin not initialized".to_string());
        }

        match command {
            "git-status" | "gs" => self.git_status(),
            "git-branch" | "gb" => self.git_branch(),
            "git-log" | "gl" => self.git_log(),
            "git-diff" | "gd" => self.git_diff(),
            "git-remote" | "gr" => self.git_remote(),
            "git-info" | "gi" => self.git_info(),
            "help" => Some(self.help_text()),
            _ => None,
        }
    }

    fn git_status(&self) -> Option<String> {
        self.run_git_command(&["status", "--short"])
    }

    fn git_branch(&self) -> Option<String> {
        match self.run_git_command(&["branch", "--show-current"]) {
            Some(branch) if !branch.trim().is_empty() => {
                Some(format!("Current branch: {}", branch.trim()))
            }
            _ => Some("Not on any branch".to_string()),
        }
    }

    fn git_log(&self) -> Option<String> {
        self.run_git_command(&["log", "--oneline", "-10"])
    }

    fn git_diff(&self) -> Option<String> {
        self.run_git_command(&["diff", "--stat"])
    }

    fn git_remote(&self) -> Option<String> {
        self.run_git_command(&["remote", "-v"])
    }

    fn git_info(&self) -> Option<String> {
        let branch = self.run_git_command(&["branch", "--show-current"])
            .unwrap_or_else(|| "unknown".to_string());
        
        let status = self.run_git_command(&["status", "--short"])
            .unwrap_or_else(|| "".to_string());
        
        let remote = self.run_git_command(&["remote", "get-url", "origin"])
            .unwrap_or_else(|| "No remote".to_string());

        Some(format!(
            "Git Repository Info:\n  Branch: {}\n  Remote: {}\n  Status: {}\n",
            branch.trim(),
            remote.trim(),
            if status.is_empty() { "Clean" } else { "Modified" }
        ))
    }

    fn run_git_command(&self, args: &[&str]) -> Option<String> {
        match Command::new("git").args(args).output() {
            Ok(output) => {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Some(format!(
                        "Git error: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            }
            Err(e) => Some(format!("Failed to run git: {}", e)),
        }
    }

    fn help_text(&self) -> String {
        r#"Git Integration Plugin Commands:
  git-status (gs)  - Show git status
  git-branch (gb)  - Show current branch
  git-log (gl)     - Show recent commits
  git-diff (gd)    - Show file changes
  git-remote (gr)  - Show remote URLs
  git-info (gi)    - Show repository info
  help             - Show this help message"#
            .to_string()
    }

    fn cleanup(&mut self) {
        self.initialized = false;
        println!("[Git Plugin] Cleaned up!");
    }
}

/// Plugin entry point
#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut GitPlugin {
    Box::into_raw(Box::new(GitPlugin::new()))
}

/// Destroy plugin
#[no_mangle]
pub extern "C" fn _plugin_destroy(plugin: *mut GitPlugin) {
    if !plugin.is_null() {
        unsafe {
            let mut plugin = Box::from_raw(plugin);
            plugin.cleanup();
        }
    }
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn _plugin_name(plugin: *const GitPlugin) -> *const c_char {
    if plugin.is_null() {
        return std::ptr::null();
    }
    unsafe { (*plugin).name }
}

/// Initialize plugin
#[no_mangle]
pub extern "C" fn _plugin_init(plugin: *mut GitPlugin) {
    if !plugin.is_null() {
        unsafe {
            (*plugin).init();
        }
    }
}

/// Handle command
#[no_mangle]
pub extern "C" fn _plugin_handle_command(
    plugin: *const GitPlugin,
    command: *const c_char,
) -> *mut c_char {
    if plugin.is_null() || command.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let command_str = CStr::from_ptr(command).to_str().unwrap();
        if let Some(result) = (*plugin).handle_command(command_str) {
            CString::new(result).unwrap().into_raw()
        } else {
            std::ptr::null_mut()
        }
    }
}
