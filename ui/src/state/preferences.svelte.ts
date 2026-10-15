/** Las preferencias del Miembro en esta ventana: se cargan al abrirla y siguen los cambios hechos en cualquier otra. */
import { errorMessage, t } from "../i18n";
import { api } from "../lib/api";
import { DEFAULT_PREFERENCES, withDefaults, type Preferences } from "../lib/preferences";
import { toasts } from "./toasts.svelte";

export const preferences = $state<Preferences>({ ...DEFAULT_PREFERENCES });

/** Carga las preferencias y escucha los cambios; aplica el Tema cada vez. */
export async function loadPreferences() {
  void api.onPreferencesChanged((raw) => apply(withDefaults(raw)));
  apply(withDefaults(await api.readPreferences().catch(() => null)));
}

/** Cambia una preferencia y la guarda; el resto de ventanas se entera por el evento. */
export async function setPreference<K extends keyof Preferences>(key: K, value: Preferences[K]) {
  apply({ ...preferences, [key]: value });
  try {
    await api.savePreferences({ ...preferences });
  } catch (code) {
    toasts.error(errorMessage(code));
  }
}

/** Abre Ajustes en su ventana. */
export function openSettings() {
  api.openSettings(t("settings.title")).catch((code) => toasts.error(errorMessage(code)));
}

function apply(next: Preferences) {
  Object.assign(preferences, next);
  // Sin atributo, el Tema Sistema: lo resuelve `prefers-color-scheme` en app.css.
  if (next.theme === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = next.theme;
}
