"""M10: tamanho em disco de uma pasta ou arquivo (instalador, pacote). Uso: python -I harness/tamanho.py <caminho>..."""
import os, sys

for p in sys.argv[1:]:
    if os.path.isfile(p):
        t = os.path.getsize(p)
    else:
        t = sum(os.path.getsize(os.path.join(d, f)) for d, _, fs in os.walk(p) for f in fs)
    print(f"{t / 1048576:10.1f} MB  {p}")
