#!/bin/bash
# Build script for creating Linux distribution packages of Furnace
# Generates: .deb, .rpm, AppImage, and tar.gz packages

set -e

echo "=========================================="
echo "Furnace Linux Distribution Builder"
echo "=========================================="
echo ""

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Building Furnace version: $VERSION"
echo ""

# Create output directory
OUTPUT_DIR="dist"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Step 1: Build the release binary with GPU acceleration
echo "==> Step 1: Building release binary with GPU acceleration..."
cargo build --release --features gpu
echo "✓ GPU-accelerated binary built successfully"
echo ""

# Step 2: Build .deb package (Debian/Ubuntu)
echo "==> Step 2: Building .deb package..."
if ! command_exists cargo-deb; then
    echo "Installing cargo-deb..."
    cargo install cargo-deb
fi

cargo deb --no-build --output "$OUTPUT_DIR/furnace_${VERSION}_amd64.deb"
echo "✓ .deb package created: $OUTPUT_DIR/furnace_${VERSION}_amd64.deb"
echo ""

# Step 3: Build .rpm package (Fedora/RHEL/CentOS)
echo "==> Step 3: Building .rpm package..."
if ! command_exists cargo-generate-rpm; then
    echo "Installing cargo-generate-rpm..."
    cargo install cargo-generate-rpm
fi

# Check binary exists before packaging
if [ ! -f target/release/furnace ]; then
    echo "Error: Binary not found at target/release/furnace"
    exit 1
fi

# Generate RPM (stripping happens after to avoid modifying the binary before packaging tools process it)
cargo generate-rpm --output "$OUTPUT_DIR/furnace-${VERSION}-1.x86_64.rpm"

# Strip debug symbols for smaller subsequent packages (doesn't affect the already-created RPM)
strip target/release/furnace || echo "Warning: Failed to strip binary"

echo "✓ .rpm package created: $OUTPUT_DIR/furnace-${VERSION}-1.x86_64.rpm"
echo ""

# Step 4: Build .pkg.tar.zst package (Arch Linux)
echo "==> Step 4: Building .pkg.tar.zst package..."
if command_exists makepkg; then
    # Save current directory
    ORIGINAL_DIR="$PWD"
    
    # Create temporary directory for Arch package build
    ARCH_BUILD_DIR="$OUTPUT_DIR/arch-build"
    mkdir -p "$ARCH_BUILD_DIR"
    
    # Copy PKGBUILD to build directory
    cp PKGBUILD "$ARCH_BUILD_DIR/"
    
    # Update version in PKGBUILD
    sed -i "s/^pkgver=.*/pkgver=${VERSION}/" "$ARCH_BUILD_DIR/PKGBUILD"
    
    # Build package
    # Note: --skipinteg is used because the PKGBUILD has no source array (building from local source)
    # and makepkg would fail trying to verify checksums for non-existent remote sources
    cd "$ARCH_BUILD_DIR"
    if PKGDEST="$PWD" makepkg -f --skipinteg 2>&1 | tee makepkg.log; then
        # Return to original directory
        cd "$ORIGINAL_DIR"
        
        # Move package to output directory if it was created
        if ls "$ARCH_BUILD_DIR"/*.pkg.tar.zst 1> /dev/null 2>&1; then
            mv "$ARCH_BUILD_DIR"/*.pkg.tar.zst "$OUTPUT_DIR/"
            echo "✓ .pkg.tar.zst package created: $OUTPUT_DIR/furnace-${VERSION}-1-x86_64.pkg.tar.zst"
        else
            echo "⚠ .pkg.tar.zst package not found after build"
        fi
    else
        echo "⚠ Arch package build failed (this is optional)"
        echo "Note: makepkg may fail outside of Arch Linux environment"
        # Return to original directory even on failure
        cd "$ORIGINAL_DIR"
    fi
else
    echo "⚠ makepkg not found - skipping Arch Linux package"
    echo "   Install makepkg (Arch Linux) to build .pkg.tar.zst packages"
fi
echo ""

# Step 5: Build AppImage (Universal Linux)
echo "==> Step 5: Building AppImage..."
APPIMAGE_DIR="$OUTPUT_DIR/AppDir"
mkdir -p "$APPIMAGE_DIR/usr/bin"
mkdir -p "$APPIMAGE_DIR/usr/share/applications"
mkdir -p "$APPIMAGE_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPIMAGE_DIR/usr/share/doc/furnace"

# Copy binary
cp target/release/furnace "$APPIMAGE_DIR/usr/bin/"

# Create desktop entry for system installation
cat > "$APPIMAGE_DIR/usr/share/applications/furnace.desktop" << 'EOF'
[Desktop Entry]
Type=Application
Name=Furnace
Comment=High-performance terminal emulator
Exec=furnace
Icon=furnace
Terminal=true
Categories=System;TerminalEmulator;
Keywords=terminal;emulator;shell;
EOF

# Create root desktop entry (required by AppImage spec)
cat > "$APPIMAGE_DIR/furnace.desktop" << 'EOF'
[Desktop Entry]
Type=Application
Name=Furnace
Comment=High-performance terminal emulator
Exec=furnace
Icon=furnace
Terminal=true
Categories=System;TerminalEmulator;
EOF

# Copy icon (required by AppImage spec)
if [ -f "furnace.png" ]; then
    cp furnace.png "$APPIMAGE_DIR/usr/share/icons/hicolor/256x256/apps/furnace.png"
    cp furnace.png "$APPIMAGE_DIR/furnace.png"
else
    echo "Warning: furnace.png not found. AppImage may not have an icon."
fi

# Copy documentation
cp README.md LICENSE "$APPIMAGE_DIR/usr/share/doc/furnace/" 2>/dev/null || true
cp config.example.lua "$APPIMAGE_DIR/usr/share/doc/furnace/" 2>/dev/null || true

# Create AppRun script
cat > "$APPIMAGE_DIR/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin/:${PATH}"
exec "${HERE}/usr/bin/furnace" "$@"
EOF
chmod +x "$APPIMAGE_DIR/AppRun"

# Download appimagetool if not present
if [ ! -f "appimagetool-x86_64.AppImage" ]; then
    echo "Downloading appimagetool..."
    echo "Note: Downloading from continuous release. For production, consider pinning a specific version."
    
    if wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"; then
        chmod +x appimagetool-x86_64.AppImage
        
        # Verify the download (basic sanity checks)
        # Note: continuous releases don't have stable checksums, but we can validate basic properties
        if [ ! -x "appimagetool-x86_64.AppImage" ] || [ ! -s "appimagetool-x86_64.AppImage" ]; then
            echo "Warning: Downloaded appimagetool appears invalid. Skipping AppImage creation."
            rm -f appimagetool-x86_64.AppImage
        else
            # Check file size is reasonable (appimagetool is typically ~5MB)
            file_size=$(stat -c%s "appimagetool-x86_64.AppImage" 2>/dev/null || stat -f%z "appimagetool-x86_64.AppImage" 2>/dev/null)
            if [ "$file_size" -lt 1000000 ]; then
                echo "Warning: Downloaded file is suspiciously small. Skipping AppImage creation."
                rm -f appimagetool-x86_64.AppImage
            fi
        fi
    else
        echo "Warning: Could not download appimagetool. Skipping AppImage creation."
        echo "You can manually download it from https://github.com/AppImage/AppImageKit/releases"
    fi
fi

if [ -f "appimagetool-x86_64.AppImage" ] && [ -x "appimagetool-x86_64.AppImage" ]; then
    # Use APPIMAGE_EXTRACT_AND_RUN to work in environments without FUSE
    ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 ./appimagetool-x86_64.AppImage "$APPIMAGE_DIR" "$OUTPUT_DIR/furnace-${VERSION}-x86_64.AppImage"
    echo "✓ AppImage created: $OUTPUT_DIR/furnace-${VERSION}-x86_64.AppImage"
else
    echo "⚠ AppImage creation skipped (appimagetool not available)"
fi
echo ""

# Step 6: Create tar.gz archive with install script
echo "==> Step 6: Building tar.gz archive..."
TARBALL_DIR="$OUTPUT_DIR/furnace-${VERSION}"
mkdir -p "$TARBALL_DIR"

# Copy files
cp target/release/furnace "$TARBALL_DIR/"
cp README.md LICENSE "$TARBALL_DIR/" 2>/dev/null || true
cp config.example.lua "$TARBALL_DIR/" 2>/dev/null || true

# Create install script
cat > "$TARBALL_DIR/install.sh" << 'EOF'
#!/bin/bash
# Furnace Terminal Emulator - Installation Script

set -e

echo "=========================================="
echo "Furnace Terminal Emulator - Installer"
echo "=========================================="
echo ""

# Check if running as root for system-wide install
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
    DOC_DIR="/usr/local/share/doc/furnace"
    echo "Installing system-wide to $INSTALL_DIR"
else
    INSTALL_DIR="$HOME/.local/bin"
    DOC_DIR="$HOME/.local/share/doc/furnace"
    echo "Installing to user directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi

# Install binary
echo "Installing furnace binary..."
cp furnace "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/furnace"

# Install documentation
echo "Installing documentation..."
mkdir -p "$DOC_DIR"
cp README.md LICENSE "$DOC_DIR/" 2>/dev/null || true
cp config.example.lua "$DOC_DIR/" 2>/dev/null || true

echo ""
echo "✓ Installation complete!"
echo ""
echo "Furnace has been installed to: $INSTALL_DIR/furnace"
echo ""

# Check if directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠ NOTE: $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add it to your PATH by adding this line to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo ""
fi

echo "To run Furnace, type: furnace"
echo ""
EOF
chmod +x "$TARBALL_DIR/install.sh"

# Create the tarball
cd "$OUTPUT_DIR"
tar czf "furnace-${VERSION}-linux-x86_64.tar.gz" "furnace-${VERSION}"
cd ..
echo "✓ Tar.gz archive created: $OUTPUT_DIR/furnace-${VERSION}-linux-x86_64.tar.gz"
echo ""

# Step 7: Generate checksums
echo "==> Step 7: Generating checksums..."
cd "$OUTPUT_DIR"

# Generate checksums for all package files
> SHA256SUMS  # Create empty file
for file in furnace*.deb furnace*.rpm furnace*.pkg.tar.zst furnace*.AppImage furnace*.tar.gz; do
    if [ -f "$file" ]; then
        sha256sum "$file" >> SHA256SUMS
    fi
done

cd ..
echo "✓ Checksums generated: $OUTPUT_DIR/SHA256SUMS"
echo ""

# Summary
echo "=========================================="
echo "Build Summary"
echo "=========================================="
echo ""
echo "Distribution packages created in: $OUTPUT_DIR/"
echo ""
ls -lh "$OUTPUT_DIR" | grep -E '\.(deb|rpm|pkg\.tar\.zst|AppImage|tar\.gz|SHA256SUMS)$' || ls -lh "$OUTPUT_DIR"
echo ""
echo "Package formats:"
echo "  • .deb          - Debian/Ubuntu (apt/dpkg)"
echo "  • .rpm          - Fedora/RHEL/CentOS (dnf/yum)"
echo "  • .pkg.tar.zst  - Arch Linux (pacman)"
echo "  • .AppImage     - Universal Linux (portable)"
echo "  • .tar.gz       - Manual installation (with install script)"
echo ""
echo "✓ All distribution packages built successfully! 🔥"
echo ""
