# Contribuindo

Obrigado por considerar uma contribuição ao TelaLeve. O projeto integra Rust, Tauri, GStreamer e uma interface TypeScript; mudanças pequenas e verificáveis facilitam a revisão.

## Preparar o ambiente

Consulte [README.md](README.md) e [Linux](docs/LINUX.md) para os requisitos de sistema. Em geral, o fluxo inicial é:

```sh
./scripts/install-deps.sh
./scripts/diagnose.sh
npm ci
```

Os comandos de metadados de release exigem Python 3.11 ou superior. O instalador não instala Rust; prepare o toolchain estável antes dos checks nativos.

## Testes

Execute as verificações relevantes para a mudança. O [workflow Linux](.github/workflows/linux.yml) define os ambientes e comandos de CI; consulte os resultados da execução para confirmar o que passou.

```sh
npm run check
npm run build:web
npm test
npm run test:linux
npm run test:release
npm run format:check
npm run test:core
npm run test:native
npm run lint:rust
```

| Suíte | Cobertura |
| --- | --- |
| `tests/frontend/` | Tela cheia, histórico e integridade dos assets e rotas |
| `tests/linux/` | Instalador, diagnóstico e empacotador com ferramentas simuladas |
| `tests/release/` | SemVer, versões dos manifests e validação da tag |
| `tests/core/` | Modelo Rust e seleção de backend sem GTK/GStreamer |
| `src-tauri/src/` | Mídia, captura e controle de sessão |

Compile os assets antes dos testes frontend. As suítes Python não instalam pacotes nem alteram a sessão gráfica. A integração X11 é ignorada por padrão; para executá-la com servidor gráfico virtual e plugins GStreamer disponíveis:

```sh
xvfb-run -a env XDG_SESSION_TYPE=x11 cargo test --locked --manifest-path src-tauri/Cargo.toml -- --include-ignored
```

### Validação manual

Antes de anunciar suporte em um ambiente, verifique:

- Pareamento, aprovação no desktop, recusa de código expirado e de segundo receptor.
- Vídeo no Android físico, codecs disponíveis e saída de tela cheia.
- Troca de monitor/janela e qualidade; falha ou remoção da origem.
- Interrupção da rede, reconexão e novo pareamento após expiração.
- Transmissão por 30 minutos, observando estabilidade e uso de memória.
- Seleção e cancelamento pelo portal em Wayland, em desktops representativos.
- Instalação em sistemas limpos e uso em hardware ARM64.

Testes X11 virtuais não comprovam transmissão em Wayland ou Android físico. Registre ambiente, versão/commit, passos e resultados na issue ou no pull request correspondente. O estado de compatibilidade e as pendências ficam no README.

## Pull requests

- Descreva o problema e a solução, incluindo contexto necessário.
- Mantenha cada PR focado e evite misturar refatorações não relacionadas.
- Inclua testes ou explique por que não são aplicáveis.
- Atualize a documentação quando mudar configuração, instalação, compatibilidade ou comportamento visível.
- Informe limitações conhecidas e plataformas efetivamente testadas.
- Não inclua binários, artefatos de build, dados pessoais ou segredos.

O projeto ainda está em desenvolvimento e a validação de algumas combinações de desktop, compositor e dispositivo permanece manual. Consulte o README antes de propor mudanças que alterem essas expectativas.

## Fontes e commits

Edite HTML, TypeScript e CSS em `frontend/` e rode `npm run build:web`; não edite `web/` manualmente. Use títulos curtos com prefixos `fix:`, `feat:`, `docs:`, `style:` ou `chore:`. Explique o motivo no corpo quando necessário. Na `main`, pushes geram prévias após os builds; branches de trabalho usam pull requests. A versão SemVer só muda quando preparada explicitamente.
