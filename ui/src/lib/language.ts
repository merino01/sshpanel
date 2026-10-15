/**
 * Idiomas de la interfaz. El Miembro elige uno en Ajustes o deja «system», que
 * sigue el idioma del equipo: español si es español, y si no, inglés.
 */

export const LANGUAGES = ["es", "en"] as const;

export type Language = (typeof LANGUAGES)[number];

/** La preferencia que sigue el idioma del equipo. */
export const SYSTEM_LANGUAGE = "system";

/** El idioma cuando el del equipo no es ninguno de los de la app. */
const FALLBACK: Language = "en";

/** El nombre de cada idioma en ese mismo idioma, para que lo reconozca quien no entiende el activo. */
export const LANGUAGE_NAMES: Record<Language, string> = { es: "Español", en: "English" };

export function isLanguage(value: unknown): value is Language {
  return typeof value === "string" && (LANGUAGES as readonly string[]).includes(value);
}

/** Si `value` vale como preferencia de idioma: «system» o uno de `LANGUAGES`. */
export function isLanguagePreference(value: unknown): value is string {
  return value === SYSTEM_LANGUAGE || isLanguage(value);
}

/**
 * El idioma que se ve con la preferencia `preference`. Con «system», el de
 * `systemLanguage` (una etiqueta BCP 47 como `es-ES`) si la app lo tiene y, si
 * no, inglés.
 */
export function resolveLanguage(preference: string, systemLanguage: string | undefined): Language {
  if (isLanguage(preference)) return preference;
  const primary = (systemLanguage ?? "").split("-")[0].toLowerCase();
  return isLanguage(primary) ? primary : FALLBACK;
}
