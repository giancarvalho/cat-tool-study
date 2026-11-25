// Marcações de tempo (M1..M10). O driver externo de teclas vem na etapa de harness;
// aqui registramos o lado do app: t_recebe, t_editor, t_previa (performance.now, ms).
declare global {
  interface Window {
    cat: {
      rpc(metodo: string, params: unknown): Promise<{ ok: boolean; resultado: any; erro?: string; t_core_us: number }>;
      args(): Promise<{ teste: boolean; doc?: string; docs?: string; script?: string; out?: string; tm?: number }>;
      abrir(): Promise<string | null>;
      urlArquivo(caminho: string): string;
      log(rows: unknown[]): void;
    };
    __cat?: any;
  }
}

const buf: Record<string, unknown>[] = [];
let pendingKey: { t: number; k: string } | null = null;

export const now = () => performance.now();
export const marcarTecla = (k: string) => (pendingKey = { t: now(), k });
export const teclaPendente = () => pendingKey;
export const limparTecla = () => (pendingKey = null);

/** Executa fn depois do quadro seguinte à mudança (rAF duplo). */
export function aposQuadro(fn: (t: number) => void) {
  requestAnimationFrame(() => requestAnimationFrame(() => fn(now())));
}

// Os instantes saem em tempo de parede UTC (µs), o mesmo relógio do driver (spec/SCRIPT.md §1).
const ORIGEM_US = performance.timeOrigin * 1000;
const CAMPOS_T = ["t_recebe_us", "t_editor_us", "t_previa_us"];
export function registrar(row: Record<string, unknown>) {
  for (const c of CAMPOS_T) if (typeof row[c] === "number") row[c] = Math.round((row[c] as number) + ORIGEM_US);
  buf.push(row);
  if (buf.length >= 200) flush();
}
export function flush() {
  if (buf.length) window.cat.log(buf.splice(0, buf.length));
}
setInterval(flush, 500); // não perder linhas quando o driver encerra o app
export const registrosPendentes = () => buf.length;
