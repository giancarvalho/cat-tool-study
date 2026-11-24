export type Inline =
  | { k: "t"; v: string; b?: boolean; i?: boolean }
  | { k: "g"; id: number; tipo: "abre" | "fecha" | "vazia" | "link"; rotulo: string };

export interface Seg {
  id: number;
  bloco_id: number;
  ordem: number;
  origem: Inline[];
  destino: Inline[];
  estado: string;
  versao: number;
  pos: unknown;
  base?: Inline[]; // último destino confirmado (Esc descarta para ele)
}

export interface Bloco {
  id: number;
  tipo: string;
  nivel: number;
  meta: unknown;
}

export const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

export function chipLabel(n: Extract<Inline, { k: "g" }>): string {
  if (n.tipo === "abre") return String(n.id);
  if (n.tipo === "fecha") return "/" + n.id;
  return n.rotulo;
}

/** Chips para a lista (todos os tags viram chips). */
export function htmlLista(v: Inline[]): string {
  let h = "";
  for (const n of v) h += n.k === "t" ? esc(n.v) : `<span class="chip">${esc(chipLabel(n))}</span>`;
  return h;
}

/** Prévia: b/i/link viram estilo; demais tags (br) viram chips pequenos (spec §2). */
export function htmlPrevia(v: Inline[]): string {
  let h = "";
  for (const n of v) {
    if (n.k === "t") {
      h += esc(n.v);
      continue;
    }
    const r = n.rotulo;
    if (r === "b") h += "<b>";
    else if (r === "/b") h += "</b>";
    else if (r === "i") h += "<i>";
    else if (r === "/i") h += "</i>";
    else if (r === "link") h += "<a>";
    else if (r === "/link") h += "</a>";
    else h += `<span class="chip chip-p">${esc(r)}</span>`;
  }
  return h;
}

export function textLen(v: Inline[]): number {
  let n = 0;
  for (const x of v) n += x.k === "t" ? x.v.length : 3;
  return n;
}
