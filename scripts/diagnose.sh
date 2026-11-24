#!/bin/sh
set -u
failed=0
printf 'Sessão: %s\nDisplay: %s\nWayland: %s\nArquitetura: %s\n' "${XDG_SESSION_TYPE:-desconhecida}" "${DISPLAY:-ausente}" "${WAYLAND_DISPLAY:-ausente}" "$(uname -m)"
portal=false
if [ "${XDG_SESSION_TYPE:-}" = wayland ]; then portal=true
elif [ "${XDG_SESSION_TYPE:-}" != x11 ] && [ -n "${WAYLAND_DISPLAY:-}" ]; then portal=true; fi
if "$portal"; then capture=pipewiresrc; printf 'Captura: portal/PipeWire\n'
else capture=ximagesrc; printf 'Captura: X11\n'; [ -n "${DISPLAY:-}" ] || { printf 'AUSENTE sessão gráfica X11\n'; failed=1; }; fi
if ! command -v gst-inspect-1.0 >/dev/null 2>&1; then
    printf 'AUSENTE gst-inspect-1.0; instale as ferramentas GStreamer.\n'; failed=1
else
    for element in "$capture" videoconvert videorate videoscale videobox appsink appsrc vp8enc rtpvp8pay webrtcbin nicesrc; do
        if gst-inspect-1.0 "$element" >/dev/null 2>&1; then printf 'OK %s\n' "$element"; else printf 'AUSENTE %s\n' "$element"; failed=1; fi
    done
    printf '\nEncoders opcionais:\n'
    for element in vah264enc nvh264enc qsvh264enc; do
        if gst-inspect-1.0 "$element" >/dev/null 2>&1; then printf '%s instalado (exige teste de funcionamento)\n' "$element"; fi
    done
fi
if "$portal"; then
    if command -v gdbus >/dev/null 2>&1 && gdbus introspect --session --dest org.freedesktop.portal.Desktop --object-path /org/freedesktop/portal/desktop 2>/dev/null | grep -q 'org.freedesktop.portal.ScreenCast'; then
        printf 'OK portal ScreenCast (a seleção real ainda precisa ser testada)\n'
    else printf 'AUSENTE portal ScreenCast acessível; configure o backend do desktop conforme docs/LINUX.md.\n'; failed=1; fi
fi
exit "$failed"
