<!-- La ventana de Ajustes: sus avisos van aparte de los de la ventana principal. -->
<script lang="ts">
  import TitleBar from "./components/TitleBar.svelte";
  import Toasts from "./components/Toasts.svelte";
  import { t } from "./i18n";
  import { api } from "./lib/api";
  import Settings from "./screens/settings/Settings.svelte";

  // El título sigue al idioma: Rust la abrió con el del momento.
  $effect(() => {
    document.title = t("settings.title");
    api.setWindowTitle(document.title).catch(() => {});
  });
</script>

<div class="grid h-screen grid-rows-[auto_minmax(0,1fr)] bg-surface glass:bg-glass">
  <TitleBar>
    <span class="font-semibold">{t("settings.title")}</span>
  </TitleBar>
  <Settings />
</div>
<Toasts />
