/**
 * Atajos de la app: qué combinación lanza cada acción. Las combinaciones se
 * escriben como «Mod+Shift+N», donde «Mod» es Ctrl, o Cmd en macOS. Los de las
 * pestañas de la Terminal van aparte, en `terminalTabs`.
 */

export type Action = "settings";

export type Shortcuts = Record<Action, string>;

export const DEFAULT_SHORTCUTS: Shortcuts = {
  settings: "Mod+,",
};

type Keys = Pick<KeyboardEvent, "key" | "ctrlKey" | "shiftKey" | "altKey" | "metaKey">;

/** Si el sistema es macOS, donde los atajos usan Cmd en vez de Ctrl. */
export const isMac = typeof navigator !== "undefined" && /Mac/i.test(navigator.platform || navigator.userAgent);

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

/** La combinación de `event` en el formato de los atajos, o `null` si solo se ha pulsado un modificador. */
export function comboOf(event: Keys, mac = isMac): string | null {
  if (MODIFIER_KEYS.has(event.key)) return null;
  const parts = [
    (mac ? event.metaKey : event.ctrlKey) && "Mod",
    (mac ? event.ctrlKey : event.metaKey) && (mac ? "Ctrl" : "Meta"),
    event.altKey && "Alt",
    event.shiftKey && "Shift",
    event.key.length === 1 ? event.key.toUpperCase() : event.key,
  ];
  return parts.filter(Boolean).join("+");
}

/** La acción que lanza `event` con los atajos `shortcuts`, si alguna. */
export function actionFor(event: Keys, shortcuts: Shortcuts = DEFAULT_SHORTCUTS, mac = isMac): Action | null {
  const combo = comboOf(event, mac);
  if (!combo) return null;
  const found = Object.entries(shortcuts).find(([, shortcut]) => shortcut === combo);
  return found ? (found[0] as Action) : null;
}
