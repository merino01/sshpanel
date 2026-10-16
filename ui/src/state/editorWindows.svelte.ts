/**
 * Las ventanas del Editor vistas desde la principal: abrir un archivo en la
 * suya, enterarse de que una ha perdido la conexión y no perder cambios sin
 * guardar al cerrar la app o cambiar de Perfil. La ventana del propio Editor
 * está en `state/editor`.
 */
import { errorMessage, locale, t } from "../i18n";
import { api } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { formatSize } from "../lib/format";
import { baseName } from "../lib/paths";
import { fileTitle, isLarge, isText } from "../lib/textFile";
import { confirm } from "./confirm.svelte";
import { localCopies } from "./localCopies.svelte";
import { inWorkspace, showConnectionLost } from "./session.svelte";
import { toasts } from "./toasts.svelte";

/**
 * Abre el archivo `path` del Servidor en su ventana del Editor, si es de
 * texto. Si pasa de 5 MB, pregunta antes.
 */
export function openInEditor(path: string) {
  return open(path, () => toasts.error(t("editor.notText", { name: baseName(path) })));
}

/** Lo que hace el doble clic: abre `path` en el Editor si es de texto y, si no, con la aplicación del equipo. */
export function openFile(path: string) {
  return open(path, () => localCopies.open(path));
}

/** Abre `path` en el Editor si es de texto; si no, llama a `notText`. */
async function open(path: string, notText: () => unknown) {
  const name = baseName(path);
  let head;
  try {
    head = await api.readHead(path);
  } catch (code) {
    return failed(code);
  }
  if (!isText(head.bytes)) return notText();
  if (isLarge(head.size)) {
    const confirmed = await confirm({
      title: t("editor.largeTitle", { name, size: formatSize(head.size, locale.current) }),
      body: t("editor.largeHelp"),
      confirmLabel: t("editor.largeConfirm"),
    });
    if (!confirmed) return;
  }
  try {
    await api.openEditor(path, fileTitle(path));
  } catch (code) {
    failed(code);
  }
}

function failed(code: unknown) {
  if (isConnectionError(code)) showConnectionLost(code);
  else toasts.error(errorMessage(code));
}

/**
 * Desde la ventana principal: si una ventana del Editor pierde la conexión al
 * guardar, se ofrece «Reintentar» aquí; y al cerrar la app con cambios sin
 * guardar en el Editor, se pregunta antes.
 */
export function watchEditors() {
  void api.onEditorConnectionLost((code) => {
    if (inWorkspace()) showConnectionLost(code);
  });
  void api.guardClose(() =>
    askToDiscard({ title: t("editor.quitTitle"), body: t("editor.quitHelp"), confirmLabel: t("editor.quitConfirm") }),
  );
}

/**
 * Antes de cambiar de Perfil: cierra las ventanas del Editor, que son de otro
 * Servidor, preguntando antes si alguna tiene cambios sin guardar. Dice si se
 * puede seguir.
 */
export async function closeEditors(): Promise<boolean> {
  const confirmed = await askToDiscard({
    title: t("editor.switchTitle"),
    body: t("editor.switchHelp"),
    confirmLabel: t("editor.switchConfirm"),
  });
  if (confirmed) await api.closeEditors().catch(() => {});
  return confirmed;
}

/** Si no hay cambios sin guardar en el Editor, sí; si los hay, lo que conteste el Miembro a `question`. */
async function askToDiscard(question: Parameters<typeof confirm>[0]): Promise<boolean> {
  const unsaved = await api.unsavedEditors().catch(() => 0);
  return unsaved === 0 || confirm(question);
}
