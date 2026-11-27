"""Emenda 10: pacote do Flutter com o Fleather 1.28.0 corrigido (harness/patches), só para a sensibilidade do M7.
Copia apps/flutter e o Fleather do cache do pub para uma pasta temporária, aplica o patch, compila e grava
dist-pacotes/flutter-corrigido e harness/candidatos_corrigido.json. Não altera o app do estudo.
Uso: python -I harness/flutter_corrigido.py"""
import json, os, shutil, subprocess, sys, tempfile

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FLUTTER = r"C:\dev\flutter\bin\flutter.bat"
FLEATHER = os.path.join(os.environ["LOCALAPPDATA"], "Pub", "Cache", "hosted", "pub.dev", "fleather-1.28.0")
PATCH = os.path.join(RAIZ, "harness", "patches", "fleather-1.28.0.patch")


def run(cmd, cwd):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, errors="replace")
    if r.returncode:
        sys.exit(f"falhou: {cmd}\n{r.stdout[-1500:]}\n{r.stderr[-1500:]}")


def main():
    t = tempfile.mkdtemp(prefix="fl_corrigido_")
    app, lib = os.path.join(t, "app"), os.path.join(t, "fleather")
    shutil.copytree(os.path.join(RAIZ, "apps", "flutter"), app, ignore=shutil.ignore_patterns("build", ".dart_tool", "ephemeral"))
    shutil.copytree(FLEATHER, lib)
    run(["git", "apply", "-p3", "--ignore-whitespace", PATCH], lib)
    ed = open(os.path.join(lib, "lib", "src", "widgets", "editor.dart"), encoding="utf-8").read()
    assert "removeObserver(this);\n    super.dispose();" in ed and "addPersistentFrameCallback" not in ed and "oldWidget.clipboardStatus.dispose()" in ed, "patch não aplicado"
    with open(os.path.join(app, "pubspec.yaml"), "a", encoding="utf-8") as f:
        f.write("\ndependency_overrides:\n  fleather:\n    path: ../fleather\n")
    run([FLUTTER, "build", "windows", "--release"], app)
    cfg = json.load(open(os.path.join(app, ".dart_tool", "package_config.json"), encoding="utf-8"))
    usada = next(p["rootUri"] for p in cfg["packages"] if p["name"] == "fleather")
    assert "pub.dev" not in usada, "build usou o Fleather do cache: " + usada
    dst = os.path.join(RAIZ, "dist-pacotes", "flutter-corrigido")
    shutil.rmtree(dst, ignore_errors=True)
    shutil.copytree(os.path.join(app, "build", "windows", "x64", "runner", "Release"), dst)
    shutil.copy(os.path.join(RAIZ, "dist-pacotes", "flutter", "catcore.exe"), dst)
    c = json.load(open(os.path.join(RAIZ, "harness", "candidatos_pacote.json"), encoding="utf-8"))["flutter"]
    exe = "dist-pacotes/flutter-corrigido/catstudy.exe"
    c["cmd"][0], c["exes_gpu"], c["versao"] = exe, [exe], c["versao"] + ", fleather 1.28.0 com harness/patches/fleather-1.28.0.patch (Emenda 10)"
    json.dump({"flutter": c}, open(os.path.join(RAIZ, "harness", "candidatos_corrigido.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=2)
    print("pronto:", dst, "| fleather usado:", usada)


main()
