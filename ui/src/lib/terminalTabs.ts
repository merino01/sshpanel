/** Pestañas de Terminal: qué número lleva una nueva, cuál queda al cerrar o al cambiar, y los atajos. */

/** Lo mínimo de una pestaña: su número interno y el que se ve («Terminal 2»). */
export interface TabRef {
  id: number;
  number: number;
}

/** El número más bajo que no usa ninguna pestaña, empezando por 1. */
export function nextNumber(tabs: TabRef[]): number {
  const used = new Set(tabs.map((tab) => tab.number));
  let n = 1;
  while (used.has(n)) n++;
  return n;
}

/** Qué pestaña queda activa al cerrar `id`: la de su derecha o, si era la última, la de su izquierda. */
export function activeAfterClose(tabs: TabRef[], id: number, active: number | null): number | null {
  if (active !== id) return active;
  const i = tabs.findIndex((tab) => tab.id === id);
  const rest = tabs.filter((tab) => tab.id !== id);
  return rest[Math.min(i, rest.length - 1)]?.id ?? null;
}

/** La pestaña a `step` posiciones de `active`, dando la vuelta. */
export function cycle(tabs: TabRef[], active: number | null, step: number): number | null {
  if (!tabs.length) return null;
  const i = Math.max(0, tabs.findIndex((tab) => tab.id === active));
  return tabs[(((i + step) % tabs.length) + tabs.length) % tabs.length].id;
}

export type TerminalShortcut = "new" | "close" | "next" | "previous";

/** Atajos de las pestañas: Ctrl+Mayús+T, Ctrl+Mayús+W, Ctrl+AvPág y Ctrl+RePág. */
export function shortcutFor(event: Pick<KeyboardEvent, "key" | "ctrlKey" | "shiftKey" | "altKey" | "metaKey">): TerminalShortcut | null {
  if (!event.ctrlKey || event.altKey || event.metaKey) return null;
  const key = event.key.toLowerCase();
  if (event.shiftKey && key === "t") return "new";
  if (event.shiftKey && key === "w") return "close";
  if (!event.shiftKey && key === "pagedown") return "next";
  if (!event.shiftKey && key === "pageup") return "previous";
  return null;
}
