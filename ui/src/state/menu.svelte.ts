/** Menú contextual abierto, si lo hay. Lo pinta `components/ContextMenu.svelte`, montado una sola vez. */

export interface MenuItem {
  label: string;
  action: () => void;
  /** Borra algo: se pinta en el color de error. */
  destructive?: boolean;
}

export interface OpenMenu {
  label: string;
  items: MenuItem[];
  /** Elemento sobre el que se abrió; recibe el foco al cancelar. */
  anchor: HTMLElement;
  /** Dónde se abrió con el ratón; `null` si fue con el teclado (tecla Menú o Shift+F10). */
  pointer: { x: number; y: number } | null;
}

export const menu = $state<{ current: OpenMenu | null }>({ current: null });

/**
 * Abre un menú contextual junto al puntero o, si se abrió con el teclado,
 * junto a `anchor`. Sustituye al que hubiera abierto.
 */
export function openMenu(event: MouseEvent, anchor: HTMLElement, label: string, items: MenuItem[]) {
  event.preventDefault();
  event.stopPropagation();
  const fromKeyboard = event.clientX === 0 && event.clientY === 0;
  menu.current = { label, items, anchor, pointer: fromKeyboard ? null : { x: event.clientX, y: event.clientY } };
}

export function closeMenu() {
  menu.current = null;
}
