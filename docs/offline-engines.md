<!-- docs/offline-engines.md -->
# Motor local integrado

Novas configurações usam **Local integrado · modelos Argos**. Configurações existentes recebem essa opção em memória, preservando provider, endpoint e consentimentos anteriores. Somente Salvar grava a configuração migrada.

O LibreTranslate utiliza Argos e acrescenta um servidor HTTP. Incorporamos inferência com **CTranslate2 4.8.2** e **SentencePiece 0.2.2**, lendo modelos Argos diretamente, em um processo Python 3.12 empacotado pelo PyInstaller 6.22.3. O computador final não precisa de Python, pip, Docker, porta, chave de API ou GPU. LibreTranslate HTTP continua disponível para servidores externos. Fontes: [LibreTranslate](https://github.com/LibreTranslate/LibreTranslate), [formato de pacotes Argos](https://github.com/argosopentech/argos-translate/blob/master/argostranslate/package.py), [CTranslate2](https://opennmt.net/CTranslate2/), [sidecar Tauri](https://v2.tauri.app/develop/sidecar/).

Não embarcamos a biblioteca Python Argos, Stanza/MiniSBD, Torch ou LibreTranslate/Flask. A divisão simples em frases/linhas difere do Argos completo; saída/qualidade idêntica não são prometidas. O modo local preserva quebras de linha e normaliza marcadores SentencePiece, conservando underscores. Não oferece detecção automática, tom/tratamento ou variantes regionais: escolha origem explícita. Frases acima de 1.024 tokens ou saída que atinge 2.048 tokens falham sem truncamento oculto. Traduções precisam revisão.

## Instalação dos idiomas

Em Configurações → Local integrado, cada botão Baixar mostra par, versão, tamanho real, origem, licença e checksum. Listar, abrir o app e traduzir não baixam modelos. O backend permite somente IDs do catálogo embarcado, baixa sem credenciais/texto, limita bytes/tempo e verifica SHA-256 antes da extração e novamente no worker. Extração verifica caminhos, limite expandido e metadados; instala atomicamente somente dados de inferência em `local-models` no diretório de dados do app. Preserva README, manifest, recibo e atribuições. Downloads falhos são descartados; não existe fallback remoto.

Catálogo verificado em 05/10/2026 com arquivos do [índice oficial](https://github.com/argosopentech/argospm-index):

| Par direto | Versão | Bytes baixados | Licença/evidência |
|---|---|---:|---|
| PT → EN | 1.9 | 69.447.231 | README: OPUS-MT original CC-BY-4.0 |
| EN → PT | 1.9 | 66.179.184 | README: OPUS-MT original CC-BY-4.0 |
| DE → EN | 1.3 | 150.512.831 | Declaração do mantenedor: binários Argos MIT/CC0 |
| EN → DE | 1.3 | 150.508.297 | Declaração do mantenedor: binários Argos MIT/CC0 |

Hashes completos/créditos em `engine/catalog.json`. Os modelos alemães não têm licença individual dentro do arquivo: a evidência é [a declaração oficial do mantenedor](https://github.com/argosopentech/argos-translate/issues/533), apresentada como tal na interface. Português identifica Jörg Tiedemann e Santhosh Thottingal, OPUS-MT, EAMT 2020, Lisboa, e CC-BY-4.0 do modelo original. Não inferimos licença pela MIT do aplicativo. `engine/THIRD_PARTY.md` acompanha cada modelo; as licenças reais de Python/dependências são coletadas no executável.

PT ↔ DE usa PT → EN → DE ou DE → EN → PT. A UI mostra os pares diretos e a rota intermediária, que pode reduzir qualidade. O catálogo local inicial é PT/EN/DE; outros idiomas continuam nos demais providers. Pesos ficam fora do Git e do instalador.

## Ciclo de vida e distribuição

Worker iniciado sob demanda por requisição, um por vez. Texto passa por pipe privado stdio, sem argumentos, HTTP ou logs. O Rust mantém um pipe de propriedade aberto: cancelamento/timeout/saída fecham esse pipe e encerram também o filho de inferência do bootloader PyInstaller. Modelo liberado ao concluir; não há servidor residente. A inicialização por chamada tem custo, medido na validação.

Prepare com Python 3.12: `python scripts/prepare-engine.py`. Dependências/versões/hashes PyPI fixados em `engine/requirements.lock`. O script cria binário para a arquitetura nativa e copia a `src-tauri/binaries/` com o sufixo de target exigido pelo Tauri. Instaladores incluem `local-engine` via `bundle.externalBin`. A versão portátil precisa dos dois executáveis na mesma pasta.

Build e inferência executados em Windows x64. Workflows preparam Linux x64/macOS Intel/ARM, ainda sem execução/QA nessas máquinas. Bergamot/Marian continuam alternativas documentais, sem segundo motor integrado. O teste sem rede bloqueia conexões no nível socket Python; não equivale a um teste com firewall do sistema operacional.
