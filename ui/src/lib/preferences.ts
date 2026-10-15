/**
 * Preferencias del Miembro (Ajustes). El fichero las guarda tal cual; aquí se
 * completan con los valores por defecto, para que una preferencia nueva no
 * rompa un fichero antiguo ni un valor que ya no existe deje la app a medias.
 */

import { isLanguagePreference, SYSTEM_LANGUAGE } from "./language";
import { validOverrides, type ShortcutOverrides } from "./shortcuts";
import { isTheme, SYSTEM_THEME } from "./themes";

export interface Preferences {
  /** El id de un Tema de `themes/`, o «system», que sigue el modo claro u oscuro del equipo. */
  theme: string;
  /** Solo los Atajos que el Miembro ha cambiado; el resto, los de por defecto. */
  shortcuts: ShortcutOverrides;
  /** «es», «en», o «system», que sigue el idioma del equipo. */
  language: string;
}

export const DEFAULT_PREFERENCES: Preferences = { theme: SYSTEM_THEME, shortcuts: {}, language: SYSTEM_LANGUAGE };

/** Las preferencias de `raw` (lo leído del fichero), con el valor por defecto en las que faltan o no valen. */
export function withDefaults(raw: unknown): Preferences {
  const saved = typeof raw === "object" && raw !== null ? (raw as Record<string, unknown>) : {};
  return {
    theme: isTheme(saved.theme) ? saved.theme : DEFAULT_PREFERENCES.theme,
    shortcuts: validOverrides(saved.shortcuts),
    language: isLanguagePreference(saved.language) ? saved.language : DEFAULT_PREFERENCES.language,
  };
}
