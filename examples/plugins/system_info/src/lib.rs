use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process::Command;

/// System Info Plugin - Display system information
#[repr(C)]
pub struct SystemInfoPlugin {
    name: *const c_char,
    version: *const c_char,
    initialized: bool,
}

impl SystemInfoPlugin {
    fn new() -> Self {
        Self {
            name: CString::new("System Info").unwrap().into_raw(),
            version: CString::new("0.1.0").unwrap().into_raw(),
            initialized: false,
        }
    }

    fn init(&mut self) {
        self.initialized = true;
        println!("[System Info Plugin] Initialized!");
    }

    fn handle_command(&self, command: &str) -> Option<String> {
        if !self.initialized {
            return Some("Plugin not initialized".to_string());
        }

        match command {
            "sysinfo" | "si" => self.get_full_info(),
            "cpu" => self.get_cpu_info(),
            "mem" => self.get_memory_info(),
            "disk" => self.get_disk_info(),
            "os" => self.get_os_info(),
            "network" => self.get_network_info(),
            "uptime" => self.get_uptime(),
            "help" => Some(self.help_text()),
            _ => None,
        }
    }

    fn get_full_info(&self) -> Option<String> {
        let mut info = String::from("=== System Information ===\n\n");
        
        if let Some(os) = self.get_os_info() {
            info.push_str(&format!("OS:\n{}\n\n", os));
        }
        
        if let Some(cpu) = self.get_cpu_info() {
            info.push_str(&format!("CPU:\n{}\n\n", cpu));
        }
        
        if let Some(mem) = self.get_memory_info() {
            info.push_str(&format!("Memory:\n{}\n\n", mem));
        }
        
        if let Some(uptime) = self.get_uptime() {
            info.push_str(&format!("Uptime:\n{}\n", uptime));
        }

        Some(info)
    }

    fn get_cpu_info(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            self.run_command("lscpu", &[])
        }
        
        #[cfg(target_os = "windows")]
        {
            self.run_command("wmic", &["cpu", "get", "name"])
        }
        
        #[cfg(target_os = "macos")]
        {
            self.run_command("sysctl", &["-n", "machdep.cpu.brand_string"])
        }
    }

    fn get_memory_info(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            self.run_command("free", &["-h"])
        }
        
        #[cfg(target_os = "windows")]
        {
            self.run_command("wmic", &["OS", "get", "FreePhysicalMemory,TotalVisibleMemorySize"])
        }
        
        #[cfg(target_os = "macos")]
        {
            self.run_command("vm_stat", &[])
        }
    }

    fn get_disk_info(&self) -> Option<String> {
        #[cfg(unix)]
        {
            self.run_command("df", &["-h"])
        }
        
        #[cfg(windows)]
        {
            self.run_command("wmic", &["logicaldisk", "get", "name,size,freespace"])
        }
    }

    fn get_os_info(&self) -> Option<String> {
        #[cfg(unix)]
        {
            self.run_command("uname", &["-a"])
        }
        
        #[cfg(windows)]
        {
            self.run_command("ver", &[])
        }
    }

    fn get_network_info(&self) -> Option<String> {
        #[cfg(unix)]
        {
            self.run_command("ifconfig", &[])
        }
        
        #[cfg(windows)]
        {
            self.run_command("ipconfig", &[])
        }
    }

    fn get_uptime(&self) -> Option<String> {
        #[cfg(unix)]
        {
            self.run_command("uptime", &[])
        }
        
        #[cfg(windows)]
        {
            self.run_command("net", &["statistics", "workstation"])
        }
    }

    fn run_command(&self, cmd: &str, args: &[&str]) -> Option<String> {
        match Command::new(cmd).args(args).output() {
            Ok(output) => {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Some(format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            Err(e) => Some(format!("Failed to run {}: {}", cmd, e)),
        }
    }

    fn help_text(&self) -> String {
        r#"System Info Plugin Commands:
  sysinfo (si)  - Show full system information
  cpu           - Show CPU information
  mem           - Show memory information
  disk          - Show disk usage
  os            - Show OS information
  network       - Show network configuration
  uptime        - Show system uptime
  help          - Show this help message"#
            .to_string()
    }

    fn cleanup(&mut self) {
        self.initialized = false;
        println!("[System Info Plugin] Cleaned up!");
    }
}

/// Plugin entry point
#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut SystemInfoPlugin {
    Box::into_raw(Box::new(SystemInfoPlugin::new()))
}

/// Destroy plugin
#[no_mangle]
pub extern "C" fn _plugin_destroy(plugin: *mut SystemInfoPlugin) {
    if !plugin.is_null() {
        unsafe {
            let mut plugin = Box::from_raw(plugin);
            plugin.cleanup();
        }
    }
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn _plugin_name(plugin: *const SystemInfoPlugin) -> *const c_char {
    if plugin.is_null() {
        return std::ptr::null();
    }
    unsafe { (*plugin).name }
}

/// Initialize plugin
#[no_mangle]
pub extern "C" fn _plugin_init(plugin: *mut SystemInfoPlugin) {
    if !plugin.is_null() {
        unsafe {
            (*plugin).init();
        }
    }
}

/// Handle command
#[no_mangle]
pub extern "C" fn _plugin_handle_command(
    plugin: *const SystemInfoPlugin,
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
