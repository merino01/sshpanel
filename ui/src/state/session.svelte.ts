/** Qué pantalla se ve y cómo se llega a ella: arrancar, importar el Perfil y conectar. */
import { open } from "@tauri-apps/plugin-dialog";
import { branding } from "../branding.generated";
import { errorMessage, t } from "../i18n";
import { api, type AppState } from "../lib/api";
import { enterExplorer } from "./explorer.svelte";
import { closeMenu } from "./menu.svelte";
import { transfers } from "./transfers.svelte";

export type Screen =
  | { name: "starting" }
  | { name: "welcome"; error?: string }
  /** El Servidor aún no acepta la clave del Miembro. */
  | { name: "key"; notice: string }
  | { name: "connecting" }
  | { name: "problem"; message: string }
  | { name: "explorer" }
  | { name: "trash" };

export const session = $state<{ screen: Screen; appState: AppState | null }>({ screen: { name: "starting" }, appState: null });

export function show(screen: Screen) {
  closeMenu();
  session.screen = screen;
}

/** Se ha caído la conexión: se explica y se ofrece «Reintentar». */
export function showConnectionLost(code: unknown) {
  show({ name: "problem", message: errorMessage(code) });
}

export async function start() {
  try {
    session.appState = await api.appState();
    if (session.appState.profile) await connect();
    else show({ name: "welcome" });
  } catch (code) {
    show({ name: "welcome", error: errorMessage(code) });
  }
}

export async function connect() {
  show({ name: "connecting" });
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
  if (typeof path !== "string") return;
  try {
    session.appState = await api.importProfile(path);
  } catch (code) {
    return show({ name: "welcome", error: errorMessage(code) });
  }
  await connect();
}
