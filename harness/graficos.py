"""Gráficos do artigo só Windows (SVG, sem dependências). Uso: python -I -X utf8 harness/graficos.py
Lê results/fase3/{latencias,memoria,m7}.csv e dist-pacotes/tamanhos.json; grava artigo/graficos/*.svg, em português e em inglês (*.en.svg)."""
import csv, json, os

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
F3, SAIDA = os.path.join(RAIZ, "results", "fase3"), os.path.join(RAIZ, "artigo", "graficos")
APPS = ["electron", "tauri", "avalonia", "flutter"]
NOMES = dict(zip(APPS, ["Electron", "Tauri", "Avalonia", "Flutter"]))
FUNDO, TINTA, TINTA2, GRADE, AZUL = "#fcfcfb", "#0b0b0b", "#52514e", "#e1e0d9", "#2a78d6"
LARGURA, ESQ, DIR, LINHA = 720, 96, 120, 28


def ler(nome):
    return list(csv.DictReader(open(os.path.join(F3, nome), encoding="utf-8")))


EN = {"meta": "target",
      "Tempos no perfil restrito, p95": "Timings in the restricted profile, p95",
      "Tecla até editor (ms)": "Key to editor (ms)", "Confirmar segmento (ms)": "Confirm segment (ms)",
      "Busca por concordância (ms)": "Concordance search (ms)", "Prévia do documento compilado (s)": "Compiled document preview (s)",
      "Memória e tamanho do instalador": "Memory and installer size",
      "Memória com 3 600 segmentos abertos (MiB)": "Memory with 3,600 segments open (MiB)",
      "Memória em uso prolongado, minutos 20–60 (MiB)": "Memory in prolonged use, minutes 20–60 (MiB)",
      "Crescimento da memória (MiB/min)": "Memory growth (MiB/min)", "Instalador (MiB)": "Installer (MiB)"}
ARQUIVOS_EN = {"tempos.svg": "timings.en.svg", "memoria-instalador.svg": "memory-installer.en.svg"}
INGLES = False


def tr(s):
    return EN[s] if INGLES else s


def br(v, casas):
    v += 1e-9 if casas == 2 else 0  # 7,845 vira 7,85, como nas tabelas
    s = f"{v:,.{casas}f}"
    return s if INGLES else s.replace(",", " ").replace(".", ",")


def texto(x, y, s, cor=TINTA2, tam=13, ancora="start", peso=400):
    s = s.replace("&", "&amp;").replace("<", "&lt;")
    return f'<text x="{x:.1f}" y="{y:.1f}" fill="{cor}" font-size="{tam}" text-anchor="{ancora}" font-weight="{peso}">{s}</text>'


def painel(y, titulo, valores, casas, meta=None):
    """valores: {app: número} (barra) ou {app: (mín, máx)} (faixa). Eixo sempre a partir de zero."""
    faixa = isinstance(valores[APPS[0]], tuple)
    topo = max([(v[1] if faixa else v) for v in valores.values()] + ([meta] if meta else []))
    px = lambda v: ESQ + v / (topo * 1.04) * (LARGURA - ESQ - DIR)
    o = [texto(16, y + 16, tr(titulo), TINTA, 14, peso=600)]
    y0 = y + 28
    o.append(f'<line x1="{ESQ}" y1="{y0}" x2="{ESQ}" y2="{y0 + LINHA * 4}" stroke="{GRADE}" stroke-width="1"/>')
    if meta:
        o.append(f'<line x1="{px(meta):.1f}" y1="{y0}" x2="{px(meta):.1f}" y2="{y0 + LINHA * 4}" stroke="{TINTA2}" stroke-width="1" stroke-dasharray="4 3"/>')
        o.append(texto(px(meta), y0 + LINHA * 4 + 14, f"{tr('meta')} {br(meta, 0)}", tam=11, ancora="middle"))
    for i, app in enumerate(APPS):
        cy, v = y0 + LINHA * i + LINHA / 2, valores[app]
        o.append(texto(ESQ - 10, cy + 4, NOMES[app], ancora="end"))
        if faixa:
            a, b = v
            o.append(f'<line x1="{px(a):.1f}" y1="{cy}" x2="{px(b):.1f}" y2="{cy}" stroke="{AZUL}" stroke-width="6" stroke-linecap="round"/>')
            o += [f'<circle cx="{px(x):.1f}" cy="{cy}" r="5" fill="{AZUL}" stroke="{FUNDO}" stroke-width="2"/>' for x in (a, b)]
            rotulo, fim = f"{br(a, casas)}–{br(b, casas)}", px(b)
        else:
            w = max(px(v) - ESQ, 2)
            r = min(4, w / 2)
            o.append(f'<path d="M{ESQ} {cy - 8}h{w - r:.1f}a{r} {r} 0 0 1 {r} {r}v{16 - 2 * r}a{r} {r} 0 0 1 -{r} {r}h-{w - r:.1f}z" fill="{AZUL}"/>')
            rotulo, fim = br(v, casas), px(v)
        o.append(texto(LARGURA - 16, cy + 4, rotulo, TINTA, ancora="end"))
    return o, y0 + LINHA * 4 + (30 if meta else 16)


def gravar(nome, titulo, paineis):
    corpo, y = [], 8
    for p in paineis:
        o, y = painel(y, *p)
        corpo += o
        y += 8
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{LARGURA}" height="{y}" viewBox="0 0 {LARGURA} {y}" role="img" aria-label="{tr(titulo)}" '
           f'font-family="system-ui, -apple-system, Segoe UI, sans-serif"><rect width="{LARGURA}" height="{y}" fill="{FUNDO}"/>' + "".join(corpo) + "</svg>\n")
    nome = ARQUIVOS_EN[nome] if INGLES else nome
    open(os.path.join(SAIDA, nome), "w", encoding="utf-8", newline="\n").write(svg)
    print(nome, LARGURA, y)


def main():
    os.makedirs(SAIDA, exist_ok=True)
    lat = [r for r in ler("latencias.csv") if r["perfil"] == "restrito"]

    def faixa(metrica, escala=1):
        out = {}
        for app in APPS:
            v = [float(r["p95_ms"]) * escala for r in lat if r["app"] == app and r["metrica"] == metrica]
            out[app] = (min(v), max(v))
        return out

    gravar("tempos.svg", "Tempos no perfil restrito, p95", [
        ("Tecla até editor (ms)", faixa("M1"), 1, 16),
        ("Confirmar segmento (ms)", faixa("M3"), 1, 50),
        ("Busca por concordância (ms)", faixa("M8"), 1, 100),
        ("Prévia do documento compilado (s)", faixa("M11", .001), 2),
    ])
    mem = [r for r in ler("memoria.csv") if r["perfil"] == "restrito" and r["documento"] == "longo" and r["rotulo"] == "aberto"]
    m7 = {r["app"]: r for r in ler("m7.csv") if r["app"] != "flutter" or r["rodada"] == "f3-restrito-m7-flutter"}
    tam = json.load(open(os.path.join(RAIZ, "dist-pacotes", "tamanhos.json"), encoding="utf-8"))["tamanhos"]
    gravar("memoria-instalador.svg", "Memória e tamanho do instalador", [
        ("Memória com 3 600 segmentos abertos (MiB)", {r["app"]: float(r["privada_mb_p50"]) for r in mem}, 0),
        ("Memória em uso prolongado, minutos 20–60 (MiB)", {a: float(m7[a]["mb_20a60min"]) for a in APPS}, 0),
        ("Crescimento da memória (MiB/min)", {a: float(m7[a]["inclinacao_mb_min"]) for a in APPS}, 2),
        ("Instalador (MiB)", {a: float(tam[a]["instalador_mb"]) for a in APPS}, 1),
    ])


for INGLES in (False, True):
    main()
