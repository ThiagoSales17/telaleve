import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, access } from 'node:fs/promises';

const web = new URL('../../web/', import.meta.url);
async function modules(url, seen = new Set()) {
  if (seen.has(url.href)) return seen;
  seen.add(url.href);
  const source = await readFile(url, 'utf8');
  for (const match of source.matchAll(/(?:from\s*|import\s*)['"]([^'"]+)['"]/g)) {
    assert.ok(match[1].startsWith('.'), 'O navegador exige imports relativos sem bundler');
    await modules(new URL(match[1], url), seen);
  }
  return seen;
}

test('desktop e viewer referenciam assets existentes após o build', async () => {
  for (const page of ['index.html', 'viewer.html']) {
    const url = new URL(page, web);
    const html = await readFile(url, 'utf8');
    for (const match of html.matchAll(/(?:src|href)="([^"]+)"/g)) {
      const asset = new URL(match[1], url);
      await access(asset);
      if (asset.pathname.endsWith('.js')) await modules(asset);
    }
  }
});

test('servidor publica os módulos e estilos usados pelo receptor', async () => {
  const server = await readFile(new URL('../../src-tauri/src/server.rs', import.meta.url), 'utf8');
  const viewer = await readFile(new URL('viewer.html', web), 'utf8');
  const script = [...viewer.matchAll(/src="([^"]+)"/g)][0][1];
  const graph = await modules(new URL(script, web));
  for (const url of graph) {
    const path = new URL(url).pathname.slice(web.pathname.length);
    assert.ok(server.includes(`"/${path}"`), `Rota ausente: ${path}`);
    assert.ok(server.includes(`../../web/${path}`), `Asset Rust ausente: ${path}`);
  }
  assert.ok(server.includes('"/style.css"'));
});
