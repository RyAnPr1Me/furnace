# Maintainer: RyAnPr1Me
pkgname=furnace
pkgver=1.0.0
pkgrel=1
pkgdesc="High-performance, memory-safe terminal emulator written in Rust"
arch=('x86_64')
url="https://github.com/RyAnPr1Me/furnace"
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
source=()
sha256sums=()

build() {
    cd "$srcdir/../.."
    cargo build --release --features gpu --locked
}

package() {
    cd "$srcdir/../.."
    
    # Install binary
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    
    # Install license
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    
    # Install example configuration
    install -Dm644 config.example.lua "$pkgdir/usr/share/doc/$pkgname/config.example.lua"
}
