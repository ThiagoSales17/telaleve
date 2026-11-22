#!/bin/sh
# Package an already compiled native binary; no SDK or root access is required.
set -eu
cd "$(dirname "$0")/.."
binary=${1:-src-tauri/target/release/telaleve}
[ -x "$binary" ] || { printf 'Binário não encontrado: %s\n' "$binary" >&2; exit 1; }
command -v flatpak >/dev/null 2>&1 || { printf 'Instale Flatpak para gerar o pacote.\n' >&2; exit 1; }
arch=$(flatpak --default-arch)
case "$arch" in x86_64|aarch64) ;; *) printf 'Arquitetura Flatpak ainda não suportada: %s\n' "$arch" >&2; exit 1 ;; esac
version=$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')
app_id=io.github.ThiagoSales17.TelaLeve
branch=stable
runtime_version=51
build_dir=$(mktemp -d)
trap 'rm -rf "$build_dir"' EXIT HUP INT TERM
app="$build_dir/app"
mkdir -p "$app/files/bin" "$app/files/share/applications" "$app/files/share/icons/hicolor/256x256/apps" "$app/files/share/licenses/$app_id"
install -m755 "$binary" "$app/files/bin/telaleve"
install -m644 src-tauri/icons/icon.png "$app/files/share/icons/hicolor/256x256/apps/$app_id.png"
install -m644 LICENSE "$app/files/share/licenses/$app_id/LICENSE"
cat > "$app/metadata" <<EOF
[Application]
name=$app_id
runtime=org.gnome.Platform/$arch/$runtime_version
command=telaleve
EOF
cat > "$app/files/share/applications/$app_id.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=TelaLeve
Comment=Compartilhar tela com Android na rede local
Exec=telaleve
Icon=$app_id
Terminal=false
Categories=Utility;Network;
EOF
flatpak build-finish --command=telaleve --socket=wayland --socket=fallback-x11 --share=network --share=ipc --device=dri "$app"
flatpak build-export "$build_dir/repo" "$app" "$branch"
mkdir -p dist
output="dist/TelaLeve-$version-$arch.flatpak"
flatpak build-bundle --arch="$arch" --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo "$build_dir/repo" "$output" "$app_id" "$branch"
printf '\nPacote: %s\nInstale com: flatpak install --user %s\n' "$output" "$output"
printf 'O runtime GNOME e o portal ScreenCast do desktop precisam estar disponíveis; veja LINUX.md.\n'
