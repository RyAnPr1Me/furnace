use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process::Command;

/// Weather Plugin - Get weather information via wttr.in
#[repr(C)]
pub struct WeatherPlugin {
    name: *const c_char,
    version: *const c_char,
    initialized: bool,
    default_location: String,
}

impl WeatherPlugin {
    fn new() -> Self {
        Self {
            name: CString::new("Weather").unwrap().into_raw(),
            version: CString::new("0.1.0").unwrap().into_raw(),
            initialized: false,
            default_location: String::new(),
        }
    }

    fn init(&mut self) {
        self.initialized = true;
        println!("[Weather Plugin] Initialized!");
    }

    fn handle_command(&self, command: &str) -> Option<String> {
        if !self.initialized {
            return Some("Plugin not initialized".to_string());
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.get(0).copied() {
            Some("weather") => {
                let location = parts.get(1).copied().unwrap_or("auto");
                self.get_weather(location)
            }
            Some("weather-short") => {
                let location = parts.get(1).copied().unwrap_or("auto");
                self.get_weather_short(location)
            }
            Some("forecast") => {
                let location = parts.get(1).copied().unwrap_or("auto");
                self.get_forecast(location)
            }
            Some("moon") => self.get_moon_phase(),
            Some("help") => Some(self.help_text()),
            _ => None,
        }
    }

    fn get_weather(&self, location: &str) -> Option<String> {
        self.fetch_wttr(location, &[])
    }

    fn get_weather_short(&self, location: &str) -> Option<String> {
        self.fetch_wttr(location, &["format=3"])
    }

    fn get_forecast(&self, location: &str) -> Option<String> {
        self.fetch_wttr(location, &["format=v2"])
    }

    fn get_moon_phase(&self) -> Option<String> {
        self.fetch_wttr("Moon", &[])
    }

    fn fetch_wttr(&self, location: &str, params: &[&str]) -> Option<String> {
        let mut url = format!("https://wttr.in/{}", location);
        
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        match Command::new("curl")
            .args(&["-s", &url])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Some("Failed to fetch weather data. Is curl installed?".to_string())
                }
            }
            Err(_) => Some(
                "curl not found. Please install curl to use weather plugin.".to_string()
            ),
        }
    }

    fn help_text(&self) -> String {
        r#"Weather Plugin Commands:
  weather [location]       - Get full weather report (default: auto-detect)
  weather-short [location] - Get one-line weather info
  forecast [location]      - Get weather forecast
  moon                     - Get moon phase
  help                     - Show this help message

Examples:
  weather              - Current location weather
  weather London       - London weather
  weather-short NYC    - New York City brief weather
  forecast Tokyo       - Tokyo forecast
  moon                 - Current moon phase

Note: Requires curl to be installed. Uses wttr.in service."#
            .to_string()
    }

    fn cleanup(&mut self) {
        self.initialized = false;
        println!("[Weather Plugin] Cleaned up!");
    }
}

/// Plugin entry point
#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut WeatherPlugin {
    Box::into_raw(Box::new(WeatherPlugin::new()))
}

/// Destroy plugin
#[no_mangle]
pub extern "C" fn _plugin_destroy(plugin: *mut WeatherPlugin) {
    if !plugin.is_null() {
        unsafe {
            let mut plugin = Box::from_raw(plugin);
            plugin.cleanup();
        }
    }
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn _plugin_name(plugin: *const WeatherPlugin) -> *const c_char {
    if plugin.is_null() {
        return std::ptr::null();
    }
    unsafe { (*plugin).name }
}

/// Initialize plugin
#[no_mangle]
pub extern "C" fn _plugin_init(plugin: *mut WeatherPlugin) {
    if !plugin.is_null() {
        unsafe {
            (*plugin).init();
        }
    }
}

/// Handle command
#[no_mangle]
pub extern "C" fn _plugin_handle_command(
    plugin: *const WeatherPlugin,
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
