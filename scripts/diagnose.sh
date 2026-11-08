#!/bin/sh
set -u
printf 'Sessão: %s\nDisplay: %s\n' "${XDG_SESSION_TYPE:-desconhecida}" "${DISPLAY:-ausente}"
for element in ximagesrc input-selector videoconvert videoscale videobox vp8enc rtpvp8pay webrtcbin nicesrc; do
 if gst-inspect-1.0 "$element" >/dev/null 2>&1; then printf 'OK %s\n' "$element"; else printf 'AUSENTE %s\n' "$element"; fi
done
printf '\nEncoders opcionais:\n'
for element in vah264enc nvh264enc qsvh264enc; do gst-inspect-1.0 "$element" >/dev/null 2>&1 && printf '%s instalado (ainda exige teste de funcionamento)\n' "$element"; done
exit 0
