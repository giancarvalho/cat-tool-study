"""Funções compartilhadas de rodada.py (preferência gráfica por executável)."""
import os
RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DRIVER = os.path.join(RAIZ, "harness", "driver", "target", "release", "catdriver.exe")


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
