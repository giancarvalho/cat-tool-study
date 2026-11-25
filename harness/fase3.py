"""Fase 3: rodadas de medição valendo, em sequência e com retomada (pula o que já tem rodada.json).
Cobertura aprovada por Gian (2026-11-24): Restrito completo, Completo reduzido.
  - medicao.json (M1, M2, M3, M5, M6, M8, M9, M11, M7 parcial): Restrito em 7 documentos; Completo em 4 (pequeno, pior caso, texto longo real, PPTX real);
  - rolagem.json (M4, PresentMon): mesmos documentos de cada perfil;
  - uso_prolongado.json (M7, 60 min por candidato): só no Restrito.
Sementes registradas: SEMENTE_BASE + índice do item. Candidatos: harness/candidatos_pacote.json (builds de produção).
Uso: python -I harness/fase3.py [--listar] [--so-etapa restrito-medicao,restrito-m7,restrito-m4,completo-medicao,completo-m4] [--reps 10] [--pausa 15]
Registro em results/fase3/ordem.json; log em results/fase3/log.txt."""
import argparse, json, os, shutil, subprocess, sys, time

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SEMENTE_BASE = 20261124
DOCS = {
    "pequeno": "corpus/out/pequeno.docx",
    "medio": "corpus/out/medio.docx",
    "longo": "corpus/out/longo.docx",
    "pior_caso": "corpus/out/pior_caso.docx",
    "real_layout": "corpus/real/real_layout.docx",
    "real_texto": "corpus/real/real_texto.docx",
    "real_pptx": "corpus/real/real_pptx.pptx",
}
RESTRITO = list(DOCS)
COMPLETO = ["pequeno", "pior_caso", "real_texto", "real_pptx"]
CICLO_M7 = ["corpus/out/pequeno.docx", "corpus/out/medio.docx", "corpus/out/longo.docx"]


def plano(reps, pausa):
    itens = []
    k = 0

    def add(etapa, nome, doc, perfil, roteiro, extra=()):
        nonlocal k
        k += 1
        gpu = "integrada" if perfil == "restrito" else "nvidia"
        cmd = [sys.executable, "-I", "harness/rodada.py", "--nome", nome, "--doc", DOCS.get(doc, doc), "--perfil", perfil, "--reps", str(reps),
               "--semente", str(SEMENTE_BASE + k), "--roteiro", roteiro, "--candidatos", "harness/candidatos_pacote.json", "--gpu", gpu, "--pausa", str(pausa), *extra]
        itens.append({"etapa": etapa, "nome": nome, "cmd": cmd, "semente": SEMENTE_BASE + k})

    for d in RESTRITO:
        add("restrito-medicao", f"f3-restrito-medicao-{d}", d, "restrito", "spec/roteiros/medicao.json")
    add("restrito-m7", "f3-restrito-m7", "pequeno", "restrito", "spec/roteiros/uso_prolongado.json",
        ["--docs-ciclo", ";".join(CICLO_M7), "--memoria-cada", "60"])
    itens[-1]["cmd"][itens[-1]["cmd"].index("--reps") + 1] = "1"
    for d in RESTRITO:
        add("restrito-m4", f"f3-restrito-m4-{d}", d, "restrito", "spec/roteiros/rolagem.json", ["--presentmon"])
    for d in COMPLETO:
        add("completo-medicao", f"f3-completo-medicao-{d}", d, "completo", "spec/roteiros/medicao.json")
    for d in COMPLETO:
        add("completo-m4", f"f3-completo-m4-{d}", d, "completo", "spec/roteiros/rolagem.json", ["--presentmon"])
    return itens


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--listar", action="store_true")
    ap.add_argument("--so-etapa", default="")
    ap.add_argument("--reps", type=int, default=10)
    ap.add_argument("--pausa", type=int, default=15)
    a = ap.parse_args()
    itens = plano(a.reps, a.pausa)
    if a.so_etapa:
        itens = [i for i in itens if i["etapa"] in a.so_etapa.split(",")]
    os.makedirs(os.path.join(RAIZ, "results", "fase3"), exist_ok=True)
    json.dump([{k: v for k, v in i.items() if k != "cmd"} | {"cmd": " ".join(i["cmd"][2:])} for i in itens], open(os.path.join(RAIZ, "results", "fase3", "ordem.json"), "w", encoding="utf-8"), indent=1)
    if a.listar:
        for i in itens:
            print(i["etapa"], i["nome"], "semente", i["semente"])
        return
    log = open(os.path.join(RAIZ, "results", "fase3", "log.txt"), "a", encoding="utf-8")
    for i in itens:
        if os.path.exists(os.path.join(RAIZ, "results", i["nome"], "rodada.json")):
            print("já feito:", i["nome"], flush=True)
            continue
        pasta = os.path.join(RAIZ, "results", i["nome"])
        if os.path.exists(pasta):
            shutil.rmtree(pasta)  # rodada interrompida: recomeça do zero (evita linhas duplicadas)
        t0 = time.time()
        msg = f"[{time.strftime('%H:%M:%S')}] início {i['nome']}"
        print(msg, flush=True)
        log.write(msg + "\n")
        log.flush()
        r = subprocess.run(i["cmd"], cwd=RAIZ, capture_output=True, text=True, encoding="utf-8", errors="replace")
        msg = f"[{time.strftime('%H:%M:%S')}] fim {i['nome']} ({(time.time() - t0) / 60:.1f} min) código {r.returncode}"
        print(msg, flush=True)
        log.write(msg + "\n" + (r.stdout[-1500:] if r.returncode else "") + (r.stderr[-800:] if r.returncode else "") + "\n")
        log.flush()
    print("Fase 3 concluída (todas as etapas listadas).", flush=True)


main()
