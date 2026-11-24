# Contribuindo

Obrigado por considerar uma contribuição ao TelaLeve. O projeto integra Rust, Tauri, GStreamer e uma interface TypeScript; mudanças pequenas e verificáveis facilitam a revisão.

## Preparar o ambiente

Consulte [README.md](README.md) e [Linux](LINUX.md) para os requisitos de sistema. Em geral, o fluxo inicial é:

```sh
./scripts/install-deps.sh
./scripts/diagnose.sh
npm ci
```

## Verificações antes de abrir um pull request

Execute os checks relevantes para a mudança. O workflow [Linux](.github/workflows/linux.yml) é a referência para os comandos de CI.

```sh
npm run check
npm run build:web
npm test
python3 tests/linux-scripts.test.py
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --locked --manifest-path core-tests/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Os testes nativos de captura dependem de X11 e plugins GStreamer; veja `VALIDATION.md` antes de executá-los. Não declare validação em Wayland ou Android físico sem realizar e registrar os testes correspondentes.

## Pull requests

- Descreva o problema e a solução, incluindo contexto necessário.
- Mantenha cada PR focado e evite misturar refatorações não relacionadas.
- Inclua testes ou explique por que não são aplicáveis.
- Atualize a documentação quando mudar configuração, instalação, compatibilidade ou comportamento visível.
- Informe limitações conhecidas e plataformas efetivamente testadas.
- Não inclua binários, artefatos de build, dados pessoais ou segredos.

O projeto ainda está em desenvolvimento e a validação de algumas combinações de desktop, compositor e dispositivo permanece manual. Consulte o README antes de propor mudanças que alterem essas expectativas.
