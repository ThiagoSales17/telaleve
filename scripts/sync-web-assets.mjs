import { copyFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const root = new URL('../', import.meta.url);
mkdirSync(new URL('web/', root), { recursive: true });
for (const [source, destination] of [
  ['frontend/desktop/index.html', 'web/index.html'],
  ['frontend/viewer/index.html', 'web/viewer.html'],
  ['frontend/styles/main.css', 'web/style.css'],
])
  copyFileSync(fileURLToPath(new URL(source, root)), fileURLToPath(new URL(destination, root)));
