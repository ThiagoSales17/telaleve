declare global { interface Window { __TAURI__: { core: { invoke: (name: string, args?: Record<string, unknown>) => Promise<any> } } } }
const $ = (id: string): any => document.getElementById(id);
const invoke = (name: string, args?: Record<string, unknown>) => window.__TAURI__.core.invoke(name, args);
const states = { aguardando: 'Aguardando celular', solicitando_aprovacao: 'Aprovação pendente', conectando: 'Conectando…', transmitindo: 'Transmitindo', reconectando: 'Reconectando · até 30 segundos', encerrado: 'Encerrado', origem_perdida: 'Origem indisponível · selecione e aplique outra tela', erro_porta_9876: 'A porta 9876 está ocupada', erro_servidor: 'Servidor indisponível' };
async function run(fn) { $('error').hidden = true; try { await fn(); } catch(e) { $('error').textContent = String(e); $('error').hidden = false; } }
let portalCapture = false;
async function refresh() { const selected = $('source').value; const sources = await invoke('list_sources'); $('source').replaceChildren(...sources.map(s => new Option(s.width && s.height ? `${s.name} · ${s.width}×${s.height}` : s.name, s.id))); if(sources.some(s=>s.id===selected)) $('source').value=selected; }
$('refresh').onclick = () => run(async()=> {
 if(portalCapture) await invoke('change_source',{source:'portal',quality:$('quality').value,reselect:true});
 await refresh();
});
$('begin').onclick = () => run(async()=> { const result = await invoke('begin',{source:$('source').value,ip:$('network').value,quality:$('quality').value}); $('qr').innerHTML = result.qr; $('url').textContent = result.url; $('url').removeAttribute('href'); $('pair').hidden = false; });
$('switch').onclick = () => run(()=>invoke('change_source',{source:$('source').value,quality:$('quality').value}));
$('approve').onclick = () => run(()=>invoke('approve',{allow:true}));
$('deny').onclick = () => run(()=>invoke('approve',{allow:false}));
$('end').onclick = () => run(async()=>{await invoke('end'); $('pair').hidden=true;});
run(async()=>{portalCapture=(await invoke('capture_backend'))==='portal'; if(portalCapture) $('refresh').textContent='Escolher tela ou janela'; await refresh(); const networks=await invoke('networks'); $('network').replaceChildren(...networks.map(n=>new Option(`${n.name} · ${n.ip}`,n.ip)));});
setInterval(()=>run(async()=>{const s=await invoke('status');$('status').textContent=states[s.state]||s.state;$('metrics').textContent=s.codec?`${s.codec} · ${(s.bitrate/1e6).toFixed(1)} Mbps`:'';$('pending').hidden=s.state!=='solicitando_aprovacao';if(!['aguardando','solicitando_aprovacao'].includes(s.state))$('pair').hidden=true;}),1000);

export {};
