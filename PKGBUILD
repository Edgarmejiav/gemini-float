# Maintainer: lega <lega@cachyos>
pkgname=gemini-float
pkgver=1.0.0
pkgrel=1
pkgdesc="Floating Google Gemini desktop wrapper for Linux (GNOME/Wayland)"
arch=('x86_64')
url="https://github.com/lega/gemini-float"
license=('MIT')
depends=(
    'webkit2gtk-4.1'
    'gtk3'
    'libappindicator-gtk3'
    'hicolor-icon-theme'
)
makedepends=(
    'rust'
    'cargo'
    'nodejs'
    'pnpm'
)
options=('!strip' '!debug')
install=gemini-float.install
source=("$pkgname-$pkgver.tar.gz::https://github.com/lega/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    pnpm install --frozen-lockfile
    pnpm tauri build --no-bundle
}

package() {
    cd "$pkgname-$pkgver"

    # Binario en /usr/bin (estándar FHS para binarios de usuario)
    install -Dm755 "src-tauri/target/release/$pkgname" \
        "$pkgdir/usr/bin/$pkgname"

    # Icono en la jerarquía estándar hicolor
    install -Dm644 "src-tauri/icons/icon.png" \
        "$pkgdir/usr/share/icons/hicolor/256x256/apps/$pkgname.png"
    install -Dm644 "src-tauri/icons/128x128.png" \
        "$pkgdir/usr/share/icons/hicolor/128x128/apps/$pkgname.png"
    install -Dm644 "src-tauri/icons/32x32.png" \
        "$pkgdir/usr/share/icons/hicolor/32x32/apps/$pkgname.png"

    # Archivo .desktop (estándar XDG/Freedesktop)
    install -Dm644 "$pkgname.desktop" \
        "$pkgdir/usr/share/applications/$pkgname.desktop"

    # Licencia
    install -Dm644 LICENSE \
        "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
