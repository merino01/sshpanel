/**
 * Atajos de la app: qué combinaciones lanzan cada acción, dónde valen y cómo
 * se leen. Las combinaciones se escriben como «Mod+Shift+N», donde «Mod» es
 * Ctrl, o Cmd en macOS; así un Atajo guardado vale en los dos sistemas. Los de
 * las pestañas de la Terminal van aparte, en `terminalTabs`, y no se pueden
 * cambiar.
 */

/** Las acciones con Atajo, en el orden en que se listan en Ajustes. */
export const ACTIONS = ["newFolder", "upload", "rename", "trash", "open", "back", "reload", "copyPath", "settings", "terminal"] as const;

export type Action = (typeof ACTIONS)[number];

/** Cada acción con sus combinaciones; casi todas tienen una, «Carpeta anterior» dos. */
export type Shortcuts = Record<Action, string[]>;

/** Lo que el Miembro ha cambiado respecto a los Atajos por defecto; es lo que se guarda en las preferencias. */
export type ShortcutOverrides = Partial<Shortcuts>;

/** Si el sistema es macOS, donde los atajos usan Cmd en vez de Ctrl. */
export const isMac = typeof navigator !== "undefined" && /Mac/i.test(navigator.platform || navigator.userAgent);

/** Los Atajos por defecto. En macOS, «Mover a la Papelera» es Cmd+Retroceso, como en el Finder. */
export function defaultShortcuts(mac = isMac): Shortcuts {
  return {
    newFolder: ["Mod+Shift+N"],
    upload: ["Mod+U"],
    rename: ["F2"],
    trash: [mac ? "Mod+Backspace" : "Delete"],
    open: ["Enter"],
    back: ["Alt+ArrowLeft", "Backspace"],
    reload: ["F5"],
    copyPath: ["Mod+Shift+C"],
    settings: ["Mod+,"],
    terminal: ["Mod+`"],
  };
}

export const DEFAULT_SHORTCUTS = defaultShortcuts();

/** Los Atajos activos: los por defecto con los cambios del Miembro encima. */
export function resolveShortcuts(overrides: ShortcutOverrides = {}, mac = isMac): Shortcuts {
  return { ...defaultShortcuts(mac), ...overrides };
}

/** `overrides` con `action` pasando a `combos`; si coincide con el de por defecto, deja de guardarse. */
export function withShortcut(overrides: ShortcutOverrides, action: Action, combos: string[], mac = isMac): ShortcutOverrides {
  const next = { ...overrides, [action]: combos };
  if (defaultShortcuts(mac)[action].join("\n") === combos.join("\n")) delete next[action];
  return next;
}

/** Lo guardado en las preferencias, sin acciones que no existen ni valores que no son listas de combinaciones. */
export function validOverrides(raw: unknown): ShortcutOverrides {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) return {};
  const valid = (combos: unknown): combos is string[] =>
    Array.isArray(combos) && combos.length > 0 && combos.every((combo) => typeof combo === "string" && combo !== "");
  return Object.fromEntries(
    Object.entries(raw).filter(([action, combos]) => (ACTIONS as readonly string[]).includes(action) && valid(combos)),
  );
}

type Keys = Pick<KeyboardEvent, "key" | "code" | "ctrlKey" | "shiftKey" | "altKey" | "metaKey">;

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "AltGraph", "Meta"]);

/**
 * Las teclas de signo por su posición (`event.code`), con el signo de un
 * teclado estadounidense. El carácter (`event.key`) cambia con Shift y según la
 * distribución (en español, Shift+, da «;») y la tecla ` es una tecla muerta
 * que ni siquiera lo da; la posición no cambia.
 */
const SYMBOL_CODES: Record<string, string> = {
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  IntlBackslash: "<",
  ...Object.fromEntries(Array.from({ length: 10 }, (_, n) => [`Digit${n}`, String(n)])),
};

/**
 * La tecla de `event`. Las letras siguen la distribución (la Z de un teclado
 * francés es su Z); los signos y los dígitos, la posición; las teclas con
 * nombre («F2», «ArrowLeft»…), su nombre.
 */
function keyOf(event: Keys): string {
  // Con Alt, macOS cambia la letra por otro carácter («å» con Alt+A).
  if (event.altKey && /^Key[A-Z]$/.test(event.code)) return event.code.slice(3);
  if (event.key === " ") return "Space";
  // Letra es lo que tiene mayúscula: «ñ» sí; «º», en la tecla de ` del teclado español, no.
  if (event.key.length === 1 && event.key.toUpperCase() !== event.key.toLowerCase()) return event.key.toUpperCase();
  if (event.key.length === 1 || event.key === "Dead") return SYMBOL_CODES[event.code] ?? event.key.toUpperCase();
  return event.key;
}

/** La combinación de `event` en el formato de los atajos, o `null` si solo se ha pulsado un modificador. */
export function comboOf(event: Keys, mac = isMac): string | null {
  if (MODIFIER_KEYS.has(event.key)) return null;
  const parts = [
    (mac ? event.metaKey : event.ctrlKey) && "Mod",
    (mac ? event.ctrlKey : event.metaKey) && (mac ? "Ctrl" : "Meta"),
    event.altKey && "Alt",
    event.shiftKey && "Shift",
    keyOf(event),
  ];
  return parts.filter(Boolean).join("+");
}

/** La acción que lanza `event` con los atajos `shortcuts`, si alguna. */
export function actionFor(event: Keys, shortcuts: Shortcuts = DEFAULT_SHORTCUTS, mac = isMac): Action | null {
  const combo = comboOf(event, mac);
  if (!combo) return null;
  return ACTIONS.find((action) => shortcuts[action].includes(combo)) ?? null;
}

/** Las combinaciones que la webview usa para recargar la página; la app las anula siempre. */
export function isReload(combo: string | null): boolean {
  return combo === "F5" || combo === "Mod+R" || combo === "Mod+Shift+R";
}

/** Las pestañas de la Terminal: Ctrl+Mayús+T, Ctrl+Mayús+W, Ctrl+AvPág y Ctrl+RePág, también Ctrl en macOS. */
function terminalTabCombos(mac: boolean): string[] {
  const ctrl = mac ? "Ctrl" : "Mod";
  return [`${ctrl}+Shift+T`, `${ctrl}+Shift+W`, `${ctrl}+PageDown`, `${ctrl}+PageUp`];
}

/** Quién ya usa una combinación: otra acción o las pestañas de la Terminal. */
export type Conflict = Action | "terminalTabs";

/** Si `combo` ya lo usa otra acción que no sea `action`, o las pestañas de la Terminal. */
export function conflictOf(shortcuts: Shortcuts, action: Action, combo: string, mac = isMac): Conflict | null {
  if (terminalTabCombos(mac).includes(combo)) return "terminalTabs";
  return ACTIONS.find((other) => other !== action && shortcuts[other].includes(combo)) ?? null;
}

/** Dónde tiene sentido cada acción: en cualquier sitio, en el Explorador o sobre una fila del Explorador. */
const SCOPES: Record<Action, "global" | "explorer" | "entry"> = {
  newFolder: "explorer",
  upload: "explorer",
  rename: "entry",
  trash: "entry",
  open: "entry",
  back: "explorer",
  reload: "explorer",
  copyPath: "explorer",
  settings: "global",
  terminal: "global",
};

/** Dónde está el Miembro al pulsar un Atajo. */
export interface ShortcutContext {
  /** El foco está en la Terminal: la consola recibe todas las teclas. */
  inTerminal: boolean;
  /** El foco está en un campo de texto. */
  inText: boolean;
  /** Hay un menú contextual o una confirmación abiertos. */
  overlay: boolean;
  /** Se ve el Explorador y no se está escribiendo un nombre. */
  explorer: boolean;
  /** Una fila del Explorador tiene el foco. */
  selected: boolean;
  /** En una pantalla de trabajo con el Servidor conectado. */
  workspace: boolean;
  advanced: boolean;
}

/** Si `action` se aplica en `ctx`. Si no, la tecla sigue su curso (en un campo de texto, borra o salta de línea). */
export function allowed(action: Action, ctx: ShortcutContext): boolean {
  if (ctx.inTerminal || ctx.overlay) return false;
  if (action === "terminal") return ctx.advanced && ctx.workspace;
  const scope = SCOPES[action];
  if (scope === "global") return true;
  if (ctx.inText || !ctx.explorer) return false;
  return scope === "explorer" || ctx.selected;
}

/** Separa «Mod+Shift+N» en modificadores y tecla, también cuando la tecla es «+». */
function split(combo: string): { modifiers: string[]; key: string } {
  const at = combo.lastIndexOf("+", combo.length - 2);
  return { modifiers: at < 0 ? [] : combo.slice(0, at).split("+"), key: combo.slice(at + 1) };
}

const ARROWS: Record<string, string> = { ArrowLeft: "←", ArrowRight: "→", ArrowUp: "↑", ArrowDown: "↓" };
const MAC_MODIFIERS: Record<string, string> = { Ctrl: "⌃", Alt: "⌥", Shift: "⇧", Mod: "⌘" };
const MAC_KEYS: Record<string, string> = { Backspace: "⌫", Delete: "⌦", Enter: "↩", Escape: "⎋", Tab: "⇥" };

/**
 * `combo` como lo lee el Miembro: en macOS con los símbolos del sistema y sin
 * «+» (⌘⇧N); fuera, con Ctrl y los nombres de `names` («Ctrl+Mayús+N»,
 * «Supr»). `names` trae los nombres de las teclas en el idioma de la interfaz.
 */
export function shortcutLabel(combo: string, names: Record<string, string> = {}, mac = isMac): string {
  const { modifiers, key } = split(combo);
  if (mac) {
    const ordered = ["Ctrl", "Alt", "Shift", "Mod"].filter((m) => modifiers.includes(m));
    return [...ordered.map((m) => MAC_MODIFIERS[m]), ARROWS[key] ?? MAC_KEYS[key] ?? names[key] ?? key].join("");
  }
  const name = (part: string) => (part === "Mod" ? "Ctrl" : (ARROWS[part] ?? names[part] ?? part));
  return [...modifiers, key].map(name).join("+");
}
