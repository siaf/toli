# Maintainer: Your Name <your.email@example.com>

pkgname=toli
pkgver=0.1.0
pkgrel=1
pkgdesc="Terminal Intelligence & Learning Operator - Natural language interface for shell commands"
arch=('x86_64')
url="https://github.com/siaf/toli"
license=('MIT')
depends=('gcc-libs')
makedepends=('rust')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 target/release/toli "$pkgdir/usr/bin/toli"

  # Install shell completions
  install -Dm644 completions/toli.bash "$pkgdir/usr/share/bash-completion/completions/toli"
  install -Dm644 completions/toli.zsh "$pkgdir/usr/share/zsh/site-functions/_toli"
  install -Dm644 completions/toli.fish "$pkgdir/usr/share/fish/vendor_completions.d/toli.fish"

  # Install shell aliases
  install -Dm644 /dev/null "$pkgdir/etc/profile.d/toli.sh"
  echo 'alias howto="toli --how"' >> "$pkgdir/etc/profile.d/toli.sh"
  echo 'alias do="toli --do"' >> "$pkgdir/etc/profile.d/toli.sh"

  # Install documentation
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}