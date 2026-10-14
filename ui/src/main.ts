import { invoke } from "@tauri-apps/api/core";
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
  app.replaceChildren(...nodes);
}

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

function showExplorer(ex: Explorer) {
  const { listing, editing } = ex;
  const shared = ex.state.profile?.shared_folder;
  const parent = parentOf(listing.path);

  const up = el("button", { className: "secondary", textContent: t("explorer.up"), disabled: parent === null });
  up.addEventListener("click", () => parent !== null && openFolder(ex, parent));
  const newFolder = el("button", { className: "secondary", textContent: t("explorer.newFolder") });
  newFolder.addEventListener("click", () => showExplorer({ ...ex, message: undefined, editing: { kind: "create", value: "" } }));

  const rows = listing.entries.map((entry) =>
    editing?.kind === "rename" && editing.name === entry.name ? nameForm(ex, editing, entry) : entryRow(ex, entry),
  );
  if (editing?.kind === "create") rows.unshift(nameForm(ex, editing));

  render(
    el("h1", {}, t("files.title")),
    el(
      "nav",
      { className: "shortcuts" },
      shortcut(ex, t("explorer.home"), ex.home),
      ...(shared ? [shortcut(ex, t("explorer.shared"), shared)] : []),
    ),
    el("div", { className: "toolbar" }, up, el("p", { className: "path" }, listing.path), newFolder),
    ...(ex.message ? [el("p", { className: "error", role: "alert" }, ex.message)] : []),
    rows.length ? el("ul", { className: "files" }, ...rows) : el("p", { className: "help" }, t("files.empty")),
  );
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
  const name = isDir
    ? el("button", { className: "name", textContent: entry.name, title: t("explorer.open", { name: entry.name }) })
    : el("span", { className: "name", title: entry.name }, entry.name);
  if (isDir) name.addEventListener("click", () => openFolder(ex, joinPath(ex.listing.path, entry.name)));

  const rename = el("button", { className: "link", textContent: t("explorer.rename") });
  rename.addEventListener("click", () =>
    showExplorer({ ...ex, message: undefined, editing: { kind: "rename", name: entry.name, value: entry.name } }),
  );

  return el(
    "li",
    { className: `entry ${entry.kind}` },
    el("span", { className: "icon", ariaHidden: "true" }, isDir ? "📁" : "📄"),
    name,
    el("span", { className: "size" }, entry.kind === "file" ? formatSize(entry.size) : ""),
    rename,
  );
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
