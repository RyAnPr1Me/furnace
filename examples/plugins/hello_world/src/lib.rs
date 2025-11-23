use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Hello World Plugin - A simple example plugin for Furnace
#[repr(C)]
pub struct HelloWorldPlugin {
    name: *const c_char,
    version: *const c_char,
    initialized: bool,
}

impl HelloWorldPlugin {
    fn new() -> Self {
        Self {
            name: CString::new("Hello World").unwrap().into_raw(),
            version: CString::new("0.1.0").unwrap().into_raw(),
            initialized: false,
        }
    }

    fn get_name(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.name).to_str().unwrap()
        }
    }

    fn get_version(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.version).to_str().unwrap()
        }
    }

    fn init(&mut self) {
        self.initialized = true;
        println!("[Hello World Plugin] Initialized!");
    }

    fn handle_command(&self, command: &str) -> Option<String> {
        if !self.initialized {
            return Some("Plugin not initialized".to_string());
        }

        match command {
            "hello" => Some("Hello from Furnace plugin!".to_string()),
            "greet" => Some("Welcome to the Furnace terminal emulator!".to_string()),
            "about" => Some(format!(
                "Hello World Plugin v{}\nA simple example plugin demonstrating the plugin API.",
                self.get_version()
            )),
            "help" => Some(
                "Available commands:\n  hello  - Say hello\n  greet  - Show welcome message\n  about  - Plugin information"
                    .to_string()
            ),
            _ => None,
        }
    }

    fn cleanup(&mut self) {
        self.initialized = false;
        println!("[Hello World Plugin] Cleaned up!");
    }
}

/// Plugin entry point
#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut HelloWorldPlugin {
    Box::into_raw(Box::new(HelloWorldPlugin::new()))
}

/// Destroy plugin instance
#[no_mangle]
pub extern "C" fn _plugin_destroy(plugin: *mut HelloWorldPlugin) {
    if !plugin.is_null() {
        unsafe {
            let mut plugin = Box::from_raw(plugin);
            plugin.cleanup();
        }
    }
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn _plugin_name(plugin: *const HelloWorldPlugin) -> *const c_char {
    if plugin.is_null() {
        return std::ptr::null();
    }
    unsafe { (*plugin).name }
}

/// Get plugin version
#[no_mangle]
pub extern "C" fn _plugin_version(plugin: *const HelloWorldPlugin) -> *const c_char {
    if plugin.is_null() {
        return std::ptr::null();
    }
    unsafe { (*plugin).version }
}

/// Initialize plugin
#[no_mangle]
pub extern "C" fn _plugin_init(plugin: *mut HelloWorldPlugin) {
    if !plugin.is_null() {
        unsafe {
            (*plugin).init();
        }
    }
}

/// Handle command
#[no_mangle]
pub extern "C" fn _plugin_handle_command(
    plugin: *const HelloWorldPlugin,
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
