# Roteiro automatizado e log de tempos (v0.1, rascunho)

## 1. Execução
- O app inicia com `--script spec/roteiros/<nome>.json` e `--out results/<run>/<app>.csv`.
- O roteiro é uma lista de **passos**; o app os executa em ordem. Os eventos de teclado vêm do **sistema operacional** (AutoHotkey no Windows, xdotool/ydotool no Linux, AppleScript no macOS), não de chamadas internas ao app, para medir o caminho real da tecla. Os passos que não são tecla (importar, abrir documento) usam a ação interna do app.
- O driver externo (`harness/driver`) lê o mesmo roteiro, envia as teclas e registra `t_envio` (relógio monotônico do SO). O app registra `t_recebe`, `t_editor`, `t_previa`. Os dois relógios são alinhados por uma marcação de sincronia no início de cada execução.
- Cada execução tem **N repetições** (10 em física, 30 em VM), a primeira descartada como aquecimento e declarada.

## 2. Passos do roteiro (tipos)
| Tipo | Parâmetros | Medida gerada |
| --- | --- | --- |
| `abrir` | documento, perfil | M5 (abrir até poder editar), M9 |
| `digitar` | texto, atraso_ms (padrão 80) | M1, M2 |
| `confirmar` | - | M3 |
| `rolar` | quadros ou distância, alvo=lista\|previa | M4 |
| `mesclar` / `dividir` | posição | M6 |
| `buscar` | modo, texto | M8 |
| `esperar` | ms | - |
| `amostrar_memoria` | rotulo | M7 |

## 3. Roteiros
- `curto.json`: abrir, 200 digitações, 20 confirmações, 3 rolagens, 5 mesclar/dividir, 10 buscas. Para validar o app.
- `medicao.json`: o mesmo, repetido 10 vezes por documento (pequeno, médio, longo, pior caso).
- `uso_prolongado.json`: ciclo digitar, confirmar, rolar, mesclar, dividir por 60 min, com `amostrar_memoria` a cada 60 s; 3 documentos abertos (alternando).
- `editor_tags.json`: casos de comportamento do editor (seleção, cursor, tags, navegação por palavra, IME pt-BR) com asserções sobre o estado final do segmento; usado na Etapa 2 em três sistemas.

## 4. Formato do log (CSV, uma linha por evento)
Colunas: `run_id, app, versao_app, so, so_versao, perfil, gpu, repeticao, passo, indice, metrica, t_envio_us, t_recebe_us, t_editor_us, t_previa_us, t_core_us, valor, unidade, aquecimento, erro`.
- `metrica` ∈ `M1`…`M10`; `valor` em milissegundos (exceto M7 em MB e M4 em % de quadros perdidos).
- Marcação de quadro: o app registra o tempo do **quadro seguinte** à mudança (Electron/Tauri: `requestAnimationFrame` duplo; Avalonia: callback pós-render; Flutter: `addPostFrameCallback`). Descrever o método de cada candidato em `spec/quadro.md` antes de medir.
- Além do CSV por execução, `results/<run>/ambiente.json` registra hardware, SO, drivers, versões exatas de todas as dependências, perfil, GPU fixada, modo de energia e temperatura da CPU por amostra.

## 5. Casos de aceitação (T-R1 a T-R8)
| Caso | Verifica |
| --- | --- |
| T-R1 | importar 3 DOCX e o XLSX; contagem de segmentos igual ao esperado (arquivo `esperado.json`) e posição de origem não vazia |
| T-R2 | rolar até o 10 000º e voltar; o segmento ativo sempre visível; alturas variáveis |
| T-R3 | digitar com tags no meio, cursor atravessa tag, apagar tag pede confirmação, Ctrl+seta respeita limite de tag, `Esc` descarta |
| T-R4 | uma tecla altera só o bloco certo da prévia |
| T-R5 | após confirmar, `segmento_versao` e `sync_queue` ganham 1 linha cada, na mesma transação |
| T-R6 | mesclar e dividir deixam tags pareadas e atualizam lista e prévia |
| T-R7 | consulta de concordância e de memória devolvem o conjunto esperado (`esperado_tm.json`) |
| T-R8 | roteiro `curto.json` termina sem erro e gera o CSV completo |

## 6. Regras de medição
Sem ferramentas de depuração abertas; app em build de produção; uma execução por vez; pausa entre execuções; ordem dos candidatos sorteada por rodada (semente registrada).
