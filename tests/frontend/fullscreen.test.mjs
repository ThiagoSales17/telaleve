import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
const source = await readFile(new URL('../../web/viewer/fullscreen.js', import.meta.url), 'utf8');
const { setupFullscreen } = await import(
  `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`
);
function fixture() {
  const doc = new EventTarget();
  const classes = new Set();
  doc.body = {
    classList: {
      toggle(name, active) {
        if (active) classes.add(name);
        else classes.delete(name);
      },
    },
  };
  const win = new EventTarget();
  const stack = [null];
  let position = 0;
  const move = () => {
    const event = new Event('popstate');
    Object.defineProperty(event, 'state', { value: stack[position] });
    win.dispatchEvent(event);
  };
  win.history = {
    pushState(state) {
      stack.splice(++position);
      stack.push(state);
    },
    back() {
      if (position > 0) {
        position--;
        move();
      }
    },
    forward() {
      if (position + 1 < stack.length) {
        position++;
        move();
      }
    },
  };
  doc.defaultView = win;
  const stage = { ownerDocument: doc };
  const button = () => ({
    hidden: false,
    attrs: {},
    setAttribute(k, v) {
      this.attrs[k] = v;
    },
  });
  const enter = button(),
    exit = button();
  doc.exitFullscreen = async () => {
    doc.fullscreenElement = null;
    doc.dispatchEvent(new Event('fullscreenchange'));
  };
  return { doc, stage, enter, classes, win, stack };
}
test('fullscreen targets only video stage and follows browser exit', async () => {
  const f = fixture();
  let requested = false;
  f.stage.requestFullscreen = function (options) {
    assert.equal(this, f.stage);
    assert.deepEqual(options, { navigationUI: 'hide' });
    requested = true;
    f.doc.fullscreenElement = f.stage;
    f.doc.dispatchEvent(new Event('fullscreenchange'));
    return Promise.resolve();
  };
  setupFullscreen(f.stage, f.enter);
  const pending = f.enter.onclick();
  assert.equal(requested, true, 'request must occur in the tap handler before awaiting');
  await pending;
  assert.equal(f.enter.attrs['aria-pressed'], 'true');
  await f.doc.exitFullscreen();
  assert.equal(f.enter.attrs['aria-pressed'], 'false');
});
test('native mode adds no history entry and browser exit restores state', async () => {
  const f = fixture();
  f.stage.requestFullscreen = async () => {
    f.doc.fullscreenElement = f.stage;
  };
  setupFullscreen(f.stage, f.enter);
  await f.enter.onclick();
  assert.equal(f.stack.length, 1);
  await f.doc.exitFullscreen();
  assert.equal(f.doc.fullscreenElement, null);
});
test('unavailable API expands viewport and Escape restores controls', async () => {
  const f = fixture();
  setupFullscreen(f.stage, f.enter);
  await f.enter.onclick();
  assert.ok(f.classes.has('immersive'));
  const event = new Event('keydown');
  Object.defineProperty(event, 'key', { value: 'Escape' });
  f.doc.dispatchEvent(event);
  assert.ok(!f.classes.has('immersive'));
});
test('denied API falls back and exit restores page', async () => {
  const f = fixture();
  f.stage.requestFullscreen = () => Promise.reject(new Error('denied'));
  setupFullscreen(f.stage, f.enter);
  await f.enter.onclick();
  assert.ok(f.classes.has('immersive'));
  f.win.history.back();
  assert.ok(!f.classes.has('immersive'));
  assert.equal(f.stack.length, 2);
  f.win.history.forward();
  assert.ok(f.classes.has('immersive'));
  f.win.history.back();
});
test('prefixed API enters and exits', async () => {
  const f = fixture();
  f.stage.webkitRequestFullscreen = () => {
    f.doc.webkitFullscreenElement = f.stage;
  };
  f.doc.exitFullscreen = undefined;
  f.doc.webkitExitFullscreen = () => {
    f.doc.webkitFullscreenElement = null;
  };
  setupFullscreen(f.stage, f.enter);
  await f.enter.onclick();
  f.doc.webkitExitFullscreen();
  f.doc.dispatchEvent(new Event('webkitfullscreenchange'));
  assert.equal(f.enter.attrs['aria-pressed'], 'false');
});

test('viewer has no exit overlay', async () => {
  const html = await readFile(new URL('../../web/viewer.html', import.meta.url), 'utf8');
  assert.ok(!html.includes('exit-fullscreen'));
  assert.ok(!html.includes('Sair da tela cheia'));
});
