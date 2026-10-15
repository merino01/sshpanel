import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { branding } from "./branding.generated";
import { t } from "./i18n";
import { actionFor, type Action } from "./lib/shortcuts";
import SettingsWindow from "./SettingsWindow.svelte";
import { loadPreferences, openSettings } from "./state/preferences.svelte";
import { start } from "./state/session.svelte";
import { listenForDrops } from "./state/transfers.svelte";
import { checkForUpdate } from "./state/update.svelte";

document.title = branding.name;
document.documentElement.style.setProperty("--accent", branding.accentColor);

// El menú nativo de la webview («Recargar», «Inspeccionar») no pinta nada en la app;
// solo se deja en los campos de texto, para poder pegar.
document.addEventListener("contextmenu", (event) => {
  if (!(event.target instanceof HTMLInputElement)) event.preventDefault();
});

// Lo que hace cada Atajo, en cualquier ventana y pantalla.
const actions: Record<Action, () => void> = {
  settings: openSettings,
};
window.addEventListener("keydown", (event) => {
  const action = actionFor(event);
  if (!action) return;
  event.preventDefault();
  actions[action]();
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
