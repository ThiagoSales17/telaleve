# TelaLeve AppImage

Arquivo: `TelaLeve-0.1.0-x86_64.AppImage` (aproximadamente 103 MB).
Alvo: Debian 13, x86_64/amd64, sessão X11. O tamanho inclui WebKitGTK e bibliotecas/plugins GStreamer; não representa o consumo de RAM.

## Executar

Na pasta onde salvou o arquivo:

```sh
chmod +x TelaLeve-0.1.0-x86_64.AppImage
./TelaLeve-0.1.0-x86_64.AppImage
```

Se o sistema não tiver suporte a FUSE:

```sh
./TelaLeve-0.1.0-x86_64.AppImage --appimage-extract-and-run
```

O segundo modo extrai temporariamente o conteúdo antes de executar. Não exige instalação por apt nem privilégios de administrador. Drivers gráficos, X11 e bibliotecas básicas do sistema continuam sendo fornecidos pelo Linux.

## Empacotamento e validação

- Binário otimizado da versão 0.1.0; AppDir preparado com linuxdeploy e convertido por appimagetool, provenientes dos repositórios oficiais.
- Incluídos WebKitWebProcess, WebKitNetworkProcess, WebKitGPUProcess, scanner GStreamer e plugins necessários para captura X11, VP8 e WebRTC/ICE.
- Avisos de copyright das bibliotecas e plugins preservados em `usr/share/doc` dentro do AppImage.
- Abertura do arquivo final validada em modo de extração: visualizador HTTP servido e conexão WebSocket com token inválido recusada.
- Seis testes passaram com caminhos de plugins do sistema desativados e somente os plugins empacotados: incluiu negociação SDP, vinte trocas entre monitor e janela e preservação da origem diante de uma solicitação inválida.
- Ainda não validado em instalação Debian limpa ou Android real. Sem benchmark comparativo contra Deskreen. Não há garantia de compatibilidade com distribuições anteriores ao Debian 13.

## Reproduzir o empacotamento

Instale as dependências Debian com `scripts/install-deps.sh`, execute `npm ci && npm run build:web` e `cargo build --release --manifest-path src-tauri/Cargo.toml`. Use linuxdeploy e appimagetool dos respectivos repositórios oficiais do GitHub, extraídos para executar sem FUSE quando necessário. O runtime x86_64 deve corresponder ao formato Type 2 do AppImage.

```sh
python3 scripts/package-appimage.py --binary src-tauri/target/release/telaleve --linuxdeploy /caminho/linuxdeploy/AppRun --appimagetool /caminho/appimagetool/AppRun --runtime /caminho/runtime-x86_64
```

O resultado é salvo em `dist/`. Os plugins vêm dos pacotes instalados no Debian 13; o script inclui WebKitGTK, auxiliares, plugins de vídeo e avisos de copyright.
