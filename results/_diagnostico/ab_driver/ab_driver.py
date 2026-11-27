"""Teste A/B: driver anterior (HEAD) x driver com guarda de foco, alternados, mesmo app e documento.
Só o trecho de digitação do roteiro de medição; perfil Restrito (iGPU), como nas rodadas comparadas."""
import json, os, statistics as st, subprocess, sys, time, winreg

RAIZ = r"C:\Projects\cat-study"
S = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(RAIZ, "harness"))
from juntar import juntar  # noqa: E402

DRV = {"antigo": os.path.join(S, "drv_antigo", "target", "release", "catdriver.exe"),
       "novo": os.path.join(RAIZ, "harness", "driver", "target", "release", "catdriver.exe")}
cands = json.load(open(os.path.join(RAIZ, "harness", "candidatos_pacote.json"), encoding="utf-8"))
rot = json.load(open(os.path.join(RAIZ, "spec", "roteiros", "medicao.json"), encoding="utf-8"))
rot["passos"] = rot["passos"][:3]
roteiro = os.path.join(S, "ab_roteiro.json")
json.dump(rot, open(roteiro, "w", encoding="utf-8"))
APPS = sys.argv[1].split(",")
ORDEM = ["antigo", "novo", "novo", "antigo", "antigo", "novo"]
doc = os.path.join(RAIZ, "corpus", "real", "real_texto.docx")

k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Microsoft\DirectX\UserGpuPreferences")
antes = {}
for a in APPS:
    for e in cands[a]["exes_gpu"]:
        p = os.path.abspath(os.path.join(RAIZ, e))
        if not os.path.exists(p):
            print("AVISO: exe de GPU não existe:", p)
        try:
            antes[p] = winreg.QueryValueEx(k, p)[0]
        except FileNotFoundError:
            antes[p] = None
        winreg.SetValueEx(k, p, 0, winreg.REG_SZ, "GpuPreference=1;")
res = {}
try:
    for i, drv in enumerate(ORDEM):
        for a in APPS:
            d = os.path.join(S, "ab", a, f"{i}_{drv}")
            os.makedirs(d, exist_ok=True)
            app_csv = os.path.join(d, "app.csv")
            if os.path.exists(app_csv):
                os.remove(app_csv)
            cmd = [c.replace("{doc}", doc).replace("{out}", app_csv) for c in cands[a]["cmd"]]
            cmd[0] = os.path.abspath(os.path.join(RAIZ, cmd[0]))
            cmd += ["--tmx", os.path.join(RAIZ, "docs", "translation memory")]
            r = subprocess.run([DRV[drv], "--roteiro", roteiro, "--saida", os.path.join(d, "driver.csv"), "--espera-csv", app_csv, "--perfil", "restrito", "--"] + cmd,
                               cwd=RAIZ, capture_output=True, text=True, timeout=300)
            if r.returncode:
                print(a, i, drv, "FALHOU", (r.stderr or r.stdout)[-300:], flush=True)
                time.sleep(15)
                continue
            linhas, _ = juntar(os.path.join(d, "driver.csv"), app_csv)
            m = [x for x in linhas if x["metrica"] == "M1"]
            e, t = st.median(x["entrada_ms"] for x in m), st.median(x["ponta_a_ponta_ms"] for x in m)
            res.setdefault((a, drv), []).append(t)
            print(f"{time.strftime('%H:%M:%S')} {a:9s} {drv:6s} n={len(m)} entrada p50 {e:.2f}  M1 p50 {t:.2f}", flush=True)
            time.sleep(10)
finally:
    for p, v in antes.items():
        if v is None:
            winreg.DeleteValue(k, p)
        else:
            winreg.SetValueEx(k, p, 0, winreg.REG_SZ, v)
for chave, v in sorted(res.items()):
    print(chave, "mediana das execuções:", round(st.median(v), 2), [round(x, 1) for x in v])
