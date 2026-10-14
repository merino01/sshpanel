import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MAX_TOASTS, SUCCESS_MS, Toasts } from "./toasts.svelte";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

const texts = (toasts: Toasts) => toasts.items.map((toast) => toast.text);

describe("Toasts", () => {
  it("cierra solos los avisos de éxito, pero no los errores", () => {
    const toasts = new Toasts();
    toasts.success("borrado");
    toasts.error("sin permiso");

    vi.advanceTimersByTime(SUCCESS_MS);

    expect(texts(toasts)).toEqual(["sin permiso"]);
  });

  it("en pausa no corre el tiempo; al seguir, solo espera lo que quedaba", () => {
    const toasts = new Toasts();
    const id = toasts.success("borrado");

    vi.advanceTimersByTime(SUCCESS_MS - 1000);
    toasts.pause(id);
    vi.advanceTimersByTime(SUCCESS_MS * 2);
    expect(texts(toasts)).toEqual(["borrado"]);

    toasts.resume(id);
    vi.advanceTimersByTime(999);
    expect(texts(toasts)).toEqual(["borrado"]);
    vi.advanceTimersByTime(1);
    expect(texts(toasts)).toEqual([]);
  });

  it(`muestra el más nuevo primero y como mucho ${MAX_TOASTS}`, () => {
    const toasts = new Toasts();
    for (const text of ["uno", "dos", "tres", "cuatro"]) toasts.error(text);

    expect(texts(toasts)).toEqual(["cuatro", "tres", "dos"]);
  });

  it("al pulsar su botón lo ejecuta y cierra el aviso", () => {
    const toasts = new Toasts();
    const run = vi.fn();
    const id = toasts.success("borrado", { label: "Deshacer", run });

    toasts.act(id);

    expect(run).toHaveBeenCalledOnce();
    expect(texts(toasts)).toEqual([]);
  });

  it("cerrarlo a mano no deja el temporizador vivo", () => {
    const toasts = new Toasts();
    const first = toasts.success("uno");
    toasts.dismiss(first);
    toasts.success("dos");

    vi.advanceTimersByTime(SUCCESS_MS - 1);

    expect(texts(toasts)).toEqual(["dos"]);
  });
});
