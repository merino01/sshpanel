import type { Api, Stamp } from "./api";
import { baseName, joinPath, parentOf } from "./paths";
import type { QueuedTransfer, TransferQueue } from "./transferQueue.svelte";
import { t } from "../i18n";
import type { Choice, Confirmation } from "../state/confirm.svelte";

/**
 * `opening`: descargándose para abrirlo; `watching`: abierto, se sube al guardar; `uploading`: subiendo un
 * guardado; `failed`: no se pudo subir el último guardado, se reintenta con el siguiente.
 */
export type LocalCopyStatus = "opening" | "watching" | "uploading" | "failed";

/** Un archivo del Servidor abierto con una aplicación del equipo, cuya copia se vigila. */
export interface LocalCopy {
  /** Número de la copia en `src-tauri` (`local_copies`). */
  id: number;
  remotePath: string;
  name: string;
  /** Ruta de la copia en el equipo; `null` mientras se descarga. */
  file: string | null;
  /** Tamaño y fecha del archivo en el Servidor al descargarlo o subirlo; si ya no coinciden, hay conflicto. */
  stamp: Stamp | null;
  status: LocalCopyStatus;
  /** La Transferencia en curso (descarga o subida), para enseñar su avance. */
  transfer?: QueuedTransfer;
  /** Se ha vuelto a guardar mientras se subía: al terminar, se sube otra vez. */
  changedAgain: boolean;
}

interface Dependencies {
  api: Pick<
    Api,
    "fileStamp" | "prepareLocalCopy" | "watchLocalCopy" | "openLocalCopy" | "forgetLocalCopy" | "clearLocalCopies" | "addDownload" | "addReplace" | "addUpload"
  >;
  queue: Pick<TransferQueue, "enqueue">;
  choose: (question: Confirmation) => Promise<Choice>;
  /** Algo ha fallado fuera de la cola, como abrir la copia con su aplicación. */
  onError: (code: unknown) => void;
}

/**
 * Los archivos abiertos en el equipo: los descarga a una copia temporal, la
 * abre con su aplicación y sube cada guardado, preguntando si el archivo
 * cambió en el Servidor. Se vigilan hasta `clear` (cambiar de Perfil o cerrar
 * la app), sin nada que gestionar. Las descargas y subidas van por la cola
 * pero ocultas: lo que hay que ver está en `active`.
 */
export class LocalCopies {
  items = $state<LocalCopy[]>([]);
  /** Archivos del Servidor que se están preparando para abrirlos, antes de entrar en `items`. */
  #preparing = new Set<string>();
  /** Cambia con cada `clear`: lo que terminara de una tanda anterior se descarta. */
  #generation = 0;
  #deps: Dependencies;

  constructor(deps: Dependencies) {
    this.#deps = deps;
  }

  /** Abre `remotePath` con la aplicación del equipo; si ya está abierto, vuelve a abrir su copia. */
  async open(remotePath: string) {
    const { api, queue, onError } = this.#deps;
    const open = this.items.find((copy) => copy.remotePath === remotePath);
    if (open) {
      if (open.status !== "opening") await api.openLocalCopy(open.id).catch(onError);
      return;
    }
    if (this.#preparing.has(remotePath)) return;
    this.#preparing.add(remotePath);
    const generation = this.#generation;
    let stamp, prepared;
    try {
      stamp = await api.fileStamp(remotePath);
      prepared = await api.prepareLocalCopy();
    } catch (code) {
      return onError(code);
    } finally {
      this.#preparing.delete(remotePath);
    }
    if (generation !== this.#generation) return void api.forgetLocalCopy(prepared.id).catch(() => {});

    const name = baseName(remotePath);
    const dir = parentOf(remotePath) ?? "/";
    const localDir = prepared.dir;
    this.items.push({ id: prepared.id, remotePath, name, file: null, stamp, status: "opening", changedAgain: false });
    const copy = this.items[this.items.length - 1];
    try {
      copy.transfer = await queue.enqueue(
        {
          direction: "download",
          name,
          hidden: true,
          describe: () => "",
          onDone: (outcome) => {
            if (!this.#isWatched(copy) || !("downloaded" in outcome)) return;
            copy.file = outcome.downloaded;
            copy.status = "watching";
            copy.transfer = undefined;
            void api.watchLocalCopy(copy.id, copy.file).catch(onError);
          },
          onFailed: (code) => {
            if (code !== "transfer.cancelled") onError(code);
            void this.#forget(copy);
          },
        },
        (id) => api.addDownload(id, dir, name, localDir),
      );
    } catch (code) {
      void this.#forget(copy);
      onError(code);
    }
  }

  /** Se ha guardado la copia `id`: se sube, o se vuelve a subir al terminar si ya se estaba subiendo. */
  changed(id: number) {
    const copy = this.items.find((item) => item.id === id);
    if (!copy || copy.status === "opening") return;
    if (copy.status === "uploading") copy.changedAgain = true;
    else void this.#replace(copy, copy.stamp);
  }

  /** Las que están pasando por algo que conviene ver: abriéndose, subiendo o sin poder subir. */
  get active(): LocalCopy[] {
    return this.items.filter((copy) => copy.status !== "watching");
  }

  /** Deja de vigilar `copy` y borra su copia del equipo. */
  async #forget(copy: LocalCopy) {
    this.items = this.items.filter((item) => item.id !== copy.id);
    await this.#deps.api.forgetLocalCopy(copy.id).catch(() => {});
  }

  /** Deja de vigilarlas todas y las borra, como al cambiar de Perfil. */
  async clear() {
    this.#generation++;
    this.#preparing.clear();
    this.items = [];
    await this.#deps.api.clearLocalCopies().catch(() => {});
  }

  /** Sube la copia encima del archivo del Servidor; con `expected`, si no ha cambiado allí. */
  async #replace(copy: LocalCopy, expected: Stamp | null) {
    const { api, queue } = this.#deps;
    copy.status = "uploading";
    copy.changedAgain = false;
    try {
      copy.transfer = await queue.enqueue(
        {
          direction: "upload",
          name: copy.name,
          hidden: true,
          changes: parentOf(copy.remotePath) ?? undefined,
          describe: () => "",
          onDone: (outcome) => {
            if ("replaced" in outcome) copy.stamp = outcome.replaced;
            this.#settled(copy, "watching");
          },
          onFailed: (code) => {
            if (code === "editor.conflict") void this.#conflict(copy);
            else this.#failed(copy, code);
          },
        },
        (id) => api.addReplace(id, copy.file!, copy.remotePath, expected),
      );
    } catch (code) {
      this.#failed(copy, code);
    }
  }

  /** El archivo ha cambiado en el Servidor: pregunta si sobrescribirlo o subir la copia al lado, que pasa a ser el vigilado. */
  async #conflict(copy: LocalCopy) {
    const { api, queue, choose } = this.#deps;
    const choice = await choose({
      title: t("editor.conflictTitle", { name: copy.name }),
      body: t("editor.conflictHelp"),
      confirmLabel: t("editor.conflictOverwrite"),
      alternativeLabel: t("editor.conflictCopy"),
    });
    if (!this.#isWatched(copy) || choice === "cancel") return this.#settled(copy, "watching");
    if (choice === "confirm") return this.#replace(copy, null);
    const dir = parentOf(copy.remotePath) ?? "/";
    try {
      copy.transfer = await queue.enqueue(
        {
          direction: "upload",
          name: copy.name,
          hidden: true,
          changes: dir,
          describe: () => "",
          onDone: async (outcome) => {
            if ("uploaded" in outcome) {
              copy.remotePath = joinPath(dir, outcome.uploaded);
              copy.name = outcome.uploaded;
              copy.stamp = await api.fileStamp(copy.remotePath).catch(() => null);
            }
            this.#settled(copy, "watching");
          },
          onFailed: (code) => this.#failed(copy, code),
        },
        (id) => api.addUpload(id, copy.file!, dir),
      );
    } catch (code) {
      this.#failed(copy, code);
    }
  }

  /** No se ha podido subir el último guardado de `copy`: se avisa y se reintenta con el siguiente. */
  #failed(copy: LocalCopy, code: unknown) {
    if (code !== "transfer.cancelled") this.#deps.onError(code);
    this.#settled(copy, "failed");
  }

  /** Ha acabado una subida de `copy`, quedando en `status`; si se volvió a guardar mientras, se sube otra vez. */
  #settled(copy: LocalCopy, status: LocalCopyStatus) {
    copy.status = status;
    copy.transfer = undefined;
    if (copy.changedAgain && this.#isWatched(copy)) void this.#replace(copy, copy.stamp);
  }

  #isWatched(copy: LocalCopy): boolean {
    return this.items.some((item) => item.id === copy.id);
  }
}
