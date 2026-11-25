"""Orquestrador de rodadas (Emenda 3, item 3).
Para cada repetição, sorteia a ordem dos candidatos (semente registrada), roda o driver, junta os CSVs e acumula
results/<nome>/medicoes.csv. A repetição 1 é marcada como aquecimento (declarada, descartada na análise).
Uso: python -I harness/rodada.py --nome r1 --doc corpus/out/pequeno.docx --perfil completo|restrito --reps 10 --semente 20261124
        [--roteiro spec/roteiros/curto.json] [--candidatos harness/candidatos.json] [--so electron,tauri] [--pausa 15]
        [--gpu integrada|nvidia] [--memoria-cada 60]"""
import argparse, csv, json, os, random, subprocess, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from juntar import juntar  # noqa: E402
import ambiente  # noqa: E402

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PRESENTMON = os.path.join(RAIZ, "harness", "tools", "PresentMon-2.6.0-x64.exe")
DRIVER = os.path.join(RAIZ, "harness", "driver", "target", "release", "catdriver.exe")
COLS = ["rodada", "app", "perfil", "documento", "repeticao", "aquecimento", "metrica", "passo", "valor_ms", "entrada_ms"]


def git(*a):
    return subprocess.run(["git", *a], capture_output=True, text=True, cwd=RAIZ).stdout.strip()


def gpu_pref(exes, modo):
    """Preferência gráfica por executável do Windows (HKCU, restaurada ao final). 1 = economia (iGPU), 2 = alto desempenho."""
    import winreg
    chave = r"Software\Microsoft\DirectX\UserGpuPreferences"
    k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, chave)
    antes = {}
    for e in exes:
        p = os.path.abspath(os.path.join(RAIZ, e))
        try:
            antes[p] = winreg.QueryValueEx(k, p)[0]
        except FileNotFoundError:
            antes[p] = None
        winreg.SetValueEx(k, p, 0, winreg.REG_SZ, f"GpuPreference={1 if modo == 'integrada' else 2};")
    return k, antes


def gpu_restaurar(k, antes):
    import winreg
    for p, v in antes.items():
        if v is None:
            try:
                winreg.DeleteValue(k, p)
            except FileNotFoundError:
                pass
        else:
            winreg.SetValueEx(k, p, 0, winreg.REG_SZ, v)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--nome", required=True)
    ap.add_argument("--doc", required=True)
    ap.add_argument("--perfil", choices=["completo", "restrito"], required=True)
    ap.add_argument("--reps", type=int, default=10)
    ap.add_argument("--semente", type=int, required=True)
    ap.add_argument("--roteiro", default="spec/roteiros/curto.json")
    ap.add_argument("--candidatos", default="harness/candidatos.json")
    ap.add_argument("--so", default="")
    ap.add_argument("--pausa", type=int, default=15)
    ap.add_argument("--gpu", choices=["integrada", "nvidia", ""], default="")
    ap.add_argument("--memoria-cada", default="")
    ap.add_argument("--tmx", default="docs/translation memory", help="pasta dos TMX para o M8 (TM real); vazio = sem TM")
    ap.add_argument("--docs-ciclo", default="", help="lista a;b;c de documentos para Ctrl+Shift+O (uso prolongado, M7)")
    ap.add_argument("--presentmon", action="store_true", help="captura o PresentMon (M4); exige privilégio elevado ou o grupo Performance Log Users")
    a = ap.parse_args()

    cands = json.load(open(os.path.join(RAIZ, a.candidatos), encoding="utf-8"))
    if a.so:
        cands = {k: v for k, v in cands.items() if k in a.so.split(",")}
    out = os.path.join(RAIZ, "results", a.nome)
    os.makedirs(out, exist_ok=True)
    ambiente.json.dump(ambiente.coletar(a.perfil, {k: v["versao"] for k, v in cands.items()}), open(os.path.join(out, "ambiente.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=2)
    rng = random.Random(a.semente)
    ordens = []
    meta = {"semente": a.semente, "commit": git("rev-parse", "HEAD"), "arvore_suja": bool(git("status", "--porcelain")), "doc": a.doc, "perfil": a.perfil, "reps": a.reps, "roteiro": a.roteiro, "gpu": a.gpu, "tmx": a.tmx, "docs_ciclo": a.docs_ciclo, "ordens": ordens}
    gk = None
    if a.gpu:
        exes = [e for c in cands.values() for e in c.get("exes_gpu", [])]
        gk = gpu_pref(exes, a.gpu)
    novo = not os.path.exists(os.path.join(out, "medicoes.csv"))
    f = open(os.path.join(out, "medicoes.csv"), "a", newline="", encoding="utf-8")
    w = csv.DictWriter(f, fieldnames=COLS)
    if novo:
        w.writeheader()
    try:
        for rep in range(1, a.reps + 1):
            ordem = list(cands)
            rng.shuffle(ordem)
            ordens.append(ordem)
            for nome in ordem:
                d = os.path.join(out, nome, f"rep{rep:02d}")
                os.makedirs(d, exist_ok=True)
                app_csv = os.path.join(d, "app.csv")
                if os.path.exists(app_csv):
                    os.remove(app_csv)
                cmd = [c.replace("{doc}", os.path.abspath(os.path.join(RAIZ, a.doc))).replace("{out}", app_csv) for c in cands[nome]["cmd"]]
                cmd[0] = os.path.abspath(os.path.join(RAIZ, cmd[0]))
                if a.tmx:
                    cmd += ["--tmx", os.path.abspath(os.path.join(RAIZ, a.tmx))]
                if a.docs_ciclo:
                    cmd += ["--docs", ";".join(os.path.abspath(os.path.join(RAIZ, x)) for x in a.docs_ciclo.split(";"))]
                dr = [DRIVER, "--roteiro", os.path.join(RAIZ, a.roteiro), "--saida", os.path.join(d, "driver.csv"), "--espera-csv", app_csv, "--perfil", a.perfil]
                if a.memoria_cada:
                    dr += ["--memoria-cada", a.memoria_cada]
                print(f"[rep {rep}/{a.reps}] {nome} ...", flush=True)
                pm = None
                if a.presentmon:
                    pm = subprocess.Popen([PRESENTMON, "--output_file", os.path.join(d, "presentmon.csv"), "--no_console_stats", "--date_time", "--stop_existing_session", "--session_name", "catstudy"], cwd=RAIZ, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                    time.sleep(2)
                    if pm.poll() is not None:
                        print("  PresentMon não iniciou:", pm.stdout.read().strip()[:300])
                        pm = None
                r = subprocess.run(dr + ["--"] + cmd, cwd=RAIZ, capture_output=True, text=True)
                if pm is not None:
                    pm.terminate()
                    try:
                        pm.wait(10)
                    except Exception:
                        pm.kill()
                if r.returncode != 0:
                    print("  FALHOU:", (r.stderr or r.stdout).strip()[:300])
                    continue
                linhas, sem = juntar(os.path.join(d, "driver.csv"), app_csv)
                for x in linhas:
                    w.writerow({"rodada": a.nome, "app": nome, "perfil": a.perfil, "documento": os.path.basename(a.doc), "repeticao": rep, "aquecimento": int(rep == 1), "metrica": x["metrica"], "passo": x["passo"], "valor_ms": x["ponta_a_ponta_ms"], "entrada_ms": x["entrada_ms"]})
                f.flush()
                if sem:
                    print("  teclas sem resposta:", {f"{k[0]}/{k[1]}": v for k, v in sem.items()})
                time.sleep(a.pausa)
    finally:
        f.close()
        if gk:
            gpu_restaurar(*gk)
        json.dump(meta, open(os.path.join(out, "rodada.json"), "w", encoding="utf-8"), indent=2)
    print("pronto:", out)


main()
