import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./styles.css";
import { branding } from "./branding.generated";
import { errorMessage, t } from "./i18n";

interface Profile {
  host: string;
  username: string;
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

function showFiles(listing: Listing) {
  const rows = listing.entries.map((entry) =>
    el(
      "li",
      { className: `entry ${entry.kind}` },
      el("span", { className: "icon", ariaHidden: "true" }, entry.kind === "dir" ? "📁" : "📄"),
      el("span", { className: "name" }, entry.name),
      el("span", { className: "size" }, entry.kind === "file" ? formatSize(entry.size) : ""),
    ),
  );
  render(
    el("h1", {}, t("files.title")),
    el("p", { className: "path" }, listing.path),
    rows.length ? el("ul", { className: "files" }, ...rows) : el("p", { className: "help" }, t("files.empty")),
  );
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
    showFiles(await invoke<Listing>("connect"));
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
