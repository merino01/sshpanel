/** Los Atajos activos en esta ventana (los por defecto con los cambios del Miembro) y cómo cambiarlos. */
import { t } from "../i18n";
import { resolveShortcuts, shortcutLabel, withShortcut, type Action, type Shortcuts } from "../lib/shortcuts";
import { preferences, setPreference } from "./preferences.svelte";

export function activeShortcuts(): Shortcuts {
  return resolveShortcuts(preferences.shortcuts);
}

/** Cambia el Atajo de `action` por `combo`; sustituye a todos los que tuviera. */
export function setShortcut(action: Action, combo: string) {
  void setPreference("shortcuts", withShortcut(preferences.shortcuts, action, [combo]));
}

/** Vuelve a los Atajos por defecto. */
export function resetShortcuts() {
  void setPreference("shortcuts", {});
}

/** Las teclas con nombre que cambian con el idioma; el resto se leen igual en todos. */
const NAMED_KEYS = ["Shift", "Meta", "Delete", "Backspace", "Enter", "Escape", "Space", "Tab", "Insert", "Home", "End", "PageUp", "PageDown"];

/** `combo` como lo lee el Miembro: «Ctrl+Mayús+N», o «⇧⌘N» en macOS. */
export function label(combo: string): string {
  return shortcutLabel(combo, Object.fromEntries(NAMED_KEYS.map((key) => [key, t(`shortcuts.key.${key}`)])));
}
