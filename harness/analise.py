"""Análise (Emenda 3, item 8; PREREGISTRO seção 6): p50, p95 e IC 95% do p95 por bootstrap em dois níveis
(reamostra repetições e, dentro delas, amostras), sem a repetição de aquecimento. Meta atingida só se o limite superior
do IC do p95 estiver abaixo da meta; se só o p95 passa, "limítrofe".
Uso: python -I harness/analise.py results/<rodada>/medicoes.csv [saida.csv]"""
import csv, random, statistics, sys
from collections import defaultdict

METAS = {"M1": 16, "M2": 50, "M3": 50, "M5": 2000, "M6": 100, "M8": 100}  # ms (p95)
B = 2000


def pct(v, q):
    v = sorted(v)
    return v[min(len(v) - 1, int(round(q * (len(v) - 1))))]


def ic_p95(por_rep, semente=20261124):
    rng = random.Random(semente)
    reps = list(por_rep)
    est = []
    for _ in range(B):
        amostra = []
        for r in (rng.choice(reps) for _ in reps):
            v = por_rep[r]
            amostra += [rng.choice(v) for _ in v]
        est.append(pct(amostra, 0.95))
    est.sort()
    return est[int(0.025 * B)], est[int(0.975 * B) - 1]


def analisar(caminho):
    g = defaultdict(lambda: defaultdict(list))
    n_aquec = 0
    for r in csv.DictReader(open(caminho, encoding="utf-8")):
        if r["aquecimento"] == "1":
            n_aquec += 1
            continue
        g[(r["app"], r["perfil"], r["documento"], r["metrica"], r["passo"])][r["repeticao"]].append(float(r["valor_ms"]))
    saida = []
    for (app, perfil, doc, m, passo), por_rep in sorted(g.items()):
        todas = [x for v in por_rep.values() for x in v]
        lo, hi = ic_p95(por_rep) if len(por_rep) > 1 else (float("nan"), float("nan"))
        meta = METAS.get(m)
        p95 = pct(todas, 0.95)
        if meta is None:
            status = "registrar"
        elif p95 < meta and hi < meta:
            status = "atingida"
        elif p95 < meta:
            status = "limítrofe"
        else:
            status = "não atingida"
        saida.append({"app": app, "perfil": perfil, "documento": doc, "metrica": m, "passo": passo, "reps": len(por_rep), "n": len(todas), "p50_ms": round(statistics.median(todas), 2), "p95_ms": round(p95, 2), "ic95_p95_inf": round(lo, 2), "ic95_p95_sup": round(hi, 2), "meta_ms": meta or "", "status": status})
    return saida, n_aquec


if __name__ == "__main__":
    saida, n_aquec = analisar(sys.argv[1])
    print(f"linhas de aquecimento descartadas: {n_aquec}")
    for r in saida:
        print(f'{r["app"]:9s} {r["perfil"]:8s} {r["metrica"]:3s} {r["passo"]:10s} reps={r["reps"]:2d} n={r["n"]:5d} p50={r["p50_ms"]:8.2f} p95={r["p95_ms"]:8.2f} IC95=[{r["ic95_p95_inf"]:.2f}, {r["ic95_p95_sup"]:.2f}] meta={r["meta_ms"]} {r["status"]}')
    if len(sys.argv) > 2 and saida:
        with open(sys.argv[2], "w", newline="", encoding="utf-8") as f:
            w = csv.DictWriter(f, fieldnames=list(saida[0]))
            w.writeheader()
            w.writerows(saida)
