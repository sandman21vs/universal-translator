<!-- docs/architecture.md -->
# Arquitetura e decisões

## Stack

Tauri 2 atende tray, instância única, IPC, atalhos e autostart; não foi identificada uma limitação que justificasse Electron. React/Vite renderizam somente texto. Rust realiza HTTP, guarda configuração/segredos, cache, cancelamento e integração desktop. Dependências resolvidas em `package-lock.json` e `src-tauri/Cargo.lock`; toolchain fixada. Nome/identificador/versão ficam em `app.meta.json`; `npm run sync:meta` propaga para manifests. Não altere identificador de um app instalado sem planejar migração de configuração/cofre/autostart.

Referências consultadas em 05/10/2026: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), [tray](https://v2.tauri.app/learn/system-tray/), [atalhos](https://v2.tauri.app/plugin/global-shortcut/), [capabilities](https://v2.tauri.app/security/capabilities/), [React useSyncExternalStore](https://react.dev/reference/react/useSyncExternalStore), [Vite](https://vite.dev/guide/). Não confundir compatibilidade de compilação com QA desktop.

## Contratos e ciclo de requisição

`TranslationProvider`, `CredentialStore`, `WindowTargetService`, `SelectionService`, `TextInsertionService` são contratos pequenos. Tray/Hotkeys permanecem diretamente no plugin Tauri; uma abstração `HotkeyService` independente fica para quando houver outro adaptador. O contrato IPC transporta ID monotônico, texto e operação; o backend combina atomicamente a configuração salva (origem/destino/variante, estilo, tratamento, relação, endpoint/modelo/prompt) para realizar a requisição. Somente `translate` está implementado.

`TranslationSession` publica estados inativo/debounce/traduzindo/pronto/inserindo/erro. Editar texto invalida confirmação imediatamente, cancela o ID anterior e inicia timer; o backend também verifica ID/token/revisão. A resposta antiga é descartada em ambos os lados. Não há chamadas com texto vazio, retries automáticos nem envio a um detector separado. Editar a tradução pronta mantém vínculo com o ID original; inserir consome uma autorização de resultado. Falha preserva original/resultado e exige nova tradução antes de outra tentativa de inserção.

Cache em memória: TTL 5 minutos e até 64 entradas. A chave SHA-256 inclui texto/operação e configuração completa, inclusive prompt, estilos e autorizações; a versão de prompt faz parte da chave. Configuração/credenciais alteradas limpam cache e invalidam requisições. A tradução anterior é mantida em memória para repetir, sem reinsert automático.

## Motor integrado

O novo padrão é um worker CTranslate2/SentencePiece empacotado com Python, usando modelos Argos oficiais. Não abre porta nem exige serviço instalado. Rust inicia o binário conhecido ao lado do app, envia texto por stdio, limita resposta e concorre com cancelamento/timeout. Um pipe mantido aberto vincula a vida do filho de inferência ao chamador, inclusive com bootloader PyInstaller. Instalação explícita baixa apenas arquivos do catálogo, verifica tamanho/SHA-256, extrai em staging e renomeia atomicamente. Modelos/licenças persistem; textos continuam somente na RAM. Configurações antigas mantêm o provider e recebem a opção local na migração, sem gravação automática. [Decisão e limites](offline-engines.md).

## Providers HTTP e qualidade

Cliente Rust com TLS validado, sem redirects, conexão limitada a 5 s, corpo limitado a 1 MiB e timeout configurável. Erros de providers são mapeados para mensagens genéricas: corpos, cabeçalhos e URLs com segredos não são apresentados/logados. Google usa API key em header, sem query string. Requisições obsoletas cancelam a future HTTP; isso não garante que um servidor remoto interrompa computação ou cobrança já iniciada.

Prompt do sistema separado de conteúdo no papel user; Anthropic usa `system` de nível superior. Humor, negação, incertezas, destinatários e identificadores são instruídos, sem remover conteúdo com regex. Isso reduz risco de injection, sem garantia linguística/absoluta. Ollama envia `think:false` e limite de geração; modelos/servidores que recusarem o recurso retornam erro em vez de fallback oculto. Streaming não implementado; respostas incompletas são rejeitadas.

## Desktop

O alvo é capturado por Win32/UI Automation antes de mostrar o overlay: HWND, PID e runtime ID do campo. Os objetos COM são criados e liberados no mesmo worker. Ao inserir, o adaptador verifica janela/PID, solicita foco via `SetForegroundWindow`, aguarda liberação de modificadores, verifica runtime ID, segurança e editabilidade e envia UTF-16 via `SendInput`. Não simula `VK_RETURN`, não força foco por `AttachThreadInput` e não toca no clipboard. UIA ValuePattern ou atributo IsReadOnly do TextPattern confirma editabilidade; na dúvida, falha e oferece cópia.

`SendInput` informa eventos enfileirados, não um recibo de alteração do documento. Há uma janela residual entre verificar foco e injetar eventos; OS/aplicação podem bloquear/modificar comportamento. Não é compatibilidade universal. Unicode LF não equivale a tecla Enter, mas alguns campos podem ignorar quebras. QA externo é obrigatório. [SendInput e UIPI](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput), [restrições de foco](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow).

Seleção usa somente TextPattern.GetSelection; ausência de UIA não busca clipboard antigo. Não há fallback de copy/paste, OCR ou substituição de seleção. O overlay usa monitor/cursor e coordenadas físicas com escala; posicionamento usa retângulo do monitor, não ainda sua área de trabalho descontando taskbar. macOS/Linux retornam limitação explícita para seleção/inserção.

## Configuração e segurança

Schema 1, validação estrita, escrita em arquivo temporário + sync + persistência atômica. Arquivo inválido/incompatível é preservado e inicia configuração padrão com aviso; salvar depois é escolha explícita. Migrações de schemas futuros ainda não são necessárias/não estão implementadas. Nenhum texto ou segredo é campo da configuração.

IPC tem comandos conhecidos em `AppManifest::commands`, permissões explícitas apenas para `main`, sem origens remotas nem shell/filesystem expostos. CSP restringe scripts/rede/frames; providers só são chamados no backend. Frontend usa textarea/textContent; saída não é HTML executável. Autostart desligado por padrão.

Credenciais: keyring 3 com adaptadores nativos; sessão explícita quando cofre indisponível. Não retornam segredos do cofre ao frontend. O formulário de entrada inevitavelmente contém o segredo recém-digitado até envio, então é apagado; esse limite do modelo de ameaça é explícito. Configuração de ambiente não é lida pelo app empacotado. [keyring 3.6.3](https://docs.rs/keyring/3.6.3/keyring/).

Persistência de configuração e leitura/escrita do cofre usam workers para manter a UI disponível. Pendências estruturais: menu com destinos/provider dinâmicos, área útil do monitor, captura de caret/seleção para substituição, autostart QA, idioma UI EN selecionável, streams e histórico opt-in. Não criar abstrações/plugin executável antecipadamente.
