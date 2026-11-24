"""Gera o corpus sintetico (seed fixa). Uso: python gen_corpus.py out_dir

Regras para casar com a segmentacao da spec: cada frase termina em '.' e a
proxima comeca com maiuscula; nenhum '.', '?' ou '!' aparece dentro de frase.
Tags inline = runs formatados (negrito/italico), hyperlinks e quebras de linha.
"""
import json, random, sys, pathlib
from docx import Document
from docx.shared import Pt
import openpyxl

SEED = 20261124
WORDS = ("system record segment document review policy section value process "
         "report field table update source target model number version output "
         "window editor memory result change length order cache local remote "
         "format layout style margin header footer title notice agency rule").split()


def sentence(rng, n_fmt):
    """Retorna (lista de (texto, fmt)), n_tags). fmt in None|'b'|'i'|'br'"""
    nwords = rng.randint(8, 18)
    ws = [rng.choice(WORDS) for _ in range(nwords)]
    ws[0] = ws[0].capitalize()
    parts, i = [], 0
    spans = []
    for _ in range(n_fmt):
        if i >= nwords - 2:
            break
        a = rng.randint(i, nwords - 2)
        b = min(nwords - 1, a + rng.randint(0, 2))
        spans.append((a, b, rng.choice("bi")))
        i = b + 2
    cur, out = 0, []
    for a, b, f in spans:
        if a > cur:
            out.append((" ".join(ws[cur:a]) + " ", None))
        out.append((" ".join(ws[a:b + 1]), f))
        cur = b + 1
        if cur < nwords:
            out.append((" ", None))
    if cur < nwords:
        out.append((" ".join(ws[cur:]), None))
    out[-1] = (out[-1][0] + ".", out[-1][1])
    return out, len(spans) * 2


def add_sentence(par, rng, n_fmt, first):
    runs, ntags = sentence(rng, n_fmt)
    if not first:
        par.add_run(" ")
    for text, f in runs:
        r = par.add_run(text)
        if f == "b":
            r.bold = True
        elif f == "i":
            r.italic = True
    return ntags


def make_docx(path, n_sentences, tags_per_sentence, seed, per_par=(3, 7), tables=True):
    rng = random.Random(seed)
    doc = Document()
    doc.styles["Normal"].font.size = Pt(11)
    total, tags = 0, 0
    heading_every = 25
    while total < n_sentences:
        if total % heading_every == 0:
            h = doc.add_heading(level=1 + (total // heading_every) % 3)
            tags += add_sentence(h, rng, 0, True)
            total += 1
            continue
        kind = rng.random()
        k = min(rng.randint(*per_par), n_sentences - total)
        if tables and kind < 0.06 and total + 4 <= n_sentences:
            t = doc.add_table(rows=2, cols=2)
            for row in t.rows:
                for cell in row.cells:
                    tags += add_sentence(cell.paragraphs[0], rng, rng.randint(0, tags_per_sentence), True)
                    total += 1
            continue
        style = "List Bullet" if kind < 0.2 else None
        par = doc.add_paragraph(style=style)
        for j in range(k):
            tags += add_sentence(par, rng, rng.randint(max(0, tags_per_sentence - 1), tags_per_sentence + 1), j == 0)
            total += 1
    doc.save(path)
    return total, tags


def make_xlsx(path, n_cells, seed):
    rng = random.Random(seed)
    wb = openpyxl.Workbook()
    ws = wb.active
    ws.title = "Textos"
    n = 0
    for r in range(1, n_cells // 4 + 1):
        for c in range(1, 5):
            s, _ = sentence(rng, 0)
            ws.cell(row=r, column=c, value="".join(t for t, _ in s))
            n += 1
    wb.save(path)
    return n


if __name__ == "__main__":
    out = pathlib.Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    exp = {}
    for name, n, tps, sd in [("pequeno", 120, 1, 1), ("medio", 1200, 1, 2),
                             ("longo", 3600, 1, 3), ("pior_caso", 10000, 3, 4)]:
        segs, tags = make_docx(out / f"{name}.docx", n, tps, SEED + sd)
        exp[f"{name}.docx"] = {"segmentos": segs, "tags_aprox": tags}
    exp["textos.xlsx"] = {"segmentos": make_xlsx(out / "textos.xlsx", 20000, SEED + 9)}
    (out / "esperado.json").write_text(json.dumps(exp, indent=2))
    print(json.dumps(exp, indent=2))
