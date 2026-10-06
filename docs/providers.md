<!-- docs/providers.md -->
# Providers

| Provider | Protocolo real | Descoberta | Tom/região | Validação |
|---|---|---|---|---|
| Local integrado | CTranslate2/SentencePiece, modelos Argos, stdio privado | Catálogo embarcado + recibos de instalação | Sem tom/região/detecção automática | Sidecar Windows x64 + traduções reais PT/EN/DE |
| Ollama | POST `/api/chat`, mensagens system/user, stream false | GET `/api/tags` | Via prompt | Contrato HTTP + tradução real local |
| OpenAI/compatível | POST base `/chat/completions` | GET base `/models` (pode não existir) | Via prompt, condicionado ao modelo | Contrato HTTP, conta real não testada |
| Anthropic | POST `/v1/messages`, x-api-key, anthropic-version 2023-06-01 | GET `/v1/models` | Via system | Contrato HTTP, conta real não testada |
| Google Cloud | POST `/language/translate/v2`, x-goog-api-key, format text | GET `/language/translate/v2/languages` | Sem tom/tratamento/região nesta implementação NMT | Contrato HTTP, conta real não testada |
| LibreTranslate | POST `/translate`, api_key opcional no JSON | GET `/languages` | Sem tom/tratamento/região | Contrato HTTP, servidor real não testado |

Ollama: servidor padrão `http://localhost:11434`, modelo escolhido pelo usuário e já instalado. Não há download automático, exigência de GPU ou estimativa inventada de RAM. `/api/tags` é consultado antes de gerar para confirmar modelo e detectar metadados remote_host/remote_model, além de nomes `:cloud`; sem autorização remota, cloud é bloqueada antes do texto. Um servidor malicioso pode omitir metadados: só utilize endpoints confiáveis. A pré-consulta pode somar tempo de rede; tempo exibido inclui backend completo. `think:false` evita raciocínio adicional quando suportado. [Chat](https://docs.ollama.com/api/chat), [tags](https://docs.ollama.com/api/tags).

OpenAI: base inicial `https://api.openai.com/v1`, ID manual ou listado. Somente Chat Completions está implementado. Servidores compatíveis podem recusar temperatura, system, `/models` ou campos opcionais; erros são apresentados, sem anunciar suporte a Responses. Nenhum modelo fixo/obsoleto nem modelo Codex é requisito. Assinatura de ChatGPT não é crédito de API. [Protocolo oficial consultado](https://developers.openai.com/api/reference/resources/chat).

Anthropic: base `https://api.anthropic.com`, ID editável, header/version e system de nível superior; aceita apenas stop_reason end_turn como conclusão. Thinking/tools/streaming não habilitados. [Messages](https://platform.claude.com/docs/en/api/messages/create), [versionamento](https://platform.claude.com/docs/en/api/versioning).

Google: **Cloud Translation Basic v2**, autenticação por API key de um projeto com Translation API habilitada e faturamento aplicável. Credencial no cofre/sessão backend; enviada em `x-goog-api-key`, sem query string. Não implementa ADC/service-account JSON/OAuth nem usa GOOGLE_APPLICATION_CREDENTIALS. Restrinja sua key conforme recomendações do Google. Requer conta/uso cobrado segundo o serviço; nenhum preço fixado na documentação. Origem auto omite source; idioma regional é reduzido ao código base, saída NMT, entidades HTML retornadas são decodificadas e renderizadas como texto. Sem scraping, endpoint gratuito interno ou segundo LLM oculto. [REST v2](https://docs.cloud.google.com/translate/docs/reference/rest/v2/translate), [autenticação por API key/header](https://docs.cloud.google.com/docs/authentication/api-keys-use).

LibreTranslate: configure seu servidor, inclusive localhost; chave opcional conforme instância. Não presume gratuidade. Descoberta informa códigos suportados; erro em um par não inicia pivot/fallback escondido. [API oficial](https://docs.libretranslate.com/api/operations/translate/), [uso](https://docs.libretranslate.com/guides/api_usage/).

Todos os providers informam erro sem devolver corpos potencialmente sensíveis; limites/quota exigem nova ação do usuário, sem retries infinitos. DeepL/Gemini/HTTP personalizado e Responses ficam planejados, sem plugins arbitrários no MVP.
