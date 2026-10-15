import type { Language } from "./language";

/** Un tamaño en bytes con la unidad más grande que quede por encima de 1, con los números de `language`. */
export function formatSize(bytes: number, language: Language): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  const decimals = unit === 0 ? 0 : 1;
  const number = new Intl.NumberFormat(language, { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
  return `${number.format(value)} ${units[unit]}`;
}

/**
 * Fecha y hora, en corto y con el formato de `language`, de unos segundos desde
 * la época Unix. `timeZone` solo hace falta en los tests; si no, la del equipo.
 */
export function formatDate(seconds: number, language: Language, timeZone?: string): string {
  return new Date(seconds * 1000).toLocaleString(language, { dateStyle: "short", timeStyle: "short", timeZone });
}
