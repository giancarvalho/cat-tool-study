# Candidato Flutter + Fleather

- **Pilha:** Flutter 3.50.1 (Dart 3.14.0), Fleather 1.28.0 (editor de texto rico), scrollable_positioned_list 0.3.8 (lista com rolagem por índice), núcleo compartilhado por IPC (`lib/core_client.dart`, mesmo protocolo do Electron e do Avalonia).
- **Tags atômicas (risco da Emenda 1): resolvido.** O Fleather suporta embeds inline (`SpanEmbed`, 1 posição de documento, desenhados como `WidgetSpan`). Cada tag é um embed `tag` desenhado como chip. O cursor atravessa a tag como 1 caractere. Verificado por roteiro com teclas do sistema e leitura do SQLite: `spec/roteiros/tag_fumaca.json` (Delete sobre a abertura pede confirmação e apaga a abertura e a fechadora pareada) e `tag_palavra.json` (Ctrl+→ atravessa a tag; o texto digitado fica logo depois dela).
- **Controle de tags e teclado:** apagar com confirmação, Ctrl+←/→ com limite de tag e os atalhos do app foram escritos pelo estudo. O Fleather trata teclas por intenção (`DeleteCharacterIntent`, `ExtendSelectionToNextWordBoundaryIntent`), e um handler global de teclado **não impede** o Fleather de agir; foi preciso sobrepor as intenções com `Actions` no editor.
- **Lista e prévia:** `ScrollablePositionedList` (itens de altura variável); só o segmento ativo é `FleatherEditor`; a prévia rápida usa `RichText` por bloco com `ValueNotifier` por bloco (só o bloco do segmento muda a cada tecla).
- **Marcação de quadro:** dois `addPostFrameCallback` encadeados, com `scheduleFrame` (`spec/quadro.md`).
- **Construir e rodar:** `flutter build windows --release`; `build/windows/x64/runner/Release/catstudy.exe --limpo --doc <arquivo> --out <csv>`. Janela 1440x900 fixa (editei `windows/runner`).

## Achados de implementação (foram a maior parte do tempo)
1. **Altura do editor:** o Fleather precisa de altura limitada (`scrollable: true`) para mover o cursor na vertical; com `scrollable: false` o `Home`/`↑`/`↓` lança `Bad state: No element`. O editor ganhou altura estimada pelo texto (`TextPainter`).
2. **Teclas perdidas depois da rolagem:** `ScrollablePositionedList.jumpTo` recria a linha ativa, o nó de foco já "focado" não dispara a abertura da conexão de texto e as teclas se perdem. Resolvido pedindo o teclado de novo (`EditorState.requestKeyboard`) nos quadros seguintes.
3. **Relógio:** `DateTime.now()` tem resolução de ~15 ms no Windows; a origem do relógio de parede precisa ser lida na virada do valor, ou os tempos saem deslocados até 15 ms entre app e driver (afetava 15% das teclas no pareamento).
4. **Entrada composta:** a conexão de texto padrão do Flutter recebe o acento morto do sistema; não foi testada com `´`+`a` por roteiro (pendente, assim como nos outros).

## Estado (2026-11-24)
- T-R8 passa (200 de 200 teclas, M1, M2, M3, M5, M6, M8). T-R3 verificado por fumaça (acima). T-R2 a T-R7 completos por roteiro com leitura de estado: pendentes, como nos outros.
- Pendente: seletor de arquivo, exportação e prévia compilada (Emenda 5), tamanho do pacote (`build/.../Release`).
