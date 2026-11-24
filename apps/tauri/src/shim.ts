// Ponte window.cat sobre os comandos do Tauri (invoke). Mesma interface do preload do Electron.
const invoke = (cmd: string, args?: Record<string, unknown>): Promise<any> => (window as any).__TAURI__.core.invoke(cmd, args);

(window as any).cat = {
  rpc: (metodo: string, params: unknown) => invoke("rpc", { metodo, params }),
  args: () => invoke("args"),
  abrir: () => invoke("abrir"),
  log: (rows: unknown[]) => void invoke("log", { rows }),
};
