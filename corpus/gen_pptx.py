"""PPTX sintético (semente fixa) para testes de importação, exportação e prévia compilada. Uso: python gen_pptx.py out/sintetico.pptx [slides]"""
import random, sys
from pptx import Presentation
from pptx.util import Inches, Pt

saida = sys.argv[1] if len(sys.argv) > 1 else "out/sintetico.pptx"
n = int(sys.argv[2]) if len(sys.argv) > 2 else 40
rnd = random.Random(20261124)
pal = "system record transfer client report review update access policy order value service plan batch audit".split()
def frase(k):
    return " ".join(rnd.choice(pal) for _ in range(k)).capitalize() + "."
prs = Presentation()
for i in range(n):
    s = prs.slides.add_slide(prs.slide_layouts[1])
    s.shapes.title.text = f"Slide {i + 1}: " + frase(3)
    tf = s.placeholders[1].text_frame
    for j in range(3):
        p = tf.paragraphs[0] if j == 0 else tf.add_paragraph()
        r = p.add_run(); r.text = frase(6) + " "
        r2 = p.add_run(); r2.text = frase(4) + " "; r2.font.bold = True
        r3 = p.add_run(); r3.text = frase(5); r3.font.italic = (j % 2 == 0)
prs.save(saida)
print(saida, n, "slides")
