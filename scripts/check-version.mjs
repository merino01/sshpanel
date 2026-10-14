// Comprueba que la versión es la misma en Cargo.toml, tauri.conf.json y
// package.json y, si se da, que coincide con la etiqueta (`v0.2.0`).
// Uso: node scripts/check-version.mjs [etiqueta]
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");

const versions = {
  "Cargo.toml": parse(read("Cargo.toml")).workspace.package.version,
  "src-tauri/tauri.conf.json": JSON.parse(read("src-tauri/tauri.conf.json")).version,
  "package.json": JSON.parse(read("package.json")).version,
};
const tag = process.argv[2];
if (tag) versions[`etiqueta ${tag}`] = tag.replace(/^v/, "");

const distinct = new Set(Object.values(versions));
if (distinct.size !== 1) {
  for (const [where, version] of Object.entries(versions)) console.error(`${where}: ${version}`);
  console.error("Las versiones no coinciden.");
  process.exit(1);
}
console.log(`Versión ${[...distinct][0]}`);
