# Spec v0.2: proposta de alterações (a aprovar pelo Gian; a v0.1 continua em vigor até lá)

Cada item vem de uma decisão já tomada (Emendas 3 a 5) ou de algo descoberto ao implementar o Electron e o Tauri. Nada altera critérios, metas nem pesos.

## Decisões já aprovadas, a incorporar ao texto
1. **R1 inclui PPTX** (Emenda 4): cada parágrafo de cada forma ou célula vira um bloco; origem = slide e forma. Teste T-R1 ganha o PPTX.
2. **Prévia dupla** (Emenda 5, §2): rápida (blocos estilizados, mede M2) e compilada (documento real via LibreOffice headless, mede M11).
3. **R9 Exportar** (Emenda 5): o núcleo exporta DOCX e PPTX com os destinos dos segmentos e tags pareadas. Teste T-R9: exportar sem edição reproduz o texto original; com edição, troca só os segmentos alterados.
4. **R7**: a TM real tem 51 769 pares EN-GB/PT-PT (Emenda 3, item 6); M8 nos dois modos.
5. **Roteiro**: o driver usa `SendInput` (Windows) em vez de AutoHotkey; relógio comum de parede em vez de marcação de sincronia; tecla `Esc` de aquecimento antes do roteiro (o WebView2 descartava a primeira tecla).

## Detalhes descobertos na implementação (precisam de decisão)
6. **Atalhos de navegação além do v0.1**: `PgUp/PgDn` (10 segmentos), `Ctrl+Home/End`, `Alt+↑/↓` (segmento anterior/seguinte). Já existem no Electron e no Tauri; entram em todos.
7. **Tags na prévia rápida**: negrito, itálico e link viram estilo; só `br` aparece como chip. (v0.1 dizia "tags como chips pequenos".)
8. **Mesclar** só atua em segmentos do mesmo bloco; entre blocos o app mostra "sem segmento seguinte no bloco" e não faz nada.
9. **XLIFF**: o brief original lista XLIFF como formato de importação e a v0.1 não o cobre. Decidir: incluir como R1b ou registrar como fora do protótipo.
10. **Seletor de arquivo (`abrir`)**: os roteiros abrem por `--doc`; o botão "Abrir" existe no Electron e não no Tauri. Decidir se o seletor é requisito do protótipo.
11. **Notas do orador do PPTX, layouts e mestres**: fora do protótipo.
