/** Las preferencias del Miembro en esta ventana: se cargan al abrirla y siguen los cambios hechos en cualquier otra. */
import { branding } from "../branding.generated";
import { errorMessage, t } from "../i18n";
import { locale, systemLanguage } from "../i18n/locale.svelte";
import { api } from "../lib/api";
import { resolveLanguage } from "../lib/language";
import { DEFAULT_PREFERENCES, withDefaults, type Preferences } from "../lib/preferences";
import { cssVariables, glassOn, resolveTheme, type ThemePalette } from "../lib/themes";
import { toasts } from "./toasts.svelte";

export const preferences = $state<Preferences>({ ...DEFAULT_PREFERENCES });

const darkScheme = window.matchMedia("(prefers-color-scheme: dark)");
// Con Sistema, el Tema cambia en cuanto el equipo pasa a claro u oscuro.
darkScheme.addEventListener("change", () => apply(preferences));

/** La paleta del Tema que se ve ahora (con Sistema, la de Claro u Oscuro), para lo que no se pinta con CSS: la Terminal. */
export const activeTheme = $state<{ palette: ThemePalette }>({
  palette: resolveTheme(DEFAULT_PREFERENCES.theme, darkScheme.matches),
});

/** Carga las preferencias y escucha los cambios; aplica el Tema y el idioma cada vez. */
export async function loadPreferences() {
  // Mientras se lee el fichero, los valores por defecto: la ventana no se pinta nunca sin colores.
  apply(preferences);
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

/**
 * Guarda las preferencias en el estado, pinta el Tema como variables CSS en
 * `:root` (con la clase `dark` en los oscuros y `data-glass` si la ventana va de
 * cristal) y cambia el idioma: los textos de la ventana se traducen solos.
 */
function apply(next: Preferences) {
  Object.assign(preferences, next);
  locale.current = resolveLanguage(next.language, systemLanguage);
  document.documentElement.lang = locale.current;
  const palette = resolveTheme(next.theme, darkScheme.matches);
  activeTheme.palette = palette;
  const root = document.documentElement.style;
  for (const [name, value] of Object.entries(cssVariables(palette, branding.accentColor))) root.setProperty(name, value);
  root.colorScheme = palette.scheme;
  document.documentElement.classList.toggle("dark", palette.scheme === "dark");
  document.documentElement.toggleAttribute("data-glass", glassOn(next.transparency, palette));
}
