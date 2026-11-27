"""Consolida a Fase 3 a partir dos CSVs brutos (reprocessa o pareamento; não usa medicoes.csv das rodadas).
Uso: python -I harness/resultados.py
--so-corrigido grava só m7_corrigido.csv (Emenda 10).
Saídas em results/fase3/: medicoes.csv, latencias.csv, sem_resposta.csv, m4.csv, m4_execucoes.csv, memoria.csv, m7.csv.
Regras: repetição 1 descartada (menos no M7, que tem uma só); o M7 do Flutter vem de f3-restrito-m7-flutter (Emenda 7);
só entram execuções que chegaram ao fim; results/_descartadas e <rodada>/_falhas ficam fora."""
import csv, glob, json, math, os, statistics, sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from juntar import juntar  # noqa: E402
from analise import analisar, pct  # noqa: E402
import m4  # noqa: E402
import fase3_plano  # noqa: E402

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RES = os.path.join(RAIZ, "results")
SAIDA = os.path.join(RES, "fase3")
APPS = ["electron", "tauri", "avalonia", "flutter"]


def ler(p):
    return list(csv.DictReader(open(p, encoding="utf-8", errors="replace")))


def gravar(nome, linhas):
    with open(os.path.join(SAIDA, nome), "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(linhas[0]))
        w.writeheader()
        w.writerows(linhas)
    print(nome, len(linhas), "linhas")


def execucoes(rodada):
    for app in APPS:
        for d in sorted(glob.glob(os.path.join(RES, rodada, app, "rep*"))):
            drv = os.path.join(d, "driver.csv")
            if os.path.exists(drv) and any(r["passo"] == "fim" for r in ler(drv)):
                yield app, int(os.path.basename(d)[3:]), d


def regressao(x, y):
    """inclinação por mínimos quadrados e IC 95% (aproximação t = 2,02 para ~40 pontos)."""
    n, mx, my = len(x), statistics.fmean(x), statistics.fmean(y)
    sxx = sum((a - mx) ** 2 for a in x)
    b = sum((a - mx) * (c - my) for a, c in zip(x, y)) / sxx
    res = sum((c - (my + b * (a - mx))) ** 2 for a, c in zip(x, y))
    ep = math.sqrt(res / (n - 2) / sxx)
    return b, b - 2.02 * ep, b + 2.02 * ep


def linha_m7(app, nome, d):
    dr = ler(os.path.join(d, "driver.csv"))
    t0 = next(int(r["t_envio_us"]) for r in dr if r["passo"] == "inicio")
    am = [((int(r["t_us"]) - t0) / 6e7, float(r["privada_mb"])) for r in ler(os.path.join(d, "memoria.csv")) if r["rotulo"] == "auto"]
    am = [(t, v) for t, v in am if t >= 0]
    jan = [(t, v) for t, v in am if 20 <= t <= 60]
    b, lo, hi = regressao([t for t, _ in jan], [v for _, v in jan])
    ini = [v for t, v in am if t < 5]
    return {"app": app, "rodada": nome, "amostras": len(am), "duracao_min": round(am[-1][0], 1), "mb_0a5min": round(statistics.median(ini), 1),
            "mb_20a60min": round(statistics.median([v for _, v in jan]), 1), "mb_max": round(max(v for _, v in am), 1),
            "inclinacao_mb_min": round(b, 3), "ic95_inf": round(lo, 3), "ic95_sup": round(hi, 3),
            "crescimento_continuo": "sim" if b > 0.5 and lo > 0 else "não"}


def m7_corrigido():
    """Emenda 10: M7 do Flutter com o Fleather corrigido, à parte (não entra em m7.csv)."""
    linhas = [linha_m7(app, "f3-restrito-m7-flutter-corrigido", d) for app, _, d in execucoes("f3-restrito-m7-flutter-corrigido")]
    if linhas:
        gravar("m7_corrigido.csv", linhas)


def main():
    medicoes, sem, quadros, memoria, m7 = [], defaultdict(int), [], [], []
    for item in fase3_plano.itens():
        nome, etapa, perfil, doc = item
        if etapa.endswith("medicao"):
            for app, rep, d in execucoes(nome):
                linhas, faltas = juntar(os.path.join(d, "driver.csv"), os.path.join(d, "app.csv"))
                for x in linhas:
                    medicoes.append({"rodada": nome, "app": app, "perfil": perfil, "documento": doc, "repeticao": rep, "aquecimento": int(rep == 1),
                                     "metrica": x["metrica"], "passo": x["passo"], "valor_ms": x["ponta_a_ponta_ms"], "entrada_ms": x["entrada_ms"]})
                if rep > 1:
                    for (passo, m), n in faltas.items():
                        sem[(app, perfil, doc, m, passo)] += n
                mem = ler(os.path.join(d, "memoria.csv"))
                for r in {r["rotulo"]: r for r in mem}.values():
                    if rep > 1:
                        memoria.append({"app": app, "perfil": perfil, "documento": doc, "repeticao": rep, "rotulo": r["rotulo"], "privada_mb": float(r["privada_mb"]), "processos": int(r["n_processos"])})
        elif etapa.endswith("-m4"):
            for app, rep, d in execucoes(nome):
                for t in m4.analisar(os.path.join(d, "presentmon.csv"), os.path.join(d, "driver.csv")):
                    quadros.append({"app": app, "perfil": perfil, "documento": doc, "repeticao": rep, "trecho": t["trecho"], "erro": t.get("erro", ""),
                                    **{k: t.get(k, "") for k in ("duracao_s", "apresentados", "exibidos", "descartados", "perdidos_pct", "longos_pct")}})
        else:
            for app, rep, d in execucoes(nome):
                if app == "flutter" and nome == "f3-restrito-m7":
                    continue
                m7.append(linha_m7(app, nome, d))
    gravar("medicoes.csv", medicoes)
    lat, n_aq = analisar(os.path.join(SAIDA, "medicoes.csv"))
    gravar("latencias.csv", lat)
    gravar("sem_resposta.csv", [{"app": a, "perfil": p, "documento": d, "metrica": m, "passo": s, "sem_resposta": n} for (a, p, d, m, s), n in sorted(sem.items())])
    gravar("m4_execucoes.csv", quadros)
    g = defaultdict(list)
    for q in quadros:
        if q["repeticao"] > 1:
            g[(q["app"], q["perfil"], q["documento"])].append(q)
    resumo = []
    for (app, perfil, doc), qs in sorted(g.items()):
        ok = [q for q in qs if not q["erro"]]
        p = [q["perdidos_pct"] for q in ok]
        resumo.append({"app": app, "perfil": perfil, "documento": doc, "trechos": len(ok), "com_erro": len(qs) - len(ok),
                       "perdidos_p50": round(statistics.median(p), 2) if p else "", "perdidos_p95": round(pct(p, .95), 2) if p else "",
                       "longos_p50": round(statistics.median([q["longos_pct"] for q in ok]), 2) if ok else "",
                       "exibidos_p50": statistics.median([q["exibidos"] for q in ok]) if ok else "",
                       "descartados_p50": statistics.median([q["descartados"] for q in ok]) if ok else ""})
    gravar("m4.csv", resumo)
    g = defaultdict(list)
    proc = defaultdict(set)
    for r in memoria:
        g[(r["app"], r["perfil"], r["documento"], r["rotulo"])].append(r["privada_mb"])
        proc[(r["app"], r["perfil"], r["documento"], r["rotulo"])].add(r["processos"])
    gravar("memoria.csv", [{"app": k[0], "perfil": k[1], "documento": k[2], "rotulo": k[3], "reps": len(v), "privada_mb_p50": round(statistics.median(v), 1),
                            "privada_mb_max": round(max(v), 1), "processos": "/".join(map(str, sorted(proc[k])))} for k, v in sorted(g.items())])
    gravar("m7.csv", m7)
    tela_parada()
    m7_corrigido()


def tela_parada():
    """Emenda 9: as quatro rodadas feitas com a tela parada, reportadas à parte como sensibilidade."""
    linhas_s = []
    for nome, etapa, perfil, doc in fase3_plano.itens():
        pasta = os.path.join("_descartadas", nome + "-estado_lento")
        if not os.path.isdir(os.path.join(RES, pasta)):
            continue
        for app, rep, d in execucoes(pasta):
            for x in juntar(os.path.join(d, "driver.csv"), os.path.join(d, "app.csv"))[0]:
                linhas_s.append({"rodada": nome, "app": app, "perfil": perfil, "documento": doc, "repeticao": rep, "aquecimento": int(rep == 1),
                                 "metrica": x["metrica"], "passo": x["passo"], "valor_ms": x["ponta_a_ponta_ms"], "entrada_ms": x["entrada_ms"]})
    gravar("tela_parada_medicoes.csv", linhas_s)
    gravar("tela_parada.csv", analisar(os.path.join(SAIDA, "tela_parada_medicoes.csv"))[0])


m7_corrigido() if "--so-corrigido" in sys.argv else main()
