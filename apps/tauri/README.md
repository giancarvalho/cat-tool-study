# Candidato Tauri 2

- **Front-end:** o mesmo código do renderer do Electron (`apps/electron/src/renderer`, ProseMirror, lista virtualizada, prévia), empacotado por `build.mjs` com uma ponte `window.cat` própria (`src/shim.ts`, sobre `invoke`). Dependências do ProseMirror resolvidas em `apps/electron/node_modules` (versões fixas lá).
- **Núcleo:** a lib `catcore` roda **em processo** (comandos `rpc`, `args`, `abrir`, `log` em `src-tauri/src/main.rs`); os comandos são assíncronos para não bloquear a thread da interface. Despacho comum em `core/src/rpc.rs`.
- **Webview:** WebView2 do sistema (Edge 158 nesta máquina). Não vem junto do executável.
- **Construir:** `npm install`, `npm run build` (executável em `src-tauri/target/release/catstudy-tauri.exe`, 6,0 MB sem instalador). Instalador (`tauri build`, M10) fica para a etapa de empacotamento.
- **Rodar:** `catstudy-tauri.exe --limpo --doc <arquivo> --out <csv>`.

## Estado (2026-11-24)
- T-R8 passa: `curto.json` roda até o fim com teclas do sistema (200 digitações, 20 confirmações, 5 mesclar, 5 dividir, 10 buscas), 200 de 200 teclas recebidas.
- T-R2 a T-R7 passam por roteiro (`harness/funcional.py`, instantâneo de estado no Ctrl+Alt+F9).
- **Pendente:** seletor de arquivo (`abrir`); pacote de instalação.
- **Achado de harness:** o WebView2 descartou a primeira tecla enviada após focar a janela (o Electron não). O driver agora envia `Esc` de aquecimento para todos os candidatos antes do roteiro.
- Diferenças de comparação: o Electron roda o núcleo como processo filho (IPC), o Tauri em processo; é a decisão do plano (§3), e o custo de fronteira faz parte do que cada casca oferece.

- Prévia compilada (Emenda 5, M11) integrada: Ctrl+Shift+P, Ctrl+Shift+→/←, Esc; T-R9 passa (ver `harness/README.md`).
