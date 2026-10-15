/**
 * Preferencias del Miembro (Ajustes). El fichero las guarda tal cual; aquí se
 * completan con los valores por defecto, para que una preferencia nueva no
 * rompa un fichero antiguo ni un valor que ya no existe deje la app a medias.
 */

export const THEMES = ["system", "light", "dark"] as const;

export type Theme = (typeof THEMES)[number];

export interface Preferences {
  /** «system» sigue el modo claro u oscuro del equipo. */
  theme: Theme;
}

export const DEFAULT_PREFERENCES: Preferences = { theme: "system" };

/** Las preferencias de `raw` (lo leído del fichero), con el valor por defecto en las que faltan o no valen. */
export function withDefaults(raw: unknown): Preferences {
  const saved = typeof raw === "object" && raw !== null ? (raw as Record<string, unknown>) : {};
  return {
    theme: THEMES.includes(saved.theme as Theme) ? (saved.theme as Theme) : DEFAULT_PREFERENCES.theme,
  };
}
