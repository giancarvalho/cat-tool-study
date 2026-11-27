# Electron (candidato finalista)

Electron 44.4.5 + prosemirror-view 1.42.8 (+ prosemirror-model 1.25.13, mesma versão da dependência do view). Build com esbuild.
Núcleo: processo filho `catcore` (JSON-RPC), caminho em `CATCORE`; sem a variável, `catcore` (`catcore.exe` no Windows) ao lado do executável ou na pasta `resources` do pacote; senão, `core/target/release/`.
Os testes (`--teste`) leem o corpus de `CORPUS_DIR`, ou de `corpus/out` se a variável não for informada, e gravam o resultado em `TEST_RESULT`, ou em `test-result.json` nesta pasta.
Pacotes para macOS e Linux (teste de fumaça): `npm run build && npx electron-builder --mac` (ou `--linux`), configurados em `electron-builder.yml`.

```
npm install
npm run build
npx electron . --doc ../../corpus/out/pequeno.docx --tm-sintetico 100000
npx electron . --teste --tm-sintetico 100000   # casos funcionais T-R2..T-R7
```

## Decisões de implementação (a refletir na spec v0.2)
- Todos os segmentos são carregados na abertura (em páginas de 2000); a lista é virtualizada (alturas estimadas e depois medidas). Documentos de até 10 mil segmentos.
- Atalhos adicionais para navegar na lista: PgUp/PgDn (±10), Ctrl+Home/Ctrl+End (primeiro/último), Alt+↑/↓ (anterior/próximo).
- Prévia: `b`, `i` e `link` aplicam estilo; só tags sem renderização (`br`) aparecem como chips pequenos.
- Ctrl+←/→: param antes e depois de cada tag (a tag conta como uma parada), e em fim de palavra (espaços são pulados antes da palavra).
- Rascunho do segmento fica na memória até `Ctrl+Enter`; `Esc` volta ao último confirmado.
- Mesclar só junta segmentos do mesmo bloco; dividir recebe a posição em átomos (caractere ou tag).
- Teclas dos testes funcionais vêm de `webContents.sendInputEvent` + `insertText`, não do sistema operacional. A medição (M1, M2) usa o driver externo, ainda não construído (harness).

- Prévia compilada (Emenda 5, M11) integrada: Ctrl+Shift+P, Ctrl+Shift+→/←, Esc; T-R9 passa (ver `harness/README.md`).
