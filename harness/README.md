# Harness de medição (Windows)

## driver (`harness/driver`, Rust)
Envia teclas do **sistema operacional** com `SendInput` (o mesmo caminho do AutoHotkey; nenhuma instalação). Lança o app, espera a linha `M5` no CSV do app, coloca a janela em primeiro plano (aborta se não conseguir) e executa o roteiro.

```powershell
cargo build --release --manifest-path harness/driver/Cargo.toml
harness/driver/target/release/catdriver.exe --roteiro spec/roteiros/curto.json --saida results/run/driver.csv --espera-csv results/run/electron.csv -- <app.exe> <args do app> --out results/run/electron.csv
python -I harness/juntar.py results/run/driver.csv results/run/electron.csv results/run/juncao.csv
```

- **Relógio comum:** tempo de parede UTC em µs. Driver: `GetSystemTimePreciseAsFileTime`. App: `performance.timeOrigin + performance.now()`. Substitui a "marcação de sincronia" da spec v0.1; a verificação é que `t_recebe − t_envio` fique em ~1 ms (medido: p50 0,5 a 1,2 ms).
- `juntar.py` pareia cada tecla com a linha do app que a recebeu e calcula `entrada` (SO→app) e `ponta-a-ponta` (tecla enviada → quadro seguinte).
- Passos implementados: `abrir` (feito pelo app via `--doc`), `digitar`, `confirmar`, `mesclar`, `dividir`, `rolar` (PgUp/PgDn), `buscar`, `tecla`, `esperar`, `repetir`. `amostrar_memoria` registra a memória privada da árvore de processos (definição do pré-registro) em `memoria.csv`; `--memoria-cada N` amostra a cada N s. Pendentes: M4 (PresentMon, aguardando autorização do download) e escolha do modo de busca (concordância/memória).

## Limitações conhecidas
- O `curto.json` vai ao segmento 1 (Ctrl+Home, Alt+↓) porque o 0 está sozinho no bloco; os 5 mesclar e 5 dividir passam (corrigido). Em outros documentos o par do mesmo bloco muda de posição.
- Layout de teclado: `VkKeyScanW` usa o layout ativo; o texto dos roteiros é ASCII.
- Linux (xdotool/ydotool) e macOS (AppleScript) ficam para a Etapa 2.

## Emenda 3: ferramentas
| Ferramenta | Função |
| --- | --- |
| `driver --perfil restrito` | Job Object: 4 primeiros núcleos lógicos (máscara 0x0F, núcleos de desempenho) e 8 GB para o app e filhos |
| `driver --autoteste N` | janela própria sem casca; piso do SendInput até a mensagem (medido: p50 0,50 ms, p95 1,26 ms, máx 1,84 ms, 299 de 300 teclas) |
| `rodada.py` | sorteia a ordem por repetição (semente), roda driver, junta e acumula `medicoes.csv`; repetição 1 = aquecimento; `--gpu integrada|nvidia` define a preferência gráfica por executável (HKCU, restaurada ao final); grava `ambiente.json` e `rodada.json` (commit, semente, ordens) |
| `analise.py` | p50, p95, IC 95% do p95 (bootstrap em dois níveis, semente fixa) e status atingida/limítrofe/não atingida |
| `ambiente.py` | hardware, SO, plano de energia, tomada, Smart App Control, Defender, versões |
| `tamanho.py` | M10 (tamanho de pacote/instalador) |
| M9 | `juntar.py`: do spawn do processo até o app iniciar a abertura do documento |

Limitações declaradas: a temperatura da CPU não é exposta pelo Windows neste notebook (`n/d`; exigiria uma ferramenta externa); o Restrito limita núcleos e memória, mas não reproduz a iGPU do perfil sem `--gpu integrada` e sem verificação por PresentMon; M2 no Electron é marcado no mesmo quadro de M1 (a prévia atualiza de forma síncrona no mesmo ciclo).

## M4 (quadros perdidos) com PresentMon
- **Binário:** `harness/tools/PresentMon-2.6.0-x64.exe` (não versionado), baixado de `github.com/GameTechDev/PresentMon/releases/tag/v2.6.0` em 2026-11-24 com autorização do Gian. 980 320 bytes, SHA-256 `B2A706BC6AD475749E3B7E3409263AA1E6906D45BDCF993F6DBC0F660188F1AF`, assinatura digital válida da Intel Corporation.
- **Privilégio:** a captura por ETW exige administrador ou o grupo "Performance Log Users". Sem isso, `PresentMon` termina com "access denied" (verificado). **M4 ainda não foi medido.**
- **Roteiro:** `spec/roteiros/rolagem.json` (passo `rolar_roda`: roda do mouse sobre a lista, 60 eventos por segundo, 5 s para baixo e 5 s para cima; verificado que a lista rola). O driver grava `roda_inicio`, `roda_fim` e os PIDs da árvore do app.
- **Captura:** `rodada.py --presentmon` inicia o PresentMon antes de cada execução e grava `presentmon.csv`; `m4.py` filtra por janela e PIDs e calcula `perdidos_pct` (1 menos quadros exibidos pelo esperado a 60 Hz), `longos_pct` (intervalos acima de 1,5 vsync) e `descartados`. O `m4.py` foi validado só com um CSV sintético; os nomes de coluna do PresentMon real precisam ser conferidos no primeiro uso.
