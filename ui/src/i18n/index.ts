import es from "./es.json";

const messages: Record<string, string> = es;

/** Devuelve el texto de la interfaz para `key`, sustituyendo `{variables}`. */
export function t(key: string, vars: Record<string, string> = {}): string {
  const template = messages[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (_, name: string) => vars[name] ?? `{${name}}`);
}
