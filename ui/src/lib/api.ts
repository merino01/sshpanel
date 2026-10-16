/**
 * Comandos del núcleo, uno por función y con sus tipos. Es el único sitio que
 * llama a `invoke`: si cambia un comando en `src-tauri/src/commands/`, se
 * cambia aquí. Los errores llegan como el código del núcleo (`files.not_found`…).
 */
import { Channel, invoke } from "@tauri-apps/api/core";
import { emit, emitTo, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Preferences } from "./preferences";

export interface Profile {
  host: string;
  port: number;
  username: string;
  fingerprint: string;
  mode: "simple" | "advanced";
  shared_folder: string | null;
  /** Perfil de un Administrador: puede crear Perfiles. Falta si es `false`. */
  admin?: boolean;
}

/** Lo que elige el Administrador al crear un Perfil; la Huella la calcula el núcleo. */
export interface NewProfile {
  host: string;
  port: number;
  username: string;
  mode: Profile["mode"];
  shared_folder: string | null;
  admin: boolean;
}

export interface AppState {
  profile: Profile | null;
  public_key: string | null;
}

export interface Entry {
  name: string;
  kind: "file" | "dir" | "symlink";
  size: number;
  /** Segundos desde la época Unix. */
  modified: number | null;
}

/** Contenido de una carpeta del Servidor: carpetas primero, luego por nombre. */
export interface Listing {
  path: string;
  entries: Entry[];
}

/** Un archivo o carpeta de la Papelera. */
export interface Trashed {
  id: string;
  name: string;
  original_dir: string;
  kind: Entry["kind"];
  size: number;
  /** Cuándo se borró, en segundos desde la época Unix. */
  deleted: number | null;
}

/** Dónde ha quedado algo restaurado; `name` puede no ser el original. */
export interface Restored {
  dir: string;
  name: string;
}

/** Avance de una Transferencia, en bytes. */
export interface Progress {
  done: number;
  total: number;
}

/** Tamaño y fecha de modificación de un archivo al leerlo o guardarlo en el Editor. */
export interface Stamp {
  size: number;
  /** Segundos desde la época Unix. */
  modified: number | null;
}

/** El principio de un archivo (como mucho 8 KB) y su tamaño entero. */
export interface Head {
  bytes: number[];
  size: number;
}

/** Un archivo de texto leído entero. */
export interface TextFile {
  text: string;
  stamp: Stamp;
}

/** La copia guardada al lado de un archivo que cambió en el Servidor. */
export interface TextCopy {
  path: string;
  stamp: Stamp;
}

/** Dónde ha quedado una Transferencia terminada; al sustituir un archivo, cómo ha quedado. */
export type Outcome = { uploaded: string } | { downloaded: string } | { replaced: Stamp };

/** Una subcarpeta nueva y vacía de la carpeta temporal, para descargar una copia que abrir en el equipo. */
export interface PreparedCopy {
  id: number;
  dir: string;
}

export const api = {
  appState: () => invoke<AppState>("app_state"),
  importProfile: (path: string) => invoke<AppState>("import_profile", { path }),
  /** Lee un fichero de Perfil sin importarlo, para enseñar qué contiene. */
  readProfile: (path: string) => invoke<Profile>("read_profile", { path }),
  /** El Perfil abierto con doble clic que aún no se ha recogido, si lo hay. */
  takeOpenedProfile: () => invoke<string | null>("take_opened_profile"),
  /** Avisa cuando se abre un Perfil con doble clic con la app ya en marcha; recógelo con `takeOpenedProfile`. */
  onProfileOpened: (handler: () => void) => listen("profile-opened", handler),
  connect: () => invoke<Listing>("connect"),
  /** Crea el fichero de Perfil `path`, conectando con el Servidor para calcular su Huella. */
  createProfile: (profile: NewProfile, path: string) => invoke<Profile>("create_profile", { profile, path }),

  listDir: (path: string) => invoke<Listing>("list_dir", { path }),
  createFolder: (parent: string, name: string) => invoke<void>("create_folder", { parent, name }),
  rename: (dir: string, from: string, to: string) => invoke<void>("rename", { dir, from, to }),
  /** Tamaño y fecha de modificación actuales del archivo `path`. */
  fileStamp: (path: string) => invoke<Stamp>("file_stamp", { path }),

  /** El principio del archivo `path`, para saber si es de texto (ver `lib/textFile`) antes de abrirlo. */
  readHead: (path: string) => invoke<Head>("read_head", { path }),
  /** Lee entero un archivo de texto; falla con `editor.not_text` si no es UTF-8. */
  readText: (path: string) => invoke<TextFile>("read_text", { path }),
  /**
   * Guarda `text` en el archivo `path`, que tiene que existir, y devuelve su `Stamp` nuevo. Con `expected`,
   * falla con `editor.conflict` si el archivo ya no tiene ese `Stamp`; con `null`, lo sobrescribe sin mirar.
   */
  writeText: (path: string, text: string, expected: Stamp | null) => invoke<Stamp>("write_text", { path, text, expected }),
  /** Guarda `text` en un archivo nuevo al lado de `path`, con el primer nombre libre. */
  writeTextCopy: (path: string, text: string) => invoke<TextCopy>("write_text_copy", { path, text }),
  /** Abre `path` en su propia ventana del Editor, o la pone delante si ya está abierta. */
  openEditor: (path: string, title: string) => invoke<void>("open_editor", { path, title }),
  /** El archivo que tiene abierto esta ventana del Editor. */
  editorFile: () => invoke<string | null>("editor_file"),
  /** Esta ventana del Editor pasa a tener el archivo `path`. */
  moveEditor: (path: string) => invoke<void>("move_editor", { path }),
  /** Apunta si esta ventana del Editor tiene cambios sin guardar. */
  setUnsaved: (unsaved: boolean) => invoke<void>("set_unsaved", { unsaved }),
  /** Cuántas ventanas del Editor tienen cambios sin guardar. */
  unsavedEditors: () => invoke<number>("unsaved_editors"),
  /** Cierra todas las ventanas del Editor, sin preguntar. */
  closeEditors: () => invoke<void>("close_editors"),
  /** La barra de título es de la interfaz (`TitleBar.svelte`): estos son sus botones en Windows y Linux. */
  minimizeWindow: () => getCurrentWindow().minimize(),
  toggleMaximizeWindow: () => getCurrentWindow().toggleMaximize(),
  /** Cierra esta ventana como su botón del sistema: pasa por `guardClose`. */
  closeWindow: () => getCurrentWindow().close(),
  /** Antes de cerrar esta ventana pregunta a `allow`; si responde `false`, no se cierra. */
  guardClose: (allow: () => Promise<boolean>) =>
    getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await allow())) event.preventDefault();
    }),
  /** Avisa a todas las ventanas de que la principal acaba de conectar con el Servidor. */
  announceConnected: () => emit("connected"),
  onConnected: (handler: () => void) => listen("connected", handler),
  /** Desde el Editor: avisa a la ventana principal de que se ha caído la conexión, con el código del error. */
  reportConnectionLost: (code: unknown) => emitTo("main", "editor-connection-lost", String(code)),
  onEditorConnectionLost: (handler: (code: string) => void) =>
    listen<string>("editor-connection-lost", (event) => handler(event.payload)),

  /** Devuelve su id en la Papelera, para deshacerlo con `restore`. */
  trash: (dir: string, name: string) => invoke<string>("trash", { dir, name }),
  listTrash: () => invoke<Trashed[]>("list_trash"),
  restore: (id: string) => invoke<Restored>("restore", { id }),
  /** Solo en Modo avanzado. */
  deleteForever: (id: string) => invoke<void>("delete_forever", { id }),
  /** Solo en Modo avanzado. */
  emptyTrash: () => invoke<void>("empty_trash"),

  /**
   * Abre la Terminal `id` (solo en Modo avanzado); puede haber varias. Lo que
   * escribe el Servidor llega a `onOutput` en bytes.
   */
  openTerminal: (id: number, cols: number, rows: number, onOutput: (data: Uint8Array) => void, onClosed: () => void) => {
    const output = new Channel<ArrayBuffer>();
    output.onmessage = (data) => onOutput(new Uint8Array(data));
    const closed = new Channel<null>();
    closed.onmessage = onClosed;
    return invoke<void>("open_terminal", { id, cols, rows, onOutput: output, onClosed: closed });
  },
  terminalInput: (id: number, data: string) => invoke<void>("terminal_input", { id, data }),
  resizeTerminal: (id: number, cols: number, rows: number) => invoke<void>("resize_terminal", { id, cols, rows }),
  closeTerminal: (id: number) => invoke<void>("close_terminal", { id }),

  addUpload: (id: number, local: string, remoteDir: string) => invoke<void>("add_upload", { id, local, remoteDir }),
  addDownload: (id: number, remoteDir: string, name: string, localDir: string) =>
    invoke<void>("add_download", { id, remoteDir, name, localDir }),
  /**
   * Sustituye el archivo `remote` por la copia `local` abierta en el equipo. Con `expected`, falla con
   * `editor.conflict` si el archivo ya no tiene ese `Stamp`; con `null`, lo sobrescribe sin mirar.
   */
  addReplace: (id: number, local: string, remote: string, expected: Stamp | null) =>
    invoke<void>("add_replace", { id, local, remote, expected }),
  /** Lanza la Transferencia `id`, o la reanuda donde se quedó, avisando de cada avance. */
  runTransfer: (id: number, onProgress: (progress: Progress) => void) => {
    const channel = new Channel<Progress>();
    channel.onmessage = onProgress;
    return invoke<Outcome>("run_transfer", { id, onProgress: channel });
  },
  cancelTransfer: (id: number) => invoke<void>("cancel_transfer", { id }),

  /** Archivos abiertos en el equipo: prepara la subcarpeta donde descargar la copia. */
  prepareLocalCopy: () => invoke<PreparedCopy>("prepare_local_copy"),
  /** Abre con su aplicación la copia `file`, descargada en la subcarpeta de `id`, y empieza a vigilarla. */
  watchLocalCopy: (id: number, file: string) => invoke<void>("watch_local_copy", { id, file }),
  /** Vuelve a abrir con su aplicación la copia `id`. */
  openLocalCopy: (id: number) => invoke<void>("open_local_copy", { id }),
  /** Deja de vigilar la copia `id` y la borra del equipo. */
  forgetLocalCopy: (id: number) => invoke<void>("forget_local_copy", { id }),
  /** Deja de vigilar todas las copias y las borra del equipo. */
  clearLocalCopies: () => invoke<void>("clear_local_copies"),
  /** Avisa con su número cuando se guarda una copia vigilada. */
  onLocalCopyChanged: (handler: (id: number) => void) => listen<number>("local-copy-changed", (event) => handler(event.payload)),

  /** La versión nueva publicada en las Releases, o `null` si ya es la última. */
  checkUpdate: () => invoke<{ version: string } | null>("check_update"),
  /** Instala la versión que encontró `checkUpdate` y reinicia la app. */
  installUpdate: () => invoke<void>("install_update"),
  /** Abre en el navegador la página de la última Release. */
  openReleases: () => invoke<void>("open_releases"),

  /** Las preferencias guardadas en el equipo, sin completar (`null` si no hay); ver `lib/preferences`. */
  readPreferences: () => invoke<unknown>("read_preferences"),
  /** Guarda las preferencias y avisa a todas las ventanas con `onPreferencesChanged`. */
  savePreferences: (preferences: Preferences) => invoke<void>("save_preferences", { preferences }),
  onPreferencesChanged: (handler: (preferences: unknown) => void) =>
    listen<unknown>("preferences-changed", (event) => handler(event.payload)),
  /** Abre la ventana de Ajustes, o la pone delante si ya está abierta. */
  openSettings: (title: string) => invoke<void>("open_settings", { title }),
  /** Cambia el título de esta ventana, para que siga al idioma activo. */
  setWindowTitle: (title: string) => getCurrentWindow().setTitle(title),
};

export type Api = typeof api;
