#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
npm ci
npm run build
printf '\nPacote disponível em src-tauri/target/release/bundle/deb/\n'
