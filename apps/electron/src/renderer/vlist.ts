export interface VLOpts {
  scroller: HTMLElement;
  inner: HTMLElement;
  count: () => number;
  est: (i: number) => number;
  fill: (i: number, el: HTMLElement) => void;
  cls: (i: number) => string;
  onRender?: () => void;
}

/** Lista virtualizada com altura variável: alturas estimadas, refinadas ao medir as linhas visíveis. */
export class VList {
  h: number[] = [];
  top: number[] = [0];
  els = new Map<number, HTMLElement>();
  private scheduled = false;
  constructor(private o: VLOpts) {
    o.scroller.addEventListener("scroll", () => this.schedule(), { passive: true });
  }
  private schedule() {
    if (this.scheduled) return;
    this.scheduled = true;
    requestAnimationFrame(() => {
      this.scheduled = false;
      this.render();
    });
  }
  reset() {
    for (const el of this.els.values()) el.remove();
    this.els.clear();
    const n = this.o.count();
    this.h = Array.from({ length: n }, (_, i) => this.o.est(i));
    this.recompute();
    this.render();
  }
  private recompute() {
    const n = this.h.length;
    this.top = new Array(n + 1);
    this.top[0] = 0;
    for (let i = 0; i < n; i++) this.top[i + 1] = this.top[i] + this.h[i];
    this.o.inner.style.height = this.top[n] + "px";
  }
  private find(y: number): number {
    let lo = 0,
      hi = this.h.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (this.top[mid] <= y) lo = mid;
      else hi = mid - 1;
    }
    return Math.max(0, lo);
  }
  render() {
    const n = this.h.length;
    if (!n) return;
    const sc = this.o.scroller;
    for (let pass = 0; pass < 3; pass++) {
      const first = Math.max(0, this.find(sc.scrollTop) - 4);
      const last = Math.min(n - 1, this.find(sc.scrollTop + sc.clientHeight) + 4);
      for (const [i, el] of this.els) {
        if (i < first || i > last) {
          el.remove();
          this.els.delete(i);
        }
      }
      const fresh: number[] = [];
      for (let i = first; i <= last; i++) {
        if (this.els.has(i)) continue;
        const el = document.createElement("div");
        el.className = this.o.cls(i);
        el.dataset.i = String(i);
        this.o.fill(i, el);
        this.o.inner.appendChild(el);
        this.els.set(i, el);
        fresh.push(i);
      }
      let changed = false;
      for (const i of fresh) {
        const hh = this.els.get(i)!.offsetHeight;
        if (hh && hh !== this.h[i]) {
          this.h[i] = hh;
          changed = true;
        }
      }
      if (changed) this.recompute();
      for (const [i, el] of this.els) el.style.top = this.top[i] + "px";
      if (!changed) break;
    }
    this.o.onRender?.();
  }
  /** Reconstrói uma linha já renderizada (conteúdo mudou). */
  refreshRow(i: number) {
    const el = this.els.get(i);
    if (!el) return;
    el.className = this.o.cls(i);
    this.o.fill(i, el);
    const hh = el.offsetHeight;
    if (hh && hh !== this.h[i]) {
      this.h[i] = hh;
      this.recompute();
      for (const [j, e] of this.els) e.style.top = this.top[j] + "px";
    }
  }
  scrollToIndex(i: number) {
    const sc = this.o.scroller;
    i = Math.max(0, Math.min(this.h.length - 1, i));
    const t = this.top[i],
      b = this.top[i + 1];
    if (t < sc.scrollTop) sc.scrollTop = t;
    else if (b > sc.scrollTop + sc.clientHeight) sc.scrollTop = Math.max(0, b - sc.clientHeight);
    this.render();
  }
  isVisible(i: number): boolean {
    const sc = this.o.scroller;
    return this.top[i] >= sc.scrollTop - 1 && this.top[i + 1] <= sc.scrollTop + sc.clientHeight + 1;
  }
}
