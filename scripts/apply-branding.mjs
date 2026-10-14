// Vuelca branding.toml en la configuración de Tauri, la interfaz y los iconos.
// Ejecutar con `npm run branding` tras editar branding.toml.
import { execFileSync } from "node:child_process";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const branding = parse(readFileSync(resolve(root, "branding.toml"), "utf8"));

const required = ["name", "identifier", "profile_extension", "icon", "accent_color"];
for (const key of required) {
  if (typeof branding[key] !== "string" || branding[key].trim() === "") {
    throw new Error(`branding.toml: falta el campo "${key}"`);
  }
}
if (!/^[a-z0-9]+$/.test(branding.profile_extension)) {
  throw new Error("branding.toml: profile_extension solo admite minúsculas y dígitos");
}
if (!/^#[0-9a-fA-F]{6}$/.test(branding.accent_color)) {
  throw new Error("branding.toml: accent_color debe ser #rrggbb");
}

const confPath = resolve(root, "src-tauri/tauri.conf.json");
const conf = JSON.parse(readFileSync(confPath, "utf8"));
conf.productName = branding.name;
conf.identifier = branding.identifier;
conf.app.windows[0].title = branding.name;
conf.bundle.fileAssociations = [
  {
    ext: [branding.profile_extension],
    name: `Perfil de ${branding.name}`,
    description: `Perfil de ${branding.name}`,
    role: "Editor",
  },
];
writeFileSync(confPath, JSON.stringify(conf, null, 2) + "\n");

const generated = `// Generado por scripts/apply-branding.mjs desde branding.toml. No editar.
export const branding = {
  name: ${JSON.stringify(branding.name)},
  profileExtension: ${JSON.stringify(branding.profile_extension)},
  accentColor: ${JSON.stringify(branding.accent_color)},
} as const;
`;
writeFileSync(resolve(root, "ui/src/branding.generated.ts"), generated);

execFileSync(
  "npx",
  ["tauri", "icon", resolve(root, branding.icon), "-o", resolve(root, "src-tauri/icons")],
  { cwd: root, stdio: "inherit", shell: process.platform === "win32" },
);
// La app es solo de escritorio: fuera los iconos móviles.
for (const dir of ["android", "ios"]) {
  rmSync(resolve(root, "src-tauri/icons", dir), { recursive: true, force: true });
}
console.log(`Branding aplicado: ${branding.name} (.${branding.profile_extension})`);
