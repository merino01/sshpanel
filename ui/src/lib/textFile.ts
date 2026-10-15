/**
 * Archivos de texto para el Editor: qué cuenta como texto, cuándo un archivo es
 * grande, cómo acaban sus líneas y qué lenguaje resaltar cuando la extensión no
 * lo dice.
 */
import { baseName, parentOf } from "./paths";

/** Por encima de este tamaño (5 MiB) se avisa antes de abrir un archivo en el Editor. */
export const LARGE_FILE = 5 * 1024 * 1024;

/** Si un archivo es de texto, por su primer trozo (`head`): lo es si no tiene ningún byte nulo. */
export function isText(head: ArrayLike<number>): boolean {
  for (let i = 0; i < head.length; i++) if (head[i] === 0) return false;
  return true;
}

/** Si un archivo de `size` bytes es tan grande que conviene avisar antes de abrirlo. */
export function isLarge(size: number): boolean {
  return size > LARGE_FILE;
}

/** Cómo acaban las líneas de `text`, según la primera: CRLF (Windows) o LF. Sin saltos, LF. */
export function lineSeparator(text: string): "\r\n" | "\n" {
  const newline = text.indexOf("\n");
  return newline > 0 && text[newline - 1] === "\r" ? "\r\n" : "\n";
}

/** Ficheros de configuración que se reconocen por su nombre y no por la extensión. */
const BY_NAME: Record<string, string> = {
  ".bashrc": "shell",
  ".bash_profile": "shell",
  ".bash_aliases": "shell",
  ".bash_logout": "shell",
  ".profile": "shell",
  ".zshrc": "shell",
  ".zprofile": "shell",
  ".env": "properties",
  ".gitconfig": "properties",
  ".editorconfig": "properties",
};

/** Extensiones de configuración tipo INI que no reconoce el resaltado por sí solo. */
const BY_EXTENSION: Record<string, string> = {
  zsh: "shell",
  env: "properties",
  conf: "properties",
  cnf: "properties",
  cfg: "properties",
};

/**
 * El lenguaje con el que resaltar el archivo `name` cuando no basta con su
 * extensión (`.bashrc`, `app.conf`…), o `null` para que lo elija el Editor.
 */
export function languageHint(name: string): string | null {
  const lower = name.toLowerCase();
  if (Object.hasOwn(BY_NAME, lower)) return BY_NAME[lower];
  const dot = lower.lastIndexOf(".");
  if (dot <= 0) return null;
  const extension = lower.slice(dot + 1);
  return Object.hasOwn(BY_EXTENSION, extension) ? BY_EXTENSION[extension] : null;
}

/** El título de la ventana del Editor: «• notas.txt — /home/alice», con «•» si hay cambios sin guardar. */
export function fileTitle(path: string, unsaved = false): string {
  return `${unsaved ? "• " : ""}${baseName(path)} — ${parentOf(path) ?? "/"}`;
}
