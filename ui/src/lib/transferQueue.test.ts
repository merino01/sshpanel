import { describe, expect, it, vi } from "vitest";
import type { Outcome, Progress } from "./api";
import { MAX_PARALLEL, TransferQueue, type NewTransfer } from "./transferQueue.svelte";

/** Núcleo simulado: cada Transferencia lanzada queda pendiente hasta que el test la termina. */
function fakeCore() {
  const running = new Map<number, { resolve: (o: Outcome) => void; reject: (code: string) => void }>();
  const api = {
    runTransfer: vi.fn(
      (id: number, _onProgress: (p: Progress) => void) =>
        new Promise<Outcome>((resolve, reject) => running.set(id, { resolve, reject })),
    ),
    cancelTransfer: vi.fn(async (_id: number) => {}),
  };
  const finish = async (id: number, outcome: Outcome = { uploaded: "x" }) => {
    running.get(id)!.resolve(outcome);
    await flush();
  };
  const failWith = async (id: number, code: string) => {
    running.get(id)!.reject(code);
    await flush();
  };
  return { api, finish, failWith };
}

const flush = () => new Promise((resolve) => setTimeout(resolve));

function setup() {
  const core = fakeCore();
  const onChanged = vi.fn();
  const onLost = vi.fn();
  const queue = new TransferQueue({ api: core.api, onChanged, onLost });
  const upload = (name: string): NewTransfer => ({ direction: "upload", name, changes: "/home/alice", describe: () => `subido ${name}` });
  const add = async (...names: string[]) => {
    for (const name of names) await queue.enqueue(upload(name), async () => {});
    await flush();
  };
  const status = () => queue.items.map((tr) => tr.status);
  return { ...core, queue, onChanged, onLost, add, status };
}

describe("TransferQueue", () => {
  it(`lanza ${MAX_PARALLEL} a la vez y la siguiente espera su turno`, async () => {
    const { queue, add, finish, status, onChanged } = setup();
    await add("a", "b", "c", "d");

    expect(status()).toEqual(["running", "running", "running", "waiting"]);

    await finish(queue.items[0].id);
    expect(status()).toEqual(["done", "running", "running", "running"]);
    expect(queue.items[0].outcome).toBe("subido a");
    expect(onChanged).toHaveBeenCalledWith("/home/alice");
  });

  it("al perder la conexión deja las que iban cortadas y siguen al reanudar", async () => {
    const { queue, add, failWith, status, onLost, api } = setup();
    await add("a", "b", "c", "d");

    await failWith(queue.items[0].id, "connection.lost");
    await failWith(queue.items[1].id, "connection.lost");

    expect(status()).toEqual(["lost", "lost", "running", "waiting"]);
    expect(onLost).toHaveBeenCalledTimes(1);

    queue.resume();
    await flush();
    expect(status()).toEqual(["running", "running", "running", "waiting"]);
    expect(api.runTransfer).toHaveBeenCalledTimes(5);
  });

  it("cancelar una en espera la quita de la cola sin lanzarla", async () => {
    const { queue, add, status, api, onChanged } = setup();
    await add("a", "b", "c", "d");

    await queue.cancel(queue.items[3]);

    expect(status()).toEqual(["running", "running", "running", "cancelled"]);
    expect(api.cancelTransfer).toHaveBeenCalledWith(queue.items[3].id);
    expect(onChanged).toHaveBeenCalledTimes(1);
  });

  it("cancelar una en curso espera a que el núcleo la pare", async () => {
    const { queue, add, failWith, status } = setup();
    await add("a");

    await queue.cancel(queue.items[0]);
    expect(queue.items[0].cancelling).toBe(true);
    expect(status()).toEqual(["running"]);

    await failWith(queue.items[0].id, "transfer.cancelled");
    expect(status()).toEqual(["cancelled"]);
    expect(queue.items[0].cancelling).toBe(false);
  });

  it("una fallida se puede reintentar", async () => {
    const { queue, add, failWith, status, finish } = setup();
    await add("a");

    await failWith(queue.items[0].id, "files.permission_denied");
    expect(status()).toEqual(["failed"]);

    queue.retry(queue.items[0]);
    await flush();
    expect(status()).toEqual(["running"]);
    await finish(queue.items[0].id);
    expect(status()).toEqual(["done"]);
  });

  it("quitar las terminadas deja el resto", async () => {
    const { queue, add, finish, status } = setup();
    await add("a", "b");
    await finish(queue.items[0].id);

    queue.clearFinished();

    expect(status()).toEqual(["running"]);
  });

  it("avisa de cómo acaba a quien la puso en la cola, salvo si se corta la conexión", async () => {
    const { queue, api, finish, failWith } = setup();
    const onDone = vi.fn();
    const onFailed = vi.fn();
    const track = (name: string): NewTransfer => ({ direction: "upload", name, describe: () => "", onDone, onFailed });
    const [a, b, c, d] = await Promise.all(["a", "b", "c", "d"].map((name) => queue.enqueue(track(name), async () => {})));
    await flush();

    await queue.cancel(d);
    expect(onFailed).toHaveBeenLastCalledWith("transfer.cancelled");

    await finish(a.id, { replaced: { size: 3, modified: 9 } });
    expect(onDone).toHaveBeenCalledWith({ replaced: { size: 3, modified: 9 } });

    await failWith(b.id, "editor.conflict");
    expect(onFailed).toHaveBeenLastCalledWith("editor.conflict");

    await failWith(c.id, "connection.lost");
    expect(onFailed).toHaveBeenCalledTimes(2);
    expect(api.runTransfer).toHaveBeenCalledTimes(3);
  });

  it("las ocultas no se pintan y salen de la cola al acabar, salvo si se corta la conexión", async () => {
    const { queue, api, finish, failWith } = setup();
    const hidden = (name: string): NewTransfer => ({ direction: "upload", name, describe: () => "", hidden: true });
    const [a, b, c] = await Promise.all(["a", "b", "c"].map((name) => queue.enqueue(hidden(name), async () => {})));
    await queue.enqueue({ direction: "upload", name: "d", describe: () => "" }, async () => {});
    await flush();

    expect(queue.visible.map((tr) => tr.name)).toEqual(["d"]);

    await finish(a.id);
    await failWith(b.id, "files.permission_denied");
    expect(api.cancelTransfer).toHaveBeenCalledWith(b.id);
    await failWith(c.id, "connection.lost");
    expect(queue.items.map((tr) => tr.name)).toEqual(["c", "d"]);

    queue.resume();
    await flush();
    expect(queue.items[0].status).toBe("running");
  });
});
