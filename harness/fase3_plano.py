"""Itens da Fase 3 como (rodada, etapa, perfil, documento), lidos de results/fase3/ordem.json (gerado por fase3.py)."""
import json, os

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def itens():
    for i in json.load(open(os.path.join(RAIZ, "results", "fase3", "ordem.json"), encoding="utf-8")):
        a = i["cmd"].split(" --")
        perfil = next(x.split(" ", 1)[1] for x in a if x.startswith("perfil "))
        doc = next(x.split(" ", 1)[1] for x in a if x.startswith("doc "))
        yield i["nome"], i["etapa"], perfil, os.path.splitext(os.path.basename(doc))[0]
