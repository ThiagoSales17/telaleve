#!/bin/sh
# Install build/runtime dependencies without changing the session or audio setup.
set -eu
dry_run=false
case "${1:-}" in
    --dry-run) dry_run=true ;;
    '') ;;
    *) printf 'Uso: %s [--dry-run]\n' "$0" >&2; exit 2 ;;
esac
os_release=${TELALEVE_OS_RELEASE:-/etc/os-release}
[ -r "$os_release" ] || { printf 'Distribuição não identificada; consulte LINUX.md (Flatpak).\n' >&2; exit 1; }
# os-release is a trusted system configuration; the override is useful for tests.
. "$os_release"
family=''
for candidate in "${ID:-}" ${ID_LIKE:-}; do
    case "$candidate" in
        debian|ubuntu|linuxmint|pop) family=debian; break ;;
        fedora|rhel|centos) family=fedora; break ;;
        arch|manjaro|endeavouros) family=arch; break ;;
        opensuse*|suse|sles) family=suse; break ;;
    esac
done
run() {
    if "$dry_run"; then printf '%s' "$1"; shift; printf ' %s' "$@"; printf '\n';
    elif [ "$(id -u)" = 0 ]; then "$@";
    else sudo "$@"; fi
}
case "$family" in
    debian)
        run apt-get update
        run apt-get install -y build-essential pkg-config curl python3 npm libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev libxdo-dev libx11-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-nice gstreamer1.0-pipewire xdg-desktop-portal
        ;;
    fedora)
        run dnf install -y gcc gcc-c++ make pkgconf-pkg-config curl python3 nodejs npm gtk3-devel webkit2gtk4.1-devel librsvg2-devel openssl-devel libX11-devel libxdo-devel gstreamer1-devel gstreamer1-plugins-base-devel gstreamer1-plugins-bad-free-devel gstreamer1-plugins-base gstreamer1-plugins-good gstreamer1-plugins-bad-free libnice-gstreamer1 pipewire-gstreamer xdg-desktop-portal
        ;;
    arch)
        run pacman -S --needed --noconfirm base-devel pkgconf curl python nodejs npm gtk3 webkit2gtk-4.1 librsvg openssl libx11 xdotool gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad libnice gst-plugin-pipewire xdg-desktop-portal
        ;;
    suse)
        run zypper --non-interactive install gcc gcc-c++ make pkg-config curl python3 nodejs npm gtk3-devel webkit2gtk3-soup3-devel librsvg-devel libopenssl-devel libX11-devel libxdo-devel gstreamer-devel gstreamer-plugins-base-devel gstreamer-plugins-bad-devel gstreamer-plugins-base gstreamer-plugins-good gstreamer-plugins-bad gstreamer-libnice gstreamer-plugin-pipewire xdg-desktop-portal
        ;;
    *) printf 'Use o pacote Flatpak ou prepare as dependências descritas em LINUX.md para %s.\n' "${ID:-este sistema}" >&2; exit 1 ;;
esac
printf '\nPara Wayland/Flatpak, habilite o backend ScreenCast do seu desktop (GNOME, KDE, wlr ou Hyprland); veja LINUX.md.\n'
printf 'Rust estável deve estar instalado. Verifique o ambiente com ./scripts/diagnose.sh.\n'
