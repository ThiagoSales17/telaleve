# Testes

| Diretório | O que verifica | Comando |
| --- | --- | --- |
| `frontend/` | Tela cheia, histórico e integridade dos assets/rotas | `npm test` |
| `linux/` | Instalador, diagnóstico e empacotador com ferramentas simuladas | `npm run test:linux` |
| `release/` | SemVer, versão dos manifests e validação da tag | `npm run test:release` |
| `core/` | Modelo Rust e seleção de backend sem GTK/GStreamer | `npm run test:core` |
| `src-tauri/src/` | Testes nativos próximos ao código de mídia e sessão | `npm run test:native` |

Execute `npm run build:web` antes dos testes frontend. As suítes Python não instalam pacotes nem alteram a sessão gráfica. Os testes de `core/` importam os mesmos módulos usados pelo aplicativo.

A integração de captura X11 é ignorada por padrão e exige servidor gráfico e plugins GStreamer. Execute em uma tela virtual isolada:

```sh
xvfb-run -a env XDG_SESSION_TYPE=x11 cargo test --locked --manifest-path src-tauri/Cargo.toml -- --include-ignored
```

A integração virtual não substitui validar Wayland, Android físico ou uma sessão prolongada. Veja [validação manual](../docs/VALIDATION.md).
