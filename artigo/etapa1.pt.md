# Qual casca desktop para um editor de tradução? Resultados da Etapa 1 (Windows)

Rascunho de 27/11/2026. **Resultados provisórios:** só Windows; a consistência do editor entre sistemas (28% do peso) ainda não foi medida.

## Resumo

Implementamos o mesmo protótipo de editor de tradução em quatro cascas desktop (Electron, Tauri 2, Avalonia 12 e Flutter com o editor Fleather), com um núcleo comum em Rust, e medimos latência, memória e tamanho sob um protocolo registrado antes das medições e emendado ao longo do caminho (todas as emendas são públicas).

- Os quatro ficam abaixo do limite eliminatório de digitação (p95 < 33 ms no perfil restrito) e atingiram todas as metas de latência nas condições padronizadas.
- Electron, Tauri e Avalonia estão em **empate técnico** na pontuação parcial. O Flutter fica atrás, principalmente pela memória.
- O Flutter, com o Fleather 1.28.0 como publicado, mostrou crescimento contínuo de memória em uma hora de uso. Rastreamos a parte ligada à navegação até a biblioteca de edição.
- Com três correções na biblioteca, esse crescimento desaparece na hora de teste (sensibilidade, fora da pontuação).
- Não há vencedor nesta etapa. O critério de maior peso depende do Linux e do macOS.

## 1. Pergunta e método

Para um editor desktop de documentos estruturados (ferramenta CAT), com trabalho offline, fila de sincronização e prévia ao vivo: qual casca usar?

| Candidato | Versão | Editor do segmento |
| --- | --- | --- |
| Electron | 44.4.5 | ProseMirror |
| Tauri 2 | WebView2 158.0.4370.33 | ProseMirror (mesmo front-end do Electron) |
| Avalonia | 12.2.0 (.NET 10) | controle próprio, escrito para o estudo |
| Flutter | 3.50.1 | Fleather 1.28.0 |

**Protótipo.** Importação de DOCX, XLSX e PPTX; lista virtualizada; editor só no segmento ativo, com tags atômicas; prévia por tecla; SQLite com fila de sincronização; mesclar e dividir; busca em memória de tradução. A importação, o banco, a busca e a exportação ficam no núcleo Rust (`catcore`), usado como processo separado em três candidatos e dentro do processo no Tauri. Os testes funcionais por teclado (T-R2 a T-R7 e T-R9) passam nos quatro.

**Equipamento.** Notebook com Intel Core Ultra 7 255HX, 31 GB, GPU integrada Intel e NVIDIA RTX 5070 Laptop, tela de 240 Hz, Windows 11, na tomada. Perfil **Restrito** (principal): 4 núcleos, 8 GB, GPU integrada. Perfil **Completo**: sem limites, NVIDIA.

**Medição.** Teclas enviadas pelo sistema operacional (`SendInput`); o app registra o quadro seguinte à mudança. Piso do harness: p50 0,50 ms, p95 1,26 ms. Dez repetições com ordem sorteada, a primeira descartada; p50, p95 e intervalo de 95% do p95 por bootstrap. Meta atingida só se o limite superior do intervalo fica abaixo da meta. Memória: soma da memória privada de todos os processos. Rolagem: PresentMon.

**Documentos.** Quatro sintéticos (120 a 10 000 segmentos) e três reais: um documento do governo dos EUA com layout complexo (329 segmentos), um texto corrido longo (*Moby Dick*, digitalização com ruído de OCR, 5 424 segmentos) e uma apresentação PPTX privada (596 slides, 500 MB, 1 718 segmentos), cujo conteúdo não é publicado. Memória de tradução real DGT-TM, 51 769 pares EN-GB/PT-PT.

## 2. Resultados

### 2.1 Latência (p95 em ms; faixa entre documentos)

| Métrica (meta) | Perfil | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- | --- |
| M1 tecla até editor (< 16) | Restrito | 8,6–9,5 | 8,8–10,1 | 6,8–8,6 | 10,9–12,8 |
| M1 | Completo | 8,1–8,2 | 8,3–8,5 | 6,2–7,4 | 11,0–11,2 |
| M3 confirmar (< 50) | Restrito | 16,5–22,1 | 10,7–12,6 | 15,1–38,4 | 14,9–28,6 |
| M6 mesclar/dividir (< 100), pior p95 | Restrito | 34,4 | 23,1 | 32,3 | 33,3 |
| M8 busca na memória (< 100) | Restrito | 14,9–23,5 | 11,9–13,4 | 13,3–21,3 | 13,7–24,3 |
| M5 abrir 10 000 segmentos (< 2 000), p50 | Restrito | 656 | 483 | 1 117 | 595 |

- As 308 combinações com meta (app, perfil, documento, métrica, passo) ficaram como "atingida".
- **M2 (tecla até prévia, meta < 50)** saiu igual ao M1: nos quatro, o app marca o mesmo quadro para o editor e para a prévia. Ela não acrescenta informação sobre o M1.
- **M9 (início até começar a abrir o documento)**, p50 de 3,4 a 4,2 s, sem diferença relevante: todos carregam a memória de tradução de 51 769 pares antes de abrir, o que domina o tempo. Não mede o início da casca isolada; partida a frio e com cache não foram separadas. Houve um início isolado de 24,4 s no Electron, sem causa determinada.
- **M11 (prévia compilada, sem meta)**: 1,2 a 8,1 s conforme o documento, praticamente igual nos quatro, porque a conversão pelo LibreOffice domina.
- Mesclar é recusado uma vez por execução em documentos reais (par de segmentos em blocos diferentes), igual nos quatro.

### 2.2 Rolagem (M4)

Com a definição registrada (quadros perdidos em relação a 60 Hz), os quatro perdem 0% nos documentos que permitem rolagem contínua. A tela é de 240 Hz e os candidatos apresentam taxas diferentes (cerca de 240 quadros por segundo em Electron e Tauri, perto de 100 no Avalonia e de 123 no Flutter), diferença que a métrica não captura. No documento pequeno a rolagem chega ao fim e o valor não mede desempenho.

### 2.3 Memória

| Memória privada, todos os processos (MB), Restrito | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- |
| Documento de 3 600 segmentos recém-aberto | 275 | 264 | 231 | 345 |
| Uso prolongado, mediana dos minutos 20–60 | 329 | 330 | 221 | 710 |
| Máximo em 60 minutos | 356 | 354 | 250 | 859 |
| Inclinação (MB/min) | 0,28 | 0,24 | 0,31 | **7,85** |
| Crescimento contínuo (critério registrado) | não | não | não | **sim** |

O uso prolongado é um roteiro de 60 minutos com ciclos de digitar, confirmar, rolar, mesclar e dividir, trocando entre três documentos, um aberto por vez. Foi feito só no perfil Restrito, uma execução por candidato.

#### Origem do crescimento no Flutter

Em testes curtos, um tipo de operação por vez, o crescimento veio principalmente da troca de segmento (cerca de 60 KB por troca). Cada troca cria um editor e descarta o anterior; três pontos do Fleather 1.28.0 podem reter o editor descartado: um observador não removido no `dispose`, uma rotina de quadro que não pode ser removida e um ouvinte preso a um objeto antigo.

| Teste de navegação (cerca de 2,5 min) | MB/min |
| --- | --- |
| Biblioteca como publicada | 18,7 |
| Só a primeira correção | 19,1 |
| Só a segunda | 19,9 |
| Só a terceira | 16,6 |
| As três juntas | 0,8 |

Pares de correções não foram testados. **Sensibilidade (decidida depois de ver o resultado oficial; Emenda 10):**

| Uso prolongado de 60 minutos, Restrito | Como publicada (oficial) | Com as três correções |
| --- | --- | --- |
| Mediana dos minutos 20–60 (MB) | 710 | 382 |
| Máximo (MB) | 859 | 397 |
| Inclinação (MB/min), IC 95% | 7,85 (7,68 a 8,01) | 0,01 (−0,12 a 0,15) |
| Crescimento contínuo | sim | não |

Uma execução de cada, com o mesmo roteiro, documentos e semente; na corrigida todas as 7 200 teclas tiveram resposta e não houve falha. Os testes funcionais T-R2 a T-R7 e T-R9 passam no pacote corrigido.

A pontuação usa a biblioteca como publicada. O patch (`harness/patches/`) não foi revisado pelos autores do Fleather; latências não foram medidas com ele, e os testes funcionais não cobrem seleção por mouse nem área de transferência.

### 2.4 Tamanho

| | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- |
| Instalador (MB) | 110,6 | 2,2 | 49,6 | 9,7 |
| Pasta instalada (MB) | 370,2 | 6,3 | 206,8 | 30,1 |

Mesmo script de instalador e mesma compressão para os quatro. O número do Tauri **não inclui o WebView2**, presumido no sistema. Não houve otimização de tamanho (por exemplo, remoção de idiomas ou *trimming*). Tamanho de atualização não medido.

### 2.5 Esforço e ecossistema

O estudo foi implementado por um agente de IA, e as horas por requisito, previstas no pré-registro, **não foram registradas**. Linhas de código próprias (sem testes nem núcleo): Electron 1 066, Tauri 1 081 (front-end reaproveitado mais ponte), Avalonia 1 053, Flutter 1 107. A proximidade das linhas não indica esforço igual: o Avalonia exigiu escrever o editor com tags, e o Flutter exigiu contornos de foco, altura e teclado no Fleather. Registro em `results/diario-bugs.md`.

Ecossistema (bibliotecas na linguagem natural de cada candidato, nota 1 a 5): Electron 4 (falta DOCX com ida e volta), Tauri 3 (Rust: DOCX não verificado e sem XLIFF maduro; usando bibliotecas JavaScript no front-end, 4), Avalonia 4 (Open XML SDK e ClosedXML; XLIFF 1.2 manual), Flutter 3 (DOCX e XLIFF manuais). Fontes em `phase0/fichas.csv`, itens 25 a 29; a maturidade do SQLite não foi levantada.

## 3. Pontuação parcial

Regra do pré-registro; as escolhas de agregação que ele não fixava estão na Emenda 11, **escrita depois de conhecidos os resultados** e aprovada em 27/11.

| Critério (peso) | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- |
| Desempenho (22%) | 4,75 | 4,75 | 4,86 | 4,71 |
| Memória (17%) | 4 | 4 | 5 | 1 |
| Ecossistema (11%) | 4 | 3 | 4 | 3 |
| Instalador (5%) | 1 | 5 | 1 | 1 |
| **Parcial, 55 pontos de peso** | **4,03** | **4,19** | **4,38** | **2,89** |
| Variante com esforço por linhas (todos 5), 72 pontos | 4,26 | 4,38 | 4,53 | 3,38 |

**Sensibilidade** (`results/fase4/sensibilidade.csv`), em 60 combinações das escolhas de cálculo:
- Tauri fica em primeiro em 36 e Avalonia em 24. A ordem entre os dois **depende da escolha de cálculo**.
- Tauri e Avalonia ficam a menos de 10% do primeiro nas 60; Electron em 45; Flutter em nenhuma.
- Usando a memória com documento recém-aberto em vez do uso prolongado, o Tauri passa à frente (4,50 contra 4,38) e o Flutter sobe a 3,50.
- Com a memória do Flutter corrigido, a nota de memória dele sobe de 1 para 4 e a parcial de 2,89 para 3,81, ainda 13% abaixo do primeiro. Essa variante não entra na contagem das 60 combinações.
- A consistência (28%) pode inverter a ordem: na parcial principal, uma vantagem maior que 0,37 ponto já poria o Tauri à frente do Avalonia, e maior que 0,69 poria o Electron.

## 4. Eliminatórios nesta etapa

| Critério | Situação |
| --- | --- |
| Estável nos três sistemas | não avaliado (Etapa 2) |
| Tags protegidas iguais nos três sistemas | não avaliado (Etapa 2) |
| p95 de digitação < 33 ms no Restrito | os quatro passam |
| Licença comercial sem custo por usuário | os quatro frameworks e os editores usados são de código aberto permissivo; o editor pago da Avalonia não foi usado |

## 5. Ameaças à validade

- **Um só computador** e um só sistema.
- **Tela parada (Emendas 8 e 9).** Sem nada sendo redesenhado na tela, o p95 de digitação e confirmação sobe cerca de 6 a 9 ms em Electron, Tauri e Avalonia; no Flutter quase não muda. As metas são julgadas com uma pequena animação na tela. Em quatro rodadas completas com a tela parada, 107 de 112 combinações atingem a meta, 4 ficam limítrofes (Avalonia) e 1 não atinge (Tauri, texto longo real, M1: p95 17,1 ms). Não se sabe o mecanismo, nem o que redesenhava a tela nas sete rodadas mantidas, feitas antes da animação.
- A regra de invalidação usa a própria métrica M1 e foi criada depois da primeira leitura dos resultados. As rodadas descartadas são reportadas. As rodadas de rolagem do perfil Completo não têm par para conferir o efeito.
- **Correções no meio do caminho:** guarda de foco no driver e repetição de falhas (Emenda 7; as rodadas anteriores não a tinham); correção do pareamento entre tecla e resposta, com reprocessamento de tudo; Avalonia com *swap chain* DXGI para a rolagem ser mensurável (Emenda 6).
- **Marcação interna de quadro:** não inclui a varredura da tela, e o método difere por framework.
- **Front-end compartilhado** entre Electron e Tauri: não representa o esforço de quem começa pelo Tauri. Os dois receberam a mesma correção das setas ao lado de tags no ProseMirror.
- **Avalonia** reflete um editor feito pelo estudo, não o RichTextEditor do fornecedor. **Flutter** representa um editor específico (Fleather), não o framework em geral.
- **Memória:** uma execução de 60 minutos por candidato, só no Restrito, um documento aberto por vez (o pré-registro citava três). O M7 do Flutter foi refeito por uma lacuna sem resposta na primeira execução.
- **M8** com 51 769 pares, não os 100 mil previstos. Temperatura da CPU não disponível. Exportação de XLSX não implementada. Prévia compilada só sob demanda. IME de CJK fora do protótipo.
- **Uso de IA:** implementação, harness, análise e este rascunho foram produzidos com um agente de IA, sob decisões do autor.

## 6. Próximos passos

A Etapa 2 (Linux e macOS) mede estabilidade, tags atômicas e consistência do editor, hoje a maior incerteza. Ela só começa com autorização explícita.

## Reprodução

`PREREGISTRO.md` (protocolo e emendas), `spec/` (especificação e roteiros), `harness/` (driver, rodadas, análise: `resultados.py`, `pontuacao.py`), `results/` (dados brutos e consolidados). Os documentos privados e reais não são versionados; os sintéticos são gerados por `corpus/gen_corpus.py`.
