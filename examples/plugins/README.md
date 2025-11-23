# Furnace Plugins

This directory contains example plugins for the Furnace terminal emulator.

## Available Plugins

### 1. Hello World (`hello_world`)
A simple example plugin demonstrating the basic plugin API.

**Commands:**
- `hello` - Say hello
- `greet` - Show welcome message
- `about` - Plugin information
- `help` - Show help

**Build:**
```bash
cd hello_world
cargo build --release
```

**Usage:**
```
# Load plugin
load-plugin ./target/release/libfurnace_plugin_hello_world.so

# Use commands
hello
greet
about
```

---

### 2. Git Integration (`git_integration`)
Provides convenient git commands directly in your terminal.

**Commands:**
- `git-status` (`gs`) - Show git status
- `git-branch` (`gb`) - Show current branch
- `git-log` (`gl`) - Show recent commits
- `git-diff` (`gd`) - Show file changes
- `git-remote` (`gr`) - Show remote URLs
- `git-info` (`gi`) - Show repository info

**Build:**
```bash
cd git_integration
cargo build --release
```

**Usage:**
```
# Show current branch
git-branch

# Quick status
gs

# Repository info
gi
```

**Requirements:** Git must be installed

---

### 3. Weather (`weather`)
Fetch weather information using wttr.in service.

**Commands:**
- `weather [location]` - Get full weather report
- `weather-short [location]` - Get one-line weather
- `forecast [location]` - Get weather forecast
- `moon` - Get moon phase

**Build:**
```bash
cd weather
cargo build --release
```

**Usage:**
```
# Current location
weather

# Specific city
weather London

# Brief info
weather-short NYC

# Moon phase
moon
```

**Requirements:** curl must be installed

---

### 4. System Info (`system_info`)
Display detailed system information.

**Commands:**
- `sysinfo` (`si`) - Full system information
- `cpu` - CPU information
- `mem` - Memory information
- `disk` - Disk usage
- `os` - OS information
- `network` - Network configuration
- `uptime` - System uptime

**Build:**
```bash
cd system_info
cargo build --release
```

**Usage:**
```
# Full system info
sysinfo

# Just CPU
cpu

# Memory stats
mem
```

---

### 5. Text Processor (`text_processor`)
Text manipulation and processing utilities.

**Commands:**
- `upper <text>` - Convert to UPPERCASE
- `lower <text>` - Convert to lowercase
- `reverse <text>` - Reverse text
- `count <text>` - Count chars/words/lines
- `trim <text>` - Remove whitespace
- `title <text>` - Title Case
- `slug <text>` - URL-friendly slug
- `base64 <text>` - Base64 encoding
- `hash <text>` - Generate hash
- `rot13 <text>` - ROT13 cipher

**Build:**
```bash
cd text_processor
cargo build --release
```

**Usage:**
```
# Convert case
upper hello world
→ HELLO WORLD

# Create slug
slug Hello World!
→ hello-world

# Base64 encode
base64 secret
→ c2VjcmV0

# Reverse text
reverse furnace
→ ecanruf
```

---

## Building All Plugins

From the `examples/plugins` directory:

```bash
# Build all plugins
cargo build --release --workspace

# Plugins will be in:
# ./target/release/libfurnace_plugin_hello_world.{so,dll,dylib}
# ./target/release/libfurnace_plugin_git.{so,dll,dylib}
# ./target/release/libfurnace_plugin_weather.{so,dll,dylib}
# ./target/release/libfurnace_plugin_sysinfo.{so,dll,dylib}
# ./target/release/libfurnace_plugin_text.{so,dll,dylib}
```

## Installing Plugins

1. **Copy to plugins directory:**
```bash
mkdir -p ~/.furnace/plugins
cp target/release/*.so ~/.furnace/plugins/
```

2. **Add to configuration (`~/.furnace/config.yaml`):**
```yaml
plugins:
  - "~/.furnace/plugins/libfurnace_plugin_git.so"
  - "~/.furnace/plugins/libfurnace_plugin_weather.so"
  - "~/.furnace/plugins/libfurnace_plugin_text.so"
```

3. **Load at runtime:**
Use the command palette (Ctrl+P):
```
load-plugin ~/.furnace/plugins/libfurnace_plugin_git.so
```

## Creating Your Own Plugin

See `PLUGIN_DEVELOPMENT.md` in the root directory for a comprehensive guide to creating custom plugins.

**Quick Start:**

1. Create a new library:
```bash
cargo new --lib my_plugin
```

2. Set `crate-type` in `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]
```

3. Implement the plugin interface:
```rust
#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut MyPlugin {
    Box::into_raw(Box::new(MyPlugin::new()))
}

#[no_mangle]
pub extern "C" fn _plugin_handle_command(
    plugin: *const MyPlugin,
    command: *const c_char,
) -> *mut c_char {
    // Handle commands
}
```

4. Build and test:
```bash
cargo build --release
```

## Plugin Safety

All plugins in this directory follow these safety principles:

1. **Memory Safety**: No memory leaks, proper cleanup
2. **Error Handling**: Graceful error messages
3. **Resource Management**: Proper initialization/cleanup
4. **FFI Safety**: Safe C interface with proper null checks

## Testing Plugins

Each plugin can be tested individually:

```bash
cd plugin_name
cargo test
```

## Troubleshooting

**Plugin not loading:**
- Check file permissions
- Verify correct library format (.so on Linux, .dll on Windows, .dylib on macOS)
- Check for missing dependencies

**Command not working:**
- Use `help` command to see available commands
- Check if required external tools are installed (git, curl, etc.)

**Build errors:**
- Ensure Rust toolchain is up to date: `rustup update`
- Check for missing system dependencies

## Platform-Specific Notes

### Linux
- Plugins use `.so` extension
- May need `libloading` permissions

### Windows
- Plugins use `.dll` extension
- May need to allow DLL loading

### macOS
- Plugins use `.dylib` extension
- May need to approve plugin loading in Security & Privacy

## License

All example plugins are provided under the MIT license.
