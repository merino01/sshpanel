/**
 * El idioma que se ve en esta ventana. Empieza con el del equipo, para que la
 * ventana no se pinte en otro, y lo cambian las preferencias al cargarse y con
 * cada cambio hecho en cualquier ventana (`state/preferences`).
 */
import { resolveLanguage, SYSTEM_LANGUAGE, type Language } from "../lib/language";

/** El idioma del equipo, en BCP 47 (`es-ES`). */
export const systemLanguage: string | undefined = typeof navigator === "undefined" ? undefined : navigator.language;

export const locale = $state<{ current: Language }>({ current: resolveLanguage(SYSTEM_LANGUAGE, systemLanguage) });
