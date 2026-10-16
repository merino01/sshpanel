/** La Administración: sus secciones y las Claves públicas autorizadas de cada usuario, leídas del Servidor. */
import { errorMessage } from "../i18n";
import { api, type UserKeys } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { resetCreateForm } from "./createProfile.svelte";
import { session, show, showConnectionLost } from "./session.svelte";

export const admin = $state<{ keys: UserKeys[] | null; loading: boolean; error?: string }>({ keys: null, loading: false });

/** Abre la Administración en las Claves, leyéndolas de nuevo. */
export function openAdmin() {
  resetCreateForm();
  show({ name: "admin", section: "keys" });
  void loadKeys();
}

export function showSection(section: "keys" | "createProfile") {
  if (session.screen.name === "admin") show({ name: "admin", section });
}

/** Lee las claves de cada usuario. */
export async function loadKeys() {
  admin.loading = true;
  admin.error = undefined;
  try {
    admin.keys = await api.authorizedKeys();
  } catch (code) {
    if (isConnectionError(code)) return showConnectionLost(code);
    admin.error = errorMessage(code);
  } finally {
    admin.loading = false;
  }
}
