// Casos funcionais T-R2 a T-R7 do Electron (teclas via sendInputEvent, que passa pelo pipeline de entrada do Chromium).
// O roteiro de medição com teclas do sistema operacional (driver externo) é outra etapa (harness).
import type { BrowserWindow } from "electron";
import { writeFileSync } from "node:fs";
import * as path from "node:path";

// CORPUS_DIR: pasta do corpus quando o app está empacotado (o caminho relativo ao código não existe no pacote).
const corpus = (n: string) => path.resolve(process.env.CORPUS_DIR || path.resolve(__dirname, "../../../corpus/out"), n);
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function rodarTestes(win: BrowserWindow, _doc?: string): Promise<boolean> {
  const wc = win.webContents;
  const js = (c: string) => wc.executeJavaScript(c, true);
  const key = async (k: string, mods: ("control" | "shift" | "alt")[] = [], wait = 40) => {
    wc.sendInputEvent({ type: "keyDown", keyCode: k, modifiers: mods });
    if (k.length === 1 && !mods.includes("control") && !mods.includes("alt")) wc.sendInputEvent({ type: "char", keyCode: k, modifiers: mods });
    wc.sendInputEvent({ type: "keyUp", keyCode: k, modifiers: mods });
    await sleep(wait);
  };
  const digitar = async (s: string) => {
    for (const c of s) {
      wc.sendInputEvent({ type: "keyDown", keyCode: c });
      await wc.insertText(c);
      wc.sendInputEvent({ type: "keyUp", keyCode: c });
      await sleep(40);
    }
  };
  const abrir = async (nome: string) => {
    await js(`window.__cat.abrirDoc(${JSON.stringify(corpus(nome))})`);
    for (let i = 0; i < 100 && !(await js("window.__cat.pronto()")); i++) await sleep(50);
    await sleep(200);
    win.show();
    win.focus();
    wc.focus();
  };
  const resultados: { caso: string; ok: boolean; detalhe: string }[] = [];
  const caso = async (nome: string, fn: () => Promise<string | void>) => {
    try {
      const d = (await fn()) || "";
      resultados.push({ caso: nome, ok: true, detalhe: d });
      console.log("PASSOU ", nome, d);
    } catch (e: any) {
      resultados.push({ caso: nome, ok: false, detalhe: String(e.message || e) });
      console.log("FALHOU ", nome, String(e.message || e));
    }
  };
  const ok = (c: boolean, m: string) => {
    if (!c) throw new Error(m);
  };
  const tagsNoEditor = async (): Promise<number> => js(`window.__cat.edInlines().filter(x=>x.k==='g').length`);
  const posPrimeiraTag = async (): Promise<number> =>
    js(`(()=>{let p=1;for(const x of window.__cat.edInlines()){if(x.k==='g')return p;p+=x.v.length}return -1})()`);

  // T-R2
  await caso("T-R2 lista 10 mil segmentos", async () => {
    await abrir("pior_caso.docx");
    ok((await js("window.__cat.estado().total")) === 10000, "total != 10000");
    await key("End", ["control"], 300);
    ok((await js("window.__cat.estado().active")) === 9999, "Ctrl+End não foi ao último");
    ok(await js("window.__cat.visivel(9999)"), "último não visível");
    await key("Home", ["control"], 300);
    ok((await js("window.__cat.estado().active")) === 0 && (await js("window.__cat.visivel(0)")), "Ctrl+Home falhou");
    await key("PageDown", [], 200);
    ok((await js("window.__cat.estado().active")) === 10, "PgDn não avançou 10");
  });

  await abrir("pequeno.docx");
  const idx: number = await js("window.__cat.acharComTags(2)");
  ok(idx >= 0, "sem segmento com 2 tags");

  await caso("T-R3 tags atômicas e confirmação de apagar", async () => {
    await js(`window.__cat.setActive(${idx})`);
    await sleep(150);
    wc.focus();
    const antes = await tagsNoEditor();
    const p = await posPrimeiraTag();
    await js(`window.__cat.colocarCursor(${p})`);
    await key("Right");
    ok((await js("window.__cat.selecao().head")) === p + 1, "seta não atravessou a tag como 1 unidade");
    await key("Backspace");
    ok(await js("window.__cat.modalAberto()"), "Backspace sobre tag não abriu confirmação");
    await key("Escape");
    ok(!(await js("window.__cat.modalAberto()")) && (await tagsNoEditor()) === antes, "Não deveria apagar");
    await key("Backspace");
    await key("Enter");
    ok((await tagsNoEditor()) === antes - 2, `Sim deveria apagar a tag e a pareada (antes ${antes}, depois ${await tagsNoEditor()})`);
    await key("Escape"); // descarta
    ok((await tagsNoEditor()) === antes, "Esc não descartou");
    // navegação por palavra: para antes e depois da tag
    await js(`window.__cat.colocarCursor(1)`);
    const heads: number[] = [];
    for (let i = 0; i < 40; i++) {
      await key("Right", ["control"], 20);
      const h = await js("window.__cat.selecao().head");
      if (heads[heads.length - 1] === h) break;
      heads.push(h);
    }
    ok(heads.includes(p) && heads.includes(p + 1), `Ctrl+→ deveria parar antes (${p}) e depois (${p + 1}) da tag; paradas: ${heads.join(",")}`);
    return `tags=${antes}, paradas=${heads.length}`;
  });

  await caso("T-R4 prévia incremental", async () => {
    await js(`window.__cat.setActive(${idx})`);
    await sleep(150);
    wc.focus();
    await key("End", ["control"], 40); // fim do doc do editor: Ctrl+End é do app (último segmento); volta
    await js(`window.__cat.setActive(${idx})`);
    await sleep(150);
    await js(`window.__cat.colocarCursor(1)`);
    const outros = await js(`window.__cat.previaOutros(${idx})`);
    await digitar("qz");
    await sleep(150);
    const html: string = await js(`window.__cat.previaHtml(${idx})`);
    ok(html.includes("qz"), "prévia não mostrou o texto digitado");
    ok((await js(`window.__cat.previaOutros(${idx})`)) === outros, "outros blocos da prévia mudaram");
  });

  await caso("T-R5 confirmar grava versão e fila", async () => {
    await js(`window.__cat.setActive(${idx})`);
    await sleep(100);
    const f0 = await js("window.__cat.fila()");
    await digitar("a");
    await key("Enter", ["control"], 300);
    const f1 = await js("window.__cat.fila()");
    ok(f1 === f0 + 1, `fila ${f0} -> ${f1}`);
    ok((await js("window.__cat.estado().active")) === idx + 1, "não avançou para o próximo");
    const v = await js(`window.__cat.segmento(${idx}).versao`);
    ok(v >= 1, "versão não incrementou");
  });

  await caso("T-R6 mesclar e dividir", async () => {
    const i: number = await js(`(()=>{for(let i=0;i<100;i++){if(window.__cat.segmento(i).bloco_id===window.__cat.segmento(i+1).bloco_id)return i}return -1})()`);
    ok(i >= 0, "sem vizinhos no mesmo bloco");
    await js(`window.__cat.setActive(${i})`);
    await sleep(100);
    wc.focus();
    const n0 = await js("window.__cat.contagem()");
    await key("M", ["control"], 400);
    ok((await js("window.__cat.contagem()")) === n0 - 1, "mesclar não reduziu a contagem");
    ok((await js("window.__cat.estado().total")) === n0 - 1, "lista não refletiu");
    await js(`window.__cat.colocarCursor(10)`);
    await key("D", ["control", "shift"], 400);
    ok((await js("window.__cat.contagem()")) === n0, "dividir não restaurou a contagem");
    ok((await js("window.__cat.estado().total")) === n0, "lista não refletiu o dividir");
  });

  await caso("T-R7 busca na memória", async () => {
    await js(`(()=>{const b=document.getElementById('busca');b.focus();b.value='system record'})()`);
    await key("Enter", [], 300);
    const t: string = await js("window.__cat.resultadosBusca()");
    ok(t.length > 5 && !t.includes("sem resultados"), `sem resultados de concordância: ${t.slice(0, 60)}`);
  });

  const todos = resultados.every((r) => r.ok);
  // TEST_RESULT: caminho do resultado quando o app está empacotado (a pasta do código é somente leitura).
  writeFileSync(process.env.TEST_RESULT || path.resolve(__dirname, "../test-result.json"),JSON.stringify({ quando: new Date().toISOString(), resultados }, null, 2));
  console.log(todos ? "TODOS PASSARAM" : "HÁ FALHAS");
  return todos;
}
