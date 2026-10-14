/**
 * Comandos del núcleo, uno por función y con sus tipos. Es el único sitio que
 * llama a `invoke`: si cambia un comando en `src-tauri/src/commands/`, se
 * cambia aquí. Los errores llegan como el código del núcleo (`files.not_found`…).
 */
import { Channel, invoke } from "@tauri-apps/api/core";

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

/** Dónde ha quedado una Transferencia terminada. */
export type Outcome = { uploaded: string } | { downloaded: string };

export const api = {
  appState: () => invoke<AppState>("app_state"),
  importProfile: (path: string) => invoke<AppState>("import_profile", { path }),
  connect: () => invoke<Listing>("connect"),
  /** Crea el fichero de Perfil `path`, conectando con el Servidor para calcular su Huella. */
  createProfile: (profile: NewProfile, path: string) => invoke<Profile>("create_profile", { profile, path }),

  listDir: (path: string) => invoke<Listing>("list_dir", { path }),
  createFolder: (parent: string, name: string) => invoke<void>("create_folder", { parent, name }),
  rename: (dir: string, from: string, to: string) => invoke<void>("rename", { dir, from, to }),

  trash: (dir: string, name: string) => invoke<void>("trash", { dir, name }),
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
  /** Lanza la Transferencia `id`, o la reanuda donde se quedó, avisando de cada avance. */
  runTransfer: (id: number, onProgress: (progress: Progress) => void) => {
    const channel = new Channel<Progress>();
    channel.onmessage = onProgress;
    return invoke<Outcome>("run_transfer", { id, onProgress: channel });
  },
  cancelTransfer: (id: number) => invoke<void>("cancel_transfer", { id }),
};

export type Api = typeof api;
