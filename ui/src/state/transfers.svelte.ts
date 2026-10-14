/** La cola de Transferencias de la app y cómo se le añaden: selector, arrastrar y soltar, descargar. */
import { downloadDir } from "@tauri-apps/api/path";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import { api } from "../lib/api";
import { baseName } from "../lib/paths";
import { TransferQueue } from "../lib/transferQueue.svelte";
import { explorer, refresh } from "./explorer.svelte";
import { inWorkspace, session, showConnectionLost } from "./session.svelte";

export const transfers = new TransferQueue({
  api,
  onChanged: (dir) => refresh(dir),
  onLost: (code) => {
    if (inWorkspace()) showConnectionLost(code);
  },
});

/** Carpeta a la que se subiría lo que se está arrastrando sobre la ventana, si se suelta. */
export const drop = $state<{ target: string | null }>({ target: null });

/** Sube a `dir` los archivos que elija el Miembro con el selector del sistema. */
export async function pickUploads(dir: string) {
  const paths = await open({ title: t("transfers.uploadDialog"), multiple: true });
  for (const path of paths ?? []) void queueUpload(path, dir);
}

function queueUpload(local: string, dir: string) {
  const name = baseName(local);
  return transfers.enqueue(
    {
      direction: "upload",
      name,
      changes: dir,
      describe: (outcome) => {
        const saved = "uploaded" in outcome ? outcome.uploaded : name;
        return saved === name ? t("transfers.uploaded", { path: dir }) : t("transfers.uploadedAs", { name: saved, path: dir });
      },
    },
    (id) => api.addUpload(id, local, dir),
  );
}

/** Carpeta del equipo elegida en la última descarga, para proponerla la próxima vez. */
let lastDownloadDir: string | undefined;

/** Descarga `name` de `dir` a la carpeta del equipo que elija el Miembro. */
export async function pickDownload(dir: string, name: string) {
  const defaultPath = lastDownloadDir ?? (await downloadDir().catch(() => undefined));
  const localDir = await open({ title: t("transfers.downloadDialog", { name }), directory: true, defaultPath });
  if (typeof localDir !== "string") return;
  lastDownloadDir = localDir;
  await transfers.enqueue(
    {
      direction: "download",
      name,
      describe: (outcome) => t("transfers.downloaded", { path: "downloaded" in outcome ? outcome.downloaded : localDir }),
    },
    (id) => api.addDownload(id, dir, name, localDir),
  );
}

/** Arrastrar archivos o carpetas sobre la ventana los sube a la carpeta del Explorador en pantalla. */
export function listenForDrops() {
  void getCurrentWebview().onDragDropEvent(({ payload }) => {
    const target = session.screen.name === "explorer" ? explorer.listing.path : null;
    drop.target = target !== null && (payload.type === "enter" || payload.type === "over") ? target : null;
    if (target !== null && payload.type === "drop") for (const path of payload.paths) void queueUpload(path, target);
  });
}
