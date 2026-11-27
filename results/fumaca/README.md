# Teste de fumaça do Electron no macOS e no Ubuntu

27 de novembro de 2026. O app Electron do estudo (`apps/electron/`) compila, passa nos casos funcionais e vira pacote que abre no macOS e no Ubuntu, em runners gratuitos do GitHub Actions. O Windows entra na mesma matriz como controle.

Não é medição nem a Etapa 2 do estudo. O pré-registro e o artigo não mudam.

Workflow: [`.github/workflows/fumaca-electron.yml`](../../.github/workflows/fumaca-electron.yml). PRs: [1](https://github.com/giancarvalho/cat-tool-study/pull/1), [2](https://github.com/giancarvalho/cat-tool-study/pull/2), [3](https://github.com/giancarvalho/cat-tool-study/pull/3), [4](https://github.com/giancarvalho/cat-tool-study/pull/4).

## 1. Resultado por sistema

| | Windows (controle) | macOS | Ubuntu 24.04 |
|---|---|---|---|
| Núcleo compila | sim | sim | sim |
| Testes do núcleo (`cargo test`) | 10 de 10 | 10 de 10 | 10 de 10 |
| T-R2 a T-R7, desenvolvimento (X11 virtual no Ubuntu) | passam | passam | passam |
| Pacote gerado | não se aplica | `.zip` com o `.app` | `.AppImage` e `.deb` |
| T-R2 a T-R7 no pacote | não se aplica | passam (`.zip`) | passam (`.AppImage` e `.deb`) |
| Tamanho do pacote | não se aplica | `.zip`: 122,1 MiB | `.AppImage`: 121,4 MiB; `.deb`: 96,2 MiB |
| T-R2 a T-R7 em Wayland | não se aplica | não se aplica | passam (versão de desenvolvimento) |
| Execução | [46053540530](https://github.com/giancarvalho/cat-tool-study/actions/runs/46053540530) | [46053540530](https://github.com/giancarvalho/cat-tool-study/actions/runs/46053540530) | [46053540530](https://github.com/giancarvalho/cat-tool-study/actions/runs/46053540530) |

Uma única execução cobre a tabela inteira (commit `f92dd39`, passo 4).

Os tamanhos são dos arquivos de pacote, não do app instalado, e não se comparam com os pacotes do Windows do estudo, feitos por outro método (`harness/empacotar.py`).

Os testes rodam com `--tm-sintetico 100000` e com o corpus sintético de `corpus/gen_corpus.py`. Nos pacotes, rodam sem `CATCORE`, para o app usar o núcleo que vai dentro do pacote, e fora da pasta do repositório (o `.zip` é extraído numa pasta temporária; o `.deb` é instalado em `/opt`).

## 2. Versões

| | Imagem do runner | Arquitetura |
|---|---|---|
| `windows-latest` | `win25-vs2026` 20261123.321.1 | x86_64 |
| `macos-latest` | `macos26` 20261117.0516.1 | arm64 |
| `ubuntu-24.04` | `ubuntu24` 20261122.23.1 | x86_64 |

São as imagens da execução da tabela. Outras execuções do mesmo dia no macOS receberam a `macos26` 20261123.0526.1: o GitHub não fixa a versão da imagem.

Nos três: Electron 44.4.5, Rust 1.99.1, Node 22.23.4 e Python 3.12.10, as mesmas versões do Windows. Nenhuma precisou mudar. Ferramenta nova: `electron-builder` 26.18.0, só para os pacotes de macOS e Linux.

## 3. Mudanças de código

| Arquivo | Mudança | Motivo |
|---|---|---|
| `apps/electron/src/main.ts` | Procura `catcore.exe` no Windows e `catcore` nos demais: em `CATCORE`, ao lado do executável, em `process.resourcesPath` e em `core/target/release` | O nome do binário muda com o sistema, e no pacote o núcleo fica em `resources` |
| `apps/electron/src/tests.ts` | Variável `CORPUS_DIR` | O caminho relativo ao código não existe dentro do pacote |
| `apps/electron/src/tests.ts` | Variável `TEST_RESULT` | A pasta do código no pacote é somente leitura, e o resultado precisa ser gravado fora dela |
| `apps/electron/src/renderer/vlist.ts` | `scrollToIndex` reajusta a rolagem enquanto a medição das linhas tirar a linha de destino da vista | Correção de um erro exposto no Ubuntu (ver abaixo). **Muda o comportamento do app; aprovada pelo Gian** |
| `apps/electron/electron-builder.yml`, `package.json`, `package-lock.json` | Configuração e dependência de empacotamento | Gerar `.zip`, `.AppImage` e `.deb` com o `catcore` do sistema |
| `apps/electron/.gitignore`, `apps/electron/README.md` | Ignora `release/`; documenta o acima | |

Sem as variáveis novas, os caminhos no Windows são os mesmos de antes.

As medições do estudo foram feitas com o código anterior à mudança em `vlist.ts`. Ela não altera a posição final quando a linha de destino já cabe na tela depois da primeira rolagem; nos testes locais no Windows, com Segoe UI, a geometria após Ctrl+End ficou igual à de antes. O desempenho não foi medido de novo.

### Configuração dos runners (não muda o app)

| Sistema | Configuração |
|---|---|
| Ubuntu | `sysctl kernel.apparmor_restrict_unprivileged_userns=0`, seguido de `unshare -Ur true` para falhar se não valer |
| Ubuntu | `xvfb-run` com tela de 1920x1080 |
| Windows | `Set-DisplayResolution` de 1024x768 para 1920x1080 |

### Wayland

No Ubuntu, `npm test` rodou dentro de um Weston 13.0.0 sem tela (`weston --backend=headless --renderer=pixman --width=1920 --height=1080`), com `--ozone-platform=wayland` e sem a variável `DISPLAY`, para o app não ter como voltar ao X11. T-R2 a T-R7 passaram na primeira tentativa. O log traz avisos que não afetaram os testes: nenhum dispositivo DRM (`drmGetDevices2`), cursores de arrastar ausentes, uma asserção do GDK sobre o teclado e falhas de conexão com o D-Bus.

O passo usa `continue-on-error`: uma falha em Wayland não deixa o workflow vermelho. O resultado aparece no aviso "wayland" da execução.

## 4. Problemas encontrados

| Problema | Onde | Causa | Solução |
|---|---|---|---|
| Electron abortava ao abrir (`The SUID sandbox helper binary was found, but is not configured correctly`) | Ubuntu, [primeira execução](https://github.com/giancarvalho/cat-tool-study/actions/runs/46048895870) | O AppArmor do Ubuntu 24.04 restringe namespaces de usuário, e o `chrome-sandbox` do `node_modules` não tem dono `root` nem modo 4755 | Liberar os namespaces no runner. O app não roda com `--no-sandbox` |
| T-R2 falhava: "último não visível" | Windows, primeira execução | A tela do runner (1024x768) é menor que a janela do app (1440x900) | Tela de 1920x1080 no runner |
| T-R2 falhava com a mesma mensagem, já com tela de 1920x1080 | Ubuntu, [segunda execução](https://github.com/giancarvalho/cat-tool-study/actions/runs/46050148077) | `scrollToIndex` calculava a rolagem com alturas estimadas e não a reajustava depois de medir. Com fonte mais larga que a Segoe UI, o último segmento ficava abaixo da área visível. Reproduzido no Windows trocando a fonte por Verdana (cerca de 20 px de sobra) | O agente parou e consultou o Gian; correção em `vlist.ts` |
| Passo "Resumo" falhava | macOS | O `paste` do macOS exige `-` para ler a entrada padrão | Corrigido no workflow |
| Pacotes não eram gerados quando o teste de desenvolvimento falhava | Ubuntu | Condição dos passos do workflow | Passos de pacote rodam mesmo após a falha |
| T-R7 falha com `npm test` sem memória de tradução | todos (visto no Windows local) | A busca não encontra nada numa memória vazia | O CI roda `npm test -- --tm-sintetico 100000` |
| O `electron-builder` põe `--no-sandbox` no atalho `.desktop` do AppImage por padrão | Linux | Padrão da ferramenta | `executableArgs: []` |

## 5. O que não foi coberto

- Medições de latência e memória.
- Assinatura e notarização no macOS. O teste executa o binário dentro do `.app` direto; abrir um arquivo baixado pelo Finder, com o Gatekeeper, não foi testado.
- T-R9 (prévia compilada com LibreOffice).
- IME, mouse e área de transferência. As teclas dos testes vêm do Chromium (`sendInputEvent`), não do sistema operacional.
- Placas NVIDIA e Macs com processador Intel.
- **Ubuntu com a restrição de namespaces ativa**, que é o padrão do 24.04. Todos os testes do Ubuntu rodaram com ela desligada:
  - o `.deb` instala um perfil do AppArmor, mas o teste não mostra que ele basta com a restrição ativa;
  - o script de inicialização do `.AppImage`, gerado pelo `electron-builder`, acrescenta `--no-sandbox` sozinho quando `unshare -Ur true` falha. Num sistema com a restrição, o AppImage pode abrir **sem o sandbox do Chromium**.
- Montagem do `.AppImage` com FUSE (o teste usa `--appimage-extract-and-run`).
- Ícone próprio nos pacotes: usam o ícone padrão do Electron.
- Wayland além do que está acima: só a versão de desenvolvimento (não os pacotes), só no Weston, com renderização por software. GNOME, KDE e placa de vídeo real não foram testados.
