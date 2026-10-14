import { Channel, invoke } from "@tauri-apps/api/core";
import { downloadDir } from "@tauri-apps/api/path";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import "./styles.css";
import { branding } from "./branding.generated";
import { errorMessage, t } from "./i18n";

interface Profile {
  host: string;
  username: string;
  shared_folder: string | null;
}
interface AppState {
  profile: Profile | null;
  public_key: string | null;
}
interface Entry {
  name: string;
  kind: "file" | "dir" | "symlink";
  size: number;
  modified: number | null;
}
interface Listing {
  path: string;
  entries: Entry[];
}
interface Progress {
  done: number;
  total: number;
}

const app = document.querySelector<HTMLElement>("#app")!;
document.title = branding.name;
document.documentElement.style.setProperty("--accent", branding.accentColor);

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Partial<HTMLElementTagNameMap[K]> = {},
  ...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
  const node = Object.assign(document.createElement(tag), props);
  node.append(...children);
  return node;
}

function render(...nodes: Node[]) {
  closeMenu();
  current = null;
  app.replaceChildren(...nodes);
}

interface MenuItem {
  label: string;
  action: () => void;
}

/** Menú contextual abierto, si lo hay. Vive fuera de `#app` para que `render()` no lo destruya a medias. */
let menu: HTMLElement | null = null;

function closeMenu() {
  menu?.remove();
  menu = null;
}

/**
 * Abre un menú contextual junto al puntero o, si se abrió con el teclado
 * (tecla Menú o Shift+F10), junto a `anchor`. Al cancelarlo con Escape el foco
 * vuelve a `anchor`.
 */
function showContextMenu(event: MouseEvent, anchor: HTMLElement, label: string, items: MenuItem[]) {
  event.preventDefault();
  event.stopPropagation();
  closeMenu();

  const buttons = items.map(({ label, action }) => {
    const button = el("button", { role: "menuitem", textContent: label, tabIndex: -1 });
    button.addEventListener("click", () => {
      closeMenu();
      action();
    });
    return button;
  });
  const list = el("ul", { className: "context-menu", role: "menu", ariaLabel: label }, ...buttons.map((b) => el("li", { role: "none" }, b)));
  list.addEventListener("keydown", (e) => {
    const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const target = { ArrowDown: current + 1, ArrowUp: current - 1, Home: 0, End: buttons.length - 1 }[e.key];
    if (target !== undefined) {
      e.preventDefault();
      buttons[(target + buttons.length) % buttons.length].focus();
    } else if (e.key === "Escape" || e.key === "Tab") {
      e.preventDefault();
      closeMenu();
      anchor.focus();
    }
  });
  document.body.append(list);
  menu = list;

  const fromKeyboard = event.clientX === 0 && event.clientY === 0;
  const rect = anchor.getBoundingClientRect();
  const x = fromKeyboard ? rect.left + 16 : event.clientX;
  const y = fromKeyboard ? rect.bottom : event.clientY;
  list.style.left = `${Math.max(4, Math.min(x, innerWidth - list.offsetWidth - 4))}px`;
  list.style.top = `${Math.max(4, Math.min(y, innerHeight - list.offsetHeight - 4))}px`;
  buttons[0]?.focus();
}

document.addEventListener("pointerdown", (e) => menu && !menu.contains(e.target as Node) && closeMenu(), true);
addEventListener("blur", closeMenu);
addEventListener("resize", closeMenu);
// El menú nativo de la webview («Recargar», «Inspeccionar») no pinta nada en la app;
// solo se deja en los campos de texto, para poder pegar.
document.addEventListener("contextmenu", (e) => {
  if (!(e.target instanceof HTMLInputElement)) e.preventDefault();
});

function showWelcome(error?: string) {
  const button = el("button", { className: "primary", textContent: t("welcome.import") });
  button.addEventListener("click", importProfile);
  render(
    el("h1", {}, t("welcome.title", { name: branding.name })),
    el("p", {}, t("welcome.subtitle")),
    button,
    ...(error ? [el("p", { className: "error", role: "alert" }, error)] : []),
  );
}

/** Clave pública del Miembro, cuando el Servidor todavía no la acepta. */
function showKey(state: AppState, notice: string) {
  const { profile, public_key } = state;
  if (!profile || !public_key) return showWelcome();

  const copy = el("button", { className: "primary", textContent: t("ready.copyKey") });
  const feedback = el("p", { className: "feedback", role: "status" });
  copy.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(public_key);
      feedback.textContent = t("ready.copied");
    } catch {
      feedback.textContent = t("ready.copyFailed");
    }
  });

  render(
    el("h1", {}, t("unauthorized.title")),
    el("p", {}, t("ready.subtitle", { username: profile.username, host: profile.host })),
    el("p", { className: "notice", role: "alert" }, notice),
    el("h2", {}, t("ready.keyHeading")),
    el("p", { className: "help" }, t("ready.keyHelp")),
    el("code", { className: "key" }, public_key),
    copy,
    feedback,
    retryButton(state),
    importAnotherButton(),
  );
}

function showProblem(state: AppState, message: string) {
  render(
    el("h1", {}, t("problem.title")),
    el("p", { className: "error", role: "alert" }, message),
    retryButton(state, "primary"),
    importAnotherButton(),
  );
}

/** Nombre que el Miembro está escribiendo: una carpeta nueva o un renombrado. */
type Editing = { kind: "create"; value: string } | { kind: "rename"; name: string; value: string };

/** Lo que muestra el Explorador. */
interface Explorer {
  state: AppState;
  home: string;
  listing: Listing;
  message?: string;
  editing?: Editing;
}

/** Explorador en pantalla, si es lo que se ve. */
let current: Explorer | null = null;

function showExplorer(ex: Explorer) {
  const { listing, editing } = ex;
  const shared = ex.state.profile?.shared_folder;
  const parent = parentOf(listing.path);

  const up = el("button", { className: "secondary", textContent: t("explorer.up"), disabled: parent === null });
  up.addEventListener("click", () => parent !== null && openFolder(ex, parent));
  const newFolder = el("button", { className: "secondary", textContent: t("explorer.newFolder") });
  newFolder.addEventListener("click", () => startCreating(ex));
  const upload = el("button", { className: "secondary", textContent: t("explorer.upload") });
  upload.addEventListener("click", () => pickUploads(listing.path));

  const rows = listing.entries.map((entry) =>
    editing?.kind === "rename" && editing.name === entry.name ? nameForm(ex, editing, entry) : entryRow(ex, entry),
  );
  if (editing?.kind === "create") rows.unshift(nameForm(ex, editing));

  // Clic derecho en el hueco de la carpeta, fuera de cualquier fila.
  const folderArea: HTMLElement = rows.length ? el("ul", { className: "files" }, ...rows) : el("p", { className: "help empty" }, t("files.empty"));
  folderArea.addEventListener("contextmenu", (event) => {
    if ((event.target as Element).closest("li")) return;
    showContextMenu(event, folderArea, t("explorer.menuFolderLabel"), [{ label: t("explorer.newFolder"), action: () => startCreating(ex) }]);
  });

  render(
    el("h1", {}, t("files.title")),
    el(
      "nav",
      { className: "shortcuts" },
      shortcut(ex, t("explorer.home"), ex.home),
      ...(shared ? [shortcut(ex, t("explorer.shared"), shared)] : []),
    ),
    el("div", { className: "toolbar" }, up, el("p", { className: "path" }, listing.path), newFolder, upload),
    ...(ex.message ? [el("p", { className: "error", role: "alert" }, ex.message)] : []),
    folderArea,
  );
  current = ex;
  app.querySelector<HTMLInputElement>(".files input")?.focus();
}

function shortcut(ex: Explorer, label: string, path: string) {
  const button = el("button", { className: "shortcut", textContent: label });
  if (ex.listing.path === path) button.ariaCurrent = "location";
  button.addEventListener("click", () => openFolder(ex, path));
  return button;
}

function entryRow(ex: Explorer, entry: Entry) {
  const isDir = entry.kind === "dir";
  const open = () => openFolder(ex, joinPath(ex.listing.path, entry.name));
  // La fila es la que recibe el foco; el botón del nombre solo sirve para el ratón.
  const name = isDir
    ? el("button", { className: "name", textContent: entry.name, title: t("explorer.open", { name: entry.name }), tabIndex: -1 })
    : el("span", { className: "name", title: entry.name }, entry.name);
  if (isDir) name.addEventListener("click", open);

  const row = el(
    "li",
    { className: `entry ${entry.kind}`, tabIndex: 0 },
    el("span", { className: "icon", ariaHidden: "true" }, isDir ? "📁" : "📄"),
    name,
    el("span", { className: "size" }, entry.kind === "file" ? formatSize(entry.size) : ""),
  );
  row.addEventListener("keydown", (event) => {
    if (event.target === row && event.key === "Enter" && isDir) open();
  });
  row.addEventListener("contextmenu", (event) =>
    showContextMenu(event, row, t("explorer.menuLabel", { name: entry.name }), entryMenuItems(ex, entry, open)),
  );
  return row;
}

/** Acciones del menú contextual de una fila. */
function entryMenuItems(ex: Explorer, entry: Entry, open: () => void): MenuItem[] {
  const items: MenuItem[] = [];
  if (entry.kind === "dir") items.push({ label: t("explorer.openShort"), action: open });
  // Un enlace roto no tiene nada que descargar.
  if (entry.kind !== "symlink") items.push({ label: t("explorer.download"), action: () => pickDownload(ex.listing.path, entry.name) });
  items.push(
    {
      label: t("explorer.rename"),
      action: () => showExplorer({ ...ex, message: undefined, editing: { kind: "rename", name: entry.name, value: entry.name } }),
    },
    { label: t("explorer.newFolder"), action: () => startCreating(ex) },
  );
  return items;
}

function startCreating(ex: Explorer) {
  showExplorer({ ...ex, message: undefined, editing: { kind: "create", value: "" } });
}

/** Fila con un campo para escribir el nombre de una carpeta nueva o el nuevo nombre de `entry`. */
function nameForm(ex: Explorer, editing: Editing, entry?: Entry) {
  const label = editing.kind === "create" ? t("explorer.newFolderLabel") : t("explorer.renameLabel", { name: editing.name });
  const input = el("input", { type: "text", value: editing.value, ariaLabel: label, spellcheck: false });
  const submit = el("button", {
    type: "submit",
    className: "primary small",
    textContent: editing.kind === "create" ? t("explorer.create") : t("explorer.save"),
  });
  const cancel = el("button", { type: "button", className: "link", textContent: t("explorer.cancel") });
  const close = () => showExplorer({ ...ex, message: undefined, editing: undefined });
  cancel.addEventListener("click", close);
  input.addEventListener("keydown", (event) => event.key === "Escape" && close());
  // Al renombrar, selecciona el nombre sin la extensión.
  if (editing.kind === "rename") {
    const dot = editing.value.lastIndexOf(".");
    queueMicrotask(() => input.setSelectionRange(0, dot > 0 && entry?.kind !== "dir" ? dot : editing.value.length));
  }

  const form = el("form", { className: "name-form" }, input, submit, cancel);
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    submit.disabled = true;
    const dir = ex.listing.path;
    try {
      if (editing.kind === "create") await invoke("create_folder", { parent: dir, name: input.value });
      else await invoke("rename", { dir, from: editing.name, to: input.value });
      await openFolder(ex, dir);
    } catch (code) {
      fail({ ...ex, editing: { ...editing, value: input.value } }, code);
    }
  });

  return el(
    "li",
    { className: `entry editing ${entry?.kind ?? "dir"}` },
    el("span", { className: "icon", ariaHidden: "true" }, entry && entry.kind !== "dir" ? "📄" : "📁"),
    form,
  );
}

async function openFolder(ex: Explorer, path: string) {
  try {
    const listing = await invoke<Listing>("list_dir", { path });
    showExplorer({ state: ex.state, home: ex.home, listing });
  } catch (code) {
    fail(ex, code);
  }
}

/** Una operación del Explorador ha fallado: se explica sin salir de la carpeta, salvo si se ha caído la conexión. */
function fail(ex: Explorer, code: unknown) {
  if (code === "connection.lost" || code === "connection.not_connected") showProblem(ex.state, errorMessage(code));
  else showExplorer({ ...ex, message: errorMessage(code) });
}

/** Dónde ha quedado una Transferencia terminada. */
type Outcome = { uploaded: string } | { downloaded: string };

/** Una Transferencia de la cola. */
interface Transfer {
  id: number;
  direction: "upload" | "download";
  name: string;
  /** `lost`: cortada al perder la conexión; sigue sola al volver a conectar. */
  status: "waiting" | "running" | "done" | "failed" | "lost" | "cancelled";
  progress: Progress | null;
  /** Se ha pedido cancelarla y aún no se ha parado. */
  cancelling?: boolean;
  /** Al terminar, dónde ha quedado; si falla, qué ha pasado. */
  outcome?: string;
  /** Explica dónde ha quedado al terminar. */
  describe: (outcome: Outcome) => string;
  /** Carpeta del Servidor que cambia al terminar, para refrescarla si está en pantalla. */
  changes?: string;
}

/** Cuántas Transferencias van a la vez; las demás esperan su turno. */
const MAX_PARALLEL = 3;
const transfers: Transfer[] = [];
let nextTransferId = 1;
/** Falso desde que se pierde la conexión hasta que se recupera: mientras, no se lanza nada. */
let online = true;
/** Vive fuera de `#app` para seguir a la vista al navegar o al perder la conexión. */
const transferPanel = el("aside", { className: "transfers", ariaLabel: t("transfers.title"), hidden: true });
document.body.append(transferPanel);

/** Registra la Transferencia en el núcleo con `add` y la pone en la cola. */
async function enqueue(transfer: Pick<Transfer, "direction" | "name" | "describe" | "changes">, add: (id: number) => Promise<unknown>) {
  const id = nextTransferId++;
  await add(id);
  transfers.push({ ...transfer, id, status: "waiting", progress: null });
  renderTransfers();
  pump();
}

/** Lanza Transferencias en espera, por orden de llegada, hasta tener {@link MAX_PARALLEL} en curso. */
function pump() {
  if (!online) return;
  let running = transfers.filter((tr) => tr.status === "running").length;
  for (const transfer of transfers) {
    if (running >= MAX_PARALLEL) break;
    if (transfer.status !== "waiting") continue;
    running++;
    void runTransfer(transfer);
  }
}

/** Lanza la Transferencia, o la reanuda donde se quedó si ya lo intentó antes. */
async function runTransfer(transfer: Transfer) {
  transfer.status = "running";
  transfer.outcome = undefined;
  renderTransfers();
  const onProgress = new Channel<Progress>();
  onProgress.onmessage = (progress) => {
    transfer.progress = progress;
    if (transfer.status === "running") renderTransfers();
  };
  try {
    transfer.outcome = transfer.describe(await invoke<Outcome>("run_transfer", { id: transfer.id, onProgress }));
    transfer.status = "done";
  } catch (code) {
    if (code === "transfer.cancelled") {
      transfer.status = "cancelled";
      transfer.outcome = t("transfers.cancelled");
    } else if (code === "connection.lost" || code === "connection.not_connected") {
      transfer.status = "lost";
      transfer.outcome = t("transfers.lost");
      if (online && current) fail(current, code);
      online = false;
    } else {
      transfer.status = "failed";
      transfer.outcome = errorMessage(code);
    }
  }
  transfer.cancelling = false;
  renderTransfers();
  if (transfer.status !== "lost") refreshChanged(transfer);
  pump();
}

/** Refresca la carpeta que cambia `transfer` si está en pantalla: al terminar, o al borrar lo que dejó a medias. */
function refreshChanged(transfer: Transfer) {
  if (current && !current.editing && current.listing.path === transfer.changes) void openFolder(current, transfer.changes);
}

/** Al volver a conectar, lo que se cortó sigue donde se quedó. */
function resumeTransfers() {
  online = true;
  for (const transfer of transfers) if (transfer.status === "lost") transfer.status = "waiting";
  renderTransfers();
  pump();
}

/** Para la Transferencia y borra lo que había creado. Si está en curso, `runTransfer` recoge el final. */
async function cancelTransfer(transfer: Transfer) {
  if (transfer.cancelling) return;
  if (transfer.status === "running") {
    transfer.cancelling = true;
  } else {
    transfer.status = "cancelled";
    transfer.outcome = t("transfers.cancelled");
  }
  renderTransfers();
  const wasRunning = transfer.cancelling;
  await invoke("cancel_transfer", { id: transfer.id }).catch(() => {});
  // Si estaba en curso, `runTransfer` refresca al pararse.
  if (!wasRunning) refreshChanged(transfer);
}

function retryTransfer(transfer: Transfer) {
  transfer.status = "waiting";
  transfer.outcome = undefined;
  renderTransfers();
  pump();
}

function renderTransfers() {
  const isFinished = (tr: Transfer) => tr.status === "done" || tr.status === "cancelled";
  const clear = el("button", { className: "link", textContent: t("transfers.clear"), hidden: !transfers.some(isFinished) });
  clear.addEventListener("click", () => {
    transfers.splice(0, transfers.length, ...transfers.filter((tr) => !isFinished(tr)));
    renderTransfers();
  });
  transferPanel.hidden = transfers.length === 0;
  transferPanel.replaceChildren(
    el("header", {}, el("h2", {}, t("transfers.title")), clear),
    el("ul", {}, ...transfers.map(transferRow)),
  );
}

function transferRow(transfer: Transfer) {
  const { status, progress } = transfer;
  const isUpload = transfer.direction === "upload";
  // Sin `value`, la barra se muestra indeterminada mientras se prepara.
  const bar = el("progress", { max: 1 });
  if (status === "done") bar.value = 1;
  else if (progress) bar.value = progress.total ? Math.min(1, progress.done / progress.total) : 1;
  else if (status !== "running") bar.value = 0;

  let detail = transfer.outcome ?? t("transfers.waiting");
  if (status === "running") {
    detail = progress
      ? t("transfers.progress", { done: formatSize(Math.min(progress.done, progress.total)), total: formatSize(progress.total) })
      : t("transfers.preparing");
  }

  const actions = el("span", { className: "actions" });
  if (status === "failed") actions.append(transferButton(t("transfers.retry"), t("transfers.retryLabel", { name: transfer.name }), () => retryTransfer(transfer)));
  if (status !== "done" && status !== "cancelled") {
    const cancel = transferButton(t("transfers.cancel"), t("transfers.cancelLabel", { name: transfer.name }), () => cancelTransfer(transfer));
    cancel.disabled = !!transfer.cancelling;
    actions.append(cancel);
  }
  return el(
    "li",
    { className: `transfer ${status}` },
    el("span", { className: "icon", title: t(isUpload ? "transfers.upload" : "transfers.download") }, isUpload ? "↑" : "↓"),
    el("span", { className: "name", title: transfer.name }, transfer.name),
    actions,
    bar,
    el("span", { className: status === "failed" ? "detail error" : "detail", title: detail }, detail),
  );
}

function transferButton(text: string, label: string, action: () => void) {
  const button = el("button", { className: "link", textContent: text, ariaLabel: label, title: label });
  button.addEventListener("click", action);
  return button;
}

/** Sube a `dir` los archivos que elija el Miembro con el selector del sistema. */
async function pickUploads(dir: string) {
  const paths = await open({ title: t("transfers.uploadDialog"), multiple: true });
  for (const path of paths ?? []) void queueUpload(path, dir);
}

function queueUpload(local: string, dir: string) {
  const name = local.split(/[\\/]/).filter(Boolean).pop() ?? local;
  return enqueue(
    {
      direction: "upload",
      name,
      changes: dir,
      describe: (outcome) => {
        const saved = "uploaded" in outcome ? outcome.uploaded : name;
        return saved === name ? t("transfers.uploaded", { path: dir }) : t("transfers.uploadedAs", { name: saved, path: dir });
      },
    },
    (id) => invoke("add_upload", { id, local, remoteDir: dir }),
  );
}

/** Carpeta del equipo elegida en la última descarga, para proponerla la próxima vez. */
let lastDownloadDir: string | undefined;

/** Descarga `name` de `dir` a la carpeta del equipo que elija el Miembro. */
async function pickDownload(dir: string, name: string) {
  const defaultPath = lastDownloadDir ?? (await downloadDir().catch(() => undefined));
  const localDir = await open({ title: t("transfers.downloadDialog", { name }), directory: true, defaultPath });
  if (typeof localDir !== "string") return;
  lastDownloadDir = localDir;
  await enqueue(
    {
      direction: "download",
      name,
      describe: (outcome) => t("transfers.downloaded", { path: "downloaded" in outcome ? outcome.downloaded : localDir }),
    },
    (id) => invoke("add_download", { id, remoteDir: dir, name, localDir }),
  );
}

// Arrastrar archivos o carpetas sobre la ventana los sube a la carpeta que se está viendo.
void getCurrentWebview().onDragDropEvent(({ payload }) => {
  const target = current?.listing.path;
  if (target !== undefined && (payload.type === "enter" || payload.type === "over")) {
    document.body.dataset.drop = t("transfers.dropHere", { path: target });
    return;
  }
  delete document.body.dataset.drop;
  if (target !== undefined && payload.type === "drop") for (const path of payload.paths) void queueUpload(path, target);
});

/** Carpeta que contiene `path`, o `null` en la raíz. */
function parentOf(path: string): string | null {
  const trimmed = path.replace(/\/+$/, "");
  if (trimmed === "") return null;
  const slash = trimmed.lastIndexOf("/");
  return slash <= 0 ? "/" : trimmed.slice(0, slash);
}

function joinPath(dir: string, name: string): string {
  return `${dir.replace(/\/+$/, "")}/${name}`;
}

function formatSize(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${unit === 0 ? value : value.toFixed(1)} ${units[unit]}`;
}

function retryButton(state: AppState, className = "secondary") {
  const button = el("button", { className, textContent: t("connect.retry") });
  button.addEventListener("click", () => connect(state));
  return button;
}

function importAnotherButton() {
  const button = el("button", { className: "link", textContent: t("ready.importAnother") });
  button.addEventListener("click", importProfile);
  return button;
}

async function connect(state: AppState) {
  render(el("h1", {}, t("connect.title")), el("p", { role: "status" }, t("connect.wait")));
  try {
    const listing = await invoke<Listing>("connect");
    showExplorer({ state, home: listing.path, listing });
    resumeTransfers();
  } catch (code) {
    if (code === "connection.unauthorized") showKey(state, errorMessage(code));
    else showProblem(state, errorMessage(code));
  }
}

async function importProfile() {
  const path = await open({
    title: t("welcome.dialogTitle"),
    multiple: false,
    filters: [{ name: t("welcome.profileFiles", { name: branding.name }), extensions: [branding.profileExtension] }],
  });
  if (typeof path !== "string") return;
  try {
    await connect(await invoke<AppState>("import_profile", { path }));
  } catch (code) {
    showWelcome(errorMessage(code));
  }
}

async function start() {
  try {
    const state = await invoke<AppState>("app_state");
    state.profile ? await connect(state) : showWelcome();
  } catch (code) {
    showWelcome(errorMessage(code));
  }
}

void start();
