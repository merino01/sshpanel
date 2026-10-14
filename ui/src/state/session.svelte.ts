/** Qué pantalla se ve y cómo se llega a ella: arrancar, importar el Perfil y conectar. */
import { open } from "@tauri-apps/plugin-dialog";
import { branding } from "../branding.generated";
import { errorMessage, t } from "../i18n";
import { api, type AppState } from "../lib/api";
import { confirm } from "./confirm.svelte";
import { enterExplorer } from "./explorer.svelte";
import { closeMenu } from "./menu.svelte";
import { resetTerminals } from "./terminal.svelte";
import { toasts } from "./toasts.svelte";
import { transfers } from "./transfers.svelte";

export type Screen =
  | { name: "starting" }
  | { name: "welcome"; error?: string }
  /** El Servidor aún no acepta la clave del Miembro. */
  | { name: "key"; notice: string }
  | { name: "connecting" }
  | { name: "problem"; message: string }
  | { name: "explorer" }
  | { name: "trash" }
  /** Solo en Modo avanzado. */
  | { name: "terminal" }
  /** Para el Administrador; `back` es la pantalla a la que se vuelve. */
  | { name: "createProfile"; back: Screen };

/**
 * Si cada pantalla es de trabajo con el Servidor conectado: llevan la barra
 * lateral, las Terminales siguen abiertas y, si se cae la conexión, se sale de
 * ellas a «Reintentar». Si depende de cómo se llegó, es una función. Al añadir
 * una pantalla, TypeScript obliga a decidirlo aquí.
 */
const WORKSPACE: Record<Screen["name"], boolean | ((screen: Screen) => boolean)> = {
  starting: false,
  welcome: false,
  key: false,
  connecting: false,
  problem: false,
  explorer: true,
  trash: true,
  terminal: true,
  // Se llega desde la barra lateral o desde la Bienvenida, sin Perfil.
  createProfile: (screen) => screen.name === "createProfile" && isWorkspace(screen.back),
};

function isWorkspace(screen: Screen): boolean {
  const workspace = WORKSPACE[screen.name];
  return typeof workspace === "function" ? workspace(screen) : workspace;
}

export const session = $state<{ screen: Screen; appState: AppState | null }>({ screen: { name: "starting" }, appState: null });

/** Si se está en una pantalla de trabajo con el Servidor conectado (Explorador, Papelera, Terminal…). */
export function inWorkspace(): boolean {
  return isWorkspace(session.screen);
}

/** Si el Perfil es de Modo avanzado: Terminal y borrar para siempre de la Papelera. */
export function isAdvanced(): boolean {
  return session.appState?.profile?.mode === "advanced";
}

export function show(screen: Screen) {
  closeMenu();
  session.screen = screen;
}

/** Se ha caído la conexión: se explica y se ofrece «Reintentar». */
export function showConnectionLost(code: unknown) {
  show({ name: "problem", message: errorMessage(code) });
}

export async function start() {
  void api.onProfileOpened(() => void openPendingProfile());
  try {
    session.appState = await api.appState();
    if (await openPendingProfile()) return;
    if (session.appState.profile) await connect();
    else show({ name: "welcome" });
  } catch (code) {
    show({ name: "welcome", error: errorMessage(code) });
  }
}

export async function connect() {
  show({ name: "connecting" });
  resetTerminals();
  try {
    enterExplorer(await api.connect());
    transfers.resume();
  } catch (code) {
    if (code !== "connection.unauthorized") show({ name: "problem", message: errorMessage(code) });
    else if (session.appState?.profile && session.appState.public_key) show({ name: "key", notice: errorMessage(code) });
    else show({ name: "welcome" });
  }
}

/** Pide el fichero de Perfil con el selector del sistema, lo importa y conecta. */
export async function importProfile() {
  const path = await open({
    title: t("welcome.dialogTitle"),
    multiple: false,
    filters: [{ name: t("welcome.profileFiles", { name: branding.name }), extensions: [branding.profileExtension] }],
  });
  if (typeof path === "string") await importProfileFrom(path);
}

/**
 * Importa el Perfil abierto con doble clic, si lo hay. Si ya había otro y es
 * distinto, pregunta antes de cambiar. Devuelve si lo ha importado.
 */
export async function openPendingProfile(): Promise<boolean> {
  const path = await api.takeOpenedProfile().catch(() => null);
  if (!path) return false;
  const current = session.appState?.profile;
  if (current) {
    let next;
    try {
      next = await api.readProfile(path);
    } catch (code) {
      toasts.error(errorMessage(code));
      return false;
    }
    if (JSON.stringify(next) === JSON.stringify(current)) return false;
    const confirmed = await confirm({
      title: t("open.confirmTitle", { username: next.username, host: next.host }),
      body: t("open.confirmHelp", { username: current.username, host: current.host }),
      confirmLabel: t("open.confirm"),
    });
    if (!confirmed) return false;
  }
  await importProfileFrom(path);
  return true;
}

/** Importa el Perfil del fichero `path` y conecta con él. */
async function importProfileFrom(path: string) {
  try {
    session.appState = await api.importProfile(path);
  } catch (code) {
    return show({ name: "welcome", error: errorMessage(code) });
  }
  await connect();
}
