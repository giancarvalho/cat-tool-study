# Especificação funcional comum (v0.1, rascunho)

Vale para os quatro candidatos. Mudar esta especificação depois do início da implementação exige registro e aprovação do Gian. Fidelidade: janela 1440x900, sem animações, build de produção, sem devtools, mesmos dados.

## 1. Modelo de dados
- **Documento**: lista ordenada de **blocos** (parágrafo, título 1 a 3, item de lista, linha de tabela). Cada bloco tem 1 ou mais **segmentos** (frases).
- **Segmento**: `id`, `block_id`, `ordem`, `origem` e `destino` como sequência de **nós inline**, `estado` (`novo`, `rascunho`, `confirmado`), `versao`, e **posição de origem** (`parte` do pacote, `caminho` XML, `intervalo` de runs; XLSX: `planilha`, `célula`).
- **Nó inline**: `texto{valor, negrito, itálico}` ou `tag{id, tipo, rotulo}`. Tipos de tag: `abre`, `fecha`, `vazia` (ex.: quebra, campo, imagem inline), `link`. Tags de abertura e fechamento pareiam por `id`.
- Texto destino é copiado da origem na primeira abertura (tags incluídas).

## 2. Interface (idêntica nos quatro)
Layout: barra superior (nome do arquivo, contador `n/total`, botões **Mesclar**, **Dividir**, caixa de busca), **lista de segmentos** à esquerda (60% da largura), **prévia** à direita (40%), barra inferior (estado, tempo do último confirmar). Sem animações, sem ícones decorativos.
- **Lista virtualizada**: colunas `#`, `origem`, `destino`; altura variável por linha (texto quebrado em linhas). Só o **segmento ativo** é editável; os demais são texto estático, tags desenhadas como chips.
- **Segmento ativo**: caixa de edição do destino no lugar da célula, com borda destacada. Origem somente leitura.
- **Chips de tag**: retângulo arredondado com o rótulo (`1`, `/1`, `br`, `link`), fundo cinza, texto monoespaçado; uma unidade para cursor e seleção.
- **Prévia comum (aprovada)**: coluna de blocos estilizados (parágrafo, título, item de lista com marcador, linha de tabela em grade simples); negrito, itálico, link sublinhado; tags como chips pequenos; sem imagens nem fidelidade de página. Trecho do segmento ativo destacado. A cada alteração só o bloco afetado é redesenhado.

## 3. Requisitos funcionais (numeração do brief)

**R1. Importar DOCX e XLSX.** Pelo núcleo (§5). Gera segmentos e blocos com posição de origem. Frases separadas por regra simples fixa (`.`, `?`, `!` seguidos de espaço e maiúscula; sem lista de abreviações), idêntica em todos. Importa também o arquivo `.xlsx` (uma célula de texto = 1 bloco). Exportar fica fora do protótipo (cobrir só a posição de origem).

**R2. Lista virtualizada.** Rolar do segmento 1 ao 10 000 sem travar; a altura de cada linha depende do texto; atalhos: `PgUp`, `PgDn`, `Ctrl+Home`, `Ctrl+End`. O segmento ativo sempre visível (rola até ele).

**R3. Editor só no segmento ativo, tags atômicas.**
- Digitar texto insere na posição do cursor.
- Tags são atômicas: cursor passa por elas como por 1 caractere; seleção as inclui por inteiro; `Backspace`/`Delete` sobre uma tag **pede confirmação** (diálogo simples "Apagar tag N? Sim/Não") antes de apagar; apagar uma abertura apaga também a fechadora pareada (a mesma confirmação).
- `Ctrl+←/→` (e `Option` no macOS) move por palavra, **tratando a tag como limite de palavra** (nunca entra nela, nunca a pula).
- `Ctrl+Enter` confirma o segmento e vai para o próximo; `Ctrl+Shift+Enter` confirma e fica; `Esc` descarta a edição não confirmada do segmento.
- Colar texto puro funciona; colar com tags não é suportado no protótipo.
- IME: entrada composta (pt-BR com acento morto, `´` + `a`) deve produzir `á` sem perder tags vizinhas. CJK fora do protótipo (registrar como limitação).

**R4. Prévia incremental.** Cada alteração do destino atualiza só o bloco correspondente na prévia (texto + chips). A atualização acontece a cada tecla (sem debounce).

**R5. SQLite local.** `Ctrl+Enter` grava, numa só transação: nova linha em `segmento_versao` (`segmento_id`, `versao`, `destino` serializado, `criado_em`) e uma linha em `sync_queue` (`op`, `segmento_id`, `versao`, `payload`, `criado_em`, `estado='pendente'`). Sem servidor; a fila só acumula. Banco em modo WAL, arquivo em pasta de dados do usuário.

**R6. Mesclar e dividir.** *Mesclar*: junta o segmento ativo com o seguinte (destino concatenado com um espaço; tags preservadas; abertura e fechamento continuam pareadas). *Dividir*: no cursor, parte o destino em dois segmentos (origem acompanha a primeira ocorrência de limite de frase mais próxima; se não houver, a origem fica inteira no primeiro e o segundo recebe origem vazia). Ambos atualizam lista e prévia e gravam operação na fila.

**R7. Busca na memória e concordância.** Tabela `tm(id, origem, destino)` com 100 mil pares. Caixa de busca na barra superior, 2 modos:
- *Concordância*: texto contido na origem ou no destino (FTS5, `trigram`).
- *Memória*: similaridade com a origem do segmento ativo (bigramas de caracteres, Jaccard), retorna os 5 melhores com pontuação ≥ 0,5.
Resultados em lista abaixo da caixa; sem inserir no segmento.

**R8. Roteiro automatizado.** Ver `SCRIPT.md`. Cada app expõe o roteiro por um **modo de teste** ativado pela variável `CATSTUDY_SCRIPT=<caminho>` ou argumento `--script`; fora desse modo, o app não tem código de medição ativo além de marcações leves de tempo.

## 4. Atalhos fixos
| Ação | Windows/Linux | macOS |
| --- | --- | --- |
| Confirmar e próximo | Ctrl+Enter | Cmd+Enter |
| Mesclar com seguinte | Ctrl+M | Cmd+M |
| Dividir no cursor | Ctrl+Shift+D | Cmd+Shift+D |
| Busca | Ctrl+F | Cmd+F |
| Palavra | Ctrl+←/→ | Option+←/→ |

## 5. Núcleo compartilhado (fronteira)
Processo `catcore` em Rust, protocolo JSON-RPC por stdin/stdout, uma mensagem por linha. Métodos mínimos:
`importar(caminho) -> {doc_id, n_segmentos}`, `segmentos(doc_id, de, ate) -> [segmento]`, `blocos(doc_id, ids) -> [bloco]`, `salvar_segmento(segmento_id, destino) -> {versao}`, `mesclar(segmento_id)`, `dividir(segmento_id, posicao)`, `buscar(modo, texto) -> [resultado]`, `fila(estado) -> n`. Tempo gasto dentro do núcleo é marcado com `t_core_us` em cada resposta, para separar custo do núcleo e custo da casca.
Exceção: o Tauri chama a mesma lib em processo (sem IPC) e registra esse fato.

## 6. Critérios de aceitação por requisito
Cada requisito tem um teste automatizado (§ `SCRIPT.md`, casos `T-R1` a `T-R8`) que roda no modo de teste e devolve `passou`/`falhou`. A Fase 2 termina quando cada candidato passa todos os casos ou há registro do requisito que não cumpre.

## 7. Fora do protótipo
Exportar, autenticação, sync real, memória de tradução como alinhamento, QA, colar com tags, CJK, imagens e caixas de texto na prévia.
