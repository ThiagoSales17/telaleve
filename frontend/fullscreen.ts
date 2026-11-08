type FullscreenDocument = Document & { webkitFullscreenElement?: Element; webkitExitFullscreen?: () => void | Promise<void> };
type FullscreenStage = HTMLElement & { webkitRequestFullscreen?: () => void | Promise<void> };

export function setupFullscreen(stage: FullscreenStage, enter: HTMLButtonElement) {
  const doc = stage.ownerDocument as FullscreenDocument;
  const body = doc.body;
  const win = doc.defaultView!;
  let expanded = false;
  const nativeActive = () => doc.fullscreenElement === stage || doc.webkitFullscreenElement === stage;
  function update() {
    const active = expanded || nativeActive();
    body.classList.toggle('immersive', expanded);
    enter.setAttribute('aria-pressed', String(active));

  }
  async function leave() {
    if (expanded) { win.history.back(); return; }
    try {
      if (nativeActive()) {
        if (doc.exitFullscreen) await doc.exitFullscreen();
        else if (doc.webkitExitFullscreen) await doc.webkitExitFullscreen();
      }
    } catch { /* The browser may already have exited fullscreen. */ }
    update();
  }
  function expandViewport() {
    win.history.pushState({telaleveExpanded: true}, '');
    expanded = true;
  }
  async function toggle() {
    if (expanded || nativeActive()) { await leave(); return; }
    // Invoke directly from the tap, before any asynchronous operation.
    try {
      if (stage.requestFullscreen) await stage.requestFullscreen({ navigationUI: 'hide' });
      else if (stage.webkitRequestFullscreen) await stage.webkitRequestFullscreen();
      else expandViewport();
    } catch {
      // Embedded browsers can reject the API; still use the available viewport.
      expandViewport();
    }
    update();
  }
  enter.onclick = toggle;
  win.addEventListener('popstate', event => {
    expanded = (event as PopStateEvent).state?.telaleveExpanded === true;
    update();
  });
  doc.addEventListener('fullscreenchange', update);
  doc.addEventListener('webkitfullscreenchange', update);
  doc.addEventListener('keydown', event => { if (event.key === 'Escape' && expanded) void leave(); });
  update();
}
