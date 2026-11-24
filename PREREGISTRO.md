# Pré-registro (APROVADO pelo Gian em 2026-11-24)

Aprovado em 2026-11-24, antes de qualquer medição. Os pontos da seção 11 sobre Flutter e Avalonia continuam abertos e serão registrados como emenda datada quando o Gian decidir. Nenhuma medição foi feita. Qualquer mudança posterior exige registro do motivo e aprovação do Gian.

## 1. Pergunta
Para um editor desktop de documentos estruturados (CAT Tool) com trabalho offline, sync e prévia ao vivo, qual casca usar em Windows, macOS e Linux?

## 2. Candidatos
| Candidato | Papel | Editor | Estado |
| --- | --- | --- | --- |
| Electron | Finalista | ProseMirror (prosemirror-view >= 1.42.8) | confirmado |
| Tauri 2 | Finalista | ProseMirror | confirmado |
| Avalonia 12 | Finalista | RichTextEditor (Pro) ou controle próprio | **ABERTO**: licença de teste e confirmação escrita da Avalonia |
| Flutter | Representante | Super Editor ou Fleather | **ABERTO**: decisão do Gian (ver `phase0/lacunas.md`) |

Versões exatas de cada framework, SO, driver e dependência são registradas em cada execução.

## 3. Perguntas de pesquisa (sem resultado esperado)
1. A latência de digitação cabe em um quadro (16 ms, p95) em cada candidato, no perfil restrito?
2. O editor com tags (seleção, cursor, navegação por palavra) se comporta igual nos três sistemas?
3. Quanto de memória (todos os processos) com o mesmo documento, e quanto cresce em 1 hora?
4. A prévia incremental atualiza em menos de 50 ms (p95) com 10 mil segmentos?
5. Quanto esforço (horas por requisito) cada candidato exige?

## 4. Critérios eliminatórios
1. Roda estável nos três sistemas (inclui Linux Wayland; NVIDIA se houver).
2. Tags inline protegidas funcionam igual nos três sistemas.
3. p95 de tecla até editor redesenhado < 33 ms no perfil restrito.
4. Licença compatível com uso comercial, sem custo por usuário.

Nota: a Etapa 1 (só Windows) não elimina por 1 e 2. O custo anual de licença da Avalonia Pro (por pessoa, 1 pessoa) **não é por usuário final**; o critério 4 é lido assim. **A confirmar com o Gian.**

## 5. Métricas e metas (p95)
| # | Métrica | Meta |
| --- | --- | --- |
| 1 | Tecla até editor redesenhado | < 16 ms |
| 2 | Tecla até prévia atualizada | < 50 ms |
| 3 | Confirmar até salvo e próximo em foco | < 50 ms |
| 4 | Quadros perdidos na rolagem | < 5% |
| 5 | Abrir documento longo até editar | < 2 s |
| 6 | Mesclar/dividir até lista e prévia corretas | < 100 ms |
| 7 | Memória (todos os processos) com 3 docs e após 1 h | sem crescimento contínuo |
| 8 | Busca em 100 mil pares | < 100 ms |
| 9 | Início do app (frio e com cache) | registrar |
| 10 | Instalador, atualização, build na CI | registrar |

## 6. Estatística
10 repetições por medida em máquina física (30 em VMs). Mediana, p95 e IC 95%. Descartar só a primeira execução (aquecimento), declarada. Definições propostas, **a aprovar**:
- "Crescimento contínuo" (métrica 7): inclinação de regressão linear da memória por minuto, nos últimos 40 min de 60, com IC 95% que exclua zero e > 0,5 MB/min.
- Meta atingida: p95 abaixo do limite **e** limite superior do IC 95% do p95 abaixo do limite; se só o p95 passa, marcar "limítrofe".
- Memória: soma de todos os processos do app (Windows: PrivateMemorySize64 somado pela árvore de processos; Linux: PSS; macOS: equivalente, a definir na Etapa 2).

## 7. Perfis e ambiente
Completo (todos os núcleos, sem limite, NVIDIA) e Restrito (4 núcleos, 8 GB, Intel integrada; perfil principal para eliminatórios). Notebook na tomada, mesmo modo de energia, pausas para esfriar, ordem sorteada, temperatura da CPU registrada, GPU fixada e registrada. Números de VM nunca se misturam com os do PC físico.

## 8. Pontuação (pesos fixados)
| Critério | Peso |
| --- | --- |
| Consistência do editor entre sistemas | 28% |
| Desempenho percebido | 22% |
| Memória em uso prolongado | 17% |
| Esforço de implementação | 17% |
| Ecossistema DOCX/Excel/XLIFF/SQLite | 11% |
| Tamanho do instalador e da atualização | 5% |

Empate técnico: diferença total < 10%; desempate pelo critério de maior peso; apresentar lado a lado.

### Regra de conversão em notas 1 a 5 (PROPOSTA, a aprovar)
- **Desempenho (22%)**: média das notas de métricas 1, 2, 4, 6. Por métrica: 5 se p95 <= 50% da meta; 4 se <= 75%; 3 se <= 100%; 2 se <= 150%; 1 se > 150%.
- **Memória (17%)**: nota 1 a 5 pela posição relativa à menor memória entre candidatos (razão r = valor/menor): r<=1,25→5; <=1,75→4; <=2,5→3; <=4→2; >4→1; **subtrair 1 ponto (mín. 1) se houver crescimento contínuo**.
- **Consistência do editor (28%)**: proporção de testes de comportamento (seleção, cursor, tags, navegação por palavra, IME) que passam nos três sistemas: >=95%→5; >=85%→4; >=70%→3; >=50%→2; <50%→1.
- **Esforço (17%)**: horas totais (agente + revisão humana) por requisito, relativo ao menor: razão <=1,25→5; <=1,75→4; <=2,5→3; <=4→2; >4→1.
- **Ecossistema (11%)**: 5 = biblioteca madura, mantida e licença comercial livre para DOCX, XLSX, XLIFF e SQLite; subtrair 1 por requisito que exija implementação manual de OOXML/XLIFF ou licença paga; mín. 1. Cada subtração documentada com link.
- **Instalador (5%)**: tamanho do instalador e da atualização, mesma escala de razão da memória.

## 9. Protótipo, corpus e Etapas
Os 8 requisitos do protótipo, o corpus (FAA .docx, U.S. Code/Gutenberg via pandoc, DGT-TM 100 mil pares EN-PT) e as duas etapas seguem o documento-base. Etapa 1 só Windows, com ponto de parada ao final.

## 10. Regras de condução
Sem dados do cliente. Sem runner self-hosted. Repositório público. Uso de agentes de IA declarado. Parar e perguntar ao Gian nos casos listados no documento-base.

## 11. Pontos abertos para aprovação
1. Flutter: manter (Fleather ou Super Editor), ou retirar.
2. Avalonia: como cobrir o editor (trial com confirmação escrita ou licença Pro).
3. Leitura do eliminatório de licença (seção 4).
4. Regras de conversão e definições estatísticas (seções 6 e 8).
