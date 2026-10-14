import type { Api, Outcome, Progress } from "./api";
import { isConnectionError } from "./errors";
import { errorMessage, t } from "../i18n";

/** Cuántas Transferencias van a la vez; las demás esperan su turno. */
export const MAX_PARALLEL = 3;

/** `lost`: cortada al perder la conexión; sigue sola al volver a conectar. */
export type TransferStatus = "waiting" | "running" | "done" | "failed" | "lost" | "cancelled";

/** Lo que hace falta para poner una Transferencia en la cola. */
export interface NewTransfer {
  direction: "upload" | "download";
  name: string;
  /** Explica dónde ha quedado al terminar. */
  describe: (outcome: Outcome) => string;
  /** Carpeta del Servidor que cambia al terminar, para refrescarla si está en pantalla. */
  changes?: string;
}

/** Una Transferencia de la cola. */
export interface QueuedTransfer extends NewTransfer {
  id: number;
  status: TransferStatus;
  progress: Progress | null;
  /** Se ha pedido cancelarla y aún no se ha parado. */
  cancelling: boolean;
  /** Al terminar, dónde ha quedado; si falla, qué ha pasado. */
  outcome?: string;
}

interface Dependencies {
  api: Pick<Api, "runTransfer" | "cancelTransfer">;
  /** La carpeta `dir` del Servidor ha cambiado: al terminar una Transferencia o al borrar lo que dejó a medias. */
  onChanged: (dir: string) => void;
  /** Se acaba de perder la conexión; solo se avisa una vez hasta que se recupera. */
  onLost: (code: unknown) => void;
}

export function isFinished(transfer: QueuedTransfer): boolean {
  return transfer.status === "done" || transfer.status === "cancelled";
}

/**
 * Cola de Transferencias: lanza hasta {@link MAX_PARALLEL} por orden de
 * llegada, las cancela o las reintenta, y al perder la conexión deja las que
 * iban como `lost` para seguir donde se quedaron al reconectar. Vive en
 * memoria mientras la app está abierta.
 */
export class TransferQueue {
  items = $state<QueuedTransfer[]>([]);
  #nextId = 1;
  /** Falso desde que se pierde la conexión hasta que se recupera: mientras, no se lanza nada. */
  #online = true;
  #deps: Dependencies;

  constructor(deps: Dependencies) {
    this.#deps = deps;
  }

  /** Registra la Transferencia en el núcleo con `add` y la pone en la cola. */
  async enqueue(transfer: NewTransfer, add: (id: number) => Promise<unknown>) {
    const id = this.#nextId++;
    await add(id);
    this.items.push({ ...transfer, id, status: "waiting", progress: null, cancelling: false });
    this.#pump();
  }

  /** Para la Transferencia y borra lo que había creado. Si está en curso, `#run` recoge el final. */
  async cancel(transfer: QueuedTransfer) {
    if (transfer.cancelling) return;
    const wasRunning = transfer.status === "running";
    if (wasRunning) {
      transfer.cancelling = true;
    } else {
      transfer.status = "cancelled";
      transfer.outcome = t("transfers.cancelled");
    }
    await this.#deps.api.cancelTransfer(transfer.id).catch(() => {});
    if (!wasRunning) this.#changed(transfer);
  }

  retry(transfer: QueuedTransfer) {
    transfer.status = "waiting";
    transfer.outcome = undefined;
    this.#pump();
  }

  /** Al volver a conectar, lo que se cortó sigue donde se quedó. */
  resume() {
    this.#online = true;
    for (const transfer of this.items) if (transfer.status === "lost") transfer.status = "waiting";
    this.#pump();
  }

  clearFinished() {
    this.items = this.items.filter((transfer) => !isFinished(transfer));
  }

  /** Lanza Transferencias en espera, por orden de llegada, hasta tener {@link MAX_PARALLEL} en curso. */
  #pump() {
    if (!this.#online) return;
    let running = this.items.filter((transfer) => transfer.status === "running").length;
    for (const transfer of this.items) {
      if (running >= MAX_PARALLEL) break;
      if (transfer.status !== "waiting") continue;
      running++;
      void this.#run(transfer);
    }
  }

  /** Lanza la Transferencia, o la reanuda donde se quedó si ya lo intentó antes. */
  async #run(transfer: QueuedTransfer) {
    transfer.status = "running";
    transfer.outcome = undefined;
    try {
      const outcome = await this.#deps.api.runTransfer(transfer.id, (progress) => (transfer.progress = progress));
      transfer.outcome = transfer.describe(outcome);
      transfer.status = "done";
    } catch (code) {
      if (code === "transfer.cancelled") {
        transfer.status = "cancelled";
        transfer.outcome = t("transfers.cancelled");
      } else if (isConnectionError(code)) {
        transfer.status = "lost";
        transfer.outcome = t("transfers.lost");
        if (this.#online) this.#deps.onLost(code);
        this.#online = false;
      } else {
        transfer.status = "failed";
        transfer.outcome = errorMessage(code);
      }
    }
    transfer.cancelling = false;
    if (transfer.status !== "lost") this.#changed(transfer);
    this.#pump();
  }

  #changed(transfer: QueuedTransfer) {
    if (transfer.changes !== undefined) this.#deps.onChanged(transfer.changes);
  }
}
