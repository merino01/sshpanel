/**
 * Terminales (solo en Modo avanzado), una por pestaña. Las pinta
 * `screens/terminal/`: cada pestaña le pasa lo que le llega con `setOutput` y
 * abre su sesión cuando está en `idle` y se ve.
 */
import { errorMessage } from "../i18n";
import { api } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { activeAfterClose, cycle, nextNumber } from "../lib/terminalTabs";
import { show, showConnectionLost } from "./session.svelte";

export interface TerminalTab {
  /** Número interno, el que recibe el núcleo; nunca se repite. */
  id: number;
  /** El que se ve: «Terminal 2». */
  number: number;
  status: "idle" | "opening" | "open" | "closed";
  message?: string;
}

export const terminals = $state<{ tabs: TerminalTab[]; active: number | null }>({ tabs: [], active: null });

let nextId = 1;
/** Sesión vigente de cada pestaña, para no hacer caso de lo que llegue tarde de una anterior. */
const sessions = new Map<number, number>();
let nextSession = 1;
const outputs = new Map<number, (data: Uint8Array) => void>();

function tab(id: number): TerminalTab | undefined {
  return terminals.tabs.find((t) => t.id === id);
}

export function setOutput(id: number, write: ((data: Uint8Array) => void) | null) {
  if (write) outputs.set(id, write);
  else outputs.delete(id);
}

/** Muestra las Terminales, abriendo una si no hay ninguna. */
export function openTerminal() {
  if (!terminals.tabs.length) return newTerminal();
  show({ name: "terminal" });
}

/** Abre una pestaña nueva y la muestra. */
export function newTerminal() {
  const id = nextId++;
  terminals.tabs.push({ id, number: nextNumber(terminals.tabs), status: "idle" });
  terminals.active = id;
  show({ name: "terminal" });
}

export function selectTerminal(id: number) {
  terminals.active = id;
}

/** Pasa a la pestaña a `step` posiciones de la actual. */
export function cycleTerminal(step: number) {
  terminals.active = cycle(terminals.tabs, terminals.active, step);
}

/** Cierra la pestaña y su sesión. Si era la última, vuelve al Explorador. */
export function closeTerminal(id: number) {
  terminals.active = activeAfterClose(terminals.tabs, id, terminals.active);
  terminals.tabs = terminals.tabs.filter((t) => t.id !== id);
  sessions.delete(id);
  api.closeTerminal(id).catch(() => {});
  if (!terminals.tabs.length) show({ name: "explorer" });
}

/** Abre la sesión de la pestaña `id`, de `cols` columnas y `rows` filas. */
export async function startTerminal(id: number, cols: number, rows: number) {
  const session = nextSession++;
  sessions.set(id, session);
  const current = () => sessions.get(id) === session;
  const update = (changes: Partial<TerminalTab>) => {
    const t = tab(id);
    if (t && current()) Object.assign(t, changes);
  };
  update({ status: "opening", message: undefined });
  try {
    await api.openTerminal(
      id,
      cols,
      rows,
      (data) => current() && outputs.get(id)?.(data),
      () => update({ status: "closed" }),
    );
    if (tab(id)?.status === "opening") update({ status: "open" });
  } catch (code) {
    if (!current()) return;
    if (isConnectionError(code)) return showConnectionLost(code);
    update({ status: "closed", message: errorMessage(code) });
  }
}

/** Tras cerrarse la sesión de una pestaña, abre otra en ella. */
export function reopenTerminal(id: number) {
  const t = tab(id);
  if (t) Object.assign(t, { status: "idle", message: undefined });
}

/** Olvida todas las pestañas; al conectar de nuevo el núcleo ya ha cerrado sus sesiones. */
export function resetTerminals() {
  sessions.clear();
  outputs.clear();
  Object.assign(terminals, { tabs: [], active: null });
}

export function sendInput(id: number, data: string) {
  if (tab(id)?.status !== "open") return;
  api.terminalInput(id, data).catch(() => {});
}

export function resize(id: number, cols: number, rows: number) {
  if (tab(id)?.status !== "open") return;
  api.resizeTerminal(id, cols, rows).catch(() => {});
}
