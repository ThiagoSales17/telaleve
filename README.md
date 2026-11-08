# TelaLeve

Emissor Linux/X11 em Rust + Tauri 2 e GStreamer; receptor Chrome para Android na rede local. Implementação inicial, ainda sem validação em celular real. Consulte VALIDATION.md antes de usar como versão pronta.

## Preparar e executar no Debian 13

Requer sessão X11, Rust estável e privilégios de administrador apenas para instalar dependências:

```sh
./scripts/install-deps.sh
./scripts/diagnose.sh
npm install
npm run dev
```

Selecione monitor ou janela, interface IPv4 e qualidade. Clique em **Gerar conexão**, leia o QR code no celular, toque em **Conectar** e aprove no computador. O código é de uso único e expira em cinco minutos. Use **Aplicar tela e qualidade** para trocar a origem. **Encerrar compartilhamento** revoga o acesso.

Computador e celular precisam estar na mesma rede, sem isolamento entre clientes. Permita TCP 9876 e o tráfego UDP negociado pelo WebRTC no firewall da interface local. Não configure encaminhamento de portas. A página e a sinalização são HTTP/WS locais, sem TLS; o vídeo é protegido pelo WebRTC. Use uma rede confiável. Um segundo receptor é recusado. O token não é enviado na URL HTTP: fica no fragmento do QR code, removido da barra ao carregar a página.

## Pacote Debian

```sh
./scripts/build-deb.sh
sudo apt install ./src-tauri/target/release/bundle/deb/*.deb
```

A entrega inclui um pacote otimizado inicial em `dist/telaleve_0.1.0-preview1_amd64.deb`, gerado por `scripts/package-binary.sh` a partir de `cargo build --release`. Para instalar esse pacote, use `sudo apt install ./dist/telaleve_0.1.0-preview1_amd64.deb`. O script acima permite gerar novamente o pacote pelo empacotador Tauri. A instalação em Debian limpo ainda precisa ser validada. O pacote declara plugins de vídeo, WebKitGTK e transporte ICE; encoders H.264 por hardware dependem de drivers e plugins opcionais instalados no computador.

## Código e testes

- `src-tauri/src`: comandos desktop, servidor local, controle de acesso, enumeração X11 e pipeline.
- `frontend`: fontes TypeScript; `web`: HTML/CSS e JavaScript gerado.
- `core-tests`: testa o mesmo modelo de acesso sem depender de GTK/GStreamer.

```sh
cargo test --manifest-path core-tests/Cargo.toml
npm run check
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

O encoder tenta H.264 (VA, NVIDIA ou QSV) se oferecido pelo navegador e se um pipeline de teste produzir EOS. Sem um encoder funcional, usa VP8. Não é uma certificação de compatibilidade H.264: o teste ponta a ponta ainda é obrigatório. O ajuste de bitrate usa perda e jitter relatados pelo receptor, com limites por perfil. As resoluções são limites máximos; o vídeo menor é centralizado com bordas, sem ampliação. Trocas de captura aguardam um primeiro quadro e mantêm a conexão; falhas preservam a origem anterior. Origem perdida exige escolha explícita e reinicia a negociação automaticamente após a seleção.

A sessão tenta retomar por até 30 segundos após detectar a desconexão; a detecção de conexão silenciosa usa ping a cada cinco segundos. O segredo de retomada fica apenas na memória da página. Recarregar/fechar a página exige novo QR code.

## Próximas validações obrigatórias

Executar a lista em VALIDATION.md em Debian 13 e Android antes de distribuir. Wayland, áudio, controle remoto, monitor virtual, internet e vários receptores estão fora desta versão. Não há alegação de ganho de desempenho sem medição comparativa.

## Medir consumo

Com o emissor transmitindo, encontre o PID e execute `python3 scripts/measure.py PID --seconds 1800 --output telaleve.csv`. Repita com o PID principal do Deskreen e a mesma configuração. O CSV soma RSS da árvore de processos (incluindo páginas compartilhadas contadas em cada processo) e informa CPU em relação a um núcleo. Não mede consumo da GPU nem atraso entre telas.

## AppImage

Também disponível como `TelaLeve-0.1.0-x86_64.AppImage`, sem instalação via apt. Consulte [APPIMAGE.md](APPIMAGE.md) para execução, modo sem FUSE e validações.

## GitHub

Instaladores AppImage e Debian estão nas [Releases privadas](https://github.com/ThiagoSales17/telaleve/releases). É necessário entrar na conta com acesso ao repositório. Binários, caches e dependências não são versionados no Git. Consulte HISTORY.md para a organização retrospectiva dos commits.
