"""Sonda curta (50 teclas): M1 p50 e distribuição do tempo no app. Uso: probe.py rótulo app[,app] [segura_ms ...]"""
import ctypes, json, os, statistics as st, subprocess, sys, time, winreg
from ctypes import wintypes
RAIZ = r"C:\Projects\cat-study"
S = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(RAIZ, "harness"))
from juntar import juntar
DRV = os.path.join(S, "drv_hold", "target", "release", "catdriver.exe")
cands = json.load(open(os.path.join(RAIZ, "harness", "candidatos_pacote.json"), encoding="utf-8"))
rot = json.load(open(os.path.join(RAIZ, "spec", "roteiros", "medicao.json"), encoding="utf-8"))
rot["passos"] = rot["passos"][:3]
rot["passos"][2]["vezes"] = 5
roteiro = os.path.join(S, "probe_roteiro.json")
json.dump(rot, open(roteiro, "w", encoding="utf-8"))


def estado():
    u = ctypes.windll.user32
    a, b, c = (wintypes.ULONG() for _ in range(3))
    ctypes.windll.ntdll.NtQueryTimerResolution(ctypes.byref(a), ctypes.byref(b), ctypes.byref(c))
    p = wintypes.POINT(); u.GetCursorPos(ctypes.byref(p))
    class DM(ctypes.Structure):
        _fields_ = [("n", ctypes.c_wchar * 32), ("v", wintypes.WORD * 4), ("f", wintypes.DWORD), ("u", ctypes.c_byte * 16), ("w", ctypes.c_short * 5), ("fn", ctypes.c_wchar * 32), ("lp", wintypes.WORD), ("bpp", wintypes.DWORD), ("cx", wintypes.DWORD), ("cy", wintypes.DWORD), ("df", wintypes.DWORD), ("hz", wintypes.DWORD), ("resto", wintypes.DWORD * 8)]
    dm = DM(); ctypes.c_ushort.from_buffer(dm, 68).value = ctypes.sizeof(dm)
    u.EnumDisplaySettingsW(None, -1, ctypes.byref(dm))
    class LI(ctypes.Structure):
        _fields_ = [("s", wintypes.UINT), ("t", wintypes.DWORD)]
    li = LI(8, 0); u.GetLastInputInfo(ctypes.byref(li))
    ocioso = (ctypes.windll.kernel32.GetTickCount() - li.t) / 1000
    return f"timer {c.value/1e4:.2f}ms tela {dm.hz}Hz cursor ({p.x},{p.y}) ocioso {ocioso:.0f}s"


if __name__ == "__main__":
    rotulo, apps = sys.argv[1], sys.argv[2].split(",")
    seguras = [int(x) for x in sys.argv[3:]] or [10]
    k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Microsoft\DirectX\UserGpuPreferences")
    criadas = []
    for a in apps:
        for e in cands[a]["exes_gpu"]:
            p = os.path.abspath(os.path.join(RAIZ, e))
            winreg.SetValueEx(k, p, 0, winreg.REG_SZ, "GpuPreference=1;"); criadas.append(p)
    try:
        for i, h in enumerate(seguras):
            for a in apps:
                d = os.path.join(S, "probe", f"{rotulo}_{i}_{a}_{h}_{int(time.time())}"); os.makedirs(d)
                app_csv = os.path.join(d, "app.csv")
                cmd = [c.replace("{doc}", os.path.join(RAIZ, "corpus", "real", "real_texto.docx")).replace("{out}", app_csv) for c in cands[a]["cmd"]]
                cmd[0] = os.path.abspath(os.path.join(RAIZ, cmd[0]))
                antes = estado()
                r = subprocess.run([DRV, "--roteiro", roteiro, "--saida", os.path.join(d, "driver.csv"), "--espera-csv", app_csv, "--perfil", "restrito", "--"] + cmd,
                                   cwd=RAIZ, capture_output=True, text=True, timeout=200, env=dict(os.environ, CAT_HOLD_MS=str(h)))
                if r.returncode:
                    print(rotulo, a, h, "FALHOU", (r.stderr or r.stdout)[-300:], flush=True); continue
                m = [x for x in juntar(os.path.join(d, "driver.csv"), app_csv)[0] if x["metrica"] == "M1"]
                v = sorted(float(x["app_ms"]) for x in m); q = lambda f: v[int(f * (len(v) - 1))]
                print(f"{time.strftime('%H:%M:%S')} {rotulo:10s} {a:8s} solta {h:2d}ms n={len(m)} M1 p50 {st.median(x['ponta_a_ponta_ms'] for x in m):5.1f} | app p5 {q(.05):5.1f} p50 {q(.5):5.1f} p95 {q(.95):5.1f} | antes: {antes}", flush=True)
                time.sleep(3)
    finally:
        for p in criadas:
            try: winreg.DeleteValue(k, p)
            except FileNotFoundError: pass
