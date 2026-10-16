/** Lo que la app teclea en la consola del Servidor. */

/**
 * `text` como una sola palabra para la consola, entre comillas simples: ningún
 * carácter (espacios, `$`, `;`, `*`, comillas…) tiene significado dentro.
 */
export function shellQuote(text: string): string {
  return `'${text.replaceAll("'", `'\\''`)}'`;
}
