/** Papelera: su contenido y restaurar. */
import { errorMessage, t } from "../i18n";
import { api, type Trashed } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { fail } from "./explorer.svelte";
import { show, showConnectionLost } from "./session.svelte";

export const trashView = $state<{ items: Trashed[]; notice?: string; message?: string }>({ items: [] });

/** Muestra la Papelera. Si no se puede listar, se explica en el Explorador. */
export async function openTrash(notice?: string, message?: string) {
  let items: Trashed[];
  try {
    items = await api.listTrash();
  } catch (code) {
    return fail(code);
  }
  Object.assign(trashView, { items, notice, message });
  show({ name: "trash" });
}

export async function restore(item: Trashed) {
  try {
    const { dir, name } = await api.restore(item.id);
    const notice = name === item.name ? t("trash.restored", { name, dir }) : t("trash.restoredAs", { original: item.name, name, dir });
    await openTrash(notice);
  } catch (code) {
    if (isConnectionError(code)) showConnectionLost(code);
    else await openTrash(undefined, errorMessage(code));
  }
}
