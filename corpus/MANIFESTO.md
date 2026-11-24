# Manifesto do corpus (2026-11-24)

Arquivos binários ficam fora do git (`raw/`, `out/`). Reproduzir: `python -m venv .venv`, `pip install python-docx==1.2.0 openpyxl==3.1.5 lxml`, `python gen_corpus.py out`.

## Sintéticos (gerados por `gen_corpus.py`, semente 20261124, sem origem de terceiros, sem restrição de licença)
| Arquivo | Segmentos | SHA-256 |
| --- | --- | --- |
| pequeno.docx | 120 | ccc05c90fafb9193... |
| medio.docx | 1200 | a1c7c1b65b885e7b... |
| longo.docx | 3600 | b678ecc256a85d1e... |
| pior_caso.docx | 10000 | 71ed2cd7faad9c9b... |
| textos.xlsx | 20000 | b85dcc2c75f14ee6... |

Observação: o gerador cria 'longo' com 3600 segmentos (aprox. 300 páginas) e 'pior_caso' com 10000 segmentos, média de 3,4 tags por segmento.

## Reais
| Perfil | Fonte | Licença | Estado |
| --- | --- | --- | --- |
| Layout complexo | Template de Plano de Programa de Segurança, faa.gov: https://www.faa.gov/sites/faa.gov/files/aircraft/air_cert/design_approvals/dah/pscp_template.docx | Governo dos EUA (conferir trechos de terceiros) | baixado em raw/, SHA-256 51459f88bc8050b3... |
| Layout complexo (original do brief) | omb.report 102704700.docx | domínio público | **403** em download automático; baixar manualmente se quiser |
| Texto corrido longo | Project Gutenberg / U.S. Code | domínio público nos EUA | Gutenberg fora do ar (503) em 2026-11-24; pendente |
| Memória de tradução 100 mil pares EN-PT | DGT-TM (Comissão Europeia) | reutilização com crédito | pendente (download autorizado pelo Gian) |

Enquanto o DGT-TM não for carregado, os testes usam TM sintética (, semente fixa). Os números de busca (M8) só valem na rodada de medição com a TM real.
