<!-- IMPLEMENTATION_PLAN.md -->
# Plano de implementação

Baseado em `prompt-universal-translator.md`. O diretório inicial continha somente essa especificação; não havia repositório Git nem código a preservar.

| Etapa | Critério de conclusão | Status |
|---|---|---|
| 1. Base executável | Tauri/React, configuração validada, tray, Ollama, compor → traduzir → copiar, build e testes | Implementada e compilada; Ollama real validado; cópia precisa QA manual |
| 2. Desktop | Captura de alvo antes do overlay, atalhos, seleção e inserção Unicode com foco validado no Windows; limitações explícitas nos demais SOs | Adaptador Windows implementado; QA externo pendente; macOS/X11/Wayland não implementados |
| 3. Providers/estilo | HTTP real para OpenAI Chat Completions, Anthropic Messages, Google v2 e LibreTranslate; credenciais no cofre/session; contratos testados | Cinco contratos HTTP implementados/testados; contas reais e cofres/autostart precisam QA |
| 4. Offline | Avaliação Argos/Bergamot; somente integrar se empacotamento e modelos forem verificáveis | CTranslate2/modelos Argos integrado e empacotado Windows x64; catálogo PT/EN/DE verificado, instalação explícita, rotas/pivot visíveis; QA outros SOs pendente |
| 5. Distribuição | CI, documentação, matriz de suporte, QA e instalador local quando ambiente permitir | Documentação/CI preparadas; instaladores Windows unsigned gerados; CI de outras plataformas não executada |

## Decisões iniciais

- Tauri 2 + React + Vite. HTTP, segredos e integração nativa ficam em Rust.
- Sem histórico persistente, analytics, fallback remoto ou download automático de modelos.
- Windows é o sistema disponível. Outros SOs mantêm tradução/cópia, sem anunciar inserção global não implementada.
- Rust não estava instalado no PATH. Toolchain isolada em `.tools/`, ignorada pelo controle de versão, para validar sem alterar PATH global.
- Não criar repositório remoto, publicar, fazer push ou release público.

## Validação

Registrar resultados efetivos e pendências em `docs/validation.md`. Contratos HTTP usam servidor local de teste, sem API paga ou modelo instalado. QA desktop/linguístico separado da compilação.

## Pendências revisáveis

1. Finalizar `docs/qa-checklist.md` em browser e editor nativo. O usuário passou a interagir com a build inicial; o rascunho aberto foi preservado e a automação não sobrescreveu seus dados para concluir QA.
2. Substituição de seleção exige capturar/verificar intervalo, editabilidade e alterações; nesta versão a seleção é somente leitura. Clipboard fallback não incluído porque preservar apenas texto violaria o requisito de formatos.
3. macOS Accessibility e X11 precisam implementação nativa e máquinas de teste. Wayland exige prova de portais por compositor. Não há adaptação disfarçada de suporte concluído.
4. Contas pagas/chaves de API não foram fornecidas; OpenAI/Anthropic/Google usam mocks HTTP somente nos testes. LibreTranslate real não estava executando. Nenhuma configuração de conta externa foi feita.
5. Motor local: validar builds Linux/macOS e testes com firewall do SO; ampliar catálogo somente com arquivos/licenças/checksums verificados. Revisão linguística extensa e detecção automática permanecem pendentes.
6. Streaming, histórico opt-in/retention, modos extras, menu dinâmico de provider/idiomas, atalhos de destino, localização UI EN selecionável e OCR continuam fora da primeira versão.
7. CI foi escrita, não executada remotamente; não há Git no diretório nem autorização para publicar. Assinatura/notarização depende de credenciais do mantenedor.
