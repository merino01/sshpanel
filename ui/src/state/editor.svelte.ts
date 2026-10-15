/**
 * La ventana del Editor: qué archivo tiene, si está guardado y si hay conexión.
 * El texto vive en el componente de edición (`screens/editor`), que lo da con
 * `attachText`; aquí no se copia en cada tecla. Lo que ve la ventana principal
 * de los Editores está en `state/editorWindows`.
 */
import { errorMessage, t } from "../i18n";
import { api, type Stamp, type TextFile } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { baseName } from "../lib/paths";
import { fileTitle } from "../lib/textFile";
import { choose, confirm, confirmation } from "./confirm.svelte";
import { toasts } from "./toasts.svelte";

type Phase =
  | { name: "loading" }
  | { name: "ready" }
  /** No se ha podido leer; `retry` dice si tiene sentido volver a intentarlo. */
  | { name: "failed"; message: string; retry: boolean };

interface Editor {
  /** Ruta del archivo en el Servidor; vacía hasta saberla. */
  path: string;
  phase: Phase;
  /** Hay cambios desde que se abrió o se guardó por última vez. */
  unsaved: boolean;
  saving: boolean;
  /** Al guardar se vio que no hay conexión; se quita al reconectar la principal. */
  offline: boolean;
  /** Tamaño y fecha del archivo al abrirlo o guardarlo; si al guardar ya no coinciden, hay conflicto. */
  stamp: Stamp | null;
}

export const editor = $state<Editor>({ path: "", phase: { name: "loading" }, unsaved: false, saving: false, offline: false, stamp: null });

/** Nombre del archivo abierto. */
export function fileName(): string {
  return baseName(editor.path);
}

let currentText: () => string = () => "";
/** Cuenta los cambios, para saber si hubo alguno mientras se guardaba. */
let revision = 0;

/** El componente de edición da aquí cómo leer el texto actual. */
export function attachText(read: () => string) {
  currentText = read;
}

/**
 * Prepara la ventana: escucha la reconexión de la principal y pregunta antes
 * de cerrar con cambios sin guardar. Luego carga el archivo.
 */
export async function startEditor(): Promise<TextFile | null> {
  void api.onConnected(() => {
    if (!editor.offline) return;
    editor.offline = false;
    toasts.success(t("editor.reconnected"));
  });
  void api.guardClose(allowClose);
  const path = await api.editorFile().catch(() => null);
  if (!path) {
    editor.phase = { name: "failed", message: t("editor.noFile"), retry: false };
    return null;
  }
  editor.path = path;
  updateTitle();
  return load();
}

/** Lee el archivo entero. Devuelve su texto, o `null` si no se ha podido. */
export async function load(): Promise<TextFile | null> {
  editor.phase = { name: "loading" };
  try {
    const file = await api.readText(editor.path);
    editor.stamp = file.stamp;
    editor.phase = { name: "ready" };
    return file;
  } catch (code) {
    // Sin conexión vale la pena reintentar al reconectar; si no es UTF-8 o no existe, no.
    const retry = isConnectionError(code) || code === "editor.read_failed";
    editor.phase = { name: "failed", message: errorMessage(code), retry };
    return null;
  }
}

/** El texto ha cambiado. */
export function changed() {
  revision++;
  setUnsaved(true);
}

/**
 * Guarda el texto en el Servidor. Si el archivo ha cambiado allí desde que se
 * abrió, pregunta antes. Si se ha caído la conexión, el texto se queda y se
 * avisa a la principal.
 */
export async function save() {
  if (editor.phase.name !== "ready" || editor.saving) return;
  editor.saving = true;
  const saving = revision;
  const text = currentText();
  try {
    try {
      editor.stamp = await api.writeText(editor.path, text, editor.stamp);
    } catch (code) {
      if (code !== "editor.conflict") throw code;
      if (!(await resolveConflict(text))) return;
    }
    editor.offline = false;
    // Lo escrito mientras se guardaba sigue sin guardar.
    if (revision === saving) setUnsaved(false);
  } catch (code) {
    if (isConnectionError(code)) {
      editor.offline = true;
      void api.reportConnectionLost(code);
    } else {
      toasts.error(errorMessage(code));
    }
  } finally {
    editor.saving = false;
  }
}

/**
 * El archivo ha cambiado en el Servidor: pregunta si sobrescribirlo o guardar
 * `text` como copia al lado, que pasa a ser el archivo de esta ventana. Dice
 * si se ha guardado.
 */
async function resolveConflict(text: string): Promise<boolean> {
  const choice = await choose({
    title: t("editor.conflictTitle", { name: fileName() }),
    body: t("editor.conflictHelp"),
    confirmLabel: t("editor.conflictOverwrite"),
    alternativeLabel: t("editor.conflictCopy"),
  });
  if (choice === "confirm") {
    editor.stamp = await api.writeText(editor.path, text, null);
  } else if (choice === "alternative") {
    const copy = await api.writeTextCopy(editor.path, text);
    editor.path = copy.path;
    editor.stamp = copy.stamp;
    void api.moveEditor(copy.path).catch(() => {});
    updateTitle();
    toasts.success(t("editor.savedAsCopy", { name: fileName() }));
  }
  return choice !== "cancel";
}

function setUnsaved(unsaved: boolean) {
  if (editor.unsaved === unsaved) return;
  editor.unsaved = unsaved;
  updateTitle();
  void api.setUnsaved(unsaved).catch(() => {});
}

function updateTitle() {
  const title = fileTitle(editor.path, editor.unsaved);
  document.title = title;
  void api.setWindowTitle(title).catch(() => {});
}

/** Con cambios sin guardar, pregunta antes de cerrar la ventana. */
async function allowClose(): Promise<boolean> {
  if (!editor.unsaved) return true;
  // Si ya hay una pregunta abierta, el Miembro tiene que contestarla antes.
  if (confirmation.current) return false;
  return confirm({
    title: t("editor.closeTitle", { name: fileName() }),
    body: t("editor.closeHelp"),
    confirmLabel: t("editor.closeConfirm"),
  });
}
