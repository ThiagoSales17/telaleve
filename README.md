<div align="center">

<img src="docs/assets/telaleve-banner.png" alt="TelaLeve — sua tela Linux no Android" width="100%" />

# TelaLeve

**Sua tela Linux no Android. Sem cabos, sem nuvem, na sua rede local.**

Compartilhe seu monitor ou uma janela do Linux com o Chrome no Android, sem instalar um aplicativo no celular.

![Status](https://img.shields.io/badge/status-X11%20validado-333333?style=flat-square)
![Linux](https://img.shields.io/badge/Linux-X11%20%2F%20Wayland-333333?style=flat-square&logo=linux&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-Tauri%202-B7410E?style=flat-square&logo=rust&logoColor=white)
![WebRTC](https://img.shields.io/badge/streaming-WebRTC-2563EB?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-333333?style=flat-square)

[**Download**](https://github.com/ThiagoSales17/telaleve/releases) · [Instalação](#instalação) · [Como usar](#como-usar) · [Documentação](#documentação)

</div>

---

## O que é o TelaLeve?

TelaLeve é um aplicativo desktop desenvolvido em **Rust, Tauri 2 e GStreamer** que transmite a imagem de um monitor ou janela Linux para o navegador do celular por **WebRTC**.

- Captura de monitor ou janela em X11; Wayland experimental.
- Conexão local por QR code, sem serviços de nuvem.
- Codificação H.264 quando disponível, com fallback para VP8.
- Bitrate adaptativo e quatro perfis de qualidade.
- Troca de origem durante a transmissão e tentativa de reconexão automática.
- Pareamento com aprovação explícita no computador.

> [!NOTE]
> Instalação e uso com celular Android foram validados pelo mantenedor em X11. Wayland, ARM64 e outras combinações ainda não validadas são experimentais. A versão 1.0.0 é estável para o ambiente X11 validado.

## Instalação

Baixe os pacotes na página de [Releases](https://github.com/ThiagoSales17/telaleve/releases).

### Flatpak — recomendado

Disponível para x86_64 e ARM64 (experimental). Baixe o arquivo correspondente à sua arquitetura. Exemplo para x86_64:

```bash
flatpak install --user ./TelaLeve-1.0.0-x86_64.flatpak
flatpak run io.github.ThiagoSales17.TelaLeve
```

O pacote utiliza o runtime GNOME 51. Consulte [LINUX.md](docs/LINUX.md) para requisitos e instruções adicionais.

### Debian / Ubuntu (.deb)

Após baixar o pacote correspondente à sua arquitetura, instale-o com `apt`. Exemplo para amd64:

```bash
sudo apt install ./telaleve_1.0.0_amd64.deb
```

O pacote depende de bibliotecas compatíveis com o sistema. Para maior portabilidade entre distribuições, prefira o Flatpak.

### Executar pelo código-fonte

Requer Rust estável, Node.js/npm, GTK 3, WebKitGTK 4.1 e GStreamer 1.20 ou superior.

Na raiz do projeto:

```bash
./scripts/install-deps.sh
./scripts/diagnose.sh
npm ci
npm run dev
```

## Como usar

1. Conecte o computador e o celular à mesma rede local.
2. Selecione o monitor ou a janela que deseja compartilhar. Em Wayland, autorize a captura no seletor do sistema.
3. Escolha a interface IPv4 e o perfil de qualidade.
4. Clique em **Gerar conexão** e escaneie o QR code no Android.
5. Toque em **Conectar** no Chrome e aprove a conexão no computador.

Use **Aplicar tela e qualidade** para alterar a captura ou o perfil durante a sessão. Para finalizar, clique em **Encerrar compartilhamento**.

## Como funciona

```mermaid
flowchart LR
    A["Linux · X11 / Wayland"] --> B["GStreamer"]
    B --> C["WebRTC"]
    C --> D["Chrome · Android"]
```

O aplicativo desktop utiliza Rust e Tauri 2 para gerenciar a captura, o pareamento e a sessão. O GStreamer processa e codifica o vídeo, enquanto o WebRTC realiza a transmissão pela rede local, com sinalização HTTP/WebSocket.

O bitrate é ajustado conforme a perda de pacotes e o jitter. Em caso de desconexão detectada, o sistema tenta recuperar a sessão por até 30 segundos. Recarregar ou fechar a página exige um novo pareamento.

### Perfis de qualidade

| Perfil | Resolução máxima | FPS solicitado | Bitrate inicial / máximo |
| :-- | :--: | --: | :--: |
| Econômico | 720p | 20 | 1,5 / 2,5 Mbps |
| **Equilibrado (padrão)** | 720p | 30 | 2,5 / 4 Mbps |
| Fluidez | 720p | 60 | 4 / 6 Mbps |
| Nitidez | 1080p | 30 | 4 / 6 Mbps |

A resolução e a taxa de quadros efetivas dependem da origem, do hardware, da rede e do receptor.

## Compatibilidade

| Ambiente | Suporte |
| :-- | :-- |
| X11 | Instalação e transmissão para Android físico validadas pelo mantenedor |
| Wayland | **Experimental** — captura via ScreenCast/portal e PipeWire; validação prática pendente |
| Debian / Ubuntu | Script de dependências e pacote `.deb`; instalações fora do ambiente validado são experimentais |
| Fedora / Arch / openSUSE | **Experimental** — verificações de build em CI; validação de uso real pendente |
| x86_64 | Builds e testes nativos automatizados disponíveis |
| ARM64 | **Experimental** — pacotes disponíveis; validação em equipamento real pendente |
| Android | Uso em celular físico validado pelo mantenedor; outros aparelhos e navegadores são experimentais até validação |

A validação cobre o ambiente X11 e o celular utilizados pelo mantenedor; não certifica todas as distribuições, arquiteturas, drivers ou navegadores. Combinações sem validação prática permanecem experimentais, mesmo quando compilam no CI. Consulte [LINUX.md](docs/LINUX.md) para requisitos específicos.

O escopo atual é a transmissão de vídeo para um único receptor. Áudio, controle remoto, monitor virtual e acesso pela internet não fazem parte desta versão.

## Segurança

- Código de pareamento de uso único, válido por cinco minutos.
- Aprovação obrigatória no desktop e um receptor por sessão.
- Vídeo protegido pelo WebRTC; página e sinalização locais via HTTP/WS, sem TLS.
- Token transportado no fragmento da URL, sem envio ao servidor HTTP.

> [!IMPORTANT]
> Utilize somente redes locais confiáveis. O servidor escuta em `0.0.0.0:9876`; restrinja o acesso pelo firewall, permitindo TCP 9876 e o tráfego UDP negociado pelo WebRTC na rede desejada. Não configure encaminhamento de portas no roteador.

## Testes e validação

Após instalar as dependências do projeto:

```bash
cargo test --manifest-path tests/core/Cargo.toml
npm run check
npm run build:web
npm test
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
python3 tests/linux/scripts.test.py
```

O teste nativo de integração exige uma sessão X11 acessível e os plugins GStreamer:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- --include-ignored
```

O mantenedor confirmou instalação e funcionamento em X11 com celular Android físico. Há também verificações automatizadas e testes nativos de captura X11. Wayland, ARM64 e instalações em outros ambientes permanecem experimentais até validação prática. Duração da sessão, modelos dos dispositivos, navegador e versão do pacote usado não foram registrados; não são declaradas medições de desempenho ou cobertura de todos os cenários.

Consulte [CONTRIBUTING.md](CONTRIBUTING.md#testes) para executar as suítes e conhecer os cenários de validação manual.

## Roadmap

- [x] Validar instalação e uso em X11 com celular Android físico.
- [ ] Validar captura e transmissão em desktops Wayland representativos (experimental).
- [ ] Ampliar a validação de instalação e compatibilidade, incluindo hardware ARM64 (experimental).

## Documentação

| Documento | Conteúdo |
| :-- | :-- |
| [LINUX.md](docs/LINUX.md) | Compatibilidade, dependências e empacotamento |
| [Arquitetura](docs/architecture.md) | Componentes e fluxo de dados |
| [Contribuição](CONTRIBUTING.md) | Desenvolvimento, testes e pull requests |
| [Segurança](SECURITY.md) | Uso seguro e reporte de vulnerabilidades |
| [Releases](https://github.com/ThiagoSales17/telaleve/releases) | Instaladores e versões disponíveis |

## Contribuidores

Obrigado a todas as pessoas que ajudam a construir o TelaLeve com código, documentação, testes e sugestões.

<a href="https://github.com/ThiagoSales17"><img src="https://github.com/ThiagoSales17.png?size=80" width="64" height="64" alt="ThiagoSales17" /></a>

[Ver todos os contribuidores](https://github.com/ThiagoSales17/telaleve/graphs/contributors) · [Como contribuir](CONTRIBUTING.md)

## Licença

Distribuído sob a licença [MIT](LICENSE). Componentes de terceiros mantêm suas respectivas licenças.

---

<div align="center">

**TelaLeve** · Rust + Tauri + GStreamer + WebRTC

[Downloads](https://github.com/ThiagoSales17/telaleve/releases) · [Documentação](docs/LINUX.md) · [Contribuição](CONTRIBUTING.md)

</div>
