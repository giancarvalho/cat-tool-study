"""Casos funcionais T-R2..T-R7 por roteiro com leitura de estado (driver de teclas do sistema + instantâneos Ctrl+Alt+F9).
Uso: python -I harness/funcional.py --so electron,tauri,avalonia,flutter [--casos tr2,tr3,...]
Cada app grava linhas ESTADO no CSV (JSON em percent-encoding na coluna 'erro'); este script confere as afirmações.
Posições de cursor em átomos (0 = início do segmento; tag e caractere valem 1). Resultado em results/funcional/<app>.json."""
import argparse, csv, json, os, subprocess, sys, time, urllib.parse

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DRIVER = os.path.join(RAIZ, "harness", "driver", "target", "release", "catdriver.exe")


def n_tags(e):
    return e.count("<")


def verificar(caso, s):
    def ok(c, m):
        if not c:
            raise AssertionError(m)
    if caso == "tr2":
        ok(len(s) >= 4, f"só {len(s)} instantâneos")
        ok(s[0]["total"] == 10000, f"total {s[0]['total']} != 10000")
        ok(s[1]["active"] == 9999 and s[1]["ativoVisivel"], f"Ctrl+End: ativo {s[1]['active']} visível {s[1]['ativoVisivel']}")
        ok(s[2]["active"] == 0 and s[2]["ativoVisivel"], f"Ctrl+Home: ativo {s[2]['active']} visível {s[2]['ativoVisivel']}")
        ok(s[3]["active"] == 10 and s[3]["ativoVisivel"], f"PgDn: ativo {s[3]['active']}")
    elif caso == "tr3":
        ok(len(s) >= 9, f"só {len(s)} instantâneos")
        n0 = n_tags(s[0]["editor"])
        ok(n0 >= 2 and s[0]["editor"].startswith("<"), f"segmento sem tag no início: {s[0]['editor'][:40]}")
        ok(s[0]["head"] == 0 and s[1]["head"] == 1, f"seta não atravessou a tag como 1 unidade (cabeça {s[0]['head']} -> {s[1]['head']})")
        ok(s[2]["modal"], "Backspace sobre tag não abriu confirmação")
        ok(not s[3]["modal"] and n_tags(s[3]["editor"]) == n0, "Esc deveria cancelar sem apagar")
        ok(n_tags(s[4]["editor"]) == n0 - 2, f"Enter deveria apagar a tag e a pareada (tags {n0} -> {n_tags(s[4]['editor'])})")
        ok(n_tags(s[5]["editor"]) == n0, "Esc não descartou o rascunho")
        ok(s[6]["head"] == 1, f"Ctrl+→ deveria parar depois da tag (cabeça {s[6]['head']})")
        ok(s[7]["head"] > 1, "Ctrl+→ não avançou para o fim da palavra")
        ok(s[8]["head"] == 1, f"Ctrl+← deveria voltar ao limite da tag (cabeça {s[8]['head']})")
    elif caso == "tr4":
        ok(len(s) >= 2, f"só {len(s)} instantâneos")
        ok("qz" in s[1]["editor"], "o editor não recebeu o texto digitado")
        ok(s[1]["previaAtivo"] and "qz" in s[1]["previaAtivo"], "prévia não mostrou o texto digitado")
        ok(s[1]["previaOutros"] == s[0]["previaOutros"], "outros blocos da prévia mudaram")
    elif caso == "tr5":
        ok(len(s) >= 3, f"só {len(s)} instantâneos")
        ok(s[1]["fila"] == s[0]["fila"] + 1, f"fila {s[0]['fila']} -> {s[1]['fila']}")
        ok(s[1]["active"] == s[0]["active"] + 1, "não avançou para o próximo")
        ok(s[2]["active"] == s[0]["active"] and (s[2]["versao"] or 0) >= 1, f"versão do segmento confirmado: {s[2]['versao']}")
        ok("a" in (s[2]["destinoAtivo"] or ""), "destino do segmento confirmado não tem o texto digitado")
    elif caso == "tr6":
        ok(len(s) >= 3, f"só {len(s)} instantâneos")
        n0 = s[0]["contagem"]
        ok(s[1]["contagem"] == n0 - 1 and s[1]["total"] == n0 - 1, f"mesclar: contagem {n0} -> {s[1]['contagem']}, lista {s[1]['total']}")
        ok(s[2]["contagem"] == n0 and s[2]["total"] == n0, f"dividir: contagem {s[2]['contagem']}, lista {s[2]['total']} (esperado {n0})")
    elif caso == "tr7":
        ok(len(s) >= 1, "sem instantâneo")
        r = s[0]["resultados"]
        ok(len(r) > 5 and "sem resultados" not in r, f"sem resultados de concordância: {r[:60]!r}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--so", default="electron,tauri,avalonia,flutter")
    ap.add_argument("--casos", default="tr2,tr3,tr4,tr5,tr6,tr7")
    ap.add_argument("--candidatos", default="harness/candidatos.json")
    a = ap.parse_args()
    cands = json.load(open(os.path.join(RAIZ, a.candidatos), encoding="utf-8"))
    geral = {}
    for nome in a.so.split(","):
        res = {}
        for caso in a.casos.split(","):
            rot = os.path.join(RAIZ, "spec", "roteiros", "func", caso + ".json")
            spec = json.load(open(rot, encoding="utf-8"))
            doc = spec["doc"]
            d = os.path.join(RAIZ, "results", "funcional", nome, caso)
            os.makedirs(d, exist_ok=True)
            app_csv = os.path.join(d, "app.csv")
            if os.path.exists(app_csv):
                os.remove(app_csv)
            cmd = [c.replace("{doc}", os.path.join(RAIZ, "corpus", "out", doc)).replace("{out}", app_csv) for c in cands[nome]["cmd"]]
            cmd += spec.get("args_extra", [])
            cmd[0] = os.path.abspath(os.path.join(RAIZ, cmd[0]))
            dr = [DRIVER, "--roteiro", rot, "--saida", os.path.join(d, "driver.csv"), "--espera-csv", app_csv, "--perfil", "completo", "--"] + cmd
            r = subprocess.run(dr, cwd=RAIZ, capture_output=True, text=True)
            try:
                if r.returncode != 0:
                    raise AssertionError("driver falhou: " + (r.stderr or r.stdout).strip()[:200])
                est = []
                if os.path.exists(app_csv):
                    for row in csv.DictReader(open(app_csv, encoding="utf-8")):
                        if row["metrica"] == "ESTADO":
                            est.append((int(row["passo"]), json.loads(urllib.parse.unquote(row["erro"]))))
                est = [x for _, x in sorted(est, key=lambda t: t[0])]
                verificar(caso, est)
                res[caso] = ("passou", "")
            except Exception as e:
                res[caso] = ("falhou", str(e))
            print(f"{nome:9} {caso} {res[caso][0]:7} {res[caso][1]}", flush=True)
            time.sleep(3)
        geral[nome] = res
        os.makedirs(os.path.join(RAIZ, "results", "funcional"), exist_ok=True)
        json.dump(res, open(os.path.join(RAIZ, "results", "funcional", nome + ".json"), "w", encoding="utf-8"), ensure_ascii=False, indent=1)


main()
