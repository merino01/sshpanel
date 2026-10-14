import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { branding } from "./branding.generated";
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

mount(App, { target: document.querySelector("#app")! });
listenForDrops();
void start();
void checkForUpdate();
