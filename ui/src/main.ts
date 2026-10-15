import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { branding } from "./branding.generated";
import { t } from "./i18n";
import type { Entry } from "./lib/api";
import { actionFor, allowed, comboOf, isReload, type Action, type ShortcutContext } from "./lib/shortcuts";
import SettingsWindow from "./SettingsWindow.svelte";
import { confirmation } from "./state/confirm.svelte";
import {
  askToTrash,
  copyPath,
  explorer,
  goBack,
  openEntry,
  reload,
  selectedEntry,
  startCreating,
  startRenaming,
} from "./state/explorer.svelte";
import { menu } from "./state/menu.svelte";
import { loadPreferences, openSettings } from "./state/preferences.svelte";
import { inWorkspace, isAdvanced, session, start } from "./state/session.svelte";
import { activeShortcuts } from "./state/shortcuts.svelte";
import { openTerminal } from "./state/terminal.svelte";
import { listenForDrops, pickUploads } from "./state/transfers.svelte";
import { checkForUpdate } from "./state/update.svelte";

document.title = branding.name;

// El menú nativo de la webview («Recargar», «Inspeccionar») no pinta nada en la app;
// solo se deja en los campos de texto, para poder pegar.
document.addEventListener("contextmenu", (event) => {
  if (!(event.target instanceof HTMLInputElement)) event.preventDefault();
});

// Lo que hace cada Atajo. Dónde se aplica cada uno lo decide `allowed`; las
// acciones sobre una fila solo llegan aquí con una fila con el foco.
const actions: Record<Action, () => void> = {
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

// Un solo escuchador para todos los Atajos, en cualquier ventana y pantalla.
window.addEventListener("keydown", (event) => {
  const action = actionFor(event, activeShortcuts());
  const ctx = contextOf(event);
  if (action && allowed(action, ctx)) {
    event.preventDefault();
    actions[action]();
  } else if (!ctx.inTerminal && isReload(comboOf(event))) {
    // F5 y Ctrl/Cmd+R recargarían la webview y se perdería lo que hay en pantalla.
    event.preventDefault();
  }
});

void loadPreferences();
// Todas las ventanas cargan esta interfaz; `?window=` dice cuál pintar.
const target = document.querySelector("#app")!;
if (new URLSearchParams(location.search).get("window") === "settings") {
  document.title = t("settings.title");
  mount(SettingsWindow, { target });
} else {
  mount(App, { target });
  listenForDrops();
  void start();
  void checkForUpdate();
}
