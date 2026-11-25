"""Verifica qual GPU o candidato usa sob a preferência gráfica do Windows (Restrito = Intel integrada; Completo = NVIDIA).
Roda o app (pacote) sob o driver com o roteiro de rolagem e amostra a tabela de processos do `nvidia-smi` a cada segundo; se algum PID da árvore
do app aparece como cliente da NVIDIA, a execução usou a NVIDIA. Uso: python -I harness/gpu_verificar.py [--so a,b] [--candidatos harness/candidatos_pacote.json]
Resultado: results/gpu-verificacao.json"""
import argparse, csv, json, os, re, subprocess, sys, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rodada_util as ru  # noqa: E402

RAIZ = ru.RAIZ


def pmon_pids(parar, achados):
    while not parar.is_set():
        # `pmon` não é suportado em WDDM; a tabela de processos do nvidia-smi lista os clientes gráficos (C+G, G)
        r = subprocess.run(["nvidia-smi"], capture_output=True, text=True)
        for m in re.finditer(r"\|\s+\d+\s+N/A\s+N/A\s+(\d+)\s+(?:C\+G|G|C)\s", r.stdout):
            achados.add(int(m.group(1)))
        time.sleep(0.3)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--so", default="electron,tauri,avalonia,flutter")
    ap.add_argument("--candidatos", default="harness/candidatos_pacote.json")
    a = ap.parse_args()
    cands = json.load(open(os.path.join(RAIZ, a.candidatos), encoding="utf-8"))
    res = {}
    for nome in a.so.split(","):
        c = cands[nome]
        for modo in ("integrada", "nvidia"):
            gk = ru.gpu_pref(c.get("exes_gpu", []), modo)
            d = os.path.join(RAIZ, "results", "gpu-verificacao", nome, modo)
            os.makedirs(d, exist_ok=True)
            app_csv = os.path.join(d, "app.csv")
            if os.path.exists(app_csv):
                os.remove(app_csv)
            cmd = [x.replace("{doc}", os.path.join(RAIZ, "corpus", "out", "medio.docx")).replace("{out}", app_csv) for x in c["cmd"]]
            cmd[0] = os.path.abspath(os.path.join(RAIZ, cmd[0]))
            dr = [ru.DRIVER, "--roteiro", os.path.join(RAIZ, "spec", "roteiros", "rolagem.json"), "--saida", os.path.join(d, "driver.csv"), "--espera-csv", app_csv, "--perfil", "completo", "--"] + cmd
            parar, achados = threading.Event(), set()
            th = threading.Thread(target=pmon_pids, args=(parar, achados))
            th.start()
            subprocess.run(dr, cwd=RAIZ, capture_output=True, text=True)
            parar.set()
            th.join()
            ru.gpu_restaurar(*gk)
            pids = set()
            for r in csv.DictReader(open(os.path.join(d, "driver.csv"), encoding="utf-8")):
                if r["passo"] == "pids":
                    pids |= {int(x) for x in r["tecla"].split(";") if x}
            usou = sorted(pids & achados)
            res.setdefault(nome, {})[modo] = {"pids_do_app": sorted(pids), "pids_na_nvidia": usou, "usou_nvidia": bool(usou)}
            print(f"{nome:9} preferencia={modo:9} usou_nvidia={bool(usou)} pids={usou}", flush=True)
            time.sleep(3)
    json.dump(res, open(os.path.join(RAIZ, "results", "gpu-verificacao.json"), "w", encoding="utf-8"), indent=1)


main()
