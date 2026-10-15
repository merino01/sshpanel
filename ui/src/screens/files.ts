/**
 * Las clases que comparten las listas de archivos del Explorador y la Papelera:
 * la cabecera de columnas y cada fila. Cada pantalla pone sus columnas.
 */

/** Columnas del Explorador: icono, nombre, tamaño y fecha. */
export const EXPLORER_COLUMNS = "grid-cols-[1.25rem_minmax(0,1fr)_7rem_10rem]";
/** Columnas de la Papelera: icono, nombre, de dónde vino, cuándo se borró y tamaño. */
export const TRASH_COLUMNS = "grid-cols-[1.25rem_minmax(0,1fr)_minmax(0,1fr)_10rem_7rem]";

export const HEADER = "grid shrink-0 items-center gap-3 border-b px-4 py-1.5 text-xs font-medium text-muted-foreground select-none";

/** Una fila; con `data-selected` se marca en el color de acento, como en Finder. */
export const ROW =
  "group grid h-8 cursor-default items-center gap-3 rounded-md px-2.5 select-none hover:bg-accent focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring data-selected:bg-primary data-selected:text-primary-foreground data-selected:focus-visible:outline-primary-foreground/60";

/** El texto secundario de una fila (tamaño, fecha, origen). */
export const MUTED = "truncate text-xs tabular-nums text-muted-foreground group-data-selected:text-primary-foreground/80";

export const EMPTY = "px-6 py-8 text-center text-muted-foreground";
