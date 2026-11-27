"""Fase 4: pontuação PROVISÓRIA da Etapa 1 (só Windows) pela regra do PREREGISTRO seção 8.
Uso: python -I -X utf8 harness/pontuacao.py
Lê results/fase3/*.csv e dist-pacotes/tamanhos.json; grava results/fase4/{notas,desempenho,sensibilidade}.csv e resumo.txt.
Consistência (28%) não é pontuada na Etapa 1. As escolhas que o pré-registro não fixou estão na Emenda 11
(aprovada em 27/11, escrita depois dos resultados); as alternativas são calculadas na sensibilidade."""
import csv, itertools, json, os, statistics

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
F3, F4 = os.path.join(RAIZ, "results", "fase3"), os.path.join(RAIZ, "results", "fase4")
APPS = ["electron", "tauri", "avalonia", "flutter"]
PESOS = {"desempenho": 22, "memoria": 17, "esforco": 17, "ecossistema": 11, "instalador": 5}
PESO_CONSISTENCIA = 28
# Linhas próprias em 27/11 (sem testes, dependências e núcleo comum); Tauri = front-end reaproveitado + ponte.
LINHAS = {"electron": 1066, "tauri": 1081, "avalonia": 1053, "flutter": 1107}
# Ecossistema: 5 menos 1 por formato que exige implementação manual ou licença paga (phase0/fichas.csv, 25 a 29).
ECOSSISTEMA = {"electron": 4, "tauri": 3, "avalonia": 4, "flutter": 3}


def ler(nome):
    return list(csv.DictReader(open(os.path.join(F3, nome), encoding="utf-8")))


def nota_meta(r):
    return 5 if r <= .5 else 4 if r <= .75 else 3 if r <= 1 else 2 if r <= 1.5 else 1


def nota_razao(valores):
    menor = min(valores.values())
    return {a: (5 if v / menor <= 1.25 else 4 if v / menor <= 1.75 else 3 if v / menor <= 2.5 else 2 if v / menor <= 4 else 1) for a, v in valores.items()}


def desempenho(fonte="latencias.csv", perfis=("restrito",), pior=False, m4_pequeno=False, com_m4=True):
    lat, m4 = ler(fonte), ler("m4.csv")
    notas, detalhe = {}, []
    for app in APPS:
        por_m = {}
        for m, meta in (("M1", 16), ("M2", 50), ("M4", 5), ("M6", 100)):
            if m == "M4":
                if not com_m4:
                    continue
                v = [float(r["perdidos_p95"]) for r in m4 if r["app"] == app and r["perfil"] in perfis and (m4_pequeno or r["documento"] != "pequeno")]
            else:
                v = [float(r["p95_ms"]) for r in lat if r["app"] == app and r["perfil"] in perfis and r["metrica"] == m]
            por_m[m] = nota_meta(max(v) / meta) if pior else statistics.fmean(nota_meta(x / meta) for x in v)
            detalhe.append({"app": app, "metrica": m, "combinacoes": len(v), "p95_min": round(min(v), 2), "p95_max": round(max(v), 2), "nota": round(por_m[m], 3)})
        notas[app] = statistics.fmean(por_m.values())
    return notas, detalhe


def memoria(modo="m7_mediana", corrigido=False):
    m7 = {r["app"]: r for r in ler("m7.csv")}
    if corrigido:
        m7["flutter"] = ler("m7_corrigido.csv")[0]
    if modo == "aberto":  # memória com o documento recém-aberto, média dos documentos, perfil Restrito
        mem = ler("memoria.csv")
        v = {a: statistics.fmean(float(r["privada_mb_p50"]) for r in mem if r["app"] == a and r["perfil"] == "restrito" and r["rotulo"] == "aberto") for a in APPS}
    else:
        v = {a: float(m7[a]["mb_20a60min" if modo == "m7_mediana" else "mb_max"]) for a in APPS}
    n = nota_razao(v)
    return {a: max(1, n[a] - (m7[a]["crescimento_continuo"] == "sim")) for a in APPS}, {a: round(x, 1) for a, x in v.items()}


def total(notas, criterios):
    return {a: sum(PESOS[c] * notas[c][a] for c in criterios) / sum(PESOS[c] for c in criterios) for a in APPS}


def gravar(nome, linhas):
    with open(os.path.join(F4, nome), "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(linhas[0]))
        w.writeheader()
        w.writerows(linhas)


def main():
    os.makedirs(F4, exist_ok=True)
    tam = json.load(open(os.path.join(RAIZ, "dist-pacotes", "tamanhos.json"), encoding="utf-8"))["tamanhos"]
    inst = {a: tam[a]["instalador_mb"] for a in APPS}
    desemp, detalhe = desempenho()
    mem, mem_v = memoria()
    esf = nota_razao(LINHAS)
    base = {"desempenho": desemp, "memoria": mem, "esforco": esf, "ecossistema": ECOSSISTEMA, "instalador": nota_razao(inst)}
    t1 = total(base, ["desempenho", "memoria", "ecossistema", "instalador"])
    t2 = total(base, list(PESOS))
    out = []

    def p(*a):
        s = " ".join(str(x) for x in a)
        print(s)
        out.append(s)

    p("memória M7 mediana 20-60 min (MB):", mem_v)
    for modo in ("m7_max", "aberto"):
        p("memória", modo, "(MB):", memoria(modo)[1])
    p("instalador (MB):", inst)
    for c, n in base.items():
        p(f"{c:12}", {a: round(n[a], 2) for a in APPS})
    p("PRINCIPAL sem esforço e sem consistência (55 pontos):", {a: round(v, 2) for a, v in t1.items()})
    p("com esforço por linhas (72 pontos):", {a: round(v, 2) for a, v in t2.items()})
    gravar("notas.csv", [{"app": a, **{c: round(base[c][a], 3) for c in PESOS}, "parcial_sem_esforco": round(t1[a], 3), "parcial_com_esforco_linhas": round(t2[a], 3)} for a in APPS])
    gravar("desempenho.csv", detalhe)

    var = {
        "desempenho": {"principal": desemp, "dois_perfis": desempenho(perfis=("restrito", "completo"))[0], "pior_p95": desempenho(pior=True)[0],
                       "m4_com_pequeno": desempenho(m4_pequeno=True)[0], "tela_parada_sem_m4": desempenho("tela_parada.csv", ("restrito", "completo"), com_m4=False)[0]},
        "memoria": {"principal": mem, "m7_maximo": memoria("m7_max")[0], "doc_aberto": memoria("aberto")[0]},
        "esforco": {"fora": None, "linhas": esf},
        "ecossistema": {"principal": ECOSSISTEMA, "tauri_com_bibliotecas_js": {**ECOSSISTEMA, "tauri": 4}},
    }
    if os.path.exists(os.path.join(F3, "m7_corrigido.csv")):
        var["memoria"]["flutter_corrigido"] = memoria(corrigido=True)[0]
        p("memória com o Flutter corrigido (MB):", memoria(corrigido=True)[1])
    chaves = list(var)

    def totais(escolha):
        notas = {c: var[c][v] for c, v in zip(chaves, escolha)} | {"instalador": base["instalador"]}
        return notas, total(notas, [c for c in PESOS if notas[c] is not None])

    lider, empate, n_comb = {a: 0 for a in APPS}, {a: 0 for a in APPS}, 0
    for escolha in itertools.product(*(list(var[c]) for c in chaves)):
        if "flutter_corrigido" in escolha:
            continue
        t = totais(escolha)[1]
        topo = max(t.values())
        n_comb += 1
        for a in APPS:
            lider[a] += t[a] >= topo - 1e-9
            empate[a] += (topo - t[a]) / topo < .10
    linhas = []
    for c in chaves:
        for v in var[c]:
            notas, t = totais([v if k == c else next(iter(var[k])) for k in chaves])
            linhas.append({"criterio": c, "variante": v, **{f"nota_{a}": round(notas[c][a], 2) if notas[c] else "" for a in APPS}, **{f"total_{a}": round(t[a], 2) for a in APPS}})
    gravar("sensibilidade.csv", linhas)
    for l in linhas:
        p(l["criterio"], l["variante"], "| notas", [l[f"nota_{a}"] for a in APPS], "| totais", [l[f"total_{a}"] for a in APPS])
    p(f"{n_comb} combinações (sem o Flutter corrigido). Em primeiro:", lider)
    p("dentro de 10% do primeiro (empate técnico):", empate)
    for nome, t, peso in (("sem esforço", t1, 55), ("com esforço", t2, 72)):
        for a, b in itertools.combinations(sorted(APPS, key=lambda x: -t[x]), 2):
            p(f"[{nome}] {a} - {b}: {t[a] - t[b]:.2f} ({100 * (t[a] - t[b]) / t[a]:.1f}%); b só passa a frente se a nota de consistência for maior em mais de {(t[a] - t[b]) * peso / PESO_CONSISTENCIA:.2f}")
    open(os.path.join(F4, "resumo.txt"), "w", encoding="utf-8").write("\n".join(out) + "\n")


main()
