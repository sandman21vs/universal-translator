<!-- docs/validation.md -->
# Evidências de validação — 05/10/2026

Ambiente Windows NT 10.0.26300 x64, Node 24.14.1/npm 11.19.0, Rust 1.99.0 e Visual Studio 2022 com C++/SDK. O diretório inicial continha somente a especificação e não era um repositório Git. Nenhum repositório remoto/push/release público foi criado.

## Comandos e resultados finais

Com a toolchain local habilitada conforme README:

| Comando | Resultado real |
|---|---|
| `npm install`, depois lockfile preservado | Dependências instaladas; Vitest atualizado para corrigir advisory encontrado inicialmente |
| `npm run lint` | Passou TypeScript + ESLint |
| `npm test` | 9 testes passaram em 3 arquivos, incluindo instalação explícita e disclosure de pivot |
| `npm run build` | Passou; Vite gerou assets em dist |
| `npm audit --audit-level=moderate` | Zero vulnerabilidades reportadas |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Passou |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings` | Passou após correções |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | 16 passaram; 2 testes reais ignorados por padrão |
| `npm run tauri -- build --debug --no-bundle` | Executável inicial gerado e aberto no Windows |
| `npm run desktop:build` | Build final release e 2 instaladores unsigned gerados |

Teste opt-in também executado, sem download de modelo e sem registrar conteúdo em logs:

```powershell
$env:UT_INTEGRATION_OLLAMA_MODEL = 'gemma4:e2b'
cargo test --manifest-path src-tauri/Cargo.toml --locked installed_local_ollama_opt_in -- --ignored --nocapture
```

Passou em **17.416 ms** de geração/backend para o caso de entrada maior com emoji, newline e identificadores. Asserts confirmaram resposta não vazia e preservação de `ESP32-P4`/`GPIO38`. Isso não comprova naturalidade linguística ou preservação de cada detalhe. Uma mensagem curta na interface foi observada com **5.217 ms**. Entradas/condições diferentes: não comparar como benchmark ou prometer performance universal.

O Ollama já estava instalado, mas não respondia inicialmente. Foi iniciado em processo separado para QA; modelos locais existentes foram utilizados. Nenhum modelo foi baixado. A aplicação não gerencia automaticamente a instalação/vida do servidor. Também foram encontrados modelos cloud; a versão final exige consentimento remoto antes de gerar com metadados cloud/nome :cloud, mesmo em localhost.

## Cobertura

- HTTP real contra wiremock: Ollama, OpenAI, Anthropic, Google e LibreTranslate, incluindo rotas, payloads e autenticação.
- Google: cabeçalho x-goog-api-key sem query e decodificação de entidades em texto.
- Cloud Ollama sem autorização: pré-consulta de modelos, zero POST de geração.
- Respostas truncadas recusadas; erros não contêm corpo/URL; timeout cobre corpo da resposta.
- Prompt separado do conteúdo; cache inclui estilo/configuração; configuração roundtrip atômico e arquivo inválido preservado.
- Frontend: debounce, vazio, respostas fora de ordem, invalidação imediata, Enter incompleto, Shift+Enter, Esc, falha de foco preservando texto, Unicode/multiline revisado e ausência de retry/auto-inserção.
- Windows: UTF-16/surrogates e ausência de VK_RETURN; janela inexistente falha antes do input. São testes determinísticos, não QA externo de edição.

## QA real e limites

A build inicial abriu, expôs os controles via UI Automation, permitiu configurações/listagem de modelos e apresentou uma tradução real Ollama. Computer Use foi utilizado para inspeção. Captura visual teve região parcialmente preta; não é evidência de layout completamente verificado, portanto nenhuma screenshot foi incluída como prova.

O usuário interagiu com o aplicativo durante a inspeção; seu rascunho foi preservado. Na tentativa posterior de QA com a versão final e editor nativo, **o usuário interrompeu Computer Use com Escape**. A automação foi encerrada e não foram emitidos mais comandos de controle de interface.

Assim, seleção/inserção em browser e editor nativo continuam **não validadas manualmente**. Não declarar MVP desktop concluído. Confira `qa-checklist.md`. Cofre/autostart/cópia manual, múltiplos monitores, clipboard com imagem/HTML, aplicações elevadas e demais SOs também exigem QA. Contas reais OpenAI/Anthropic/Google não foram usadas; LibreTranslate real não estava executando.

CI é preparação local, não execução remota. Arquiteturas são Windows x64, Linux x64 e macOS Intel/ARM; os labels foram conferidos na [referência oficial de runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Somente Windows foi compilado aqui. Nenhum instalador foi instalado; nenhum certificado de assinatura/notarização foi utilizado.

Aviso restante do bundler: identificador `io.github.universaltranslator.app` termina em `.app`, que Tauri desaconselha no macOS. Mantido para não trocar silenciosamente o escopo da configuração já usada durante a sessão; corrigir com migração antes da distribuição macOS. Não impediu os builds Windows.

## Artefatos da build anterior (sem motor integrado)

| Caminho relativo | Bytes | SHA-256 |
|---|---:|---|
| `src-tauri/target/release/universal-translator.exe` | 12.997.120 | `681DEBFC108B58586FFB6C62CA24B48BB0FD84FCB814CBDAB57CC484BDC695C1` |
| `src-tauri/target/release/bundle/nsis/Universal Translator_0.1.0_x64-setup.exe` | 3.185.583 | `26B21505E43E130999BDF8839CEB0A9BBB35290A6E6DF6AAFFFB4BB15B2827FF` |
| `src-tauri/target/release/bundle/msi/Universal Translator_0.1.0_x64_en-US.msi` | 4.591.616 | `74E48D274313CA170B3358F9AB7F179857668BDC2FF4B7F3ABD72EEC634D80CB` |

Para usar a versão final, se houver uma instância inicial aberta, escolha **Sair** na bandeja antes de abrir o executável release. Instância única pode apenas trazer uma build antiga já aberta ao primeiro plano. Não force encerramento de um rascunho do usuário.

## Validação do motor integrado

Python 3.12.10, CTranslate2 4.8.2 e SentencePiece 0.2.2, dependências com versões/hashes PyPI fixados. PyInstaller produz executável autocontido; nenhum modelo é baixado ao abrir/traduzir. O download real de PT→EN via Rust foi acionado explicitamente pelo teste opt-in em diretório temporário: bytes/SHA-256, extração, status e tradução passaram. A tradução desse teste levou **1.533 ms**, sem logs de conteúdo.

`python -m unittest discover -s engine -p test_engine.py`: **3 testes passaram**, incluindo checksum inválido, archive path traversal com checksum válido, modelos ausentes e origem automática sem acesso à rede.

`scripts/check-engine-integration.py` foi executado com quatro arquivos oficiais obtidos para QA em `.tools/engine-models`, separados dos dados do usuário. Instalou-os em pasta temporária, testou inferência com `socket.connect` Python bloqueado e depois o binário com **PATH/PYTHONHOME/PYTHONPATH vazios**. Asserts: saída não vazia/diferente do original, quebras preservadas, ausência de marcadores SentencePiece e nenhum stderr. Isso prova o caminho de inferência sem rede Python, sem substituir teste de firewall do SO ou revisão linguística ampla.

| Exemplo no binário final | Tempo total (inclui inicialização) |
|---|---:|
| PT → EN, duas linhas | 1.687 ms |
| EN → PT, duas linhas | 1.578 ms |
| PT → EN → DE | 1.766 ms |
| DE → EN → PT | 1.765 ms |

Medidas nesta máquina, com frases curtas e concorrência de build; não são garantias de performance. Normalização de marcador SentencePiece foi corrigida após um teste real identificar `▁` na saída. O tratamento do pipe também foi corrigido para evitar um lock de I/O buffered durante encerramento do Python.

Testes de ciclo de vida do binário final: conclusão gerenciada com pipe aberto terminou limpa; fechar pipe de propriedade cancelou e encerrou o processo. Conteúdo do executável foi inspecionado: **nenhuma biblioteca CUDA/cuDNN**; licenças upstream CTranslate2/SentencePiece/Intel OpenMP incluídas. A DLL Intel OpenMP teve hash idêntico ao wheel oficial Intel 2025.3.0, cuja licença integral foi preservada. O bundle Tauri inclui o sidecar; o usuário final não precisa de Python/Docker.

Lint, 9 testes frontend, 16 testes Rust, fmt/clippy e npm audit passaram após as alterações. Testes reais continuam opt-in. Instalação existente/provider/rascunho aberto foram preservados; o novo build usa a subpasta de target `x86_64-pc-windows-msvc`. Nenhum instalador foi executado, e nenhum modelo foi instalado no perfil do usuário. UI da nova tela não foi controlada manualmente; QA visual e de instalação no computador final permanece pendente. Outras arquiteturas têm workflows preparados, sem execução comprovada.

## Artefatos com motor integrado

Build final: `npm run tauri -- build --target x86_64-pc-windows-msvc`. O SHA-256 do sidecar copiado para a pasta release coincide com o binário testado.

| Caminho relativo | Bytes | SHA-256 |
|---|---:|---|
| `src-tauri/target/x86_64-pc-windows-msvc/release/universal-translator.exe` | 13.275.648 | `5b8bf93dfb92ba849ad7ed38f27422eb376d62148e9fd3f1dbb0255506d5fbb4` |
| `src-tauri/target/x86_64-pc-windows-msvc/release/local-engine.exe` | 41.766.368 | `d5a9a96037f1bd3da3065e6c405c17b4d260035a35a1c5012e5f621f5678ed45` |
| `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Universal Translator_0.1.0_x64-setup.exe` | 44.597.535 | `31cef7e0b9c2a93b90482afa5141b535819c6c5c2342ebd16b3425e505f6ce13` |
| `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/Universal Translator_0.1.0_x64_en-US.msi` | 46.022.656 | `9a4673cd7e4ca1458eb579ae6647e40fb9984580812f72ba3a3c3417f2b92a1c` |
