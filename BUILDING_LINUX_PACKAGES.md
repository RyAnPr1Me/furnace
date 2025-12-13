# Building Linux Distribution Packages

This document explains how to build Linux distribution packages for Furnace.

## Quick Start

To build all Linux distribution packages, run:

```bash
./build-linux-distro.sh
```

This will create packages in the `dist/` directory.

## Prerequisites

The build script will automatically install the following tools if not present:
- `cargo-deb` - For building .deb packages
- `cargo-generate-rpm` - For building .rpm packages
- `appimagetool` - For building AppImages (downloaded automatically)

System requirements:
- Rust 1.70+ (install via [rustup.rs](https://rustup.rs))
- Standard build tools (`gcc`, `make`, etc.)
- `wget` (for downloading appimagetool)

## Package Formats

The build script creates the following package formats:

### 1. .deb (Debian/Ubuntu/Linux Mint)

**Installation:**
```bash
sudo apt install ./furnace_1.0.0_amd64.deb
# or
sudo dpkg -i furnace_1.0.0_amd64.deb
```

**Removal:**
```bash
sudo apt remove furnace
# or
sudo dpkg -r furnace
```

**Features:**
- System package manager integration
- Automatic dependency resolution
- Clean installation/removal
- Installs to `/usr/bin/furnace`

### 2. .rpm (Fedora/RHEL/CentOS/openSUSE)

**Installation:**
```bash
sudo dnf install furnace-1.0.0-1.x86_64.rpm
# or
sudo rpm -i furnace-1.0.0-1.x86_64.rpm
```

**Removal:**
```bash
sudo dnf remove furnace
# or
sudo rpm -e furnace
```

**Features:**
- System package manager integration
- Automatic dependency resolution
- Clean installation/removal
- Installs to `/usr/bin/furnace`

### 3. .AppImage (Universal Linux)

**Usage:**
```bash
chmod +x furnace-1.0.0-x86_64.AppImage
./furnace-1.0.0-x86_64.AppImage
```

**Features:**
- No installation required
- Works on most Linux distributions
- Portable - can run from any location
- Self-contained with all dependencies
- Can be integrated with desktop environments

**Desktop Integration (Optional):**
```bash
# Move to a standard location
mkdir -p ~/.local/bin
mv furnace-1.0.0-x86_64.AppImage ~/.local/bin/furnace
chmod +x ~/.local/bin/furnace

# Add to PATH if needed
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

### 4. .tar.gz (Manual Installation)

**Installation:**
```bash
tar xzf furnace-1.0.0-linux-x86_64.tar.gz
cd furnace-1.0.0

# System-wide installation (requires sudo)
sudo ./install.sh

# Or user installation (no sudo required)
./install.sh
```

**Features:**
- Includes install script
- User or system-wide installation
- No package manager required
- Installs to `/usr/local/bin` (system) or `~/.local/bin` (user)

## Manual Build Process

If you want to build packages individually:

### Build .deb only:
```bash
cargo build --release --all-features
cargo install cargo-deb
cargo deb --no-build --output dist/furnace.deb
```

### Build .rpm only:
```bash
cargo build --release --all-features
cargo install cargo-generate-rpm
strip target/release/furnace
cargo generate-rpm --output dist/furnace.rpm
```

### Build AppImage only:
```bash
cargo build --release --all-features
# Then follow AppImage build steps from build-linux-distro.sh
```

### Build tar.gz only:
```bash
cargo build --release --all-features
mkdir -p dist/furnace-1.0.0
cp target/release/furnace dist/furnace-1.0.0/
cp README.md LICENSE config.example.lua dist/furnace-1.0.0/
# Create install.sh (see build-linux-distro.sh for template)
cd dist
tar czf furnace-1.0.0-linux-x86_64.tar.gz furnace-1.0.0
```

## CI/CD Integration

The GitHub Actions workflow (`.github/workflows/build-releases.yml`) automatically:
1. Builds release binaries for Linux and Windows
2. Builds all Linux distribution packages
3. Uploads packages as artifacts
4. Updates the `releases/` folder with new packages

The workflow runs on:
- Push to `main` branch
- Manual workflow dispatch

## Package Metadata

Package metadata is configured in `Cargo.toml`:
- `[package.metadata.deb]` - Debian package configuration
- `[package.metadata.generate-rpm]` - RPM package configuration

To update package information (description, dependencies, etc.), edit these sections in `Cargo.toml`.

## Checksums

The build script automatically generates SHA256 checksums for all packages in `dist/SHA256SUMS`.

To verify package integrity:
```bash
cd dist
sha256sum -c SHA256SUMS
```

## Troubleshooting

### cargo-deb not found
```bash
cargo install cargo-deb
```

### cargo-generate-rpm not found
```bash
cargo install cargo-generate-rpm
```

### AppImage build fails
The script will skip AppImage creation if `appimagetool` download fails. You can manually download it from:
https://github.com/AppImage/AppImageKit/releases

### Permission denied
Make sure the script is executable:
```bash
chmod +x build-linux-distro.sh
```

### Build fails with dependency errors
Ensure you have the required system dependencies:
```bash
# Debian/Ubuntu
sudo apt update
sudo apt install build-essential pkg-config

# Fedora/RHEL
sudo dnf install gcc make
```

## Distribution-Specific Notes

### Debian/Ubuntu
- Packages work on Ubuntu 20.04+, Debian 11+, and derivatives
- Uses `dpkg` and `apt` package managers

### Fedora/RHEL/CentOS
- Packages work on Fedora 36+, RHEL 8+, CentOS Stream 8+
- Uses `rpm`, `dnf`, or `yum` package managers

### Arch Linux
- Use the standalone binary or AppImage
- Community may create AUR packages

### Other Distributions
- AppImage and tar.gz work on most Linux distributions
- Consider using these portable formats

## Support

For issues with packaging:
1. Check that your Rust toolchain is up to date
2. Ensure all system dependencies are installed
3. Review the build script output for specific errors
4. Open an issue on GitHub with the error details

## License

Same as the main Furnace project (MIT License).
