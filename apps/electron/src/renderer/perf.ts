// Marcações de tempo (M1..M10). O driver externo de teclas vem na etapa de harness;
// aqui registramos o lado do app: t_recebe, t_editor, t_previa (performance.now, ms).
declare global {
  interface Window {
    cat: {
      rpc(metodo: string, params: unknown): Promise<{ ok: boolean; resultado: any; erro?: string; t_core_us: number }>;
      args(): Promise<{ teste: boolean; doc?: string; script?: string; out?: string; tm?: number }>;
      abrir(): Promise<string | null>;
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

export function registrar(row: Record<string, unknown>) {
  buf.push(row);
  if (buf.length >= 200) flush();
}
export function flush() {
  if (buf.length) window.cat.log(buf.splice(0, buf.length));
}
export const registrosPendentes = () => buf.length;
