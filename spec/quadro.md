# Marcação de quadro por candidato (SCRIPT.md §4)

Regra comum: o app registra o instante do **quadro seguinte** à mudança que o gerou. Os instantes saem em tempo de parede UTC em µs (o mesmo relógio do driver). Esta definição vale antes da primeira medição; mudar depois exige registro.

| Candidato | Método | Observação |
| --- | --- | --- |
| Electron | `requestAnimationFrame` duplo: o primeiro callback roda antes do próximo pintar, o segundo depois que esse quadro foi apresentado ao compositor (`aposQuadro` em `renderer/perf.ts`) | tempo = `performance.timeOrigin + performance.now()` |
| Tauri (WebView2) | o mesmo código do front-end (rAF duplo) | mesmo código; o WebView2 é Chromium |
| Avalonia 12 | a confirmar na implementação: enfileirar no `Dispatcher.UIThread` com prioridade `Render` e, dentro dele, enfileirar de novo com prioridade `Background`; registrar o instante do segundo (depois de layout e render do quadro) | documentar a API usada e testar contra o PresentMon |
| Flutter | `WidgetsBinding.instance.addPostFrameCallback` dentro do quadro que reconstruiu a árvore, mais um `addPostFrameCallback` encadeado para o quadro seguinte | idem |

## Verificação independente
Os métodos acima medem o que o app sabe sobre si. Para os quatro, o **PresentMon** (quadros apresentados pelo sistema operacional, Emenda 3 item 5) mede M4 (quadros perdidos) e serve de checagem cruzada dos tempos de quadro. Divergência sistemática entre a marcação interna e o PresentMon maior que 1 quadro (16,7 ms) invalida a marcação interna daquele candidato até corrigir.

## Limites conhecidos
- A marcação interna não vê o tempo entre apresentar e o pixel aparecer na tela (varredura do monitor); isso vale para os quatro e fica fora da métrica.
- No Electron e no Tauri, M1 e M2 são marcados no mesmo quadro porque a prévia atualiza de forma síncrona com o editor; em outro candidato a prévia pode atualizar depois, e então M2 > M1.

## M4 pelo PresentMon 2.6.0 (validado em 2026-11-24)
- Captura: `rodada.py --presentmon` (sessão ETW `catstudy`, `--date_time`), janela = `roda_inicio`..`roda_fim` do driver, processos = árvore de PIDs gravada pelo driver.
- Colunas usadas: `ProcessID`, `CPUStartDateTime`, `MsBetweenPresents`. A coluna de data sai com deslocamento de horas inteiras em relação ao relógio local (observado: 3 h); `m4.py` calibra pelo `spawn` do driver, registra `desloc_h` e `resto_s` e recusa se o resto passar de 30 s.
- Limite conhecido: o PresentMon só atribui quadros ao processo quando ele apresenta por swap chain DXGI. Electron, Tauri e Flutter aparecem com PID próprio. O Avalonia 12 (renderização padrão, DirectComposition) não gera nenhum evento com o PID próprio: os quadros aparecem no `dwm.exe`, que também atende outras janelas. M4 do Avalonia fica pendente de decisão (ver relatório).
