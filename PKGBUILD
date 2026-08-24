# Maintainer: Adriano Paonessa <https://github.com/adrianopaonessa>
pkgname=whatsapp-web-wrapper-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="A lightweight, fast, and native WhatsApp Web client for Linux built with Tauri and Rust"
arch=('x86_64')
url="https://github.com/adrianopaonessa/WhatsApp-Web-Wrapper"
license=('MIT')
depends=('webkit2gtk-4.1' 'libayatana-appindicator' 'openssl' 'librsvg' 'gtk3')
provides=('whatsapp-web-wrapper')
conflicts=('whatsapp-web-wrapper')
source_x86_64=("https://github.com/adrianopaonessa/WhatsApp-Web-Wrapper/releases/download/v${pkgver}/WhatsApp-Web-Wrapper-${pkgver}-1.x86_64.rpm")
sha256sums_x86_64=('24b817434c78cebe4c478a8bb2e8ca8db757656910a30ca89f076ef68eb78b7b')

package() {
    bsdtar -xf "WhatsApp-Web-Wrapper-${pkgver}-1.x86_64.rpm" -C "${pkgdir}"
}