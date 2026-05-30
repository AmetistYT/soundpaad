# Maintainer: AmetistYT <ametist0yt@gmail.com>
pkgname=soundpaad-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Sound pad with virtual microphone support for Linux"
arch=('x86_64')
url="https://github.com/AmetistYT/soundpaad"
license=('MIT')
depends=('gtk4' 'libadwaita' 'gstreamer' 'gst-plugins-base' 'gst-plugins-good' 'pulseaudio' 'openssl')
optdepends=('pipewire-pulse: PipeWire PulseAudio compatibility')
provides=('soundpaad')
conflicts=('soundpaad')
source=("soundpaad-${pkgver}::https://github.com/AmetistYT/soundpaad/releases/download/v${pkgver}/soundpaad"
        "com.soundpaad.app.desktop"
        "com.soundpaad.app.svg")
sha256sums=('SKIP'
            'SKIP'
            'SKIP')

package() {
    install -Dm755 "soundpaad-${pkgver}" "${pkgdir}/usr/bin/soundpaad"
    install -Dm644 "com.soundpaad.app.desktop" "${pkgdir}/usr/share/applications/com.soundpaad.app.desktop"
    install -Dm644 "com.soundpaad.app.svg" "${pkgdir}/usr/share/icons/hicolor/scalable/apps/com.soundpaad.app.svg"
}
