import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("cat", {
  rpc: (metodo: string, params: unknown) => ipcRenderer.invoke("rpc", metodo, params),
  args: () => ipcRenderer.invoke("args"),
  abrir: () => ipcRenderer.invoke("abrir"),
  urlArquivo: (p: string) => "file:///" + p.replace(/\\/g, "/"),
  log: (rows: unknown[]) => ipcRenderer.send("log", rows),
});
