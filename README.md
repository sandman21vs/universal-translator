<!-- README.md -->
# Universal Translator

Aplicativo desktop em desenvolvimento para compor mensagens, traduzir e revisar antes de copiar ou inserir. Tauri 2 + Rust + React/TypeScript. Interface inicial em pt-BR; idiomas PT, EN, DE, ES, FR e IT. Código próprio MIT; modelos e dependências conservam suas licenças.

**Versão inicial, não um MVP universal concluído.** Ollama foi utilizado realmente neste Windows; os outros providers têm implementação HTTP e contratos com servidor de teste, sem validação com contas pagas. Seleção/inserção Windows têm adaptador nativo, mas o QA externo completo está pendente. macOS abre o painel junto ao cursor em qualquer mesa (inclusive sobre apps em tela cheia) e insere a tradução no aplicativo de origem com permissão de Acessibilidade; seleção global ainda não implementada. Linux permite tradução/cópia; seleção/inserção global ainda não implementadas.

## Desenvolvimento

Use Node **24.14.1**, npm e a toolchain Rust **1.99.0** fixada em `rust-toolchain.toml`. Instale os [pré-requisitos oficiais do Tauri](https://v2.tauri.app/start/prerequisites/): no Windows, Visual Studio C++ Build Tools, Windows SDK e WebView2; no macOS, Xcode Command Line Tools; no Linux, WebKitGTK 4.1/GTK, libayatana-appindicator e dependências descritas na documentação. Não é necessário GPU para o aplicativo; os requisitos de modelos dependem do provider.

```sh
npm ci
python scripts/prepare-engine.py # Python 3.12: somente no computador de desenvolvimento
npm run desktop:dev
```

Nesta máquina Rust foi instalado sem alterar PATH global, em `.tools/`. Para usar essa instalação no PowerShell, na raiz do projeto:

```powershell
$env:RUSTUP_HOME = Join-Path (Get-Location) '.tools\rustup'
$env:CARGO_HOME = Join-Path (Get-Location) '.tools\cargo'
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
npm run desktop:dev
```

`npm run dev` abre somente a interface web em `http://127.0.0.1:1420`, mostrando aviso de backend ausente. Não há HTTP de tradução, credenciais ou mocks de produção no navegador.

## Build e validação

```sh
npm run lint
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri -- build --debug --no-bundle
npm run desktop:build
```

O build final usa `src-tauri/target/release/`. Instaladores ficam em `src-tauri/target/release/bundle/`. São unsigned; nenhuma assinatura/notarização configurada. A CI prepara compilação em Windows x64, macOS Intel/ARM e Linux x64; ainda precisa ser executada no GitHub. Consulte [validação](docs/validation.md) para distinguir comandos já executados de comandos disponíveis.

## Primeiro uso

1. Em **Configurações**, selecione **Local integrado · modelos Argos** (padrão das novas instalações).
2. Clique **Baixar** no par desejado. PT → EN: 69,4 MB; EN → PT: 66,2 MB. Para PT ↔ DE, instale também os modelos correspondentes via inglês. A UI apresenta licença e SHA-256; nenhum modelo é baixado automaticamente.
3. Escolha idiomas de origem/destino e salve. O motor já vem no instalador e funciona sem internet após instalar os idiomas. Origem automática, tom/tratamento e variantes regionais ficam disponíveis nos outros providers conforme suas capacidades. Para Ollama, inicie o servidor separado e escolha um modelo já instalado; modelos cloud exigem autorização remota explícita.
4. Escreva em Traduzir; após 500 ms de pausa, o Rust solicita a tradução. O botão Copiar mantém a tradução no clipboard por escolha explícita.
5. No Windows, para inserir, coloque foco em um campo editável externo e use `Ctrl+Alt+T`. Revise; **Enter** insere somente um resultado atual e concluído, sem enviar a mensagem. **Shift+Enter** cria uma quebra no editor; **Esc** oculta e preserva o rascunho em memória.

| Ação | Padrão Windows/Linux | macOS |
|---|---|---|
| Compor | Ctrl+Alt+T | Command+Alt+T |
| Traduzir seleção por acessibilidade (leitura) | Ctrl+Alt+S | Command+Alt+S (seleção ainda não implementada) |
| Repetir última tradução, sem inserir | Ctrl+Alt+R | Command+Alt+R |

Atalhos são editáveis e conflitos são exibidos. Não há atalhos de destino 1/2/3 nesta versão. Tray permite compor, traduzir seleção, configurar, pausar atalhos, limpar sessão e sair. Fechar a janela mantém o app na tray. Autostart está desligado inicialmente.

## Providers e privacidade

Local integrado (CTranslate2/modelos Argos), Ollama Chat, OpenAI/compatíveis via **Chat Completions**, Anthropic Messages, Google Cloud Translation Basic **v2/API key** e LibreTranslate HTTP. Endpoints/modelos configuráveis; Google fica restrito ao endpoint oficial. Tom/tratamento/região só estão disponíveis nos LLMs; Google/LibreTranslate/local recebem códigos base de idioma. Veja [providers](docs/providers.md).

Credenciais são campos somente de escrita: o valor digitado é enviado uma vez ao Rust e descartado do formulário; segredos armazenados nunca são devolvidos ao frontend. Escolha sessão ou cofre nativo (Credential Manager/Keychain/Secret Service). Se o cofre falhar, o app informa o erro e permite escolher sessão, sem criar arquivo plaintext. Credenciais são associadas ao provider **e endpoint**. A aplicação não lê cookies, logins nem credenciais de CLIs.

Não há analytics, histórico persistente, download automático ou fallback cloud. Cache/última tradução/rascunho ficam na memória. HTTP remoto exige autorização separada. Inserção Unicode e leitura por acessibilidade não modificam o clipboard; nenhuma promessa de snapshot de HTML/imagem é necessária neste fluxo. O botão Copiar substitui o clipboard deliberadamente. Veja [privacidade](docs/privacy.md) e [suporte por plataforma](docs/platform-support.md).

O motor **Local integrado** acompanha o instalador Windows e traduz com modelos Argos instalados explicitamente, sem rede. Ollama e LibreTranslate permanecem opções com servidores separados. O adaptador utiliza CTranslate2/SentencePiece diretamente e não promete saída idêntica ao servidor LibreTranslate. Distribuição, modelos e limitações em [offline](docs/offline-engines.md).

## Estrutura

```text
app.meta.json                 # nome, identificador, versão; npm run sync:meta propaga
src/                          # bridge, sessão, i18n e UI React
src-tauri/src/core.rs          # configuração, prompt, idiomas, cache
src-tauri/src/providers.rs     # contratos HTTP / descoberta
src-tauri/src/credentials.rs   # cofre nativo e sessão
src-tauri/src/platform/        # Win32/UI Automation; erro explícito nos outros SOs
src-tauri/src/lib.rs           # IPC restrito, cancelamento, tray, atalhos
tests/                        # confirmação, debounce, teclado e respostas fora de ordem
docs/                         # arquitetura, QA, privacidade e limitações
.github/workflows/            # CI e builds unsigned, sem publicação automática
```

**Screenshots:** espaço reservado para capturas reais após o QA visual. Nenhum mockup é apresentado como prova de funcionamento.

[Plano](IMPLEMENTATION_PLAN.md) · [Arquitetura](docs/architecture.md) · [Contribuir](CONTRIBUTING.md) · [Segurança](SECURITY.md)
