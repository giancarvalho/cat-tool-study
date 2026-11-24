# Fase 0: literatura (2026-11-24)

Fichas completas em `fichas.csv` (29 linhas; coluna `status` diz o que foi aberto). Limite de método: as páginas foram abertas via WebFetch, que devolve resumo por modelo pequeno; números críticos devem ser reconferidos na fonte antes do artigo.

## O que a literatura mostra

**Tema 1, benchmarks.** A única medição com versões fixas e soma de todos os processos é a Dawloom (Windows 11, out/2026): Tauri 142 MB vs Electron 149 MB (±5%), com N=5 vs N=1 e app trivial. Ganhos grandes do Tauri (172 vs 409 MB; 109 vs 128 MB) vêm de macOS/WebKit com método frouxo. A issue tauri#5889 mostra que contar memória compartilhada pode inverter o resultado. Vantagem de instalador do Tauri é robusta (1 a 18 MB vs 100 a 300 MB), mas AppImage Linux chega a ~94 MB. Nenhum benchmark confiável de Flutter desktop; Avalonia só com tamanho de pacote.

**Tema 2, migrações.** Único relato sólido, sem números: OpenCode (Tauri para Electron por WebKit). Relatos com números (besthub, Electron para Tauri, -50% memória) não descrevem método. Descartados: johalputt (404, padrão de conteúdo gerado), gerus_team (reescrita em C++, conflito de interesse), quantamind (Electron nunca construído).

**Tema 3, contenteditable.** O changelog do prosemirror-view (até 1.42.8, 24/10/2026) registra dezenas de correções por navegador para átomos inline, backspace e composição IME. Pontos que importam para tags protegidas: dummy node em Safari/Firefox, navegação por palavra no Safari pula não-texto (keymap próprio), cursor com `contenteditable=false` (corrigido em 1.42.4 só no Chrome desktop). WebKitGTK: caret fantasma (327674, aberto, baixa relevância); IME do WebKitGTK corrigido em 2024 (218148). WKWebView/IME coreano não dispara composition (xterm.js#5704).

**Tema 4, editores fora do navegador.**
- Avalonia RichTextEditor: só no Pro; sem menção a IME nem a chip inline não editável (só controles embutidos).
- Contrato 10C: trial de 30 dias, só avaliação interna, proíbe produção e distribuição, multa mínima de €5.000, **omisso sobre publicar resultados**. Pedir confirmação escrita à Avalonia antes de usar o trial neste estudo.
- Contrato 10B: assinatura anual dá direitos perpétuos sobre a versão licenciada; sem renovar, perde só as atualizações.
- Super Editor: Win/Linux "verificação irregular", versão estável com ~2 anos, sem sinal de IME desktop.
- Fleather: declara 6 plataformas, 56 issues abertas, bugs de IME em desktop (#580, #419, #420); chip atômico não verificado.

**Tema 5, CAT tools.** Trados e memoQ: Windows desktop (.NET; WPF não verificado). Nenhuma fonte primária explica o Windows-only; a explicação (legado .NET/COM, custo de portar) é inferência. Phrase: web-first, desktop multiplataforma em manutenção desde out/2025. Matecat: PHP+JS, LGPL-3.0. OmegaT: Java, GPL-3.0. CafeTran 12.5: multiplataforma, stack não confirmada. Várias páginas de fornecedores retornaram 404.

**Tema 6, bibliotecas.**
- DOCX com fidelidade de runs: Open XML SDK (C#, MIT) é o melhor; JS (docx, mammoth) não faz round-trip; docx-rs (Rust) round-trip não verificado; Dart só com `docx_creator` (licença "unknown") ou OOXML manual (`archive`+`xml`).
- XLSX: calamine só lê e rust_xlsxwriter só escreve; exceljs, SheetJS(npm) e `excel` (Dart) com pouca manutenção; **EPPlus 8 é Polyform Noncommercial** (armadilha para produto comercial).
- XLIFF: `xliff` (npm) é o mais maduro, com inline limitado; Xliff.OM só 2.0 e parado desde 2022; Rust só `bilingua-xliff` (v0.10, criado em 11/2026); Dart sem pacote.

## Lacunas que este estudo cobre
1. Nenhum benchmark mede editor de texto estruturado com tags atômicas nos três sistemas.
2. Memória de todos os processos em uso prolongado: só a Dawloom soma processos, e em app trivial.
3. Avalonia e Flutter praticamente ausentes de comparações com método.
4. Ninguém compara esforço de implementação nem consistência do editor entre motores.
5. Sem comparação de ecossistema DOCX/XLSX/XLIFF por linguagem com foco em tags inline.

## Pendências e alertas para o Gian
- **Flutter (checagem obrigatória):** Super Editor com Win/Linux não verificados e Fleather com bugs de IME. Decisão sua: manter Flutter com Fleather, trocar o editor, ou tirar da lista de representantes.
- **Avalonia:** decidir como cobrir o RichTextEditor (trial exige confirmação escrita para publicar; compra custa €899/ano).
- Reconferir na fonte original os números usados em tabelas do artigo.
- Fontes não abertas: tauri benchmark_results, repo web-to-desktop-framework-comparison, vários posts de vendor (ver `status` em `fichas.csv`).
