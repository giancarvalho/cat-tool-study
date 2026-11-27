# Commits reescritos em 27/11/2026

Antes do primeiro envio ao GitHub, o histórico local foi reescrito para retirar o nome de um arquivo privado de `corpus/converter_real.py` e `PREREGISTRO.md`. Autores, datas, mensagens e a versão final dos arquivos não mudaram. Os registros em `results/` citam os identificadores antigos.

| Antigo | Novo | Data do commit | Assunto |
| --- | --- | --- | --- |
| `5be3d31` | `5be3d31` | 2026-11-24T01:39:41-03:00 | Fase 0: fichas de literatura e resumo de lacunas |
| `9c1a582` | `9c1a582` | 2026-11-24T01:40:16-03:00 | Fase 1: pré-registro aprovado (nenhuma medição realizada) |
| `77d1048` | `77d1048` | 2026-11-24T01:40:35-03:00 | Emenda 1 ao pré-registro: Flutter com Fleather |
| `47ee9f8` | `47ee9f8` | 2026-11-24T01:41:13-03:00 | Emenda 2 ao pré-registro: Avalonia com controle próprio |
| `66edf83` | `66edf83` | 2026-11-24T02:01:51-03:00 | Fase 2: plano aprovado |
| `771ceba` | `771ceba` | 2026-11-24T02:02:00-03:00 | Corrige caminho do Flutter no plano |
| `16210df` | `16210df` | 2026-11-24T02:04:15-03:00 | Fase 2 passo A: especificação funcional e contrato do roteiro (rascunho) |
| `4b951ef` | `4b951ef` | 2026-11-24T02:09:41-03:00 | Spec v0.1 aprovada |
| `1b342ea` | `1b342ea` | 2026-11-24T02:15:18-03:00 | Passo B (parcial): gerador de corpus sintético e código do núcleo Rust (não compilado) |
| `053d6aa` | `053d6aa` | 2026-11-24T02:26:18-03:00 | Núcleo Rust compila e passa T-R1, T-R5, T-R6, T-R7; manifesto do corpus |
| `db1a196` | `db1a196` | 2026-11-24T02:26:27-03:00 | Corrige texto do manifesto |
| `35091e4` | `35091e4` | 2026-11-24T02:40:49-03:00 | Passo C: app Electron com ProseMirror passa T-R2 a T-R7 (teclas via sendInputEvent) |
| `bf7b11f` | `bf7b11f` | 2026-11-24T11:48:18-03:00 | Driver de teclas SendInput, roteiro curto.json, relógio de parede comum e junção M1/M3/M6/M8 (T-R8 no Electron) |
| `13d72ad` | `13d72ad` | 2026-11-24T12:33:26-03:00 | curto.json: mesclar/dividir no par do mesmo bloco (5/5 ok) |
| `db28498` | `4cf7f79` | 2026-11-24T12:53:36-03:00 | Emendas 3 (protocolo de medição) e 4 (PPTX exigido); docs/ fora do git |
| `e16e2f9` | `31d3deb` | 2026-11-24T12:55:22-03:00 | Núcleo: importador PPTX e leitor TMX (51769 pares EN-GB/PT-PT); corpus real de docs/ no manifesto |
| `3aa36ae` | `14cb327` | 2026-11-24T12:56:21-03:00 | Emenda 4: Gian autoriza uso do PPTX nas medições, sem citar conteúdo |
| `8f7f897` | `da8ae38` | 2026-11-24T13:05:40-03:00 | Harness Emenda 3: perfil Restrito, memória da árvore, autoteste, orquestrador, análise com bootstrap, ambiente, M9/M10 |
| `82fe20d` | `45682c5` | 2026-11-24T13:20:13-03:00 | Candidato Tauri 2 (núcleo em processo, front-end compartilhado); despacho RPC na lib; juntar.py robusto a tecla perdida; Esc de aquecimento |
| `3bdaed5` | `ff0fd40` | 2026-11-24T13:29:33-03:00 | Emenda 5: prévia dupla (rápida aproximada + compilada via LibreOffice), métrica M11 |
| `fddf01a` | `e035f91` | 2026-11-24T13:33:05-03:00 | spec/quadro.md e proposta da spec v0.2; remove resultado de teste antigo |
| `c539418` | `f51ae18` | 2026-11-24T13:35:05-03:00 | Exportação de DOCX e PPTX no núcleo (parágrafos alterados refeitos, resto copiado), método RPC exportar, testes de round-trip |
| `78f14e2` | `111f9ad` | 2026-11-24T13:52:58-03:00 | Candidato Avalonia 12 com editor próprio; roteiro curto.json passa; esforço e diário |
| `c2f0c50` | `88adb4e` | 2026-11-24T15:02:34-03:00 | Candidato Flutter + Fleather (tags atomicas por embed inline); T-R3 verificado por fumaca; quatro candidatos no orquestrador |
| `bf8b5a3` | `51933ce` | 2026-11-24T15:41:51-03:00 | M4: rolagem por roda no driver, integração e análise do PresentMon (binário fora do git; falta privilégio ETW) |
| `1135b41` | `a87c898` | 2026-11-24T18:23:49-03:00 | M4: calibração do relógio do PresentMon, tolera linhas truncadas, limite do Avalonia documentado |
| `c6c20ed` | `46da7c1` | 2026-11-24T19:03:20-03:00 | Emenda 6: Avalonia com swap chain DXGI para o M4 (PresentMon passa a ver o app); m4.py tolera linha truncada |
| `d0040f6` | `91b709c` | 2026-11-24T20:27:13-03:00 | Testes funcionais T-R2..T-R7 por roteiro com instantaneo de estado (Ctrl+Alt+F9): Electron, Tauri, Avalonia passam; setas passam por cima de tag no ProseMirror |
| `8f8c44b` | `3069db0` | 2026-11-24T20:34:45-03:00 | Testes funcionais T-R2..T-R7: Flutter com instantaneo de estado, os quatro passam; diario e READMEs |
| `16682fe` | `89b0ca1` | 2026-11-24T20:49:05-03:00 | Argumento --tmx (TMX real) nos quatro apps; T-R7r com a TM real passa nos quatro |
| `2f282f8` | `b3e3f72` | 2026-11-24T20:53:48-03:00 | Pacotes portateis de producao dos quatro candidatos, tamanhos (M10 parcial); catcore ao lado do executavel |
| `06caaee` | `403e6cf` | 2026-11-24T21:16:37-03:00 | Instaladores NSIS dos quatro (mesmo script e compressao); M10 registrado |
| `301dedf` | `41fb009` | 2026-11-24T21:19:59-03:00 | Núcleo: método previa_compilada (exporta, LibreOffice converte a página em PNG, devolve tempos) |
| `34a2ead` | `5893208` | 2026-11-24T21:32:04-03:00 | Prévia compilada (Emenda 5, M11) integrada nos quatro apps; passo previa no driver, T-R9; validada a gravação do M11 |
| `a1c50ef` | `7f378d9` | 2026-11-24T21:53:15-03:00 | Roteiros medicao.json e uso_prolongado.json; --tmx e --docs-ciclo no orquestrador; Ctrl+Shift+O alterna documentos nos quatro apps |
| `9659d7b` | `6ffed81` | 2026-11-24T23:02:18-03:00 | Fase 3: empacotar.py, fase3.py (plano com sementes e retomada), verificação de GPU (iGPU no Restrito), previa espera o M11; medicao.json sem rolagem por roda |
| `ff163cc` | `a47d5e1` | 2026-11-27T09:06:03-03:00 | Etapa 1: dados da Fase 3, pontuação provisória e relatórios (PT/EN) |
| `396fb90` | `8086ca3` | 2026-11-27T14:18:18-03:00 | Artigo só Windows e correção sobre a busca medida |
| `2b63063` | `2b6a068` | 2026-11-27T14:46:53-03:00 | Artigo só Windows em português e inglês, com gráficos e revisão |
