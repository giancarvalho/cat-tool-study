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
- Passos implementados: `abrir` (feito pelo app via `--doc`), `digitar`, `confirmar`, `mesclar`, `dividir`, `rolar` (PgUp/PgDn), `buscar`, `tecla`, `esperar`, `repetir`. Pendentes: `amostrar_memoria` (M7), medição de quadros perdidos (M4), escolha do modo de busca (concordância/memória).

## Limitações conhecidas
- O `curto.json` assume que o segmento 0 e o 1 podem estar em blocos diferentes: o primeiro `mesclar` é recusado pelo app (1 de 5) e o `dividir` seguinte divide o segmento 0. Para a medição, o roteiro deve escolher um par do mesmo bloco por posição.
- Layout de teclado: `VkKeyScanW` usa o layout ativo; o texto dos roteiros é ASCII.
- Linux (xdotool/ydotool) e macOS (AppleScript) ficam para a Etapa 2.
