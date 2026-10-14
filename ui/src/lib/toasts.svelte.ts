/** Cuánto se ve un aviso de éxito antes de cerrarse solo. */
export const SUCCESS_MS = 5000;
/** Cuántos avisos se ven a la vez; al llegar otro se va el más antiguo. */
export const MAX_TOASTS = 3;

/** Un botón del aviso, como «Deshacer». Al pulsarlo se cierra el aviso. */
export interface ToastAction {
  label: string;
  run: () => unknown;
}

export interface Toast {
  id: number;
  kind: "success" | "info" | "error";
  text: string;
  action?: ToastAction;
}

interface Timer {
  handle: ReturnType<typeof setTimeout> | null;
  /** Lo que le queda, en milisegundos, si está en pausa. */
  remaining: number;
  started: number;
}

/**
 * Avisos de lo que acaba de pasar, el más nuevo primero. Los de éxito se
 * cierran solos a los {@link SUCCESS_MS}, salvo en pausa (con el ratón o el
 * foco encima); los informativos y los errores esperan a que se cierren.
 */
export class Toasts {
  items = $state<Toast[]>([]);
  #nextId = 1;
  #timers = new Map<number, Timer>();

  success(text: string, action?: ToastAction): number {
    const id = this.#add({ kind: "success", text, action });
    this.#start(id, SUCCESS_MS);
    return id;
  }

  /** Algo que conviene saber y no caduca, como que hay una versión nueva. */
  info(text: string, action?: ToastAction): number {
    return this.#add({ kind: "info", text, action });
  }

  error(text: string, action?: ToastAction): number {
    return this.#add({ kind: "error", text, action });
  }

  dismiss(id: number) {
    const timer = this.#timers.get(id);
    if (timer?.handle) clearTimeout(timer.handle);
    this.#timers.delete(id);
    this.items = this.items.filter((toast) => toast.id !== id);
  }

  /** Pulsa el botón del aviso y lo cierra. */
  act(id: number) {
    const action = this.items.find((toast) => toast.id === id)?.action;
    this.dismiss(id);
    action?.run();
  }

  pause(id: number) {
    const timer = this.#timers.get(id);
    if (!timer?.handle) return;
    clearTimeout(timer.handle);
    timer.handle = null;
    timer.remaining -= Date.now() - timer.started;
  }

  resume(id: number) {
    const timer = this.#timers.get(id);
    if (timer && !timer.handle) this.#start(id, timer.remaining);
  }

  #add(toast: Omit<Toast, "id">): number {
    const id = this.#nextId++;
    this.items = [{ ...toast, id }, ...this.items];
    for (const old of this.items.slice(MAX_TOASTS)) this.dismiss(old.id);
    return id;
  }

  #start(id: number, ms: number) {
    this.#timers.set(id, { handle: setTimeout(() => this.dismiss(id), ms), remaining: ms, started: Date.now() });
  }
}
