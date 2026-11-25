"""Gera os pacotes de produção portáteis e os instaladores NSIS dos quatro candidatos (M10) em dist-pacotes/
e escreve harness/candidatos_pacote.json (linhas de comando usadas pelas rodadas valendo).
Uso: python -I harness/empacotar.py   (precisa de Rust, Node, .NET SDK, Flutter e do NSIS que o `tauri build` baixa)
Mesmo método para os quatro: pasta com o executável e o catcore.exe; instalador pelo mesmo script NSIS (LZMA sólido)."""
import glob, json, os, shutil, subprocess, sys, zipfile

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(RAIZ, "dist-pacotes")
FLUTTER = r"C:\dev\flutter\bin\flutter.bat"
MAKENSIS = os.path.join(os.environ["LOCALAPPDATA"], "tauri", "NSIS", "makensis.exe")
EXE = {"electron": "catstudy-electron.exe", "tauri": "catstudy-tauri.exe", "avalonia": "CatStudyAvalonia.exe", "flutter": "catstudy.exe"}


def run(cmd, cwd=RAIZ, **kw):
    print("$", " ".join(cmd) if isinstance(cmd, list) else cmd, flush=True)
    r = subprocess.run(cmd, cwd=cwd, shell=isinstance(cmd, str), **kw)
    if r.returncode:
        sys.exit(f"falhou ({r.returncode}): {cmd}")


def saida(cmd, cwd=RAIZ):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, shell=isinstance(cmd, str)).stdout.strip()


def tamanho(d):
    return sum(os.path.getsize(os.path.join(a, f)) for a, _, fs in os.walk(d) for f in fs) / 1048576


def main():
    shutil.rmtree(OUT, ignore_errors=True)
    os.makedirs(os.path.join(OUT, "instaladores"))
    core = os.path.join(RAIZ, "core", "target", "release", "catcore.exe")
    run(["cargo", "build", "--release"], cwd=os.path.join(RAIZ, "core"))

    # Electron: distribuição binária + resources/app com o bundle
    e = os.path.join(OUT, "electron")
    run("npm run build", cwd=os.path.join(RAIZ, "apps", "electron"))
    shutil.copytree(os.path.join(RAIZ, "apps", "electron", "node_modules", "electron", "dist"), e, copy_function=shutil.copy)
    os.remove(os.path.join(e, "resources", "default_app.asar"))
    os.makedirs(os.path.join(e, "resources", "app"))
    shutil.copytree(os.path.join(RAIZ, "apps", "electron", "dist"), os.path.join(e, "resources", "app", "dist"), copy_function=shutil.copy)
    shutil.copy(os.path.join(RAIZ, "apps", "electron", "package.json"), os.path.join(e, "resources", "app", "package.json"))
    os.rename(os.path.join(e, "electron.exe"), os.path.join(e, EXE["electron"]))
    shutil.copy(core, e)

    # Tauri: executável único (núcleo em processo)
    t = os.path.join(OUT, "tauri")
    run("npm run build", cwd=os.path.join(RAIZ, "apps", "tauri"))
    os.makedirs(t)
    shutil.copy(os.path.join(RAIZ, "apps", "tauri", "src-tauri", "target", "release", "catstudy-tauri.exe"), t)

    # Avalonia: publish self-contained win-x64, sem trimming
    a = os.path.join(OUT, "avalonia")
    run(["dotnet", "publish", "-c", "Release", "-r", "win-x64", "--self-contained", "true", "-o", a, "-nologo", "-v", "q"], cwd=os.path.join(RAIZ, "apps", "avalonia"))
    shutil.copy(core, a)

    # Flutter: pasta Release
    run([FLUTTER, "build", "windows", "--release"], cwd=os.path.join(RAIZ, "apps", "flutter"))
    f = os.path.join(OUT, "flutter")
    shutil.copytree(os.path.join(RAIZ, "apps", "flutter", "build", "windows", "x64", "runner", "Release"), f, copy_function=shutil.copy)
    shutil.copy(core, f)

    res = {}
    for k in EXE:
        d = os.path.join(OUT, k)
        z = os.path.join(OUT, k + ".zip")
        with zipfile.ZipFile(z, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
            for p, _, fs in os.walk(d):
                for x in fs:
                    zf.write(os.path.join(p, x), os.path.relpath(os.path.join(p, x), d))
        inst = os.path.join(OUT, "instaladores", f"CatStudy-{k}-setup.exe")
        run([MAKENSIS, "-V1", f"-DNOME=CatStudy-{k}", f"-DPASTA={d}", f"-DEXE={EXE[k]}", f"-DSAIDA={inst}", os.path.join(RAIZ, "harness", "instalador", "generico.nsi")])
        res[k] = {"pasta_mb": round(tamanho(d), 1), "zip_mb": round(os.path.getsize(z) / 1048576, 1), "instalador_mb": round(os.path.getsize(inst) / 1048576, 1)}
    json.dump({"commit": saida(["git", "rev-parse", "HEAD"]), "tamanhos": res}, open(os.path.join(OUT, "tamanhos.json"), "w", encoding="utf-8"), indent=2)

    # candidatos das rodadas valendo (a partir de candidatos.json, trocando o executável pelo pacote)
    c = json.load(open(os.path.join(RAIZ, "harness", "candidatos.json"), encoding="utf-8"))
    versoes = {
        "electron": "electron 44.4.5, pacote portátil (dist do Electron + resources/app esbuild)",
        "tauri": "tauri 2, cargo build --release, WebView2 do sistema (" + saida(["reg", "query", r"HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}", "/v", "pv"]).split()[-1] + ")",
        "avalonia": "avalonia 12.2.0, net10.0, dotnet publish win-x64 self-contained, swap chain DXGI (Emenda 6)",
        "flutter": "flutter 3.50.1, build windows --release",
    }
    for k, v in c.items():
        cmd = [x for x in v["cmd"] if x != "apps/electron"] if k == "electron" else list(v["cmd"])
        exe = f"dist-pacotes/{k}/{EXE[k]}"
        cmd[0] = exe
        v["cmd"], v["exes_gpu"], v["versao"] = cmd, [exe], versoes[k]
        if k == "tauri":  # o GPU do Tauri é do processo msedgewebview2.exe, não do executável do app
            wv = sorted(glob.glob(r"C:\Program Files (x86)\Microsoft\EdgeWebView\Application\*\msedgewebview2.exe"))
            v["exes_gpu"] += [w.replace("\\", "/") for w in wv[-1:]]
    json.dump(c, open(os.path.join(RAIZ, "harness", "candidatos_pacote.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=2)
    print(json.dumps(res, indent=1))


main()
