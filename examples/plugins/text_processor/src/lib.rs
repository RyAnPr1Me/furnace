use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Text Processor Plugin - Text manipulation utilities
#[repr(C)]
pub struct TextProcessorPlugin {
    name: *const c_char,
    version: *const c_char,
    initialized: bool,
}

impl TextProcessorPlugin {
    fn new() -> Self {
        Self {
            name: CString::new("Text Processor").unwrap().into_raw(),
            version: CString::new("0.1.0").unwrap().into_raw(),
            initialized: false,
        }
    }

    fn init(&mut self) {
        self.initialized = true;
        println!("[Text Processor Plugin] Initialized!");
    }

    fn handle_command(&self, command: &str) -> Option<String> {
        if !self.initialized {
            return Some("Plugin not initialized".to_string());
        }

        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        let cmd = parts.get(0).copied()?;
        let text = parts.get(1).copied().unwrap_or("");

        match cmd {
            "upper" => Some(text.to_uppercase()),
            "lower" => Some(text.to_lowercase()),
            "reverse" => Some(text.chars().rev().collect()),
            "count" => Some(format!("Characters: {}, Words: {}, Lines: {}",
                text.len(),
                text.split_whitespace().count(),
                text.lines().count()
            )),
            "trim" => Some(text.trim().to_string()),
            "title" => Some(self.to_title_case(text)),
            "slug" => Some(self.to_slug(text)),
            "base64" => Some(self.to_base64(text)),
            "hash" => Some(self.simple_hash(text)),
            "rot13" => Some(self.rot13(text)),
            "help" => Some(self.help_text()),
            _ => None,
        }
    }

    fn to_title_case(&self, text: &str) -> String {
        text.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn to_slug(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    fn to_base64(&self, text: &str) -> String {
        // Simple base64 encoding without external deps
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let bytes = text.as_bytes();
        let mut result = String::new();

        for chunk in bytes.chunks(3) {
            let b1 = chunk[0];
            let b2 = chunk.get(1).copied().unwrap_or(0);
            let b3 = chunk.get(2).copied().unwrap_or(0);

            result.push(CHARS[(b1 >> 2) as usize] as char);
            result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
            
            if chunk.len() > 1 {
                result.push(CHARS[(((b2 & 0x0F) << 2) | (b3 >> 6)) as usize] as char);
            } else {
                result.push('=');
            }
            
            if chunk.len() > 2 {
                result.push(CHARS[(b3 & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }

        result
    }

    fn simple_hash(&self, text: &str) -> String {
        // Simple FNV-1a hash
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in text.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{:016x}", hash)
    }

    fn rot13(&self, text: &str) -> String {
        text.chars()
            .map(|c| match c {
                'A'..='M' | 'a'..='m' => ((c as u8) + 13) as char,
                'N'..='Z' | 'n'..='z' => ((c as u8) - 13) as char,
                _ => c,
            })
            .collect()
    }

    fn help_text(&self) -> String {
        r#"Text Processor Plugin Commands:
  upper <text>    - Convert to UPPERCASE
  lower <text>    - Convert to lowercase
  reverse <text>  - Reverse text
  count <text>    - Count characters, words, lines
  trim <text>     - Remove leading/trailing whitespace
  title <text>    - Convert to Title Case
  slug <text>     - Convert to URL-friendly slug
  base64 <text>   - Encode to base64
  hash <text>     - Generate hash (FNV-1a)
  rot13 <text>    - Apply ROT13 cipher
  help            - Show this help message

Examples:
  upper hello world        → HELLO WORLD
  title hello world        → Hello World
  slug Hello World!        → hello-world
  base64 secret            → c2VjcmV0
  reverse furnace          → ecanruf"#
            .to_string()
    }

    fn cleanup(&mut self) {
        self.initialized = false;
        println!("[Text Processor Plugin] Cleaned up!");
    }
}

/// Plugin entry point
#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut TextProcessorPlugin {
    Box::into_raw(Box::new(TextProcessorPlugin::new()))
}

/// Destroy plugin
#[no_mangle]
pub extern "C" fn _plugin_destroy(plugin: *mut TextProcessorPlugin) {
    if !plugin.is_null() {
        unsafe {
            let mut plugin = Box::from_raw(plugin);
            plugin.cleanup();
        }
    }
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn _plugin_name(plugin: *const TextProcessorPlugin) -> *const c_char {
    if plugin.is_null() {
        return std::ptr::null();
    }
    unsafe { (*plugin).name }
}

/// Initialize plugin
#[no_mangle]
pub extern "C" fn _plugin_init(plugin: *mut TextProcessorPlugin) {
    if !plugin.is_null() {
        unsafe {
            (*plugin).init();
        }
    }
}

/// Handle command
#[no_mangle]
pub extern "C" fn _plugin_handle_command(
    plugin: *const TextProcessorPlugin,
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
