import type { Language } from "../lib/language";
import en from "./en.json";
import es from "./es.json";
import { locale } from "./locale.svelte";

export { locale };

/** Los textos de la interfaz en cada idioma; un test comprueba que tienen las mismas claves. */
export const MESSAGES: Record<Language, Record<string, string>> = { es, en };

/**
 * Devuelve el texto de la interfaz para `key` en el idioma activo, sustituyendo
 * `{variables}`. Lee `locale`, así que en una plantilla o un `$derived` se
 * vuelve a calcular al cambiar el idioma.
 */
export function t(key: string, vars: Record<string, string> = {}): string {
  const template = MESSAGES[locale.current][key] ?? MESSAGES.es[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (_, name: string) => vars[name] ?? `{${name}}`);
}

/** Mensaje en lenguaje simple para un código de error del núcleo. */
export function errorMessage(code: unknown): string {
  const key = `error.${String(code)}`;
  return key in MESSAGES[locale.current] ? t(key) : t("error.unknown");
}
