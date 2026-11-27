# Fase 3: estado em 2026-11-26 (medições concluídas, resultados provisórios)

Consolidação: `python -I harness/resultados.py` (CSVs nesta pasta). Nada daqui é pontuação: isso é a Fase 4.

## Cobertura
Os 24 itens do plano estão completos. As quatro rodadas de `medicao` afetadas pela tela parada foram refeitas em 26/11 (13:35 a 17:46)
sob as Emendas 8 e 9, sem falha nem interrupção. Em `latencias.csv`, as 308 combinações com meta estão como "atingida".

## Resultados principais (p95 em ms, faixa entre documentos)
| | Electron | Tauri | Avalonia | Flutter |
|---|---|---|---|---|
| M1 digitar, Restrito (meta < 16) | 8,6 a 9,5 | 8,8 a 10,1 | 6,8 a 8,6 | 10,9 a 12,8 |
| M1 digitar, Completo | 8,1 a 8,2 | 8,3 a 8,5 | 6,2 a 7,4 | 11,0 a 11,2 |
| M3 confirmar, Restrito (meta < 50) | 16,5 a 22,1 | 10,7 a 12,6 | 15,1 a 38,4 | 14,9 a 28,6 |
| M5 abrir pior caso, Restrito, p50 | 656 | 483 | 1117 | 595 |

## Sensibilidade à tela parada (`tela_parada.csv`; Emenda 9)
Quatro rodadas completas com a tela parada. M1 e M3 sobem cerca de 6 a 9 ms em Electron, Tauri e Avalonia; o Flutter quase não muda.
Das 112 combinações com meta, 107 atingem, 4 são limítrofes (Avalonia: um M1 e três M3) e 1 não atinge
(Tauri, Restrito, `real_texto`, M1: p95 17,1 ms contra meta de 16). Nenhuma chega perto do eliminatório de 33 ms.

## Diagnóstico da tela parada
- Rodadas de 26/11 entre 00:37 e 05:14 saíram com M1 p50 de ~12 a 13 ms em Electron, Tauri e Avalonia, contra ~4 a 6 ms antes.
- A guarda de foco do driver não é a causa (`results/_diagnostico/ab_driver/`).
- Sondas (`results/_diagnostico/tela_parada/`, M1 p50): sessão do assistente trabalhando (indicador animado), 3,7 a 5,9 ms (12);
  sessão parada, 12,0 a 12,8 ms (12); parada com janela animada de 90x90, 4,6 a 5,5 ms (6); parada sem a janela, 12,4 a 13,3 ms (6).
- O fator é haver ou não outra coisa sendo redesenhada. O mecanismo interno não foi determinado.
- Uma inferência anterior (o quadro esperaria a soltura da tecla) foi testada e descartada.
- Não se sabe o que redesenhava a tela nas sete rodadas de medição mantidas (24 e 25/11).
- Duas tentativas invalidadas pela Emenda 8 em 26/11 (12:17 e 12:41) estão em `_descartadas/`.
- M4: `f3-restrito-m4-pior_caso`, feita no período de tela parada, tem quadros equivalentes aos da execução anterior fora dele.
  As rodadas M4 do perfil Completo não têm par para essa comparação.

## Observações para o artigo e a Fase 4
- M7: Flutter cresce 7,8 MB/min (IC95 7,7 a 8,0) entre os minutos 20 e 60; os outros três não atendem ao critério de crescimento contínuo.
  Origem investigada em 26/11: ver a seção seguinte.
- M4: meta definida a 60 Hz em tela de 240 Hz. No documento pequeno a rolagem chega ao fim e os "perdidos" não medem desempenho.
- M6 mesclar: uma recusa por execução nos documentos reais, igual nos quatro (45 por app).
- M9: início isolado de 24,4 s no Electron (`f3-restrito-medicao-real_texto`, repetição 10); causa não determinada.

## Memória do Flutter: origem (26/11, dados em `results/_diagnostico/flutter_memoria/`)
Um tipo de operação por execução, ~2,5 min, crescimento da memória privada do processo do app:
parado, nenhum; digitar, ~2 KB por tecla; confirmar, ~3 KB; mesclar e dividir, ~49 KB por operação; navegar entre segmentos,
~23 MB/min (~60 KB por troca); trocar de documento, pouco no app (o núcleo `catcore`, comum aos quatro, cresceu 17 MB/min nesse teste acelerado).

Cada troca de segmento descarta um `FleatherEditor` e cria outro. No Fleather 1.28.0 (última versão publicada; o ramo principal tem as mesmas
linhas), três pontos podem reter o editor descartado: (A) observador do `WidgetsBinding` não removido no `dispose`; (B)
`addPersistentFrameCallback`, que não pode ser removido, a cada mudança de valor; (C) `ClipboardStatusNotifier` novo a cada reconstrução,
com o ouvinte preso no antigo.

Cópias do app, teste de navegação (`resumo_variantes.txt`): sem alteração 18,7 MB/min; só A 19,1; só B 19,9; só C 16,6; **A+B+C 0,8**.
- O crescimento na navegação vem do Fleather. Pares de correções não foram testados.
- Não medido: quanto a correção reduz no roteiro oficial do M7 (60 min); se o editor corrigido passa nos testes funcionais.
- O app do estudo não foi alterado.

## Sensibilidade do M7 com o Fleather corrigido (27/11, Emenda 10; `m7_corrigido.csv`)
Pacote de `harness/flutter_corrigido.py`. Testes funcionais: 7 de 7 (`results/funcional_corrigido/`). 60 minutos, 00:55 a 01:56, sem falha:
mediana 381,9 MB (oficial 710,3), máximo 397,4 (859,2), inclinação 0,014 MB/min, IC95 -0,122 a 0,151 (oficial 7,845): sem crescimento contínuo.
O resultado oficial e a pontuação continuam com a biblioteca como publicada. Fase 4: `results/fase4/` e `artigo/`.
