# Arquitetura

TelaLeve é um aplicativo desktop Linux com interface Tauri, emissor Rust/GStreamer e receptor web acessado pelo Chrome no Android. O fluxo de mídia usa WebRTC; a sinalização e a página do receptor são servidas localmente pelo aplicativo.

```text
Linux (X11 ou Wayland)
        │ captura (direta ou portal/PipeWire)
        ▼
Rust + GStreamer ─── sinalização HTTP/WebSocket local
        │                         │
        └──────── WebRTC ─────────┘
                                  ▼
                         Chrome no Android
```

## Componentes no repositório

- `src-tauri/src/` — aplicativo desktop, servidor local, controle de sessão e integração de mídia.
- `frontend/` — fontes TypeScript da interface desktop.
- `web/` — HTML/CSS e JavaScript do receptor e artefatos web gerados a partir de TypeScript.
- `core-tests/` — testes Rust isolados do ambiente gráfico e dos plugins de mídia.
- `tests/` — testes da interface e scripts Linux.
- `scripts/` — instalação de dependências, diagnóstico, empacotamento e medição.
- `.github/workflows/` — CI Linux e automação de releases.

## Limites importantes

A sinalização HTTP/WebSocket não usa TLS e o serviço escuta em IPv4. O uso previsto é em uma rede local confiável; não exponha diretamente o serviço à internet. O vídeo é transportado por WebRTC. Consulte o README e o guia Linux para requisitos e limitações atualizados.
