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

## Emenda 1 (2026-11-24, aprovada pelo Gian)
**Flutter:** mantido como representante, com o **Fleather** como editor. Motivo: o Super Editor tem Windows e Linux com verificação irregular e versão estável antiga (`phase0/fichas.csv`, ficha 22). Risco registrado: Fleather tem bugs abertos de IME em desktop (issues #580, #419, #420) e suporte a chip inline atômico não verificado; se o Fleather não implementar tags atômicas (requisito 3 do protótipo), parar e perguntar ao Gian. A tabela da seção 2 passa a dizer Flutter: Fleather, confirmado.
Pendente: decisão sobre a Avalonia (RichTextEditor).

## Emenda 2 (2026-11-24, aprovada pelo Gian)
**Avalonia 12:** editor de segmento será um **controle próprio** (sem RichTextEditor Pro e sem usar o trial). Motivo: o trial (contrato 10C) é só para avaliação interna e omite a publicação de resultados; evita dependência de licença e de resposta da Avalonia. As horas de implementação do controle próprio contam integralmente no critério de esforço. Registrar como limitação no artigo que o RichTextEditor pago não foi avaliado e que o resultado da Avalonia reflete um editor feito pelo estudo, não o do fornecedor. Tabela da seção 2: Avalonia, controle próprio, confirmado.
Todos os pontos abertos da seção 11 que dependiam de candidato estão resolvidos; seguem abertos 3 e 4 só se o Gian não os considerou aprovados na aprovação geral (considerados aprovados).

## Emenda 3 (2026-11-24, aprovada pelo Gian): protocolo de medição
Não altera critérios, metas nem pesos. Acrescenta etapas para que as medições da Fase 3 sejam válidas, todas definidas **antes** da primeira medição:
1. **Corpus real:** carregador TMX no núcleo (UTF-16, pares EN-GB e PT-PT); texto longo = Moby Dick (docs/mobydick.txt, digitalização com ruído de OCR, declarado) convertido em DOCX; layout complexo = CMS-10569 (docx do governo dos EUA) além do modelo da FAA; manifesto com SHA-256.
2. **Build de produção** empacotado por candidato, versões fixas gravadas em `ambiente.json` (script gerador).
3. **Orquestrador de rodadas:** ordem dos candidatos sorteada por rodada (semente registrada), 10 repetições (30 em VM) com a primeira descartada, pausa entre execuções, perfis Completo e Restrito (Restrito: 4 núcleos e 8 GB por limite de processo; iGPU por preferência gráfica do Windows), notebook na tomada, modo de energia e temperatura da CPU registrados, estado do Smart App Control registrado.
4. **M7:** amostragem da memória privada da árvore de processos a cada 60 s pelo harness; M9 e M10 por script.
5. **M4:** medição independente do app com PresentMon (quadros apresentados pelo sistema operacional), igual para os quatro; complementar à marcação interna de quadro.
6. **M8 nos dois modos** (concordância e memória) com a TM real. A TM real tem 51 769 pares EN-GB/PT-PT nos arquivos fornecidos (e não 100 mil); o M8 é reportado nesse tamanho, e o tamanho é declarado no artigo.
7. **Autoteste do harness:** driver contra um aplicativo trivial (Notepad) para medir o piso de ruído do SendInput e o desvio entre relógios; números dos candidatos só valem acima do piso.
8. **Estatística:** p95 com intervalo de confiança por bootstrap, conforme a seção 6.

## Emenda 4 (2026-11-24, aprovada pelo Gian): PPTX como formato exigido
Importar **PPTX** passa a ser requisito do protótipo (além de DOCX, XLSX e XLIFF do brief): cada caixa de texto/parágrafo vira um bloco, com a posição (slide e forma) como origem, mesma regra de segmentação. O arquivo de teste é uma apresentação PPTX em `docs/` (596 slides, 500 MB), fornecida pelo Gian, **não versionado** (`docs/` está no .gitignore) e cujo conteúdo não será publicado nem citado no artigo. O Gian disse que esse tipo de documento é comum; o Gian autorizou o uso do arquivo nas medições (2026-11-24), com a condição de que o conteúdo não seja citado nem publicado. O esforço de implementar o importador PPTX conta no critério de esforço. Requisito R1 da spec passa a incluir PPTX na v0.2.

## Emenda 5 (2026-11-24, aprovada pelo Gian): prévia dupla
Não altera critérios, metas nem pesos; define duas prévias, ambas implementadas nos quatro candidatos:
1. **Prévia rápida (aproximada):** blocos estilizados atualizados por tecla. É a que o M2 mede (tecla até prévia atualizada, meta < 50 ms). Não reproduz layout, fontes, tabelas nem paginação; declarar no artigo.
2. **Prévia compilada (documento real), sob demanda ou ao ficar ocioso (debounce):** o núcleo exporta o documento (DOCX e PPTX) com os destinos dos segmentos, um motor externo comum (LibreOffice headless, **instalação a autorizar pelo Gian**) converte em PDF/imagens, e cada casca exibe o resultado. Nova métrica **M11: pedido de visualização até a página aparecer**, registrar mediana e p95, sem meta; separar tempo de exportação, de renderização (igual nos quatro) e de exibição (o que depende da casca).
- Exportação com round-trip de tags (DOCX e PPTX) conta no critério de esforço; teste de aceitação novo: exportar sem edição reproduz o texto original, e com edição troca só os segmentos alterados.
- Diferenças de render entre LibreOffice e Word/PowerPoint são declaradas no artigo. Imagens renderizadas do PPTX privado nunca são capturadas em tela, commitadas nem publicadas (Emenda 4).
- Teste de carga da prévia rápida (blocos com mais texto e estilos) gera uma segunda curva do M2.

## Emenda 6 (2026-11-24): Avalonia com swap chain DXGI para o M4

**Motivo.** Com a composição padrão do Avalonia 12 no Windows (DirectComposition), o PresentMon não registra nenhum quadro com o PID do app: os quadros saem pelo `dwm.exe`, que atende outras janelas, e o M4 não é separável. Decidido por Gian: tentar DXGI primeiro; se não funcionasse, M4 do Avalonia seria "n/d".
**Mudança.** `Win32PlatformOptions.CompositionMode = LowLatencyDxgiSwapChain` no `Program.cs` do candidato Avalonia, para todas as métricas (não só o M4). Resultado do teste de validação (1 repetição, não é medição): o PresentMon passa a listar `CatStudyAvalonia.exe` com 547 quadros DXGI na rolagem.
**Efeito.** Rodadas anteriores do Avalonia (composição padrão) não valem para a análise e serão refeitas. A mudança será declarada no artigo.

## Emenda 7 (2026-11-26, aprovada pelo Gian): guarda de foco, repetição de falhas e rodadas refeitas

Não altera critérios, metas nem pesos. Definida no meio da Fase 3, depois de conferir contagens e integridade de 16 rodadas e **antes** de analisar os resultados de latência.

**Motivo.** (a) O driver só conferia o foco no início de cada execução; em `f3-completo-medicao-pior_caso` (Flutter, repetição 8, interrompida) as teclas passaram a ir para outra janela. (b) Execuções que falhavam não eram repetidas nem apareciam no log da Fase 3. (c) No M7 do Flutter, 9 dos 360 ciclos ficaram sem resposta num bloco de cerca de 91 s, sem causa determinada.

**Mudanças.**
1. O driver confere, antes de cada tecla e de cada evento de roda, se a janela em primeiro plano é do app medido; se não for, encerra o app e aborta a execução sem enviar mais eventos.
2. Execução que falha (foco, app que não abre em 60 s) é repetida na hora, até 3 tentativas, com 60 s de intervalo. As tentativas falhas ficam em `<rodada>/_falhas/`, fora da análise, e são listadas em `rodada.json`. Rodada com execução não recuperada é marcada incompleta e refeita.
3. Refeitas por inteiro: `f3-restrito-medicao-real_texto` (Electron, repetição 2, app não abriu) e `f3-restrito-m4-pior_caso` (Flutter, repetição 8, idem). Os dados originais ficam em `results/_descartadas/`.
4. M7 do Flutter refeito em `f3-restrito-m7-flutter`; ele substitui as linhas do Flutter em `f3-restrito-m7`, que são mantidas e declaradas. Os demais candidatos do M7 não mudam.
5. As rodadas anteriores a esta emenda foram feitas sem a guarda; o artigo declara isso.

**Pendência de análise.** `harness/juntar.py` descarta respostas acima de 1 s e atribui cada resposta ao último envio anterior, o que pode omitir ou encurtar latências altas. Os dados brutos estão preservados; o pareamento será corrigido e todas as rodadas reprocessadas antes de qualquer resultado ser reportado.

**Correção aplicada (2026-11-26, antes da primeira leitura dos resultados).** Pareamento em `harness/juntar.py`: por ordem de envio, sem corte de 1 s (limite de sanidade de 60 s). `harness/resultados.py` reprocessa todas as rodadas a partir dos CSVs brutos.
O relógio do Flutter marca a chegada até 0,24 ms antes do envio do driver, o que deslocava o pareamento em uma tecla (falsos 1,35 s no p95 de M3/M6). O pareamento aceita até 5 ms de folga; verificado após a primeira consolidação, e todas as rodadas foram reprocessadas.

## Emenda 8 (2026-11-26, aprovada pelo Gian): validade das rodadas de medição

Definida **depois** da primeira leitura dos resultados e **antes** de refazer as rodadas. Não altera metas nem pesos.

**Motivo.** Em `f3-restrito-medicao-real_texto` (rodada refeita), `f3-completo-medicao-pior_caso`, `-real_texto` e `-real_pptx`, feitas em 26/11 entre 00:37 e 05:14, todas as execuções de Electron, Tauri e Avalonia mostraram M1 p50 de cerca de 11,5 a 13,5 ms, contra 4 a 7,4 ms nas repetições 2 a 10 das demais rodadas; o Flutter não mudou. Um teste alternando o driver com e sem a guarda de foco (`results/_diagnostico/ab_driver/`) deu 11 de 12 execuções rápidas e 1 lenta, nos dois grupos há execuções rápidas e a mesma assinatura já aparecia antes da guarda. Conclusão: estado esporádico da máquina, de causa não determinada.

**Regra.** Em rodada com o roteiro `medicao.json`, se uma execução das repetições 2 em diante de Electron, Tauri ou Avalonia tiver M1 p50 acima de 9,5 ms, a rodada é interrompida e refeita inteira, após 15 minutos, até 4 tentativas; depois disso a sequência para. Não se escolhem nem se substituem execuções isoladas. As tentativas interrompidas ficam em `results/_descartadas/`. O Flutter fica fora da regra porque não mostrou o efeito. Nenhuma das rodadas mantidas viola a regra.

**Aplicação.** As quatro rodadas acima são movidas para `results/_descartadas/` (sufixo `-estado_lento`) e refeitas com as mesmas sementes. A rodagem (M4) não é refeita: `f3-restrito-m4-pior_caso`, feita nesse período, tem quadros equivalentes aos da execução anterior fora dele; as rodadas M4 do perfil Completo não têm par para essa comparação, e o artigo declara isso.

**Limitação declarada.** A regra usa a própria métrica para decidir a validade; ela pode descartar uma execução legitimamente lenta. O artigo reporta as rodadas descartadas e seus valores.

## Emenda 9 (2026-11-26, aprovada pelo Gian): condição de tela nas rodadas de medição

Substitui o diagnóstico da Emenda 8 ("causa não determinada"). Não altera metas nem pesos.

**Motivo.** As duas primeiras tentativas sob a Emenda 8 foram invalidadas. Sondas curtas (`results/_diagnostico/tela_parada/`) mostraram que o fator é a tela: com algo sendo redesenhado, M1 p50 de 3,7 a 5,9 ms (18 sondas); com a tela parada, de 12,0 a 13,3 ms (18 sondas), sem sobreposição, em Electron e Avalonia. O mecanismo (composição do Windows, driver de vídeo ou painel) não foi determinado.

**Mudança.** Nas rodadas com `medicao.json`, a `rodada.py` mantém uma janela de 90x90 com animação contínua, sempre por cima e sem foco (`harness/animador.py`). Custa processador e vídeo, igual para os quatro candidatos. A regra de validade da Emenda 8 continua.

**Aplicação.** Só as quatro rodadas da Emenda 8 são refeitas, com as mesmas sementes. As quatro versões lentas (`results/_descartadas/*-estado_lento`), completas com 10 repetições, são reportadas à parte como sensibilidade à tela parada, condição próxima do uso real.

**Limitações declaradas.** Não se sabe o que redesenhava a tela nas sete rodadas de medição mantidas (24 e 25/11); elas são aceitas porque estão na faixa rápida. Nelas não havia a janela de animação. As metas são julgadas pelas rodadas na condição padronizada; o artigo mostra também a tela parada.

## Emenda 10 (2026-11-27, aprovada pelo Gian): sensibilidade do M7 com o Fleather corrigido

Definida **depois** de conhecer o M7 oficial e **antes** de medir a versão corrigida. Não altera metas, pesos nem resultados oficiais.

**Motivo.** O Flutter mostrou crescimento contínuo de memória (M7). Testes curtos (`results/_diagnostico/flutter_memoria/`) ligaram o crescimento na navegação a três pontos do Fleather 1.28.0 (última versão publicada) que retêm o editor descartado; com as três correções juntas (`harness/patches/fleather-1.28.0.patch`) o crescimento nesse teste caiu de 18,7 para 0,8 MB/min.

**Medição.** Uma execução do roteiro `uso_prolongado.json` (60 min), perfil Restrito, com o pacote corrigido (`harness/flutter_corrigido.py`), mesmos documentos, semente (20261148), amostragem e regra de crescimento contínuo do M7 oficial; falhas são repetidas conforme a Emenda 7. Antes dela, os testes funcionais T-R2 a T-R7 e T-R9 no mesmo pacote.

**Uso do resultado.** O Flutter continua avaliado, na pontuação, com a biblioteca como publicada. A versão corrigida é reportada à parte como sensibilidade, qualquer que seja o resultado.

**Limitações declaradas.** Latências (M1 a M6, M8) não são medidas na versão corrigida; os testes funcionais cobrem comportamento por teclado, não seleção por mouse nem área de transferência; o patch não foi revisado pelos autores do Fleather.

## Emenda 11 (2026-11-27, aprovada pelo Gian): cálculo da pontuação provisória da Etapa 1

Escrita **depois** de conhecer os resultados. As seções 6 e 8 não fixam como juntar várias medições numa nota; as escolhas abaixo são usadas na pontuação provisória e as alternativas aparecem na sensibilidade (`results/fase4/sensibilidade.csv`).

1. **Desempenho:** perfil Restrito; nota por métrica e documento (e passo), média por métrica, média das métricas 1, 2, 4 e 6.
2. **M4:** o documento `pequeno` fica fora, porque a rolagem chega ao fim e os quadros "perdidos" não medem desempenho.
3. **Memória:** mediana dos minutos 20 a 60 do M7; menos 1 ponto com crescimento contínuo.
4. **Esforço:** as horas pré-registradas não foram contadas. O resultado principal deixa o critério **fora**; uma variante usa linhas de código próprias (Tauri = front-end reaproveitado mais ponte), o que é um substituto fraco.
5. **Ecossistema:** Electron 4, Tauri 3, Avalonia 4, Flutter 3, a partir das fichas 25 a 29 (SQLite não foi levantado nas fichas e não gera desconto).
6. **Instalador:** só o instalador, sem WebView2; tamanho de atualização não medido.
7. **Consistência (28%):** não pontuada na Etapa 1. O total provisório é a média ponderada dos critérios disponíveis.
