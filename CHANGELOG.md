<!-- CHANGELOG.md -->
# Changelog

## 0.1.1 — 2026-10-06

- macOS: o painel compacto abre junto ao cursor, na mesa em uso e sobre aplicativos em tela cheia.
- macOS: Inserir/Enter digita a tradução no aplicativo de origem (requer permissão de Acessibilidade).
- macOS: fechar o painel devolve o foco ao aplicativo de origem; superfície nativa com desfoque e cantos arredondados.
- Painel compacto redesenhado em tons de cinza: duas linhas (idioma + texto, idioma + tradução), configurações e copiar. Enter insere ou, sem destino, copia; Esc fecha.

## Motor local integrado — 05/10/2026

- Novo padrão em instalações novas: CTranslate2/SentencePiece com modelos Argos em sidecar empacotado, sem servidor externo.
- Instalação explícita e verificada de PT/EN/DE, créditos/licenças individuais e rotas via inglês apresentadas na interface.
- Migração preserva provider e preferências existentes; cancelamento/saída encerra também o processo de inferência.

## 0.1.0 — inicial, não publicada — 2026-10-05

- Base Tauri/React, configurações, tray, instância única, atalhos e autostart opcional.
- Debounce, resultado revisável, cópia, cancelamento e proteção contra resultados antigos.
- Ollama, OpenAI Chat Completions, Anthropic Messages, Google Cloud v2 e LibreTranslate HTTP.
- Credenciais de sessão/cofre, consentimento remoto e detecção de modelo Ollama cloud.
- Adaptador Win32/UI Automation para alvo, seleção somente leitura e inserção Unicode.
- Contratos HTTP e testes de confirmação; documentação e CI preparada.
- Pendentes: QA externo Windows, adaptações macOS/X11/Wayland, engine offline embarcada, streaming/histórico opt-in.
