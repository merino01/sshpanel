/** Rutas del Servidor, siempre con `/`, y nombres de rutas del equipo. */

/** Carpeta que contiene `path`, o `null` en la raíz. */
export function parentOf(path: string): string | null {
  const trimmed = path.replace(/\/+$/, "");
  if (trimmed === "") return null;
  const slash = trimmed.lastIndexOf("/");
  return slash <= 0 ? "/" : trimmed.slice(0, slash);
}

export function joinPath(dir: string, name: string): string {
  return `${dir.replace(/\/+$/, "")}/${name}`;
}

/** Último nombre de una ruta del equipo, con `/` o `\`. */
export function baseName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}
