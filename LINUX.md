# Compatibilidade Linux

O TelaLeve 0.2.0 amplia o projeto para desktops Linux com X11 ou Wayland. O pacote Flatpak é a opção para usar o mesmo aplicativo entre distribuições com bibliotecas diferentes. Isso não equivale a uma garantia de funcionamento em toda versão, arquitetura ou compositor existente: os resultados efetivamente obtidos ficam em VALIDATION.md.

## Formatos e requisitos

| Ambiente | Caminho de instalação | Captura |
|---|---|---|
| Debian, Ubuntu e derivados | Flatpak ou build nativo com APT | X11 ou portal/PipeWire |
| Fedora e derivados | Flatpak ou build nativo com DNF | X11 ou portal/PipeWire |
| Arch, Manjaro e derivados | Flatpak ou build nativo com Pacman | X11 ou portal/PipeWire |
| openSUSE | Flatpak ou build nativo com Zypper | X11 ou portal/PipeWire |
| Outras distribuições com Flatpak | Flatpak, com runtime GNOME 51 disponível | X11 ou portal/PipeWire |

O empacotador Flatpak aceita **x86_64 e aarch64**. Gere um binário nativo para cada arquitetura; renomear um binário não o converte. Não há artefato ARM64 validado neste computador. Sistemas sem sessão gráfica ou sem suporte ao runtime necessário precisam de preparação adicional. Em Wayland, um portal ScreenCast também é obrigatório. Não há suporte binário anunciado a 32 bits ou RISC-V.

Para compilar nativamente: Rust estável, Node.js 22 LTS/npm (ou outro Node compatível com o toolchain), compilador C/C++, GTK 3, WebKitGTK **4.1**, GStreamer **1.20 ou superior**, plugins VP8/WebRTC/ICE e bibliotecas X11. Em Wayland, também são necessários PipeWire, o elemento GStreamer `pipewiresrc` e um backend de `xdg-desktop-portal` com ScreenCast.

## Instalar um Flatpak pronto

Os pacotes x86_64 e ARM64 estão na [prévia 0.2.0](https://github.com/ThiagoSales17/telaleve/releases/tag/v0.2.0). Baixe o arquivo correspondente à arquitetura e execute na pasta do download (para ARM64, substitua `x86_64` por `aarch64`):

```sh
flatpak install --user ./TelaLeve-0.2.0-x86_64.flatpak
flatpak run io.github.ThiagoSales17.TelaLeve
```

O pacote informa o endereço do repositório de runtimes do Flathub. O gerenciador pode pedir a instalação de `org.gnome.Platform//51`. O aplicativo não precisa estar publicado no catálogo do Flathub para ser instalado pelo arquivo.

O Flatpak tem acesso à rede local, à aceleração gráfica e aos sockets gráficos necessários. Em Wayland a captura passa pelo portal e exige autorização do sistema. Em X11, a captura é direta, inclusive dentro do sandbox, para funcionar também em desktops sem portal ScreenCast. O pacote não tem acesso irrestrito aos arquivos pessoais.

## Preparar o ambiente nativo

```sh
./scripts/install-deps.sh --dry-run
./scripts/install-deps.sh
./scripts/diagnose.sh
npm ci
npm run dev
```

O instalador detecta a família da distribuição usando `/etc/os-release`; `--dry-run` apenas mostra os comandos. Ele instala dependências com APT, DNF, Pacman ou Zypper e não troca a sessão gráfica ou a configuração de áudio. Em sistemas imutáveis, prefira o Flatpak. O script não instala Rust automaticamente. Em distribuições antigas, o Node.js dos repositórios pode ser insuficiente; prepare uma versão compatível antes de `npm ci`.

Distros antigas que não oferecem WebKitGTK 4.1/GStreamer 1.20 não passam a ser compatíveis apenas por instalar um `.deb` recente. Use o Flatpak, se o runtime e o kernel forem compatíveis. Para Alpine, Gentoo e NixOS, o build nativo exige a preparação específica de dependências descrita na documentação do Tauri; o instalador automático não gerencia esses ambientes.

## Wayland e seleção de tela

Configure um backend ScreenCast compatível com o desktop, por exemplo:

- GNOME: `xdg-desktop-portal-gnome`.
- KDE Plasma: `xdg-desktop-portal-kde`.
- Sway e outros compositores wlroots: `xdg-desktop-portal-wlr`.
- Hyprland: `xdg-desktop-portal-hyprland`.

Os nomes e a disponibilidade dos pacotes variam entre distribuições. O backend GTK, sozinho, não substitui um backend ScreenCast. O compositor também determina se é possível escolher uma janela ou somente um monitor.

1. Abra o TelaLeve em sua sessão gráfica normal.
2. Clique em **Escolher tela ou janela**, ou em **Gerar conexão** para abrir a seleção na primeira vez.
3. Escolha a origem no diálogo do sistema.
4. Leia o QR code com o Android e aprove a conexão no TelaLeve.

**Aplicar tela e qualidade** reutiliza a autorização para ajustar o perfil. **Escolher tela ou janela** solicita uma nova seleção; cancelar preserva a origem anterior. **Encerrar compartilhamento** fecha a sessão de captura. Autorizações não são persistidas. Cada preparação de pipeline abre uma conexão PipeWire independente, permitindo trocas e retomadas sem reutilizar um socket encerrado.

Os quadros usam suas dimensões físicas para calcular a escala; coordenadas lógicas do portal não determinam a resolução do vídeo. Entradas pequenas são centralizadas sem ampliação.

## Gerar pacotes

Depois de preparar as dependências, na raiz do projeto:

```sh
npm ci
npm run build:web
cargo build --release --locked --manifest-path src-tauri/Cargo.toml
./scripts/package-flatpak.sh
```

O empacotador Flatpak usa o binário nativo, inclui o plugin `ximagesrc` e seu aviso de licença do sistema de build, e usa o runtime GNOME 51; não recompila o código dentro de um SDK. O ambiente de empacotamento precisa de `pkg-config` e dos plugins GStreamer de captura X11. O binário precisa ser compatível com as bibliotecas desse runtime. Antes de distribuir, teste o pacote em uma instalação limpa. Para builds reproduzíveis no runtime, use o SDK GNOME da mesma versão e as ferramentas oficiais de empacotamento Tauri/Flatpak.

Para um `.deb` da arquitetura local em Debian/Ubuntu:

```sh
./scripts/package-binary.sh
```

A versão é lida de `package.json` e a arquitetura de `dpkg --print-architecture`. Compile nativamente antes de empacotar. Esse caminho não produz RPM nem um binário universal.

O empacotador manual de AppImage em `scripts/package-appimage.py` continua específico do ambiente Debian x86_64 descrito em APPIMAGE.md. Ele inclui os plugins de captura X11 e PipeWire; não é o caminho de distribuição universal.

## Verificações

```sh
python3 tests/linux-scripts.test.py
npm run check
npm run build:web
npm test
cargo test --manifest-path core-tests/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

O workflow Linux verifica builds e testes em Ubuntu 22.04/24.04 x86_64 e Ubuntu 24.04 ARM64, exercita captura X11 em tela virtual e testa os ambientes de build de Fedora, Arch e openSUSE em containers. A seleção real em Wayland e a experiência no Android exigem validação separada. Consulte a execução do workflow para distinguir configuração de CI de testes concluídos.

## Referências de implementação

- [Dependências Linux do Tauri](https://v2.tauri.app/start/prerequisites/#linux).
- [Distribuição Flatpak com Tauri](https://v2.tauri.app/distribute/flatpak/).
- [Portal ScreenCast](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html).
