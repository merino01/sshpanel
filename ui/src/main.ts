import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { branding } from "./branding.generated";
import { t } from "./i18n";
import SettingsWindow from "./SettingsWindow.svelte";
import { loadPreferences } from "./state/preferences.svelte";
import { start } from "./state/session.svelte";
import { listenForShortcuts } from "./state/shortcuts.svelte";
import { listenForDrops } from "./state/transfers.svelte";
import { checkForUpdate } from "./state/update.svelte";

document.title = branding.name;

// El menú nativo de la webview («Recargar», «Inspeccionar») no pinta nada en la app;
// solo se deja en los campos de texto, para poder pegar.
document.addEventListener("contextmenu", (event) => {
  if (!(event.target instanceof HTMLInputElement)) event.preventDefault();
});

// Los Atajos valen en todas las ventanas: Ctrl+, abre Ajustes también desde Ajustes.
listenForShortcuts();
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
