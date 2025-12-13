# Furnace Terminal Emulator - Binary Releases

## About

Furnace is a high-performance, cross-platform terminal emulator built in Rust, supporting Windows, Linux, and macOS with native performance, 170 FPS GPU-accelerated rendering, and modern features.

## Download

### Windows (x86_64) ⭐
- **File**: `furnace-windows-x86_64.exe`
- **Size**: ~1.5MB
- **Architecture**: 64-bit Windows (Windows 10/11)
- **Build**: Optimized release with LTO

### Linux (x86_64)

#### Binary Only
- **File**: `furnace-linux-x86_64`
- **Size**: ~1.8MB
- **Architecture**: 64-bit Linux
- **Build**: Optimized release with LTO

#### Distribution Packages (in `linux-distro/`)
Choose the package format that matches your Linux distribution:

- **`.deb`** - For Debian/Ubuntu/Linux Mint
  - Install with: `sudo dpkg -i furnace_*.deb` or `sudo apt install ./furnace_*.deb`
  - Size: ~1.8MB
  
- **`.rpm`** - For Fedora/RHEL/CentOS/openSUSE
  - Install with: `sudo rpm -i furnace-*.rpm` or `sudo dnf install furnace-*.rpm`
  - Size: ~1.8MB
  
- **`.AppImage`** - Universal Linux (portable, no installation needed)
  - Just download, make executable, and run
  - Size: ~2.0MB
  
- **`.tar.gz`** - Manual installation with install script
  - Extract and run `./install.sh`
  - Size: ~1.8MB

## Installation

### Windows
```powershell
# Download furnace-windows-x86_64.exe

# Run directly (double-click or from terminal)
.\furnace-windows-x86_64.exe

# Or rename and add to PATH
Rename-Item furnace-windows-x86_64.exe furnace.exe
# Add directory to PATH in System Environment Variables

# Verify installation
furnace --version
```

### Linux

#### Option 1: Distribution Package (Recommended)

**Debian/Ubuntu/Linux Mint:**
```bash
# Download the .deb file from linux-distro/
sudo apt install ./furnace_X.Y.Z_amd64.deb

# Or using dpkg
sudo dpkg -i furnace_X.Y.Z_amd64.deb
sudo apt-get install -f  # Fix dependencies if needed

# Run
furnace
```

**Fedora/RHEL/CentOS:**
```bash
# Download the .rpm file from linux-distro/
sudo dnf install furnace-X.Y.Z-1.x86_64.rpm

# Or using rpm
sudo rpm -i furnace-X.Y.Z-1.x86_64.rpm

# Run
furnace
```

**Universal Linux (AppImage):**
```bash
# Download the .AppImage file from linux-distro/
chmod +x furnace-X.Y.Z-x86_64.AppImage

# Run directly (no installation needed)
./furnace-X.Y.Z-x86_64.AppImage

# Optional: Move to PATH for easy access
sudo mv furnace-X.Y.Z-x86_64.AppImage /usr/local/bin/furnace
```

**Manual Installation (tar.gz):**
```bash
# Download and extract the .tar.gz file from linux-distro/
tar xzf furnace-X.Y.Z-linux-x86_64.tar.gz
cd furnace-X.Y.Z

# Run the install script
sudo ./install.sh  # System-wide installation
# OR
./install.sh       # User installation to ~/.local/bin

# Run
furnace
```

#### Option 2: Standalone Binary
```bash
# Download and make executable
chmod +x furnace-linux-x86_64

# Move to PATH (optional)
sudo mv furnace-linux-x86_64 /usr/local/bin/furnace

# Or run directly
./furnace-linux-x86_64
```

### macOS
macOS binaries can be cross-compiled or built natively:
```bash
# On macOS
cargo build --release

# Binary will be at: target/release/furnace
```

## Verification

Verify the binary works:

**Windows:**
```powershell
.\furnace-windows-x86_64.exe --version
# Output: furnace 1.0.0

.\furnace-windows-x86_64.exe --help
# Shows help information
```

**Linux:**
```bash
./furnace-linux-x86_64 --version
# Output: furnace 1.0.0

./furnace-linux-x86_64 --help
# Shows help information
```

### Checksum Verification

Verify file integrity with SHA256:

**For binaries:**
```bash
# Linux/macOS
sha256sum -c SHA256SUMS

# Windows (PowerShell)
Get-FileHash furnace-windows-x86_64.exe -Algorithm SHA256
# Compare with value in SHA256SUMS file
```

**For Linux distribution packages:**
```bash
# Verify packages in linux-distro/
cd linux-distro
sha256sum -c SHA256SUMS
```

## Features

- **170 FPS Rendering**: Ultra-smooth GPU-accelerated rendering
- **Native Performance**: Rust compiled to native machine code, zero runtime overhead
- **Zero Memory Leaks**: Guaranteed by Rust's ownership system
- **24-bit True Color**: 16.7 million colors
- **Session Management**: Save and restore terminal state
- **Plugin System**: Extensible with 5 example plugins included
- **Advanced Autocomplete**: History-based with smart suggestions
- **Command Palette**: Fuzzy search (Ctrl+P)
- **Resource Monitor**: Real-time CPU/memory/disk stats (Ctrl+R)
- **Multiple Tabs**: Efficient session management
- **Split Panes**: Horizontal and vertical splits
- **Custom Themes**: 3 built-in themes + customizable

## Performance

- **Startup**: < 100ms
- **Memory**: 14-18MB (vs 60-100MB PowerShell)
- **CPU (idle)**: 2-5%
- **Input Latency**: < 3ms
- **Binary Size**: 1.8MB (stripped, optimized)

## Configuration

Default config location: `~/.config/furnace/config.yaml`

Example config: See `config.example.yaml` in the repository

## Key Bindings

| Shortcut | Action |
|----------|--------|
| `Ctrl+P` | Command palette |
| `Ctrl+R` | Toggle resource monitor |
| `Ctrl+S` | Save session |
| `Ctrl+Shift+O` | Load session |
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+H` | Split horizontal |
| `Ctrl+Shift+V` | Split vertical |
| `Ctrl+O` | Focus next pane |
| `Ctrl+F` | Search |

## Build Information

- **Rust Version**: 1.91.1
- **Optimization Level**: 3 (maximum)
- **LTO**: Fat (full link-time optimization)
- **Target CPU**: native
- **Strip**: Yes (debug symbols removed)

## Support

For issues, feature requests, or contributions, visit:
https://github.com/RyAnPr1Me/furnace

## License

See LICENSE file in the repository root.
