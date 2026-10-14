/** Papelera: su contenido, restaurar y, en Modo avanzado, borrar para siempre. */
import { errorMessage, t } from "../i18n";
import { api, type Trashed } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { confirm } from "./confirm.svelte";
import { fail } from "./explorer.svelte";
import { show, showConnectionLost } from "./session.svelte";
import { toasts } from "./toasts.svelte";

export const trashView = $state<{ items: Trashed[] }>({ items: [] });

/** Muestra la Papelera. Si no se puede listar, se explica en el Explorador. */
export async function openTrash() {
  let items: Trashed[];
  try {
    items = await api.listTrash();
  } catch (code) {
    return fail(code);
  }
  trashView.items = items;
  show({ name: "trash" });
}

export function restore(item: Trashed) {
  return change(async () => {
    const { dir, name } = await api.restore(item.id);
    return name === item.name ? t("trash.restored", { name, dir }) : t("trash.restoredAs", { original: item.name, name, dir });
  });
}

/** Borra `item` para siempre, tras confirmarlo. Solo en Modo avanzado. */
export async function deleteForever(item: Trashed) {
  const confirmed = await confirm({
    title: t("trash.deleteConfirmTitle", { name: item.name }),
    body: t("trash.deleteConfirmHelp"),
    confirmLabel: t("trash.deleteForever"),
  });
  if (!confirmed) return;
  await change(async () => {
    await api.deleteForever(item.id);
    return t("trash.deletedForever", { name: item.name });
  });
}

/** Borra para siempre todo lo de la Papelera, tras confirmarlo. Solo en Modo avanzado. */
export async function emptyTrash() {
  const confirmed = await confirm({
    title: t("trash.emptyConfirmTitle"),
    body: t("trash.emptyConfirmHelp"),
    confirmLabel: t("trash.emptyTrash"),
  });
  if (!confirmed) return;
  await change(async () => {
    await api.emptyTrash();
    return t("trash.emptied");
  });
}

/** Hace un cambio en la Papelera, avisa de cómo ha ido y la vuelve a mostrar. */
async function change(run: () => Promise<string>) {
  try {
    toasts.success(await run());
  } catch (code) {
    if (isConnectionError(code)) return showConnectionLost(code);
    toasts.error(errorMessage(code));
  }
  await openTrash();
}
