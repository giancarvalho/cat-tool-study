# Pacotes portáteis de produção (M10, parcial) — 2026-11-24

Mesmo método para os quatro: pasta de distribuição com o executável do app e o `catcore.exe` (no Tauri, o núcleo vai dentro do executável), sem instalador, sem otimização de tamanho além do padrão da ferramenta. Zip: deflate nível 9, medido com Python. Todos passam T-R3, T-R6 e T-R7r a partir da pasta do pacote (`harness/candidatos_pacote.json`).

| Candidato | Como foi montado | Pasta (MB) | Zip (MB) |
| --- | --- | ---: | ---: |
| electron | distribuição binária do Electron 44.4.5 + `resources/app` (bundle esbuild), sem pruning de idiomas | 370.1 | 151.5 |
| tauri | `cargo build --release` (WebView2 do sistema, não incluído) | 6.2 | 2.7 |
| avalonia | `dotnet publish -c Release -r win-x64 --self-contained` (sem trimming nem single-file) | 206.7 | 70.9 |
| flutter | `flutter build windows --release` (pasta Release) | 29.9 | 12.6 |

Limites: (1) não há instalador (NSIS/MSI/MSIX): as ferramentas não estão no cache e baixá-las exige autorização; (2) Tauri depende do WebView2 já presente no sistema (não contado); (3) tamanho da atualização não medido: nenhum candidato tem mecanismo de atualização implementado; (4) variantes otimizadas (trimming do .NET, pruning de locales do Electron) não foram feitas, para manter o padrão de cada ferramenta.
# Pacotes portáteis de produção (M10, parcial) — 2026-11-24

Mesmo método para os quatro: pasta de distribuição com o executável do app e o `catcore.exe` (no Tauri, o núcleo vai dentro do executável), sem instalador, sem otimização de tamanho além do padrão da ferramenta. Zip: deflate nível 9, medido com Python. Todos passam T-R3, T-R6 e T-R7r a partir da pasta do pacote (`harness/candidatos_pacote.json`).

| Candidato | Como foi montado | Pasta (MB) | Zip (MB) |
| --- | --- | ---: | ---: |
| electron | distribuição binária do Electron 44.4.5 + `resources/app` (bundle esbuild), sem pruning de idiomas | 370.1 | 151.5 |
| tauri | `cargo build --release` (WebView2 do sistema, não incluído) | 6.2 | 2.7 |
| avalonia | `dotnet publish -c Release -r win-x64 --self-contained` (sem trimming nem single-file) | 206.7 | 70.9 |
| flutter | `flutter build windows --release` (pasta Release) | 29.9 | 12.6 |

Limites: (1) não há instalador (NSIS/MSI/MSIX): as ferramentas não estão no cache e baixá-las exige autorização; (2) Tauri depende do WebView2 já presente no sistema (não contado); (3) tamanho da atualização não medido: nenhum candidato tem mecanismo de atualização implementado; (4) variantes otimizadas (trimming do .NET, pruning de locales do Electron) não foram feitas, para manter o padrão de cada ferramenta.

## Instaladores (autorizado por Gian em 2026-11-24)
Ferramenta: NSIS 3.13 (baixado pelo `tauri build` de `github.com/tauri-apps/binary-releases`, hash validado pelo Tauri). **Mesmo script e mesma compressão para os quatro** (`harness/instalador/generico.nsi`: LZMA sólido, instalação por usuário, atalho e desinstalador, sem UAC). Teste de instalação silenciosa e desinstalação feito no Flutter.

| Candidato | Instalador (MB) |
| --- | ---: |
| avalonia | 49.6 |
| electron | 110.6 |
| flutter | 9.7 |
| tauri-nativo | 2.2 |
| tauri | 2.1 |

O `tauri-nativo` é o instalador que o próprio `tauri build` gera (NSIS do Tauri, informativo; a comparação usa a linha `tauri`). Tamanho de atualização: não medido (nenhum candidato tem mecanismo de atualização).
