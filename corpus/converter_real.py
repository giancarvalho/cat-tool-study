"""Converte o texto longo real (Moby Dick, docs/mobydick.txt) em DOCX e confere os arquivos reais de docs/.
Uso: .venv/Scripts/python converter_real.py ../docs out
O texto é uma digitalização com ruído de OCR (ex.: "tbe"); isso é declarado no manifesto e não é corrigido.
Parágrafo = linhas consecutivas até linha em branco, unidas por espaço. Ignora o cabeçalho antes de "CHAPTER 1"."""
import hashlib, re, sys
from pathlib import Path
from docx import Document

docs, out = Path(sys.argv[1]), Path(sys.argv[2])
out.mkdir(exist_ok=True)
txt = (docs / "mobydick.txt").read_text(encoding="utf-8", errors="replace")
m = re.search(r"^\s*(CHAPTER|CHAP\.)\s*1\b", txt, re.M | re.I)
corpo = txt[m.start():] if m else txt
pars = [" ".join(p.split()) for p in re.split(r"\n\s*\n", corpo)]
pars = [p for p in pars if len(p) > 1]
d = Document()
for p in pars:
    d.add_paragraph(p)
d.save(out / "moby_dick.docx")
print("moby_dick.docx:", len(pars), "parágrafos")
for f in [out / "moby_dick.docx", docs / "mobydick.txt", *docs.glob("CMS-10569*.docx"), *docs.glob("*.pptx")]:
    h = hashlib.sha256()
    with open(f, "rb") as fh:
        for b in iter(lambda: fh.read(1 << 20), b""):
            h.update(b)
    print(h.hexdigest()[:16], f.name)
