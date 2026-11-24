# Plano da Fase 2: protótipo nos quatro candidatos (Windows)

APROVADO pelo Gian em 2026-11-24. Segue `PREREGISTRO.md` (emendas 1 e 2). Nada de medição nesta fase além de rodar o roteiro para ver se funciona.

## 1. Ambiente (verificado hoje)
| Item | Estado | Ação |
| --- | --- | --- |
| Node 22.23, pnpm 10.11 | ok | fixar via `.nvmrc` e `packageManager` |
| Rust/cargo 1.99.1 | ok | fixar em `rust-toolchain.toml`; instalar MSVC build tools se faltar |
| .NET SDK 8.0.428 | **só 8**; Avalonia 12 pode exigir .NET 10 (Dawloom usou SDK 10) | conferir requisito do Avalonia 12 e instalar SDK 10 se preciso (pedir ao Gian) |
| Flutter/Dart | **ausente** | instalar Flutter estável (pedir ao Gian; download e ~2 GB) |
| pandoc | **ausente** | instalar (necessário para o corpus) |
| Python 3.12 | ok | scripts do corpus e análise, venv fixado |

Instalações de ferramentas são downloads: peço autorização ao Gian item a item antes de baixar.

## 2. Estrutura do repositório
```
/spec            especificação funcional única (8 requisitos), contrato do roteiro e do log de tempos
/corpus          scripts e manifesto (origem, licença, comando); arquivos grandes no .gitignore, gerados por script
/core            lógica de domínio compartilhada (ver §3)
/apps/electron   /apps/tauri   /apps/avalonia   /apps/flutter
/harness         roteiro automatizado, coleta de métricas, scripts de análise
/results         CSVs brutos por execução (com versões)
```

## 3. Compartilhamento de lógica
- **Núcleo em Rust** (`/core`): importadores (DOCX, XLSX), segmentador, modelo de segmentos, SQLite, fila de sync, mesclar/dividir, busca na memória. Expõe uma CLI e uma lib.
  - Tauri: usa a lib direto (comandos Tauri).
  - Electron: processo filho com a CLI/IPC, ou addon nativo (napi-rs). Escolha: **processo filho**, para isolar o custo da casca e igualar a fronteira.
  - Avalonia e Flutter: chamam o mesmo binário via IPC (stdin/stdout JSON-RPC). Evita reescrever em C# e Dart; a especificação permite porque o brief diz "compartilhada onde a linguagem permitir" e o importador pode rodar fora da interface.
  - **Decisão a confirmar com o Gian:** em Avalonia e Flutter, o núcleo em IPC vs. nativo. IPC isola a casca mas tira da Avalonia/Flutter o custo e o esforço do ecossistema DOCX/Excel/SQLite, que é critério de pontuação (11%). Proposta: núcleo compartilhado para as métricas de desempenho de UI; **medir à parte** o esforço de ecossistema com um mini-spike nativo por linguagem (ler DOCX, ler XLSX, abrir SQLite).
- **Sync:** sem servidor; a fila só acumula (tabela `sync_queue`).

## 4. Corpus (script, não commitar binários grandes)
1. FAA template .docx: baixar, registrar URL e hash, conferir trechos de terceiros.
2. U.S. Code e Gutenberg: HTML para DOCX com pandoc (10, 100, 300 páginas), injeção de inline via python-docx.
3. Documento de pior caso: 10 mil segmentos, maioria com 3+ tags, gerado por script (sintético, seed fixa).
4. Excel com ~20 mil células de texto: sintético.
5. TM de 100 mil pares EN-PT: DGT-TM (TMX), amostra com seed fixa. Download grande: autorização do Gian.
Manifesto com origem, licença, comando e SHA-256 de cada arquivo.

## 5. Ordem de implementação e checkpoints
| Passo | O que | Pronto quando |
| --- | --- | --- |
| A | Especificação funcional, contrato do roteiro e do log de tempos | arquivos em `/spec` revisados |
| B | Corpus + núcleo Rust (req. 1, 5, 6, 7) com testes automatizados | testes passam nos 3 corpora |
| C | **Electron** (referência): req. 2, 3, 4, 8 | roteiro roda ponta a ponta |
| D | **Tauri**: mesmo front ProseMirror, mesma UI | idem; diário de bugs iniciado |
| E | **Avalonia 12**: lista virtualizada + controle próprio de edição | idem |
| F | **Flutter + Fleather** | idem, ou registro do requisito que falha |
Checkpoint com o Gian ao fim de B, de C e de cada candidato seguinte (relatório de fase curto).

## 6. Pontos de risco e regra de parada
- **Tags atômicas no Fleather:** primeiro experimento do passo F; se não houver embed inline não editável, parar e perguntar.
- **Controle próprio na Avalonia:** tags atômicas, cursor, seleção e navegação por palavra precisam ser construídos; orçar tempo e registrar horas por requisito.
- **Prévia incremental:** o ProseMirror renderiza a prévia como HTML; em Avalonia e Flutter a prévia é desenhada pela própria casca, o que pode exigir uma especificação "aproximada" comum (blocos de texto com estilos básicos). Definir no passo A para não favorecer ninguém.
- Duas abordagens falhas em uma fase: parar e perguntar.

## 7. Registro de esforço
Cada requisito por candidato: horas do agente (tempo de execução do trabalho) e horas de revisão humana, em `results/esforco.csv`, com data e commit. Estimativas do agente são marcadas como tal.

## 8. Fidelidade
UI idêntica, sem animações, janela 1440x900, mesmos dados, build de produção, sem devtools. Teste de aceitação visual por captura de tela comparável entre candidatos.

## 9. Perguntas para o Gian
1. Aprova o núcleo Rust compartilhado via IPC, com mini-spike nativo para medir ecossistema (§3)?
2. Autoriza instalar: .NET SDK 10 (se necessário), Flutter, pandoc, e baixar DGT-TM (~centenas de MB)?
3. Prévia comum "aproximada" como blocos estilizados é aceitável para as cascas que não usam HTML?

## 10. Decisões do Gian (2026-11-24)
1. Núcleo Rust compartilhado via IPC, com mini-spike nativo por linguagem para medir o ecossistema: aprovado (ao aprovar o plano).
2. Instalações autorizadas e feitas: .NET SDK 10.0.404, pandoc 3.14, Flutter 3.50.1 (Dart 3.14.0, em C:/dev/flutter, fora do PATH). DGT-TM autorizado, baixar no passo B.
3. Prévia comum de blocos estilizados (parágrafo, título, item de lista, linha de tabela; negrito, itálico, link; tags como chips; sem imagens nem fidelidade de página): aprovada. Limitação a declarar no artigo: a fidelidade ao layout do DOCX não é medida.
