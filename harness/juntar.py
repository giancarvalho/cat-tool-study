"""Junta o CSV do driver (t_envio) com o CSV do app (t_recebe, t_editor) e calcula a latência de ponta a ponta.
Uso: python -I harness/juntar.py results/<run>/driver.csv results/<run>/electron.csv [saida.csv]
Pareamento: cada tecla do driver com a primeira linha M1/M2/M3/M6/M8 cujo t_recebe >= t_envio (mesmo relógio de parede, µs)."""
import csv, statistics, sys

drv = list(csv.DictReader(open(sys.argv[1], encoding="utf-8")))
app = list(csv.DictReader(open(sys.argv[2], encoding="utf-8")))
passo_metrica = {"digitar": ("M1", "t_editor_us"), "confirmar": ("M3", None), "mesclar": ("M6", None), "dividir": ("M6", None), "buscar": ("M8", None)}
saida = []
sem = {}
for passo, (m, campo) in passo_metrica.items():
    envios = sorted(int(r["t_envio_us"]) for r in drv if r["passo"] == passo)
    linhas = sorted((int(r["t_recebe_us"]), r) for r in app if r["metrica"] == m and r["passo"] == passo and r["t_recebe_us"])
    j = 0
    for t in envios:
        while j < len(linhas) and linhas[j][0] < t:
            j += 1
        if j >= len(linhas):
            break
        rec, r = linhas[j]
        if rec - t > 1_000_000:  # sem resposta do app para esta tecla (ex.: mesclar recusado)
            sem[passo] = sem.get(passo, 0) + 1
            continue
        j += 1
        entrada = (rec - t) / 1000
        fim = (int(r[campo]) - t) / 1000 if campo and r[campo] else (rec - t) / 1000 + float(r["valor"])
        saida.append({"passo": passo, "metrica": m, "t_envio_us": t, "entrada_ms": round(entrada, 3), "app_ms": r["valor"], "ponta_a_ponta_ms": round(fim, 3)})
if len(sys.argv) > 3:
    with open(sys.argv[3], "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(saida[0]))
        w.writeheader()
        w.writerows(saida)
def p(v, q):
    v = sorted(v)
    return v[min(len(v) - 1, int(round(q * (len(v) - 1))))]
for passo in passo_metrica:
    v = [r["ponta_a_ponta_ms"] for r in saida if r["passo"] == passo]
    e = [r["entrada_ms"] for r in saida if r["passo"] == passo]
    if sem.get(passo):
        print(f"{passo:10s} teclas sem resposta do app: {sem[passo]}")
    if v:
        print(f"{passo:10s} n={len(v):3d}  entrada(SO->app) p50={statistics.median(e):6.2f} ms  ponta-a-ponta p50={statistics.median(v):7.2f} p95={p(v, .95):7.2f} ms")
