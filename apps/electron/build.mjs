import { build } from "esbuild";
import { cpSync, mkdirSync } from "node:fs";
mkdirSync("dist", { recursive: true });
const common = { bundle: true, sourcemap: false, minify: true, logLevel: "info" };
await build({ ...common, entryPoints: ["src/main.ts"], outfile: "dist/main.js", platform: "node", target: "node22", external: ["electron"] });
await build({ ...common, entryPoints: ["src/preload.ts"], outfile: "dist/preload.js", platform: "node", target: "node22", external: ["electron"] });
await build({ ...common, entryPoints: ["src/renderer/app.ts"], outfile: "dist/renderer.js", platform: "browser", format: "iife", target: "chrome140" });
cpSync("src/renderer/index.html", "dist/index.html");
cpSync("src/renderer/styles.css", "dist/styles.css");
