import { app, BrowserWindow, dialog, ipcMain } from "electron";
import { ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { appendFileSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as readline from "node:readline";
import { rodarTestes } from "./tests";

const argv = process.argv.slice(1);
const flag = (n: string) => argv.includes(n);
const val = (n: string) => {
  const i = argv.indexOf(n);
  return i >= 0 ? argv[i + 1] : undefined;
};
const args = {
  teste: flag("--teste"),
  doc: val("--doc"),
  script: val("--script"),
  out: val("--out"),
  tm: val("--tm-sintetico") ? Number(val("--tm-sintetico")) : undefined,
  tmx: val("--tmx"),
};

let core: ChildProcessWithoutNullStreams;
let nextId = 1;
const pend = new Map<number, (r: any) => void>();

function iniciarNucleo() {
  // pacote: catcore.exe ao lado do executável; desenvolvimento: core/target/release
  const ao_lado = path.join(path.dirname(process.execPath), "catcore.exe");
  const exe = process.env.CATCORE || (existsSync(ao_lado) ? ao_lado : path.resolve(__dirname, "../../../core/target/release/catcore.exe"));
  const dir = app.getPath("userData");
  mkdirSync(dir, { recursive: true });
  const banco = path.join(dir, "catstudy.db");
  if (args.teste || flag("--limpo")) for (const f of [banco, banco + "-wal", banco + "-shm"]) if (existsSync(f)) rmSync(f);
  core = spawn(exe, ["--banco", banco]);
  readline.createInterface({ input: core.stdout }).on("line", (l) => {
    const r = JSON.parse(l);
    pend.get(r.id)?.(r);
    pend.delete(r.id);
  });
  core.stderr.on("data", (d) => process.stderr.write(d));
}

export function rpc(metodo: string, params: unknown): Promise<any> {
  return new Promise((res) => {
    const id = nextId++;
    pend.set(id, res);
    core.stdin.write(JSON.stringify({ id, metodo, params }) + "\n");
  });
}

const COLS = ["run_id", "app", "metrica", "passo", "t_recebe_us", "t_editor_us", "t_previa_us", "t_core_us", "valor", "unidade", "erro"];
function registrar(rows: any[]) {
  if (!args.out) return;
  if (!existsSync(args.out)) {
    mkdirSync(path.dirname(args.out), { recursive: true });
    writeFileSync(args.out, COLS.join(",") + "\n");
  }
  const run = String(process.pid);
  appendFileSync(args.out, rows.map((r) => COLS.map((c) => (c === "app" ? "electron" : c === "run_id" ? run : r[c] ?? "")).join(",")).join("\n") + "\n");
}

ipcMain.handle("rpc", (_e, m, p) => rpc(m, p));
ipcMain.handle("args", () => args);
ipcMain.handle("abrir", async () => {
  const r = await dialog.showOpenDialog({ filters: [{ name: "Documentos", extensions: ["docx", "xlsx"] }], properties: ["openFile"] });
  return r.canceled ? null : r.filePaths[0];
});
ipcMain.on("log", (_e, rows) => registrar(rows));

app.whenReady().then(async () => {
  iniciarNucleo();
  if (args.tm) {
    await rpc("tm_sintetico", { n: args.tm, semente: 42 });
    await rpc("tm_aquecer", {});
  }
  if (args.tmx) {
    await rpc("tm_carregar_tmx", { pasta: args.tmx, origem: "EN-GB", destino: "PT-PT" });
    await rpc("tm_aquecer", {});
  }
  const win = new BrowserWindow({
    width: 1440,
    height: 900,
    useContentSize: true,
    resizable: false,
    show: true,
    autoHideMenuBar: true,
    webPreferences: { preload: path.join(__dirname, "preload.js"), contextIsolation: true, sandbox: true, devTools: false },
  });
  win.removeMenu();
  if (process.env.CAT_DEBUG) win.webContents.on("console-message", (_e, _l, m) => console.log("[renderer]", m));
  await win.loadFile(path.join(__dirname, "index.html"));
  if (args.teste) {
    const ok = await rodarTestes(win, args.doc);
    app.exit(ok ? 0 : 1);
  }
});
app.on("window-all-closed", () => {
  core?.stdin.write(JSON.stringify({ id: 0, metodo: "sair" }) + "\n");
  app.quit();
});
