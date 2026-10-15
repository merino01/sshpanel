/**
 * Qué icono lleva cada archivo o carpeta en el Explorador y la Papelera: por su
 * tipo y, si es un archivo, por su extensión. Solo dice la clase; el dibujo lo
 * elige `components/FileIcon.svelte`.
 */
import type { Entry } from "./api";

export type FileIconKind = "folder" | "link" | "image" | "code" | "archive" | "text" | "file";

const BY_EXTENSION: Record<string, FileIconKind> = {};
const groups: [FileIconKind, string[]][] = [
  ["image", ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "avif", "heic", "tif", "tiff"]],
  [
    "code",
    ["js", "mjs", "cjs", "ts", "tsx", "jsx", "svelte", "vue", "py", "rb", "php", "go", "rs", "java", "kt", "c", "h", "cpp", "hpp", "cs", "sh", "bash", "zsh", "fish", "sql", "html", "htm", "css", "scss", "json", "yaml", "yml", "toml", "xml", "ini", "conf", "lua", "pl", "swift"],
  ],
  ["archive", ["zip", "tar", "gz", "tgz", "bz2", "xz", "zst", "7z", "rar", "deb", "rpm"]],
  ["text", ["txt", "md", "markdown", "rst", "log", "csv", "tsv", "pdf", "doc", "docx", "odt", "rtf"]],
];
for (const [kind, extensions] of groups) for (const extension of extensions) BY_EXTENSION[extension] = kind;

export function fileIcon(entry: Pick<Entry, "name" | "kind">): FileIconKind {
  if (entry.kind === "dir") return "folder";
  if (entry.kind === "symlink") return "link";
  const dot = entry.name.lastIndexOf(".");
  // Sin punto, o solo al principio (`.bashrc`), no hay extensión.
  if (dot <= 0) return "file";
  return BY_EXTENSION[entry.name.slice(dot + 1).toLowerCase()] ?? "file";
}
