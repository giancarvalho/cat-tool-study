"""Compila variantes do app com o Fleather corrigido em cópia, uma correção por vez."""
import os, shutil, subprocess, sys
W = os.path.dirname(os.path.abspath(__file__))
ED = os.path.join(W, "fleather", "lib", "src", "widgets", "editor.dart")
ORIG = open(os.path.join(W, "editor.dart.orig"), encoding="utf-8", newline="").read().replace("\r\n", "\n")
FIX = {
    # A: observador registrado ao ganhar foco não é removido se o editor é descartado com foco
    "A": [("    clipboardStatus.dispose();\n    super.dispose();", "    clipboardStatus.dispose();\n    WidgetsBinding.instance.removeObserver(this);\n    super.dispose();")],
    # B: addPersistentFrameCallback (não pode ser removido) a cada mudança de valor
    "B": [("    SchedulerBinding.instance.addPersistentFrameCallback((timeStamp) {\n      _updateOrDisposeSelectionOverlayIfNeeded();\n    });",
           "    SchedulerBinding.instance.addPostFrameCallback((timeStamp) {\n      if (mounted) _updateOrDisposeSelectionOverlayIfNeeded();\n    });")],
    # C: o widget cria um ClipboardStatusNotifier novo a cada reconstrução; o ouvinte fica no antigo
    "C": [("    performSpellCheck(widget.controller.plainTextEditingValue.text);\n\n    _cursorController.showCursor.value",
           "    performSpellCheck(widget.controller.plainTextEditingValue.text);\n\n    if (widget.clipboardStatus != oldWidget.clipboardStatus) {\n      oldWidget.clipboardStatus.removeListener(_onChangedClipboardStatus);\n      oldWidget.clipboardStatus.dispose();\n      widget.clipboardStatus.addListener(_onChangedClipboardStatus);\n    }\n\n    _cursorController.showCursor.value")],
}
VAR = {"base": "", "A": "A", "B": "B", "C": "C", "ABC": "ABC"}
try:
    for nome, letras in VAR.items():
        s = ORIG
        for l in letras:
            for old, new in FIX[l]:
                assert s.count(old) == 1, (l, old)
                s = s.replace(old, new)
        open(ED, "w", encoding="utf-8", newline="").write(s)
        r = subprocess.run([r"C:\dev\flutter\bin\flutter.bat", "build", "windows", "--release"], cwd=os.path.join(W, "app"), capture_output=True, text=True, errors="replace")
        if r.returncode:
            print(nome, "FALHOU", r.stdout[-1500:]); sys.exit(1)
        dst = os.path.join(W, "exe", nome)
        shutil.rmtree(dst, ignore_errors=True)
        shutil.copytree(os.path.join(W, "app", "build", "windows", "x64", "runner", "Release"), dst)
        shutil.copy(r"C:\Projects\cat-study\dist-pacotes\flutter\catcore.exe", dst)
        import hashlib
        print(nome, "ok", hashlib.sha1(open(os.path.join(dst, "data", "app.so"), "rb").read()).hexdigest()[:12], flush=True)
finally:
    open(ED, "w", encoding="utf-8", newline="").write(ORIG)
