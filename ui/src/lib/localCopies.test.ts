import { describe, expect, it, vi } from "vitest";
import type { Outcome } from "./api";
import { LocalCopies } from "./localCopies.svelte";
import type { NewTransfer, QueuedTransfer } from "./transferQueue.svelte";
import type { Choice } from "../state/confirm.svelte";

const flush = () => new Promise((resolve) => setTimeout(resolve));

const STAMP = { size: 4, modified: 100 };

/** Cola simulada: guarda lo que se encola, con lo que pidió al núcleo, y deja terminarlo a mano. */
function setup(answer: Choice = "cancel") {
  const queued: { transfer: NewTransfer; added: unknown[] }[] = [];
  const api = {
    fileStamp: vi.fn(async (_path: string) => STAMP),
    prepareLocalCopy: vi.fn(async () => ({ id: 7, dir: "/tmp/copias/7" })),
    watchLocalCopy: vi.fn(async (_id: number, _file: string) => {}),
    openLocalCopy: vi.fn(async (_id: number) => {}),
    forgetLocalCopy: vi.fn(async (_id: number) => {}),
    clearLocalCopies: vi.fn(async () => {}),
    addDownload: vi.fn(async (...args: unknown[]) => void queued.at(-1)!.added.push("download", ...args.slice(1))),
    addReplace: vi.fn(async (...args: unknown[]) => void queued.at(-1)!.added.push("replace", ...args.slice(1))),
    addUpload: vi.fn(async (...args: unknown[]) => void queued.at(-1)!.added.push("upload", ...args.slice(1))),
  };
  const queue = {
    enqueue: vi.fn(async (transfer: NewTransfer, add: (id: number) => Promise<unknown>) => {
      queued.push({ transfer, added: [] });
      await add(queued.length);
      return { ...transfer, id: queued.length } as QueuedTransfer;
    }),
  };
  const choose = vi.fn(async () => answer);
  const onError = vi.fn();
  const copies = new LocalCopies({ api, queue, choose, onError });

  const done = async (outcome: Outcome) => {
    await queued.at(-1)!.transfer.onDone?.(outcome);
    await flush();
  };
  const fail = async (code: string) => {
    queued.at(-1)!.transfer.onFailed?.(code);
    await flush();
  };
  /** Abre `/home/alice/foto.png` y termina su descarga. */
  const opened = async () => {
    await copies.open("/home/alice/foto.png");
    await done({ downloaded: "/tmp/copias/7/foto.png" });
  };
  const lastAdded = () => queued.at(-1)!.added;
  return { api, queue, choose, onError, copies, queued, done, fail, opened, lastAdded };
}

describe("LocalCopies", () => {
  it("descarga el archivo a su copia sin enseñarlo en la lista, la abre y la vigila", async () => {
    const { copies, api, queued, lastAdded, done } = setup();

    await copies.open("/home/alice/foto.png");
    expect(lastAdded()).toEqual(["download", "/home/alice", "foto.png", "/tmp/copias/7"]);
    expect(queued[0].transfer.hidden).toBe(true);
    expect(copies.items).toMatchObject([{ id: 7, name: "foto.png", file: null, status: "opening" }]);

    await done({ downloaded: "/tmp/copias/7/foto.png" });
    expect(api.watchLocalCopy).toHaveBeenCalledWith(7, "/tmp/copias/7/foto.png");
    expect(copies.items).toMatchObject([{ file: "/tmp/copias/7/foto.png", stamp: STAMP, status: "watching" }]);
  });

  it("abrir otra vez el mismo archivo reabre su copia sin descargarlo", async () => {
    const { copies, api, queue, opened } = setup();
    await opened();

    await copies.open("/home/alice/foto.png");

    expect(api.openLocalCopy).toHaveBeenCalledWith(7);
    expect(queue.enqueue).toHaveBeenCalledTimes(1);
  });

  it("si la descarga falla, avisa, lo quita y borra la copia preparada", async () => {
    const { copies, api, fail, onError } = setup();
    await copies.open("/home/alice/foto.png");

    await fail("files.permission_denied");

    expect(onError).toHaveBeenCalledWith("files.permission_denied");
    expect(api.forgetLocalCopy).toHaveBeenCalledWith(7);
    expect(copies.items).toEqual([]);
  });

  it("cada guardado se sube encima del archivo, comprobando que no ha cambiado", async () => {
    const { copies, lastAdded, done, opened } = setup();
    await opened();

    copies.changed(7);
    await flush();
    expect(lastAdded()).toEqual(["replace", "/tmp/copias/7/foto.png", "/home/alice/foto.png", STAMP]);
    expect(copies.items[0].status).toBe("uploading");

    await done({ replaced: { size: 9, modified: 200 } });
    expect(copies.items[0]).toMatchObject({ status: "watching", stamp: { size: 9, modified: 200 } });
  });

  it("si una subida falla, avisa y el siguiente guardado lo reintenta", async () => {
    const { copies, queue, fail, onError, opened } = setup();
    await opened();
    copies.changed(7);
    await flush();

    await fail("files.permission_denied");
    expect(onError).toHaveBeenCalledWith("files.permission_denied");
    expect(copies.items[0].status).toBe("failed");

    copies.changed(7);
    await flush();
    expect(queue.enqueue).toHaveBeenCalledTimes(3);
    expect(copies.items[0].status).toBe("uploading");
  });

  it("los guardados durante una subida se suben juntos al terminar", async () => {
    const { copies, queue, done, opened } = setup();
    await opened();
    copies.changed(7);
    await flush();

    copies.changed(7);
    copies.changed(7);
    expect(queue.enqueue).toHaveBeenCalledTimes(2);

    await done({ replaced: STAMP });
    expect(queue.enqueue).toHaveBeenCalledTimes(3);
  });

  it("si cambió en el Servidor y se elige sobrescribir, sube sin comprobar", async () => {
    const { copies, choose, lastAdded, fail, opened } = setup("confirm");
    await opened();
    copies.changed(7);
    await flush();

    await fail("editor.conflict");

    expect(choose).toHaveBeenCalled();
    expect(lastAdded()).toEqual(["replace", "/tmp/copias/7/foto.png", "/home/alice/foto.png", null]);
  });

  it("si cambió en el Servidor y se guarda como copia, la vigilada pasa a ser la copia", async () => {
    const { copies, api, lastAdded, done, fail, opened } = setup("alternative");
    await opened();
    copies.changed(7);
    await flush();

    await fail("editor.conflict");
    expect(lastAdded()).toEqual(["upload", "/tmp/copias/7/foto.png", "/home/alice"]);

    api.fileStamp.mockResolvedValueOnce({ size: 5, modified: 300 });
    await done({ uploaded: "foto (1).png" });
    expect(copies.items[0]).toMatchObject({
      remotePath: "/home/alice/foto (1).png",
      name: "foto (1).png",
      stamp: { size: 5, modified: 300 },
      status: "watching",
    });
  });

  it("si cambió en el Servidor y se cancela, no sube nada hasta el siguiente guardado", async () => {
    const { copies, queue, fail, opened } = setup("cancel");
    await opened();
    copies.changed(7);
    await flush();

    await fail("editor.conflict");

    expect(queue.enqueue).toHaveBeenCalledTimes(2);
    expect(copies.items[0].status).toBe("watching");
  });

  it("solo están activas mientras pasa algo que conviene ver", async () => {
    const { copies, done, fail } = setup();
    await copies.open("/home/alice/foto.png");
    expect(copies.active).toHaveLength(1);

    await done({ downloaded: "/tmp/copias/7/foto.png" });
    expect(copies.active).toEqual([]);

    copies.changed(7);
    await flush();
    expect(copies.active).toHaveLength(1);
    await fail("files.permission_denied");
    expect(copies.active).toMatchObject([{ status: "failed" }]);

    copies.changed(7);
    await flush();
    await done({ replaced: STAMP });
    expect(copies.active).toEqual([]);
    expect(copies.items).toHaveLength(1);
  });

  it("al vaciar, lo que se estaba descargando ya no se vigila", async () => {
    const { copies, api, done } = setup();
    await copies.open("/home/alice/foto.png");

    await copies.clear();
    await done({ downloaded: "/tmp/copias/7/foto.png" });

    expect(api.clearLocalCopies).toHaveBeenCalled();
    expect(api.watchLocalCopy).not.toHaveBeenCalled();
    expect(copies.items).toEqual([]);
  });
});
