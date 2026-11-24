# Validação da implementação inicial

## Ambiente e evidências

- Ambiente disponível: Debian GNU/Linux 13, sessão declarada X11, arquitetura amd64.
- Seis testes Rust passaram: controle de acesso, expiração, retomada, resolução sem ampliação, identificação de codec e integração nativa.
- Integração X11: resposta SDP produzida, vinte trocas alternando monitor e janela concluídas em aproximadamente 1,2 segundo no teste inteiro; origem inválida preservou a captura anterior. O teste não conectou um receptor de mídia real e NÃO mede tempo até a imagem aparecer no celular.
- Captura de três quadros e codificação VP8 com GStreamer concluídas.
- CPU disponível: Intel Core i7-2600, 8 CPUs lógicas. GStreamer 1.26.2; GTK 3.24.49; WebKitGTK 2.54.0.
- Compilação TypeScript concluída; sintaxe JavaScript conferida com Node.
- Enumeração X11 compilada no pacote independente de testes.
- `cargo check` de todo o desktop passou. Build otimizado foi gerado com `opt-level=2` e símbolos removidos.
- Instalação global de dependências indisponível: `sudo` exige senha. Pacotes de desenvolvimento baixados e extraídos em diretório de trabalho para tentar verificar o Rust sem alterar o sistema.
- Não há Android conectado nem gravação de alta velocidade. Nenhuma medição ponta a ponta foi realizada.

## Roteiro manual de aceite

| Cenário | Resultado exigido | Estado |
|---|---|---|
| QR e aprovação | Nenhuma captura/transmissão antes de aprovar | Pendente |
| Código expirado/reutilizado e segundo receptor | Recusados | Lógica unitária parcial; integração pendente |
| Fluxo contínuo de 30 minutos | Sem quedas nem crescimento contínuo de RAM | Pendente |
| 20 trocas de monitor/janela | Sem novo pareamento; registrar duração | Pipeline nativo passou; experiência no Android pendente |
| Falha na nova captura | Captura anterior preservada | Origem com dimensões inválidas testada; timeout/permissão pendentes |
| Janela fechada/monitor removido | Avisar, suspender imagem, exigir seleção | Pendente |
| Wi-Fi interrompido e retomado | Retomar dentro de 30s após detecção | Pendente |
| Retomada após prazo | Exigir novo QR | Unitário passou; integração pendente |
| H.264 por hardware e VP8 | Vídeo decodificado no Chrome Android | Pendente |
| Pacote em Debian 13 limpo | Dependências resolvidas e aplicativo abre | Pendente |

## Comparação com Deskreen

Registrar versão/commit do Deskreen, CPU, GPU/driver, RAM, versão Chrome/Android, resolução, FPS, codec e rede. Usar a mesma origem e conteúdo nas duas aplicações. Somar RSS de todos os processos do emissor, coletar CPU e GPU, repetir os testes de vídeo estático e movimento. Gravar um contador nas duas telas com câmera de alta velocidade, medir diferenças e reportar mediana e p95. A estatística WebRTC não substitui essa medição.

Metas ainda NÃO comprovadas: RAM total pelo menos 25% menor; atraso mediano pelo menos 20% menor; CPU média sem aumento em configuração equivalente.

## Limitações conhecidas para revisar no teste real

- Monitores X11 com coordenadas negativas não são listados nesta implementação.
- Janelas minimizadas não são oferecidas como fonte.
- Preferência por H.264 ainda precisa validar perfil negociado e latência específica de cada encoder.
- A adaptação por perda/jitter é uma heurística inicial; testar comportamento com congestionamento.
- O serviço escuta todas as interfaces IPv4; selecionar uma interface altera o QR, não o endereço de escuta. O token e a aprovação controlam o acesso, mas o firewall deve restringir a rede desejada.

## Decisões confirmadas durante implementação

Capturas são pipelines independentes ligados ao transmissor por appsink/appsrc, com quadros nativos e filas limitadas. A nova captura produz um quadro antes da substituição e o pipeline WebRTC permanece o mesmo. O uso inicial de input-selector foi substituído após encontrar bloqueios em troca dinâmica. XInitThreads é chamado antes de GTK e de qualquer captura para habilitar os locks globais de Xlib.

A integração é ignorada por padrão porque requer X11 acessível. Executar explicitamente com `cargo test --manifest-path src-tauri/Cargo.toml -- --include-ignored` no computador alvo. O tempo registrado do teste não é uma medição de latência de vídeo.

## Artefato e teste de inicialização

- Pacote `dist/telaleve_0.1.0-preview1_amd64.deb` gerado com binário release otimizado, aproximadamente 2,1 MB. Esse tamanho é apenas o arquivo do pacote; as dependências de runtime são instaladas separadamente.
- Metadados Debian conferidos com `dpkg-deb --info`; verificação `ldd` não encontrou bibliotecas ausentes no ambiente disponível.
- Aplicativo nativo iniciado temporariamente e encerrado após o teste: página do visualizador servida por HTTP e WebSocket com token inválido recusado. Não iniciou uma transmissão de usuário.
- Instalação em sistema limpo, aprovação pelo desktop em conjunto com Android, codecs por hardware, sessão de 30 minutos e benchmark contra Deskreen continuam pendentes.

## AppImage

AppImage otimizado para Debian 13 amd64/X11 gerado com dependências de mídia e WebKitGTK. Abertura em modo de extração, serviço HTTP e recusa de WebSocket não autorizado passaram. Os seis testes nativos também passaram usando somente os plugins empacotados, sem caminhos de plugins do sistema. Detalhes em APPIMAGE.md. Compatibilidade em sistema limpo e Android continua pendente.

## Correção 0.1.1 — tela cheia no celular

O visualizador passa a expandir somente a área do vídeo. Adicionado botão de saída sobre o vídeo, sincronização com saída pelo navegador e modo expandido quando a API nativa é indisponível ou recusada. Layout ajustado para ocupar a altura disponível, incluindo celular na horizontal. No modo alternativo, as barras do navegador podem permanecer visíveis.

Cinco testes automatizados passaram (entrada/saída, saída externa, API ausente, API recusada e API prefixada). No navegador de teste, foram verificados os tamanhos 390×844 e 844×390; o modo expandido removeu cabeçalho e navegação, o botão de saída restaurou os controles e os limites da área de vídeo coincidiram com a viewport 844×390. O teste foi sem transmissão. Não foi possível validar no Android físico do usuário.

## Ajuste 0.1.2 — vídeo sem botão sobreposto

Removido o botão de saída do vídeo. Na tela cheia nativa, a saída fica a cargo do navegador/Voltar do Android. O modo expandido alternativo cria uma entrada temporária no histórico: Voltar restaura os controles sem sair do visualizador. Escape também restaura o modo alternativo. Seis testes frontend passaram, incluindo ausência do botão, saída nativa sem alterar histórico, saída por Voltar e restauração por Avançar. Não testado em Android físico.

## 0.1.3 — modo Fluidez

Adicionado 720p/60 FPS com bitrate inicial de 4 Mbps e máximo de 6 Mbps. Equilibrado permanece em 30 FPS por padrão. O teste nativo passou alternando vinte vezes entre monitor/janela e os perfis 30/60 FPS, conferindo as caps negociadas da captura em cada troca. Isso não comprova a taxa efetivamente exibida no Android. Seis testes Rust e os testes de tela cheia passaram. Android físico e comparação com Deskreen continuam pendentes.

## 0.2.0 — compatibilidade Linux (23/11/2026)

### Implementação

- Captura X11 direta, inclusive no Flatpak, e fallback para o desktop completo quando RandR não está disponível.
- Captura Wayland por portal ScreenCast/PipeWire, com seleção autorizada pelo sistema, conexões PipeWire independentes por pipeline e encerramento da sessão ao revogar o compartilhamento.
- A seleção disponível depende do compositor; alguns backends oferecem somente monitor. As dimensões físicas dos quadros determinam a escala, sem ampliar entradas pequenas.
- Instalador de dependências para famílias Debian/Ubuntu, Fedora, Arch e openSUSE; diagnóstico retorna erro quando faltam plugins, sessão gráfica ou portal necessário.
- Empacotador Flatpak com runtime GNOME 51, para binários nativos x86_64/aarch64. O `.deb` usa versão e arquitetura locais, sem fixar amd64 no script.

### Verificado localmente em Debian 13 x86_64

- Cinco testes do núcleo passaram, incluindo seleção de backend e serialização de origens sem expor acesso ao portal.
- Quatorze testes Rust nativos passaram, incluindo o teste de integração X11 em Xvfb, vinte trocas de captura/qualidade, preservação da origem diante de falha, ausência de ampliação de quadros pequenos e cancelamento da preparação após encerramento da sessão.
- O teste de integração passou novamente com RandR desabilitado, exercitando a captura do desktop completo.
- Oito testes dos scripts Linux passaram, sem instalar pacotes: seleção de família, derivados, distribuição desconhecida, plugins ausentes, Wayland sem X11, portal ausente e captura X11 no Flatpak sem exigir portal Wayland.
- Compilação/checagem TypeScript e suíte de tela cheia passaram. Clippy em todos os targets locais passou com avisos tratados como erro; formatação Rust e diff conferidos.
- Build otimizado gerado. As bibliotecas dinâmicas do binário 0.2.0 foram encontradas dentro do runtime GNOME 51, sem depender das bibliotecas do host.
- O pacote Flatpak inclui o plugin de captura X11 e seu aviso de licença. Os testes nativos foram executados dentro do runtime GNOME 51, em tela X11 virtual; o aplicativo GTK iniciou e serviu a página HTTP do visualizador. Isso não substitui testar com receptor real.
- O teste no runtime revelou um deadlock intermitente entre `XGrabServer` e a abertura de um novo display Xlib. As trocas X11 agora param o leitor anterior antes de preparar o próximo e retomam a origem anterior se a preparação falhar.
- O limitador usa timestamps dos quadros, sem aguardar um segundo quadro e sem depender da negociação de taxa variável do elemento `videorate`.
- Um quadro único de uma origem ao vivo é entregue sem depender de novo movimento no desktop. O encerramento da captura é detectado também por EOS. Eventos SDP/ICE de capturas encerradas não são enviados para uma nova sessão.
- Em uma sessão Wayland virtual de teste, o portal aceitou criar a sessão e selecionar a origem, mas o backend de captura recusou o formato do compositor virtual. Esse teste não concluiu a transmissão Wayland e não conta como aceite.

### Validações ainda necessárias

- Seleção e transmissão reais em desktops Wayland representativos, como GNOME e KDE, cancelamento pelo compositor, reconexão e troca de origem com vídeo no receptor.
- Android físico, sessão contínua de 30 minutos e benchmarks comparativos.
- Instalação e uso em uma amostra de sistemas limpos, com drivers e backends de portal representativos. Não é necessário testar cada distribuição existente.
- Binário/Flatpak ARM64 em equipamento real. O workflow inclui Ubuntu ARM64 e ambientes Fedora, Arch e openSUSE; consulte os resultados de cada execução antes de tratá-los como testes concluídos.

A implementação amplia a compatibilidade; não demonstra funcionamento em todas as versões de Linux, compositores, drivers e arquiteturas existentes. Os artefatos antigos 0.1.3 permanecem limitados ao ambiente descrito nas seções históricas.

### Ponto de pausa — 23/11/2026

Atividades encerradas a pedido do responsável. Na última conferência da [execução CI do código `33c70a9`](https://github.com/ThiagoSales17/telaleve/actions/runs/45293401929), Fedora 45, Arch, openSUSE Tumbleweed e Ubuntu 24.04 ARM64 concluíram com sucesso; Ubuntu 22.04 e 24.04 x86_64 ainda executavam verificações. Foi solicitado o cancelamento da execução ao pausar. Essas verificações restantes devem ser retomadas antes da publicação da prévia 0.2.0, que permanece em rascunho. O roadmap do README registra as próximas etapas.

### Reorganização do repositório — 23/11/2026

Documentação operacional movida para `docs/`, fontes de interface separadas em desktop/viewer/styles e testes agrupados em `tests/`. Os assets em `web/` são reconstruídos por `npm run build:web`; as rotas do receptor foram ajustadas e isoladas em `src-tauri/src/server.rs`.

Validação local: checagem TypeScript, Prettier e rustfmt, suítes frontend (tela cheia e integridade de assets/rotas), oito testes dos scripts Linux, cinco testes de metadados de release e cinco testes do núcleo Rust passaram. Os 14 testes nativos passaram em X11 virtual, incluindo vinte trocas de captura/qualidade. Clippy passou com avisos tratados como erro. O aplicativo iniciou na tela virtual e respondeu às rotas HTTP da página, módulos do receptor e CSS. Links locais da documentação e workflows foram conferidos.

A reorganização não adiciona evidências de transmissão em Wayland real ou Android físico; essas validações permanecem no roadmap.
