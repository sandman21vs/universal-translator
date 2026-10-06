<!-- CONTRIBUTING.md -->
# Contribuindo

Leia README, IMPLEMENTATION_PLAN e docs/architecture. Código próprio MIT, sem modelos/binários de terceiros versionados automaticamente.

Faça alterações incrementais, rode lint/test/build web, cargo fmt/clippy/test --locked. Novos providers precisam de contratos HTTP locais sem key/conta paga, limites/erros saneados e capabilities honestas. Desktop exige QA manual além de unit tests. Testes reais são opt-in, com modelo instalado escolhido explicitamente.

Não inclua texto privado, keys, headers de autenticação ou query strings sensíveis em logs/issues/fixtures. Preserve fonte/tradução quando ocorrer erro. Nenhum fallback cloud, modelo download, autostart ou histórico habilitado silenciosamente. Documente arquiteturas realmente compiladas/testadas. Para renomear, edite app.meta.json e execute npm run sync:meta; planeje migração do identificador instalado.
