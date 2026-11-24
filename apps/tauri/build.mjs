// Front-end do Tauri: o MESMO código do renderer do Electron (apps/electron/src/renderer), com uma ponte window.cat própria.
// As dependências do ProseMirror são resolvidas a partir de apps/electron/node_modules (versões fixas em apps/electron/package.json).
import { build } from "esbuild";
import { mkdirSync, readFileSync, writeFileSync, cpSync } from "node:fs";
mkdirSync("dist", { recursive: true });
await build({ entryPoints: ["src/entry.ts"], outfile: "dist/renderer.js", bundle: true, minify: true, platform: "browser", format: "iife", target: "chrome120", logLevel: "info" });
let html = readFileSync("../electron/src/renderer/index.html", "utf8");
html = html
  .replace("default-src 'self'; style-src 'self' 'unsafe-inline'", "default-src 'self'; connect-src ipc: http://ipc.localhost; style-src 'self' 'unsafe-inline'")
  .replace("CAT Study (Electron)", "CAT Study (Tauri)");
writeFileSync("dist/index.html", html);
cpSync("../electron/src/renderer/styles.css", "dist/styles.css");
