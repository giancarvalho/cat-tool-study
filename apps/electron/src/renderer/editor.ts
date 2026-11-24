import { Schema, Node as PMNode } from "prosemirror-model";
import { EditorState, Plugin, TextSelection, Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { Inline, chipLabel } from "./types";

type TagInl = Extract<Inline, { k: "g" }>;

const schema = new Schema({
  nodes: {
    doc: { content: "para" },
    para: { content: "inline*", group: "block", toDOM: () => ["p", 0] },
    text: { group: "inline" },
    tag: {
      group: "inline",
      inline: true,
      atom: true,
      selectable: true,
      draggable: false,
      attrs: { id: {}, tipo: {}, rotulo: {} },
      parseDOM: [
        {
          tag: "span.chip[data-tag-id]",
          getAttrs: (d) => {
            const e = d as HTMLElement;
            return { id: Number(e.dataset.tagId), tipo: e.dataset.tagTipo, rotulo: e.dataset.tagRotulo };
          },
        },
      ],
      toDOM: (n) => [
        "span",
        { class: "chip", contenteditable: "false", "data-tag-id": String(n.attrs.id), "data-tag-tipo": n.attrs.tipo, "data-tag-rotulo": n.attrs.rotulo },
        chipLabel({ k: "g", ...(n.attrs as any) } as TagInl),
      ],
    },
  },
});

const isOpen = (a: any) => a.tipo === "abre" || (a.tipo === "link" && !String(a.rotulo).startsWith("/"));
const isClose = (a: any) => a.tipo === "fecha" || (a.tipo === "link" && String(a.rotulo).startsWith("/"));

export function toDoc(v: Inline[]): PMNode {
  const nodes: PMNode[] = [];
  for (const n of v) {
    if (n.k === "t") {
      if (n.v) nodes.push(schema.text(n.v));
    } else nodes.push(schema.nodes.tag.create({ id: n.id, tipo: n.tipo, rotulo: n.rotulo }));
  }
  return schema.node("doc", null, [schema.node("para", null, nodes)]);
}

export function fromDoc(doc: PMNode): Inline[] {
  const out: Inline[] = [];
  doc.firstChild!.forEach((c) => {
    if (c.isText) {
      const last = out[out.length - 1];
      if (last && last.k === "t") last.v += c.text!;
      else out.push({ k: "t", v: c.text! });
    } else out.push({ k: "g", id: c.attrs.id, tipo: c.attrs.tipo, rotulo: c.attrs.rotulo });
  });
  return out;
}

const countTags = (doc: PMNode) => {
  let n = 0;
  doc.descendants((x) => {
    if (x.type === schema.nodes.tag) n++;
  });
  return n;
};

const cp = (s: string) => Array.from(s).length;

export class Editor {
  view: EditorView;
  mount: HTMLElement;
  onChange: (v: Inline[]) => void = () => {};
  confirmarTag: (rotulo: string) => Promise<boolean> = async () => true;

  constructor() {
    this.mount = document.createElement("div");
    const guard = new Plugin({
      filterTransaction: (tr, state) => {
        if (!tr.docChanged || tr.getMeta("allowTagDelete")) return true;
        return countTags(tr.doc) >= countTags(state.doc);
      },
    });
    this.view = new EditorView(
      { mount: this.mount },
      {
        state: EditorState.create({ doc: toDoc([]), plugins: [guard] }),
        dispatchTransaction: (tr: Transaction) => {
          const st = this.view.state.apply(tr);
          this.view.updateState(st);
          if (tr.docChanged) this.onChange(fromDoc(st.doc));
        },
        handleKeyDown: (_v, ev) => this.key(ev),
        attributes: { spellcheck: "false" },
      }
    );
  }

  setSegment(v: Inline[]) {
    this.view.updateState(EditorState.create({ doc: toDoc(v), plugins: this.view.state.plugins }));
  }
  getInlines(): Inline[] {
    return fromDoc(this.view.state.doc);
  }
  focus() {
    this.view.focus();
  }
  cursorAtStart() {
    this.view.dispatch(this.view.state.tr.setSelection(TextSelection.atStart(this.view.state.doc)));
  }

  /** Posição do cursor em átomos (caractere ou tag), como o núcleo conta. */
  atomIndex(): number {
    const { doc, selection } = this.view.state;
    let off = selection.head - 1,
      count = 0;
    doc.firstChild!.forEach((c) => {
      if (off <= 0) return;
      if (c.isText) {
        if (off >= c.nodeSize) count += cp(c.text!);
        else count += cp(c.text!.slice(0, off));
        off -= c.nodeSize;
      } else {
        count += 1;
        off -= 1;
      }
    });
    return count;
  }

  private items(): ("tag" | "ws" | "ch")[] {
    const out: ("tag" | "ws" | "ch")[] = [];
    this.view.state.doc.firstChild!.forEach((c) => {
      if (c.isText) for (const ch of c.text!) for (let k = 0; k < ch.length; k++) out.push(/\s/.test(ch) ? "ws" : "ch");
      else out.push("tag");
    });
    return out;
  }

  /** Navegação por palavra: tag é limite (para antes e depois dela, nunca entra nem pula). */
  private wordMove(dir: 1 | -1, extend: boolean) {
    const it = this.items(),
      n = it.length;
    const { selection, doc } = this.view.state;
    let k = selection.head - 1;
    if (dir === 1) {
      if (k >= n) return;
      if (it[k] === "tag") k++;
      else {
        while (k < n && it[k] === "ws") k++;
        while (k < n && it[k] === "ch") k++;
      }
    } else {
      if (k <= 0) return;
      if (it[k - 1] === "tag") k--;
      else {
        while (k > 0 && it[k - 1] === "ws") k--;
        while (k > 0 && it[k - 1] === "ch") k--;
      }
    }
    const pos = k + 1;
    const sel = extend ? TextSelection.create(doc, selection.anchor, pos) : TextSelection.create(doc, pos);
    this.view.dispatch(this.view.state.tr.setSelection(sel).scrollIntoView());
  }

  private key(ev: KeyboardEvent): boolean {
    const mod = ev.ctrlKey || ev.metaKey;
    if (mod && !ev.altKey && (ev.key === "ArrowLeft" || ev.key === "ArrowRight") || (ev.altKey && !ev.ctrlKey && navigator.platform.startsWith("Mac") && (ev.key === "ArrowLeft" || ev.key === "ArrowRight"))) {
      this.wordMove(ev.key === "ArrowRight" ? 1 : -1, ev.shiftKey);
      return true;
    }
    // Seta simples ao lado de uma tag: passa por cima dela como 1 unidade (o ProseMirror, por padrão, só a seleciona).
    if ((ev.key === "ArrowLeft" || ev.key === "ArrowRight") && !mod && !ev.altKey && !ev.shiftKey) {
      const { selection, tr } = this.view.state;
      const dir = ev.key === "ArrowRight" ? 1 : -1;
      if (selection instanceof TextSelection && selection.empty) {
        const n = dir === 1 ? selection.$from.nodeAfter : selection.$from.nodeBefore;
        if (n && n.type === schema.nodes.tag) {
          this.view.dispatch(tr.setSelection(TextSelection.create(this.view.state.doc, selection.from + dir)).scrollIntoView());
          return true;
        }
      }
    }
    if (ev.key === "Backspace" || ev.key === "Delete") {
      this.tryDeleteTags(ev.key === "Backspace");
      return this.pendingDelete;
    }
    return false;
  }

  private pendingDelete = false;
  private tryDeleteTags(back: boolean) {
    this.pendingDelete = false;
    const { doc, selection } = this.view.state;
    let from = selection.from,
      to = selection.to;
    if (selection.empty) {
      const n = back ? selection.$from.nodeBefore : selection.$from.nodeAfter;
      if (!n || n.type !== schema.nodes.tag) return;
      if (back) from -= 1;
      else to += 1;
    }
    const found: { pos: number; node: PMNode }[] = [];
    doc.nodesBetween(from, to, (n, p) => {
      if (n.type === schema.nodes.tag) found.push({ pos: p, node: n });
    });
    if (!found.length) return;
    this.pendingDelete = true;
    const first = found[0].node.attrs;
    this.confirmarTag(chipLabel({ k: "g", ...first } as TagInl)).then((ok) => {
      this.pendingDelete = false;
      if (ok) this.deleteTags(found, selection.empty ? null : [from, to]);
      this.view.focus();
    });
  }

  private deleteTags(found: { pos: number; node: PMNode }[], range: [number, number] | null) {
    const ranges = new Map<number, number>(); // pos -> end
    if (range) ranges.set(range[0], range[1]);
    const doc = this.view.state.doc;
    for (const f of found) {
      ranges.set(f.pos, f.pos + 1);
      const a = f.node.attrs;
      doc.descendants((n, p) => {
        if (n.type === schema.nodes.tag && n.attrs.id === a.id && p !== f.pos) {
          const par = (isOpen(a) && isClose(n.attrs)) || (isClose(a) && isOpen(n.attrs));
          if (par && !ranges.has(p)) ranges.set(p, p + 1);
        }
      });
    }
    const sorted = [...ranges.entries()].sort((x, y) => y[0] - x[0]);
    let tr = this.view.state.tr;
    let lastFrom = Infinity;
    for (const [s, e] of sorted) {
      if (e > lastFrom) continue; // sobreposto a um intervalo já removido
      tr = tr.delete(s, e);
      lastFrom = s;
    }
    tr.setMeta("allowTagDelete", true);
    this.view.dispatch(tr);
  }
}
