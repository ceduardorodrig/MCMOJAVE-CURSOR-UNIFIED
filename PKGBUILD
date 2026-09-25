# Maintainer: Carlos Eduardo Rodrigues <ceduardorodrig@gmail.com>

pkgname=mcmojave-cursor-unified
pkgver=1.0.0
pkgrel=1
pkgdesc="Dual-spec unified McMojave cursor theme (Hyprcursor vector + 1:1 calibrated XCursor)"
arch=('x86_64')
url="https://github.com/ceduardorodrig/mcmojave-cursor-unified"
license=('GPL-3.0-or-later')
makedepends=('cargo' 'librsvg' 'xorg-xcursorgen')
provides=('mcmojave-cursors' 'mcmojave-hyprcursor')
conflicts=('mcmojave-cursors' 'mcmojave-hyprcursor')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release
    ./target/release/mcmojave-cursor-unified --output dist/McMojave
}

package() {
    cd "$pkgname-$pkgver"
    install -d "$pkgdir/usr/share/icons/McMojave"
    cp -r dist/McMojave/* "$pkgdir/usr/share/icons/McMojave/"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
