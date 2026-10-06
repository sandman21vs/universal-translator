<!-- docs/privacy.md -->
# Privacidade

- Histórico persistente e analytics não estão implementados; nenhum texto de tradução é gravado em disco pelo app.
- Rascunho, cache (5 min/64 entradas), resultado e última entrada ficam em memória. Limpar sessão remove esses dados e credenciais de sessão; sair encerra o processo. Não garantimos apagamento forense de RAM, swap ou crash dumps do sistema.
- Configuração não sensível vive no diretório app_config de Tauri, `config.json`. Arquivos inválidos são preservados para recuperação.
- Credenciais persistentes vivem no Windows Credential Manager, macOS Keychain ou Linux Secret Service via keyring; opção sessão é explícita. Cofre nunca cai silenciosamente para plaintext. Credencial é associada ao provider e endpoint; mudar endpoint não reutiliza segredo anterior.
- Não há fallback entre providers. Endpoint remoto exige consentimento; mudar URL na UI remove consentimentos anteriores. HTTP remoto requer aviso e habilitação adicional; TLS é validado, redirects desabilitados.
- Endpoint localhost não prova modelo local. Metadados de cloud do Ollama exigem autorização separada antes de gerar. Servidores configurados e modelos precisam ser confiáveis; serviços externos podem reter dados segundo suas próprias políticas.
- Prompt/conteúdo são papéis separados; texto traduzido nunca executa comandos, abre URLs ou vira HTML. Prompt injection não é solução garantida só por uma instrução de sistema.
- Nenhum logger registra entrada/saída/API keys/headers. Erros HTTP são genéricos. Ferramentas de desenvolvimento, Ollama, OS e serviços podem ter seus próprios logs independentes.
- Seleção usa acessibilidade; inserção usa Unicode, sem clipboard temporário. Clipboard original (texto, HTML, imagens) não é lido nem alterado por esses fluxos. Copiar por botão é deliberado e substitui o clipboard por texto, sem restauração.
- Não há hooks globais de digitação, keylogger, /dev/input, execução elevada, OCR ou gravação de tela pelo aplicativo.
- Local integrado utiliza inferência CPU com modelos instalados, sem HTTP de tradução. Texto é enviado ao worker por pipe privado, sem argumentos ou logs; cancelamento/saída fecha o pipe e encerra o worker. Somente os pesos, README/licenças e manifest persistem.
- Baixar modelo é ação explícita: o servidor Argos recebe um pedido de arquivo e dados normais da conexão (como IP), sem texto ou credenciais. O download é verificado por bytes/SHA-256 antes de instalar. Listagem/tradução nunca inicia download.

O campo de credencial é somente escrita: o segredo digitado passa transitoriamente pelo formulário para o backend, é limpo após tentativa e não é recuperável pela UI. Não configure CSP/origens de IPC permissivas, não use builds de terceiros sem confiança e não inclua segredos nos prompts.
