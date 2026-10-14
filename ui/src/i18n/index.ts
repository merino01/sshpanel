import es from "./es.json";

const messages: Record<string, string> = es;

/** Devuelve el texto de la interfaz para `key`, sustituyendo `{variables}`. */
export function t(key: string, vars: Record<string, string> = {}): string {
  const template = messages[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (_, name: string) => vars[name] ?? `{${name}}`);
}

/** Mensaje en lenguaje simple para un código de error del núcleo. */
export function errorMessage(code: unknown): string {
  const key = `error.${String(code)}`;
  return key in messages ? t(key) : t("error.unknown");
}
