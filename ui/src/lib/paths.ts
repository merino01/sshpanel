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

/** Una carpeta con nombre propio, como «Mi carpeta». */
export interface Place {
  name: string;
  path: string;
}

/** Si `path` es `root` o está dentro de ella. Con `.` o `..` nunca, porque podría salir. */
export function isWithin(path: string, root: string): boolean {
  if (path.split("/").some((part) => part === "." || part === "..")) return false;
  const [inner, outer] = [path.replace(/\/+$/, ""), root.replace(/\/+$/, "")];
  return inner === outer || inner.startsWith(`${outer}/`);
}

/** De `places`, la más interna que contiene `path`. */
export function placeOf(path: string, places: Place[]): Place | undefined {
  return places.filter((place) => isWithin(path, place.path)).sort((a, b) => b.path.length - a.path.length)[0];
}

/**
 * Cada carpeta de la ruta, con la ruta para ir a ella: desde la raíz o, si se
 * da `base` y la contiene, desde `base` con su nombre propio.
 */
export function breadcrumbs(path: string, base?: Place): Place[] {
  const start = base && isWithin(path, base.path) ? base : { name: "/", path: "/" };
  const parts = path.slice(start.path.replace(/\/+$/, "").length).split("/").filter(Boolean);
  return [start, ...parts.map((name, i) => ({ name, path: joinPath(start.path, parts.slice(0, i + 1).join("/")) }))];
}

/** Último nombre de una ruta del equipo, con `/` o `\`. */
export function baseName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}
