/** Explorador: la carpeta en pantalla y lo que se hace en ella. */
import { errorMessage, t } from "../i18n";
import { api, type Entry, type Listing } from "../lib/api";
import type { Place } from "../lib/paths";
import { isConnectionError } from "../lib/errors";
import { show, showConnectionLost, session } from "./session.svelte";

/** Nombre que el Miembro está escribiendo: una carpeta nueva o un renombrado. */
export type Editing = { kind: "create"; value: string } | { kind: "rename"; name: string; value: string };

interface Explorer {
  /** Carpeta personal del Miembro en el Servidor. */
  home: string;
  listing: Listing;
  /** Carpetas visitadas antes de la actual, para volver con «Carpeta anterior». */
  history: string[];
  /** Qué ha hecho la última operación. */
  notice?: string;
  /** Error de la última operación. */
  message?: string;
  editing?: Editing;
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
function showListing(listing: Listing, notice?: string, back = false) {
  const previous = explorer.listing.path;
  if (!back && previous && previous !== listing.path) explorer.history.push(previous);
  Object.assign(explorer, { listing, notice, message: undefined, editing: undefined });
  show({ name: "explorer" });
}

export async function openFolder(path: string, notice?: string) {
  try {
    showListing(await api.listDir(path), notice);
  } catch (code) {
    fail(code);
  }
}

/** Vuelve a la carpeta que se veía antes de la actual. */
export async function goBack() {
  const path = explorer.history.pop();
  if (path === undefined) return;
  try {
    showListing(await api.listDir(path), undefined, true);
  } catch (code) {
    fail(code);
  }
}

/** Vuelve a listar `dir` si es la carpeta en pantalla y el Miembro no está escribiendo un nombre. */
export function refresh(dir: string) {
  if (session.screen.name === "explorer" && !explorer.editing && explorer.listing.path === dir) void openFolder(dir);
}

/**
 * Una operación ha fallado: se explica en el Explorador, sin salir de la
 * carpeta, salvo si se ha caído la conexión.
 */
export function fail(code: unknown) {
  if (isConnectionError(code)) return showConnectionLost(code);
  explorer.notice = undefined;
  explorer.message = errorMessage(code);
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
  Object.assign(explorer, { editing, notice: undefined, message: undefined });
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

/** Mueve `name`, de la carpeta en pantalla, a la Papelera. La confirmación se pide antes. */
export async function trashEntry(name: string) {
  const dir = explorer.listing.path;
  try {
    await api.trash(dir, name);
  } catch (code) {
    return fail(code);
  }
  await openFolder(dir, t("explorer.deleted", { name }));
}
