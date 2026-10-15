/**
 * Los Atajos de esta ventana: los activos (los por defecto con los cambios del
 * Miembro), cómo cambiarlos y el escuchador que los aplica. Para añadir una
 * acción: en `lib/shortcuts` (acción, Atajo por defecto y ámbito) y aquí, en
 * `ACTIONS`, lo que hace.
 */
import { t } from "../i18n";
import type { Entry } from "../lib/api";
import {
  actionFor,
  allowed,
  comboOf,
  isReload,
  resolveShortcuts,
  shortcutLabel,
  withShortcut,
  type Action,
  type ShortcutContext,
  type Shortcuts,
} from "../lib/shortcuts";
import { confirmation } from "./confirm.svelte";
import { askToTrash, copyPath, explorer, goBack, openEntry, reload, selectedEntry, startCreating, startRenaming } from "./explorer.svelte";
import { menu } from "./menu.svelte";
import { openSettings, preferences, setPreference } from "./preferences.svelte";
import { inWorkspace, isAdvanced, session } from "./session.svelte";
import { openTerminal } from "./terminal.svelte";
import { pickUploads } from "./transfers.svelte";

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

// Lo que hace cada Atajo. Dónde se aplica cada uno lo decide `allowed`; las
// acciones sobre una fila solo llegan aquí con una fila con el foco.
const ACTIONS: Record<Action, () => void> = {
  newFolder: startCreating,
  upload: () => void pickUploads(explorer.listing.path),
  rename: () => withSelected(startRenaming),
  trash: () => withSelected((entry) => void askToTrash(entry.name)),
  open: () => withSelected(openEntry),
  back: () => void goBack(),
  reload,
  copyPath: () => void copyPath(selectedEntry()?.name),
  settings: openSettings,
  terminal: openTerminal,
};

function withSelected(run: (entry: Entry) => void) {
  const entry = selectedEntry();
  if (entry) run(entry);
}

/** Aplica los Atajos en esta ventana, con un solo escuchador para todos. */
export function listenForShortcuts() {
  window.addEventListener("keydown", (event) => {
    const action = actionFor(event, activeShortcuts());
    const ctx = contextOf(event);
    if (action && allowed(action, ctx)) {
      event.preventDefault();
      ACTIONS[action]();
    } else if (!ctx.inTerminal && isReload(comboOf(event))) {
      // F5 y Ctrl/Cmd+R recargarían la webview y se perdería lo que hay en pantalla.
      event.preventDefault();
    }
  });
}

/** Dónde está el Miembro al pulsar la tecla de `event`. */
function contextOf(event: KeyboardEvent): ShortcutContext {
  const target = event.target instanceof HTMLElement ? event.target : null;
  return {
    inTerminal: !!target?.closest("[data-no-shortcuts]"),
    inText: !!target && isTextField(target),
    overlay: !!menu.current || !!confirmation.current,
    explorer: session.screen.name === "explorer" && !explorer.editing,
    selected: !!selectedEntry(),
    workspace: inWorkspace(),
    advanced: isAdvanced(),
  };
}

function isTextField(element: HTMLElement): boolean {
  if (element instanceof HTMLInputElement) return !["checkbox", "radio", "button", "submit", "reset"].includes(element.type);
  return element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement || element.isContentEditable;
}
