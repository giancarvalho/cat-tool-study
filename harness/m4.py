"""M4: quadros perdidos na rolagem, a partir do CSV do PresentMon e do driver.csv (roteiro rolagem.json).
Uso: python -I harness/m4.py <presentmon.csv> <driver.csv> [--hz 60]
Janela: de roda_inicio a roda_fim do driver (relógio de parede). Processos: a árvore de PIDs gravada pelo driver.
Definições (registradas em spec/quadro.md):
  perdidos_pct   = 1 - quadros exibidos / (duração * Hz), limitado a 0   (a rolagem contínua exige um quadro por vsync)
  longos_pct     = intervalos entre apresentações > 1,5 vsync / total de intervalos
  descartados    = quadros apresentados e não exibidos (Dropped=1 / FrameType)
Os nomes de coluna variam entre versões do PresentMon; o script aceita os da v1 e da v2 e avisa o que faltou."""
import csv, datetime, sys


def col(h, *nomes):
    m = {x.lower(): x for x in h}
    for n in nomes:
        if n.lower() in m:
            return m[n.lower()]
    return None


def epoch_us(s):
    s = s.strip()
    if "." in s:
        a, b = s.rsplit(".", 1)
        frac = (b + "000000")[:6]
        d = datetime.datetime.strptime(a, "%Y-%m-%d %H:%M:%S")
        d = d.replace(microsecond=int(frac))
    else:
        d = datetime.datetime.strptime(s, "%Y-%m-%d %H:%M:%S")
    return int(d.timestamp() * 1_000_000)  # fuso local, como o PresentMon grava


def analisar(pm_csv, drv_csv, hz=60.0):
    drv = list(csv.DictReader(open(drv_csv, encoding="utf-8")))
    ini = [int(r["t_envio_us"]) for r in drv if r["passo"] == "roda_inicio"]
    fim = [int(r["t_envio_us"]) for r in drv if r["passo"] == "roda_fim"]
    pids = {int(x) for r in drv if r["passo"] == "pids" for x in r["tecla"].split(";") if x}
    rd = csv.DictReader(open(pm_csv, encoding="utf-8"))
    h = rd.fieldnames or []
    c_pid = col(h, "ProcessID")
    c_t = col(h, "CPUStartDateTime", "PresentStartDateTime")
    c_ms = col(h, "MsBetweenPresents", "msBetweenPresents")
    c_drop = col(h, "Dropped")
    c_ft = col(h, "FrameType")
    falta = [n for n, c in (("ProcessID", c_pid), ("tempo (CPUStartDateTime)", c_t), ("MsBetweenPresents", c_ms)) if c is None]
    if falta:
        raise SystemExit(f"colunas ausentes no CSV do PresentMon: {falta}; colunas: {h}")
    saida = []
    for k, (a, b) in enumerate(zip(ini, fim)):
        linhas = []
        for r in rd if k == 0 else csv.DictReader(open(pm_csv, encoding="utf-8")):
            if int(r[c_pid]) in pids:
                t = epoch_us(r[c_t])
                if a <= t <= b:
                    linhas.append(r)
        dur = (b - a) / 1e6
        exibidos = [r for r in linhas if not (c_drop and r[c_drop] == "1") and not (c_ft and r[c_ft] in ("Dropped",))]
        descart = len(linhas) - len(exibidos)
        esperados = dur * hz
        ms = [float(r[c_ms]) for r in linhas if r[c_ms] not in ("", "NA")]
        longos = sum(1 for x in ms if x > 1.5 * 1000 / hz)
        saida.append({"trecho": k, "duracao_s": round(dur, 3), "apresentados": len(linhas), "exibidos": len(exibidos), "descartados": descart,
                      "perdidos_pct": round(max(0.0, 1 - len(exibidos) / esperados) * 100, 2) if esperados else None,
                      "longos_pct": round(100 * longos / len(ms), 2) if ms else None})
    return saida


if __name__ == "__main__":
    hz = float(sys.argv[sys.argv.index("--hz") + 1]) if "--hz" in sys.argv else 60.0
    for r in analisar(sys.argv[1], sys.argv[2], hz):
        print(r)
