export function setupFullscreen(stage, enter, exit) {
    const doc = stage.ownerDocument;
    const body = doc.body;
    let expanded = false;
    const nativeActive = () => doc.fullscreenElement === stage || doc.webkitFullscreenElement === stage;
    function update() {
        const active = expanded || nativeActive();
        body.classList.toggle('immersive', expanded);
        enter.textContent = active ? 'Sair da tela cheia' : 'Tela cheia';
        enter.setAttribute('aria-pressed', String(active));
        exit.hidden = !active;
    }
    async function leave() {
        expanded = false;
        try {
            if (nativeActive()) {
                if (doc.exitFullscreen)
                    await doc.exitFullscreen();
                else if (doc.webkitExitFullscreen)
                    await doc.webkitExitFullscreen();
            }
        }
        catch { /* The browser may already have exited fullscreen. */ }
        update();
    }
    async function toggle() {
        if (expanded || nativeActive()) {
            await leave();
            return;
        }
        // Invoke directly from the tap, before any asynchronous operation.
        try {
            if (stage.requestFullscreen)
                await stage.requestFullscreen({ navigationUI: 'hide' });
            else if (stage.webkitRequestFullscreen)
                await stage.webkitRequestFullscreen();
            else
                expanded = true;
        }
        catch {
            // Embedded browsers can reject the API; still use the available viewport.
            expanded = true;
        }
        update();
    }
    enter.onclick = toggle;
    exit.onclick = leave;
    doc.addEventListener('fullscreenchange', update);
    doc.addEventListener('webkitfullscreenchange', update);
    doc.addEventListener('keydown', event => { if (event.key === 'Escape' && expanded)
        void leave(); });
    update();
}
