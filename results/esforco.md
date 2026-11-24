# Registro de esforço (preliminar, 2026-11-24)
Não há cronômetro humano: o trabalho foi feito por um agente em uma sessão. Medidas objetivas por candidato (linhas de código próprias, sem contar dependências nem o núcleo compartilhado):

| Candidato | Linhas próprias | Observações |
| --- | --- | --- |
| Electron | ~1060 (TS: renderer 802, main/preload/testes 259) | ProseMirror dá cursor, seleção, IME e tags atômicas quase prontos; o custo foi virtualização, prévia e a configuração das versões duplicadas do `prosemirror-model` |
| Tauri | ~115 (reusa o renderer do Electron) | custo próprio é a ponte e o comando de log; o front-end foi reaproveitado, o que **não** representa o esforço de quem começa do zero com Tauri |
| Avalonia | ~980 (C#: editor 347, janela 426) | o editor de texto com tags precisou ser escrito do zero (layout, cursor, seleção, teclado); a lista e a prévia usaram controles nativos |
| Flutter | pendente | |

Para o critério de esforço, o Tauri deve ser avaliado como o front-end web mais a ponte; o artigo precisa declarar o reaproveitamento. Horas por requisito: não registradas; se o Gian quiser horas, é preciso definir como contá-las para trabalho de agente.
