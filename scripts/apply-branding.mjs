// Vuelca branding.toml en la configuración de Tauri, la interfaz, los ficheros
// de Linux (src-tauri/linux/) y los iconos.
// Ejecutar con `npm run branding` tras editar branding.toml.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
// Abrir un Perfil con doble clic: el tipo MIME es para Linux y el tipo
// exportado, para que macOS conozca la extensión.
conf.bundle.fileAssociations = [
  {
    ext: [branding.profile_extension],
    name: `Perfil de ${branding.name}`,
    description: `Perfil de ${branding.name}`,
    role: "Editor",
    mimeType: `application/x-${branding.profile_extension}`,
    exportedType: { identifier: `${branding.identifier}.profile`, conformsTo: ["public.json"] },
  },
];
// En Linux, el `.desktop` pasa el fichero abierto a la app (`%f`) y el XML de
// shared-mime-info le dice al sistema que `*.<extensión>` es un Perfil.
const mimeType = conf.bundle.fileAssociations[0].mimeType;
const mimeFile = `/usr/share/mime/packages/${branding.identifier}.xml`;
const linux = { desktopTemplate: "linux/app.desktop", files: { [mimeFile]: "linux/mime.xml" } };
conf.bundle.linux = { ...conf.bundle.linux, deb: { ...conf.bundle.linux?.deb, ...linux }, rpm: { ...conf.bundle.linux?.rpm, ...linux } };
writeFileSync(confPath, JSON.stringify(conf, null, 2) + "\n");

const linuxDir = resolve(root, "src-tauri/linux");
mkdirSync(linuxDir, { recursive: true });
writeFileSync(
  resolve(linuxDir, "app.desktop"),
  `[Desktop Entry]
Categories={{categories}}
{{#if comment}}
Comment={{comment}}
{{/if}}
Exec={{exec}} %f
StartupWMClass={{exec}}
Icon={{icon}}
Name={{name}}
Terminal=false
Type=Application
MimeType=${mimeType};
`,
);
const xmlName = `Perfil de ${branding.name}`.replace(/&/g, "&amp;").replace(/</g, "&lt;");
writeFileSync(
  resolve(linuxDir, "mime.xml"),
  `<?xml version="1.0" encoding="UTF-8"?>
<!-- Generado por scripts/apply-branding.mjs desde branding.toml. No editar. -->
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="${mimeType}">
    <comment>${xmlName}</comment>
    <glob pattern="*.${branding.profile_extension}"/>
  </mime-type>
</mime-info>
`,
);

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
