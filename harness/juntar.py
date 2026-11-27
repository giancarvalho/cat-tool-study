"""Junta o CSV do driver (t_envio) com o CSV do app (t_recebe, t_editor, t_previa) e calcula a latência de ponta a ponta.
Uso: python -I harness/juntar.py results/<run>/driver.csv results/<run>/electron.csv [saida.csv]
Pareamento (corrigido na Emenda 7), por passo e em ordem de envio: se o app gravou uma linha por tecla, a i-ésima
linha é da i-ésima tecla; se faltam linhas (ex.: mesclar recusado), cada linha, da última para a primeira, fica com o
envio livre mais recente anterior a ela. Não há corte de latência (só um limite de sanidade de 60 s).
Teclas sem linha do app são contadas como sem resposta."""
import bisect, csv, statistics, sys

# passo do driver -> lista de (métrica, passo no app, campo de fim; None = t_recebe + valor interno do app)
PASSOS = {
    "digitar": [("M1", "digitar", "t_editor_us"), ("M2", "digitar", "t_previa_us")],
    "confirmar": [("M3", "confirmar", None)],
    "mesclar": [("M6", "mesclar", None)],
    "dividir": [("M6", "dividir", None)],
    "buscar": [("M8", "buscar", None)],
    "previa": [("M11", "previa", None)],
}
LIMITE_US = 60_000_000
FOLGA_US = 5_000  # o relógio de alguns apps marca a chegada alguns ms antes do instante de envio do driver


def parear(envios, linhas):
    """envios: instantes ordenados; linhas: [(t_recebe, linha)] ordenadas. Devolve [(t_envio, t_recebe, linha)]."""
    if len(envios) == len(linhas) and all(-FOLGA_US <= rec - t <= LIMITE_US for t, (rec, _) in zip(envios, linhas)):
        return [(t, rec, r) for t, (rec, r) in zip(envios, linhas)]
    pares, j = [], len(envios) - 1
    for rec, r in reversed(linhas):
        k = min(j, bisect.bisect_right(envios, rec + FOLGA_US) - 1)
        if k < 0 or rec - envios[k] > LIMITE_US:
            continue
        pares.append((envios[k], rec, r))
        j = k - 1
    return pares[::-1]


def juntar(driver_csv, app_csv):
    drv = list(csv.DictReader(open(driver_csv, encoding="utf-8")))
    app = list(csv.DictReader(open(app_csv, encoding="utf-8")))
    saida, sem = [], {}
    for passo, specs in PASSOS.items():
        envios = sorted(int(r["t_envio_us"]) for r in drv if r["passo"] == passo)
        for metrica, passo_app, campo in specs:
            linhas = sorted(((int(r["t_recebe_us"]), r) for r in app if r["metrica"] == metrica and r["passo"] == passo_app and r["t_recebe_us"]), key=lambda x: x[0])
            pares = parear(envios, linhas)
            usados = len(pares)
            for t, rec, r in pares:
                fim = (int(r[campo]) - t) / 1000 if campo and r[campo] else (rec - t) / 1000 + float(r["valor"])
                saida.append({"passo": passo, "metrica": metrica, "t_envio_us": t, "entrada_ms": round((rec - t) / 1000, 3), "app_ms": r["valor"], "ponta_a_ponta_ms": round(fim, 3)})
            if len(envios) - usados:
                sem[(passo, metrica)] = len(envios) - usados
    # M5 (abrir até poder editar) é medido só pelo app
    for r in app:
        if r["metrica"] == "M5":
            saida.append({"passo": "abrir", "metrica": "M5", "t_envio_us": "", "entrada_ms": "", "app_ms": r["valor"], "ponta_a_ponta_ms": float(r["valor"])})
    # M9 (início do app): do spawn do processo até o app chamar abrir (t_recebe da linha M5)
    spawn = [int(r["t_envio_us"]) for r in drv if r["passo"] == "spawn"]
    m5 = [int(r["t_recebe_us"]) for r in app if r["metrica"] == "M5" and r["t_recebe_us"]]
    if spawn and m5:
        v = (m5[0] - spawn[0]) / 1000
        saida.append({"passo": "inicio", "metrica": "M9", "t_envio_us": spawn[0], "entrada_ms": "", "app_ms": "", "ponta_a_ponta_ms": round(v, 3)})
    return saida, sem


def pct(v, q):
    v = sorted(v)
    return v[min(len(v) - 1, int(round(q * (len(v) - 1))))]


if __name__ == "__main__":
    saida, sem = juntar(sys.argv[1], sys.argv[2])
    if len(sys.argv) > 3:
        with open(sys.argv[3], "w", newline="", encoding="utf-8") as f:
            w = csv.DictWriter(f, fieldnames=list(saida[0]))
            w.writeheader()
            w.writerows(saida)
    for (passo, m), n in sem.items():
        print(f"{passo:10s} {m}: teclas sem resposta do app: {n}")
    for m in sorted({r["metrica"] for r in saida}):
        for passo in sorted({r["passo"] for r in saida if r["metrica"] == m}):
            v = [r["ponta_a_ponta_ms"] for r in saida if r["metrica"] == m and r["passo"] == passo]
            e = [r["entrada_ms"] for r in saida if r["metrica"] == m and r["passo"] == passo and r["entrada_ms"] != ""]
            ent = f"entrada p50={statistics.median(e):6.2f} ms  " if e else ""
            print(f"{m:3s} {passo:10s} n={len(v):3d}  {ent}ponta-a-ponta p50={statistics.median(v):8.2f} p95={pct(v, .95):8.2f} ms")
