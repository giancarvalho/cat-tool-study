"""Com a sessão parada: alterna animador ligado/desligado, 3 vezes, em Electron e Avalonia."""
import os, subprocess, sys, time
S = os.path.dirname(os.path.abspath(__file__))
time.sleep(90)
for i, cond in enumerate(["animando", "sem_anim"] * 3):
    p = subprocess.Popen([sys.executable, "-I", os.path.join(S, "animador.py")]) if cond == "animando" else None
    time.sleep(5)
    if p and p.poll() is not None:
        print("animador encerrou sozinho, código", p.returncode, flush=True)
    try:
        subprocess.run([sys.executable, "-I", "-X", "utf8", "-u", os.path.join(S, "probe.py"), cond, "electron,avalonia", "10"])
    finally:
        if p:
            p.kill()
