#!/bin/sh
# Empacota um binário já compilado. Não substitui os testes de instalação.
set -eu
cd "$(dirname "$0")/.."
binary=${1:-src-tauri/target/release/telaleve}
[ -x "$binary" ] || { printf 'Binário não encontrado: %s\n' "$binary" >&2; exit 1; }
version=$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')
architecture=$(dpkg --print-architecture)
package_dir=$(mktemp -d)
trap 'rm -rf "$package_dir"' EXIT HUP INT TERM
mkdir -p "$package_dir/DEBIAN" "$package_dir/usr/bin" "$package_dir/usr/share/applications" "$package_dir/usr/share/icons/hicolor/256x256/apps" "$package_dir/usr/share/doc/telaleve"
install -m755 "$binary" "$package_dir/usr/bin/telaleve"
install -m644 LICENSE "$package_dir/usr/share/doc/telaleve/copyright"
install -m644 src-tauri/icons/icon.png "$package_dir/usr/share/icons/hicolor/256x256/apps/telaleve.png"
cat > "$package_dir/DEBIAN/control" <<CONTROL
Package: telaleve
Version: $version
Section: utils
Priority: optional
Architecture: $architecture
Maintainer: TelaLeve Project <noreply@localhost>
Depends: libwebkit2gtk-4.1-0, libgtk-3-0t64 | libgtk-3-0, libgstreamer1.0-0, libgstreamer-plugins-base1.0-0, libgstreamer-plugins-bad1.0-0, gstreamer1.0-x, gstreamer1.0-plugins-base, gstreamer1.0-plugins-good, gstreamer1.0-plugins-bad, gstreamer1.0-nice, gstreamer1.0-pipewire, xdg-desktop-portal
Description: Compartilhamento de tela Linux para Android na rede local
 Versão inicial, sem validação em Android real ou benchmark comparativo.
CONTROL
cat > "$package_dir/usr/share/applications/telaleve.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=TelaLeve
Comment=Compartilhar tela com Android
Exec=telaleve
Icon=telaleve
Terminal=false
Categories=Utility;Network;
DESKTOP
mkdir -p dist
dpkg-deb --root-owner-group --build "$package_dir" "dist/telaleve_${version}_${architecture}.deb"
