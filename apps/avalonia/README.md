# Candidato Avalonia 12 (editor próprio, Emenda 2)

- **Pilha:** Avalonia 12.2.0 (Avalonia, Desktop, Themes.Fluent), .NET 10 (SDK 10.0.404), interface 100% em código (sem XAML), núcleo compartilhado por IPC (`CoreClient`, JSON-RPC por stdin/stdout, mesmo protocolo do Electron).
- **Editor próprio:** `SegmentEditor.cs` (347 linhas) é um controle feito do zero: quebra de linha, chips de tag atômicos, cursor, seleção, mouse, teclado, Ctrl+setas com a tag como limite de palavra, Backspace/Delete com confirmação (apaga a tag pareada), copiar/colar texto puro. A entrada composta (acento morto pt-BR) chega pelo evento `TextInput`; **não** implementa `TextInputMethodClient`, portanto IME de CJK não funciona (fora do protótipo).
- **Lista:** `ListBox` do Avalonia com virtualização nativa e altura variável; cada linha tem origem e destino como `SegmentEditor` (só o do segmento ativo é editável). **Prévia rápida:** `ListBox` de blocos com `TextBlock` e `Run` estilizados, atualizado por bloco a cada tecla.
- **Marcação de quadro:** dois `TopLevel.RequestAnimationFrame` encadeados (equivalente ao rAF duplo; `spec/quadro.md`).
- **Construir e rodar:** `dotnet build -c Release`; `bin/Release/net10.0/CatStudyAvalonia.exe --limpo --doc <arquivo> --out <csv>`.

## Estado (2026-11-24)
- T-R8 passa: `curto.json` com teclas do sistema, 200 de 200 teclas, M1, M2, M3, M5, M6, M8 gravados. O dado chega ao SQLite (fila e versões conferidas).
- T-R2 a T-R7 passam por roteiro (`harness/funcional.py`, instantâneo de estado no Ctrl+Alt+F9).
- **Pendente:** seletor de arquivo; publicação (`dotnet publish`, tamanho do instalador: a pasta de build tem 561 MB porque inclui binários nativos de todas as plataformas; o publish para win-x64 será bem menor); exportação e prévia compilada (Emendas 5).
- **Diferenças de plataforma observadas (uma rodada de validação, não são medição):** início do app maior (JIT, sem ReadyToRun), M8 e M3 com mais cauda (a lista de resultados e a lista virtualizada rodam na thread de interface).
- **Mudanças da API do Avalonia 12 que custaram tempo:** `GotFocus/LostFocus` passaram a `FocusChangedEventArgs`; a área de transferência saiu de `GetTextAsync` para as extensões `TryGetTextAsync`/`SetTextAsync` em `Avalonia.Input.Platform`.
