import { setupFullscreen } from './fullscreen.js';
const video = document.getElementById('video'), status = document.getElementById('status'), start = document.getElementById('start');
const initial = location.hash.slice(1);
history.replaceState(null, '', location.pathname);
let resume = null, ws = null, pc = null, stopped = false, deadline = 0, timer = null, feedback = null, remoteIce = [], lastPackets = 0, lastLost = 0;
function show(text) {
    status.textContent = text;
}
function send(data) {
    if (ws?.readyState === WebSocket.OPEN)
        ws.send(JSON.stringify(data));
}
function cleanup() {
    clearInterval(feedback);
    if (pc) {
        pc.onconnectionstatechange = null;
        pc.close();
        pc = null;
    }
}
async function connect() {
    cleanup();
    remoteIce = [];
    lastPackets = 0;
    lastLost = 0;
    ws = new WebSocket(`ws://${location.host}/ws`);
    ws.onopen = () => send({ token: resume || initial, resume: !!resume });
    ws.onmessage = async ({ data }) => {
        try {
            const msg = JSON.parse(data);
            if (msg.type === 'ready') {
                pc = new RTCPeerConnection({ iceServers: [] });
                pc.addTransceiver('video', { direction: 'recvonly' });
                pc.onicecandidate = (e) => {
                    if (e.candidate)
                        send({
                            type: 'ice',
                            candidate: e.candidate.candidate,
                            sdpMLineIndex: e.candidate.sdpMLineIndex,
                        });
                };
                pc.ontrack = (e) => {
                    video.srcObject = e.streams[0] || new MediaStream([e.track]);
                    video.play().catch(() => show('Toque em Conectar para reproduzir'));
                };
                pc.onconnectionstatechange = () => {
                    if (pc.connectionState === 'connected') {
                        deadline = 0;
                        send({ type: 'connected' });
                        show('Conectado');
                    }
                    else if (pc.connectionState === 'failed') {
                        ws.close();
                    }
                };
                const offer = await pc.createOffer();
                await pc.setLocalDescription(offer);
                send({ type: 'offer', sdp: offer.sdp });
                show(msg.approved ? 'Reconectando…' : 'Aprove a conexão no computador.');
            }
            else if (msg.type === 'approved') {
                resume = msg.resume;
                show('Conectando vídeo…');
            }
            else if (msg.type === 'answer') {
                await pc.setRemoteDescription({ type: 'answer', sdp: msg.sdp });
                for (const c of remoteIce)
                    await pc.addIceCandidate(c);
                remoteIce = [];
                clearInterval(feedback);
                feedback = setInterval(async () => {
                    if (!pc || pc.connectionState !== 'connected')
                        return;
                    try {
                        const stats = await pc.getStats();
                        stats.forEach((s) => {
                            if (s.type === 'inbound-rtp' && s.kind === 'video') {
                                const received = Math.max(0, s.packetsReceived - lastPackets), lost = Math.max(0, s.packetsLost - lastLost);
                                lastPackets = s.packetsReceived;
                                lastLost = s.packetsLost;
                                send({
                                    type: 'feedback',
                                    loss: lost / Math.max(1, received + lost),
                                    jitter: s.jitter || 0,
                                });
                            }
                        });
                    }
                    catch { }
                }, 2000);
            }
            else if (msg.type === 'ice') {
                const c = {
                    candidate: msg.candidate,
                    sdpMLineIndex: msg.sdpMLineIndex,
                };
                if (pc?.remoteDescription)
                    await pc.addIceCandidate(c);
                else
                    remoteIce.push(c);
            }
            else if (msg.type === 'retry') {
                ws.close();
            }
            else if (msg.type === 'restart') {
                ws.close();
            }
            else if (msg.type === 'source_lost') {
                video.srcObject = null;
                show('A origem foi fechada. Escolha outra tela no computador.');
            }
            else if (msg.type === 'ended' || msg.type === 'error') {
                stopped = true;
                resume = null;
                cleanup();
                video.srcObject = null;
                show(msg.message || 'Compartilhamento encerrado. Gere um novo QR code.');
                ws.close();
            }
        }
        catch {
            show('Não foi possível negociar o vídeo. Gere uma nova conexão.');
        }
    };
    ws.onclose = () => {
        cleanup();
        if (stopped)
            return;
        if (!resume) {
            show('Conexão encerrada. Gere um novo QR code.');
            return;
        }
        if (!deadline)
            deadline = Date.now() + 30000;
        if (Date.now() < deadline) {
            show('Reconectando…');
            clearTimeout(timer);
            timer = setTimeout(connect, 1000);
        }
        else {
            stopped = true;
            resume = null;
            show('Tempo de reconexão encerrado. Gere um novo QR code.');
        }
    };
    ws.onerror = () => show('Confira o Wi-Fi e a porta 9876 no computador.');
}
start.onclick = () => {
    if (!initial && !resume) {
        show('Leia o QR code no computador.');
        return;
    }
    if (ws?.readyState === WebSocket.OPEN) {
        video.play();
        return;
    }
    stopped = false;
    deadline = 0;
    connect();
};
setupFullscreen(document.getElementById('stage'), document.getElementById('fullscreen'));
document.getElementById('fit').onclick = (e) => {
    const fill = video.style.objectFit !== 'cover';
    video.style.objectFit = fill ? 'cover' : 'contain';
    e.target.textContent = fill ? 'Mostrar tela inteira' : 'Preencher tela';
};
window.addEventListener('pagehide', () => {
    stopped = true;
    clearTimeout(timer);
    cleanup();
    ws?.close();
});
