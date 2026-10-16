import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { branding } from "./branding.generated";
import { t } from "./i18n";
import SettingsWindow from "./SettingsWindow.svelte";
import { loadPreferences } from "./state/preferences.svelte";
import { start } from "./state/session.svelte";
import { listenForShortcuts } from "./state/shortcuts.svelte";
import { watchLocalCopies } from "./state/localCopies.svelte";
import { listenForDrops } from "./state/transfers.svelte";
import { checkForUpdate } from "./state/update.svelte";

document.title = branding.name;

// El menú nativo de la webview («Recargar», «Inspeccionar») no pinta nada en la app;
// solo se deja en los campos de texto y en el Editor, para poder pegar.
document.addEventListener("contextmenu", (event) => {
  const editable = event.target instanceof Element && event.target.closest("[contenteditable=true]");
  if (!(event.target instanceof HTMLInputElement) && !editable) event.preventDefault();
});

// Los Atajos valen en todas las ventanas: Ctrl+, abre Ajustes también desde Ajustes.
listenForShortcuts();
void loadPreferences();
// Todas las ventanas cargan esta interfaz; `?window=` dice cuál pintar.
const target = document.querySelector("#app")!;
const windowName = new URLSearchParams(location.search).get("window");
if (windowName === "settings") {
  document.title = t("settings.title");
  mount(SettingsWindow, { target });
} else if (windowName === "editor") {
  // CodeMirror solo se descarga en las ventanas del Editor.
  void import("./EditorWindow.svelte").then(({ default: EditorWindow }) => mount(EditorWindow, { target }));
} else {
  mount(App, { target });
  listenForDrops();
  watchLocalCopies();
  void start();
  void checkForUpdate();
}
