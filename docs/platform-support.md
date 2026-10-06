<!-- docs/platform-support.md -->
# Suporte por plataforma

Status em 05/10/2026. Ambiente real: Windows NT **10.0.26300**, x86_64; registro do sistema retornou nome de produto Windows 10 Home, que não permite inferir a edição comercial exata deste build. Node 24.14.1, Rust 1.99.0, Visual Studio 2022 Community com C++/SDK.

| Função | Windows x64 | macOS Intel/ARM | Linux X11 | Linux Wayland |
|---|---|---|---|---|
| Janela React/Tauri | Implementada; executável abriu | Código/CI preparados, não testado | Código/CI preparados, não testado | Código preparado, não testado |
| Ollama compor/preview | Tradução real observada | Não testado | Não testado | Não testado |
| HTTP demais providers | Contratos locais testados | Compilação/QA pendentes | Compilação/QA pendentes | Compilação/QA pendentes |
| Tray / instância única / atalhos | Implementados; abertura/configuração observadas, QA completo pendente | Plugin disponível; não testado | Plugin disponível; não testado | Pode falhar/restringir atalhos; erro exibido |
| Seleção | UI Automation TextPattern; QA externo pendente | Planejado AX | Planejado adaptador X11 | Limitado/planejado portais |
| Inserção | Win32 Unicode, alvo/campo validados; testes de codificação; QA externo pendente | Planejado AX/permissão Accessibility | Planejado adaptador X11 | Não implementado; compor/copiar |
| Clipboard por botão | arboard; QA manual pendente | Não testado | Não testado | Não testado |
| Autostart / cofre | Implementação nativa; QA manual pendente | Plugin/keyring; não testado | Secret Service; não testado | Secret Service; não testado |
| Instaladores | Build unsigned preparado | Build unsigned preparado | .deb/.AppImage preparados | Mesmos artefatos Linux, sem suporte global inferido |

Piso de suporte do **projeto**, ainda sem QA em versões antigas: Windows 10 com WebView2, macOS 11, Ubuntu 22.04 com WebKitGTK 4.1. Requisitos reais de toolchain/dependências e distribuição devem ser verificados no sistema alvo; veja [pré-requisitos Tauri](https://v2.tauri.app/start/prerequisites/). Não anunciar mínimos só com base em CI.

Windows falha com campos de senha, read-only, UIA indisponível, runtime ID diferente, destino encerrado, foco protegido, UIPI/elevados ou modifiers retidos. Alguns editores ignoram Unicode LF. SendInput não comprova consumo no documento. Aplicativos remotos/restritos não são prometidos. Não execute como administrador por padrão.

macOS não solicita Screen Recording/Input Monitoring, porque não possui adaptação desktop que precise deles. Accessibility ficará para implementação AX com detecção/instruções, sem conceder permissão silenciosamente. Linux ainda não possui adaptador X11; não chamar a compilação Tauri de suporte a inserção X11. Wayland precisa investigação por compositor/portal, sem root ou contornar políticas. GNOME/KDE/compositores: nenhum efetivamente testado nesta tarefa.
