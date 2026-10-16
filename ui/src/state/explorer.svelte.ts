/** Explorador: la carpeta en pantalla y lo que se hace en ella. */
import { errorMessage, t } from "../i18n";
import { api, type Entry, type Listing } from "../lib/api";
import { joinPath, type Place } from "../lib/paths";
import { isConnectionError } from "../lib/errors";
import { confirm } from "./confirm.svelte";
import { openFile } from "./editorWindows.svelte";
import { show, showConnectionLost, session } from "./session.svelte";
import { toasts } from "./toasts.svelte";

/** Nombre que el Miembro está escribiendo: una carpeta nueva o un renombrado. */
export type Editing = { kind: "create"; value: string } | { kind: "rename"; name: string; value: string };

interface Explorer {
  /** Carpeta personal del Miembro en el Servidor. */
  home: string;
  listing: Listing;
  /** Carpetas visitadas antes de la actual, para volver con «Carpeta anterior». */
  history: string[];
  editing?: Editing;
  /** La fila con el foco, sobre la que actúan los Atajos de Renombrar, Mover a la Papelera y Abrir. */
  selected?: string;
}

export const explorer = $state<Explorer>({ home: "", listing: { path: "", entries: [] }, history: [] });

/**
 * En Modo simple, las únicas carpetas que se pueden usar: la personal y la
 * Carpeta compartida. `null` en Modo avanzado, sin límite. El núcleo también lo
 * comprueba; aquí sirve para no ofrecer lo que no se puede hacer.
 */
export function allowedPlaces(): Place[] | null {
  const profile = session.appState?.profile;
  if (profile?.mode !== "simple") return null;
  const shared = profile.shared_folder;
  return [{ name: t("explorer.home"), path: explorer.home }, ...(shared ? [{ name: t("explorer.shared"), path: shared }] : [])];
}

/** Recién conectado: `listing` es la carpeta personal. */
export function enterExplorer(listing: Listing) {
  explorer.home = listing.path;
  explorer.history = [];
  explorer.listing = listing;
  showListing(listing);
}

/** Muestra `listing`. Si es otra carpeta, la que se veía pasa al historial, salvo al volver atrás. */
function showListing(listing: Listing, back = false) {
  const previous = explorer.listing.path;
  if (!back && previous && previous !== listing.path) explorer.history.push(previous);
  // Al recargar la misma carpeta, la fila con el foco sigue ahí.
  const selected = previous === listing.path ? explorer.selected : undefined;
  Object.assign(explorer, { listing, editing: undefined, selected });
  show({ name: "explorer" });
}

export async function openFolder(path: string) {
  try {
    showListing(await api.listDir(path));
  } catch (code) {
    fail(code);
  }
}

/** Vuelve a la carpeta que se veía antes de la actual. */
export async function goBack() {
  const path = explorer.history.pop();
  if (path === undefined) return;
  try {
    showListing(await api.listDir(path), true);
  } catch (code) {
    fail(code);
  }
}

/** Vuelve a listar la carpeta en pantalla. */
export function reload() {
  void openFolder(explorer.listing.path);
}

/** La fila con el foco pasa a ser `name`; `undefined` si ninguna lo tiene. */
export function select(name: string | undefined) {
  explorer.selected = name;
}

/** El archivo o carpeta de la fila con el foco, si sigue en la carpeta. */
export function selectedEntry(): Entry | undefined {
  return explorer.listing.entries.find((entry) => entry.name === explorer.selected);
}

/**
 * Abre `entry`: una carpeta, en el Explorador; un archivo de texto, en su
 * ventana del Editor; cualquier otro, con la aplicación del equipo.
 */
export function openEntry(entry: Entry) {
  const path = joinPath(explorer.listing.path, entry.name);
  if (entry.kind === "dir") void openFolder(path);
  else if (entry.kind === "file") void openFile(path);
}

/** Copia la ruta de `name`, de la carpeta en pantalla, o la de la propia carpeta si no se indica. */
export async function copyPath(name?: string) {
  const path = name === undefined ? explorer.listing.path : joinPath(explorer.listing.path, name);
  try {
    await navigator.clipboard.writeText(path);
    toasts.success(t("shortcuts.pathCopied", { path }));
  } catch {
    toasts.error(t("shortcuts.copyFailed"));
  }
}

/** Vuelve a listar `dir` si es la carpeta en pantalla y el Miembro no está escribiendo un nombre. */
export function refresh(dir: string) {
  if (session.screen.name === "explorer" && !explorer.editing && explorer.listing.path === dir) void openFolder(dir);
}

/**
 * Una operación ha fallado: se explica en un aviso y se queda en el
 * Explorador, salvo si se ha caído la conexión.
 */
export function fail(code: unknown) {
  if (isConnectionError(code)) return showConnectionLost(code);
  toasts.error(errorMessage(code));
  show({ name: "explorer" });
}

export function startCreating() {
  edit({ kind: "create", value: "" });
}

export function startRenaming(entry: Entry) {
  edit({ kind: "rename", name: entry.name, value: entry.name });
}

export function stopEditing() {
  edit(undefined);
}

function edit(editing: Editing | undefined) {
  explorer.editing = editing;
}

/** Crea la carpeta o aplica el renombrado que se está escribiendo, con el nombre `value`. */
export async function submitName(value: string) {
  const editing = explorer.editing;
  if (!editing) return;
  const dir = explorer.listing.path;
  try {
    if (editing.kind === "create") await api.createFolder(dir, value);
    else await api.rename(dir, editing.name, value);
  } catch (code) {
    explorer.editing = { ...editing, value };
    return fail(code);
  }
  await openFolder(dir);
}

/** Pide confirmación y, si se da, mueve `name` a la Papelera. Dice si se confirmó. */
export async function askToTrash(name: string): Promise<boolean> {
  const confirmed = await confirm({
    title: t("trash.confirmTitle", { name }),
    body: t("trash.confirmHelp"),
    confirmLabel: t("trash.confirm"),
  });
  if (confirmed) void trashEntry(name);
  return confirmed;
}

/** Mueve `name`, de la carpeta en pantalla, a la Papelera, con «Deshacer» en el aviso. La confirmación se pide antes. */
export async function trashEntry(name: string) {
  const dir = explorer.listing.path;
  let id: string;
  try {
    id = await api.trash(dir, name);
  } catch (code) {
    return fail(code);
  }
  toasts.success(t("explorer.deleted", { name }), { label: t("toast.undo"), run: () => undoTrash(id, name) });
  await openFolder(dir);
}

/** Devuelve a su sitio lo que se acaba de mover a la Papelera. */
async function undoTrash(id: string, original: string) {
  try {
    const { dir, name } = await api.restore(id);
    toasts.success(name === original ? t("trash.restored", { name, dir }) : t("trash.restoredAs", { original, name, dir }));
    refresh(dir);
  } catch (code) {
    if (isConnectionError(code)) showConnectionLost(code);
    else toasts.error(errorMessage(code));
  }
}
