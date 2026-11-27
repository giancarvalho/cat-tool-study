"""Isola a origem do crescimento de memória do Flutter (M7): um tipo de operação por execução, ~2,5 min cada.
Uso: probe_mem.py [cenário ...] [--exe caminho] [--sufixo x]"""
import csv, json, os, statistics as st, subprocess, sys, threading, time
RAIZ = r"C:\Projects\cat-study"
S = os.path.dirname(os.path.abspath(__file__))
DRV = os.path.join(RAIZ, "harness", "driver", "target", "release", "catdriver.exe")
OUT = os.path.join(S, "mem")
DUR = 150
t = lambda c, a=150: {"tipo": "tecla", "combo": c, "atraso_ms": a}
rep = lambda n, p: {"tipo": "repetir", "vezes": n, "passos": p}
pre = [t("ctrl+home", 300), t("alt+down", 300)]
# (passos antes, corpo, segundos estimados por volta, operações contadas por volta)
CEN = {
    "ocioso": (pre, [{"tipo": "esperar", "ms": 5000}], 5.0, 0),
    "navegar": (pre, [t("alt+down", 120), t("alt+up", 120)], 0.29, 2),
    "digitar": (pre, [{"tipo": "digitar", "texto": "abcdefghij", "atraso_ms": 60}, rep(10, [t("backspace", 60)])], 1.65, 20),
    "confirmar": (pre, [t("ctrl+shift+enter", 200)], 0.22, 1),
    "mesclar_dividir": (pre, [{"tipo": "mesclar"}, {"tipo": "dividir", "posicao": 10}], 1.5, 2),
    "trocar_doc": (pre, [t("ctrl+shift+o", 2500)], 2.52, 1),
}


def amostrar(parar, linhas, t0):
    while not parar.wait(10):
        r = subprocess.run(["powershell", "-NoProfile", "-Command", "Get-Process catstudy,catcore -ErrorAction SilentlyContinue | ForEach-Object { $_.Name + ',' + $_.PrivateMemorySize64 + ',' + $_.WorkingSet64 + ',' + $_.HandleCount + ',' + $_.Threads.Count }"],
                           capture_output=True, text=True, creationflags=0x08000000)
        for l in r.stdout.split():
            n, p, w, h, th = l.split(",")
            linhas.append((round(time.time() - t0, 1), n, int(p) / 2**20, int(w) / 2**20, int(h), int(th)))


def inclinacao(pts):
    x = [a for a, _ in pts]; y = [b for _, b in pts]
    mx, my = st.fmean(x), st.fmean(y)
    return sum((a - mx) * (b - my) for a, b in pts) / sum((a - mx) ** 2 for a in x)


def main():
    a = sys.argv[1:]
    exe = os.path.join(RAIZ, "dist-pacotes", "flutter", "catstudy.exe")
    suf = ""
    if "--exe" in a:
        i = a.index("--exe"); exe = a[i + 1]; del a[i:i + 2]
    if "--sufixo" in a:
        i = a.index("--sufixo"); suf = a[i + 1]; del a[i:i + 2]
    docs = [os.path.join(RAIZ, "corpus", "out", n + ".docx") for n in ("pequeno", "medio", "longo")]
    for nome in a or list(CEN):
        antes, corpo, seg, ops = CEN[nome]
        voltas = int(DUR / seg)
        d = os.path.join(OUT, nome + suf); os.makedirs(d, exist_ok=True)
        app_csv = os.path.join(d, "app.csv")
        if os.path.exists(app_csv):
            os.remove(app_csv)
        rot = os.path.join(d, "roteiro.json")
        json.dump({"nome": nome, "passos": [{"tipo": "abrir"}, {"tipo": "esperar", "ms": 3000}, *antes, rep(voltas, corpo), {"tipo": "esperar", "ms": 2000}]}, open(rot, "w", encoding="utf-8"))
        linhas, parar, t0 = [], threading.Event(), time.time()
        th = threading.Thread(target=amostrar, args=(parar, linhas, t0)); th.start()
        try:
            r = subprocess.run([DRV, "--roteiro", rot, "--saida", os.path.join(d, "driver.csv"), "--espera-csv", app_csv, "--perfil", "restrito", "--",
                                exe, "--limpo", "--doc", docs[0], "--out", app_csv, "--docs", ";".join(docs)], cwd=RAIZ, capture_output=True, text=True, timeout=900)
        finally:
            parar.set(); th.join()
        with open(os.path.join(d, "mem.csv"), "w", newline="") as f:
            w = csv.writer(f); w.writerow(["t_s", "proc", "privada_mb", "ws_mb", "handles", "threads"]); w.writerows(linhas)
        if r.returncode:
            print(nome, "FALHOU", (r.stderr or r.stdout)[-300:], flush=True); continue
        dur = time.time() - t0
        saida = f"{nome + suf:22s} voltas {voltas} ops {voltas * ops:5d} dur {dur:5.0f}s"
        for proc in ("catstudy", "catcore"):
            p = [(x[0] / 60, x[2]) for x in linhas if x[1] == proc and x[0] > 25]
            if len(p) > 3:
                saida += f" | {proc}: {p[0][1]:.0f}->{p[-1][1]:.0f} MB, {inclinacao(p):+.2f} MB/min"
                if ops and proc == "catstudy":
                    saida += f", {inclinacao(p) * 1024 / (voltas * ops / (dur / 60)):+.1f} KB/op"
        h = [x for x in linhas if x[1] == "catstudy"]
        if h:
            saida += f" | handles {h[1 if len(h) > 1 else 0][4]}->{h[-1][4]}"
        print(saida, flush=True)
        time.sleep(5)


main()
