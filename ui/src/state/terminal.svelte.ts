/**
 * Terminal (solo en Modo avanzado): si hay una sesión abierta en el Servidor.
 * La pinta `screens/terminal/Terminal.svelte`, que le pasa lo que llega con
 * `setOutput` y la abre cuando está en `idle` y se ve.
 */
import { errorMessage } from "../i18n";
import { api } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { show, showConnectionLost } from "./session.svelte";

export const terminal = $state<{ status: "idle" | "opening" | "open" | "closed"; message?: string }>({ status: "idle" });

/** Cada sesión tiene su número, para no hacer caso de lo que llegue tarde de una anterior. */
let generation = 0;
let output: ((data: Uint8Array) => void) | null = null;

export function setOutput(write: ((data: Uint8Array) => void) | null) {
  output = write;
}

export function openTerminal() {
  show({ name: "terminal" });
}

/** Abre una sesión de `cols` columnas y `rows` filas. */
export async function startTerminal(cols: number, rows: number) {
  const current = ++generation;
  Object.assign(terminal, { status: "opening", message: undefined });
  try {
    await api.openTerminal(
      cols,
      rows,
      (data) => current === generation && output?.(data),
      () => current === generation && Object.assign(terminal, { status: "closed" }),
    );
    if (current === generation && terminal.status === "opening") terminal.status = "open";
  } catch (code) {
    if (current !== generation) return;
    if (isConnectionError(code)) return showConnectionLost(code);
    Object.assign(terminal, { status: "closed", message: errorMessage(code) });
  }
}

/** Tras cerrarse una sesión, deja abrir otra. */
export function reopenTerminal() {
  Object.assign(terminal, { status: "idle", message: undefined });
}

/** Olvida la sesión anterior; al conectar de nuevo el núcleo ya la ha cerrado. */
export function resetTerminal() {
  generation++;
  reopenTerminal();
}

export function sendInput(data: string) {
  if (terminal.status !== "open") return;
  api.terminalInput(data).catch(() => {});
}

export function resize(cols: number, rows: number) {
  if (terminal.status !== "open") return;
  api.resizeTerminal(cols, rows).catch(() => {});
}
