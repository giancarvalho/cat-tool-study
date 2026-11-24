import { Editor } from "./editor";
import { Bloco, Seg, htmlLista, htmlPrevia, textLen, esc } from "./types";
import { VList } from "./vlist";
import { aposQuadro, flush, limparTecla, marcarTecla, now, registrar, teclaPendente } from "./perf";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const rpc = async (m: string, p: unknown = {}) => {
  const r = await window.cat.rpc(m, p);
  if (!r.ok) throw new Error(r.erro);
  return r;
};
const clone = <T>(x: T): T => JSON.parse(JSON.stringify(x));

let docId = 0;
let segs: Seg[] = [];
let blocos = new Map<number, Bloco>();
let blocoOrdem: number[] = [];
let blocoSegs = new Map<number, Seg[]>();
let blocoIdx = new Map<number, number>();
let active = -1;
let nome = "";
const ed = new Editor();

// ---------- modal ----------
function confirmar(msg: string): Promise<boolean> {
  return new Promise((res) => {
    $("modal-txt").textContent = msg;
    $("modal").hidden = false;
    const h = (e: KeyboardEvent) => {
      const k = e.key;
      if (k === "Enter" || k === "Escape" || /^[ysnYSN]$/.test(k)) {
        e.preventDefault();
        e.stopPropagation();
        document.removeEventListener("keydown", h, true);
        $("modal").hidden = true;
        res(k === "Enter" || /^[ysYS]$/.test(k));
      }
    };
    document.addEventListener("keydown", h, true);
  });
}
ed.confirmarTag = (r) => confirmar(`Apagar tag ${r}? (Enter = Sim, Esc = Não)`);

// ---------- lista ----------
const cpl = 54;
const lines = (n: number, c: number) => Math.max(1, Math.ceil(n / c));
const lista = new VList({
  scroller: $("lista"),
  inner: $("lista-in"),
  count: () => segs.length,
  est: (i) => 20 * Math.max(lines(textLen(segs[i].origem), cpl), lines(textLen(segs[i].destino), cpl)) + 17,
  cls: (i) => "row" + (i === active ? " ativo" : ""),
  fill: (i, el) => {
    const s = segs[i];
    el.innerHTML = `<div class="n">${i + 1}</div><div class="o">${htmlLista(s.origem)}</div><div class="d"></div>`;
    const d = el.querySelector(".d") as HTMLElement;
    if (i === active) d.appendChild(ed.mount);
    else d.innerHTML = htmlLista(s.destino);
  },
});
$("lista").addEventListener("mousedown", (e) => {
  const row = (e.target as HTMLElement).closest(".row") as HTMLElement | null;
  if (row && Number(row.dataset.i) !== active) setActive(Number(row.dataset.i), false);
});

// ---------- prévia ----------
const previa = new VList({
  scroller: $("previa"),
  inner: $("previa-in"),
  count: () => blocoOrdem.length,
  est: (i) => {
    const b = blocos.get(blocoOrdem[i])!;
    const n = (blocoSegs.get(b.id) ?? []).reduce((a, s) => a + textLen(s.destino) + 1, 0);
    return 20 * lines(n, 72) + 8 + (b.tipo === "titulo" ? 8 : 0);
  },
  cls: (i) => {
    const b = blocos.get(blocoOrdem[i])!;
    return "blk" + (b.tipo === "item_lista" ? " li" : b.tipo === "tabela" ? " tabela" : "");
  },
  fill: (i, el) => {
    const b = blocos.get(blocoOrdem[i])!;
    const ss = blocoSegs.get(b.id) ?? [];
    const act = active >= 0 ? segs[active] : null;
    const html = ss.map((s) => (s === act ? `<span class="act">${htmlPrevia(s.destino)}</span>` : htmlPrevia(s.destino))).join(" ");
    const n = Math.min(3, Math.max(1, b.nivel));
    el.innerHTML = b.tipo === "titulo" ? `<h${n}>${html}</h${n}>` : html;
  },
});

function indexBlocos() {
  blocoSegs = new Map();
  for (const s of segs) {
    const a = blocoSegs.get(s.bloco_id);
    if (a) a.push(s);
    else blocoSegs.set(s.bloco_id, [s]);
  }
  blocoIdx = new Map(blocoOrdem.map((id, i) => [id, i]));
}

const atualizarContador = () => ($("contador").textContent = `${active + 1}/${segs.length}`);
const status = (t: string) => ($("estado").textContent = t);

// ---------- ativação e edição ----------
function setActive(i: number, scroll = true) {
  if (i < 0 || i >= segs.length) return;
  const old = active;
  active = i;
  if (old >= 0 && old !== i) {
    lista.refreshRow(old);
    const ob = blocoIdx.get(segs[old].bloco_id);
    if (ob !== undefined) previa.refreshRow(ob);
  }
  ed.setSegment(segs[i].destino);
  if (scroll) lista.scrollToIndex(i);
  else lista.render();
  lista.refreshRow(i);
  const bi = blocoIdx.get(segs[i].bloco_id)!;
  previa.scrollToIndex(bi);
  previa.refreshRow(bi);
  atualizarContador();
  ed.focus();
}

ed.onChange = (v) => {
  if (active < 0) return;
  const s = segs[active];
  s.destino = v;
  previa.refreshRow(blocoIdx.get(s.bloco_id)!); // só o bloco do segmento muda
  const k = teclaPendente();
  limparTecla();
  if (!k) return;
  aposQuadro((tq) => {
    const dt = +(tq - k.t).toFixed(3);
    registrar({ metrica: "M1", passo: "digitar", t_recebe_us: Math.round(k.t * 1000), t_editor_us: Math.round(tq * 1000), valor: dt, unidade: "ms" });
    registrar({ metrica: "M2", passo: "digitar", t_recebe_us: Math.round(k.t * 1000), t_previa_us: Math.round(tq * 1000), valor: dt, unidade: "ms" });
  });
};

async function salvarRascunho() {
  const s = segs[active];
  if (JSON.stringify(s.destino) === JSON.stringify(s.base)) return;
  const r = await rpc("salvar_segmento", { id: s.id, destino: s.destino });
  s.versao = r.resultado.versao;
  s.estado = "confirmado";
  s.base = clone(s.destino);
}

async function confirmarSeg(avancar: boolean) {
  if (active < 0) return;
  const t0 = now();
  const s = segs[active];
  const r = await rpc("salvar_segmento", { id: s.id, destino: s.destino });
  s.versao = r.resultado.versao;
  s.estado = "confirmado";
  s.base = clone(s.destino);
  const t1 = now();
  if (avancar && active + 1 < segs.length) setActive(active + 1);
  else lista.refreshRow(active);
  aposQuadro((tq) => {
    registrar({ metrica: "M3", passo: "confirmar", t_recebe_us: Math.round(t0 * 1000), t_core_us: r.t_core_us, valor: +(tq - t0).toFixed(3), unidade: "ms" });
    $("ultimo").textContent = `confirmar: ${(tq - t0).toFixed(1)} ms (núcleo+SQLite ${(t1 - t0).toFixed(1)} ms)`;
  });
}

function descartar() {
  if (active < 0) return;
  const s = segs[active];
  s.destino = clone(s.base!);
  ed.setSegment(s.destino);
  previa.refreshRow(blocoIdx.get(s.bloco_id)!);
  ed.focus();
}

async function mesclar() {
  if (active < 0 || active + 1 >= segs.length || segs[active].bloco_id !== segs[active + 1].bloco_id) return status("mesclar: sem segmento seguinte no bloco");
  const t0 = now();
  await salvarRascunho();
  const r = await rpc("mesclar", { id: segs[active].id });
  const m: Seg = r.resultado;
  m.base = clone(m.destino);
  segs.splice(active, 2, m);
  recarregarViews();
  aposQuadro((tq) => registrar({ metrica: "M6", passo: "mesclar", t_recebe_us: Math.round(t0 * 1000), t_core_us: r.t_core_us, valor: +(tq - t0).toFixed(3), unidade: "ms" }));
}

async function dividir() {
  if (active < 0) return;
  const t0 = now();
  const pos = ed.atomIndex();
  await salvarRascunho();
  const r = await rpc("dividir", { id: segs[active].id, pos });
  const [a, b]: Seg[] = r.resultado;
  a.base = clone(a.destino);
  b.base = clone(b.destino);
  segs.splice(active, 1, a, b);
  recarregarViews();
  aposQuadro((tq) => registrar({ metrica: "M6", passo: "dividir", t_recebe_us: Math.round(t0 * 1000), t_core_us: r.t_core_us, valor: +(tq - t0).toFixed(3), unidade: "ms" }));
}

function recarregarViews() {
  indexBlocos();
  const keep = $("lista").scrollTop;
  lista.reset();
  previa.reset();
  $("lista").scrollTop = keep;
  ed.setSegment(segs[active].destino);
  lista.render();
  lista.refreshRow(active);
  previa.scrollToIndex(blocoIdx.get(segs[active].bloco_id)!);
  previa.refreshRow(blocoIdx.get(segs[active].bloco_id)!);
  atualizarContador();
  ed.focus();
}

// ---------- abrir documento (M5) ----------
async function abrirDoc(caminho: string) {
  const t0 = now();
  status("importando…");
  const r = await rpc("importar", { caminho });
  docId = r.resultado.doc_id;
  nome = caminho.split(/[\\/]/).pop() || caminho;
  const total = r.resultado.n_segmentos as number;
  segs = [];
  for (let de = 0; de < total; de += 2000) segs.push(...(await rpc("segmentos", { doc_id: docId, de, n: 2000 })).resultado);
  for (const s of segs) s.base = clone(s.destino);
  const bm: Bloco[] = (await rpc("blocos_meta", { doc_id: docId })).resultado;
  blocos = new Map(bm.map((b) => [b.id, b]));
  blocoOrdem = bm.map((b) => b.id);
  indexBlocos();
  $("nome").textContent = nome;
  active = -1;
  lista.reset();
  previa.reset();
  setActive(0);
  aposQuadro((tq) => {
    registrar({ metrica: "M5", passo: "abrir", t_recebe_us: Math.round(t0 * 1000), valor: +(tq - t0).toFixed(3), unidade: "ms", erro: nome });
    status(`pronto · ${segs.length} segmentos · aberto em ${(tq - t0).toFixed(0)} ms`);
    flush();
  });
}

// ---------- busca (M8) ----------
async function buscar() {
  const modo = ($("modo") as HTMLSelectElement).value;
  const texto = modo === "memoria" ? (active >= 0 ? segs[active].origem.map((x) => (x.k === "t" ? x.v : "")).join("") : "") : ($("busca") as HTMLInputElement).value;
  const t0 = now();
  const r = await rpc("buscar", { modo, texto });
  const el = $("resultados");
  el.innerHTML = r.resultado.length
    ? r.resultado.map((x: any) => `<div><b>${x.pontuacao.toFixed(2)}</b> ${esc(x.origem)}<br><i>${esc(x.destino)}</i></div>`).join("")
    : "<div>sem resultados</div>";
  el.hidden = false;
  aposQuadro((tq) => registrar({ metrica: "M8", passo: "buscar", t_recebe_us: Math.round(t0 * 1000), t_core_us: r.t_core_us, valor: +(tq - t0).toFixed(3), unidade: "ms", erro: modo }));
}

// ---------- teclado ----------
window.addEventListener(
  "keydown",
  (e) => {
    if (!$("modal").hidden) return;
    const mod = e.ctrlKey || e.metaKey;
    const k = e.key;
    if (k.length === 1 && !mod && !e.altKey && e.target !== $("busca")) marcarTecla(k);
    if (mod && k === "Enter") {
      e.preventDefault();
      confirmarSeg(!e.shiftKey);
    } else if (k === "Escape") {
      $("resultados").hidden = true;
      if (e.target !== $("busca")) descartar();
    } else if (mod && k.toLowerCase() === "m" && !e.shiftKey) {
      e.preventDefault();
      mesclar();
    } else if (mod && e.shiftKey && k.toLowerCase() === "d") {
      e.preventDefault();
      dividir();
    } else if (mod && k.toLowerCase() === "f") {
      e.preventDefault();
      ($("busca") as HTMLInputElement).focus();
    } else if (k === "PageDown" && e.target !== $("busca")) {
      e.preventDefault();
      setActive(Math.min(segs.length - 1, active + 10));
    } else if (k === "PageUp" && e.target !== $("busca")) {
      e.preventDefault();
      setActive(Math.max(0, active - 10));
    } else if (mod && k === "Home") {
      e.preventDefault();
      setActive(0);
    } else if (mod && k === "End") {
      e.preventDefault();
      setActive(segs.length - 1);
    } else if (e.altKey && k === "ArrowDown") {
      e.preventDefault();
      setActive(active + 1);
    } else if (e.altKey && k === "ArrowUp") {
      e.preventDefault();
      setActive(active - 1);
    }
  },
  true
);
$("busca").addEventListener("keydown", (e) => {
  if ((e as KeyboardEvent).key === "Enter") buscar();
});
$("b-mesclar").addEventListener("click", mesclar);
$("b-dividir").addEventListener("click", dividir);
$("b-abrir").addEventListener("click", async () => {
  const p = await window.cat.abrir();
  if (p) abrirDoc(p);
});
$("modo").addEventListener("change", () => ($("resultados").hidden = true));

window.addEventListener('error', (e) => console.log('ERR', e.message, e.filename, e.lineno));
window.addEventListener('unhandledrejection', (e) => console.log('REJ', String((e as any).reason)));
// ---------- inicialização ----------
(async () => {
  const args = await window.cat.args();
  if (args.teste) {
    window.__cat = {
      pronto: () => active >= 0,
      abrirDoc,
      estado: () => ({ active, total: segs.length, docId }),
      setActive,
      segmento: (i: number) => clone(segs[i]),
      acharComTags: (min: number) => segs.findIndex((s) => s.destino.filter((x) => x.k === "g").length >= min && s.destino[0]?.k === "t"),
      visivel: (i: number) => lista.isVisible(i),
      linhaEl: (i: number) => !!lista.els.get(i),
      edInlines: () => ed.getInlines(),
      selecao: () => ({ head: ed.view.state.selection.head, anchor: ed.view.state.selection.anchor }),
      colocarCursor: (pos: number) => {
        const st = ed.view.state;
        ed.view.dispatch(st.tr.setSelection((st.selection.constructor as any).create(st.doc, pos)));
        ed.focus();
      },
      modalAberto: () => !$("modal").hidden,
      previaHtml: (i: number) => previa.els.get(blocoIdx.get(segs[i].bloco_id)!)?.innerHTML ?? null,
      previaOutros: (i: number) =>
        [...previa.els.entries()]
          .filter(([k]) => k !== blocoIdx.get(segs[i].bloco_id))
          .map(([, e]) => e.innerHTML)
          .join("|"),
      fila: async () => (await rpc("fila", { estado: "pendente" })).resultado,
      contagem: async () => (await rpc("contagem", { doc_id: docId })).resultado,
      resultadosBusca: () => $("resultados").innerText,
      domTexto: () => ed.view.dom.textContent!.slice(0, 40) + " | doc: " + ed.view.state.doc.textContent.slice(0, 40),
      flush,
    };
  }
  if (args.doc) await abrirDoc(args.doc);
})();
