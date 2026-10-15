<!--
  Ventana de Ajustes: las preferencias del Miembro, guardadas en su equipo. Cada
  cambio se guarda al momento y llega a las demás ventanas. Cada grupo es una
  sección, en su propio componente; de momento, la Apariencia con el Tema.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import Appearance from "./Appearance.svelte";
</script>

<main class="settings">
  <h1>{t("settings.title")}</h1>
  <p class="help">{t("settings.help")}</p>

  <Appearance />
</main>

<style>
  main.settings {
    display: block;
    max-width: 34rem;
    min-height: 0;
    margin: 0;
    padding: 1.5rem 2rem 2rem;
    text-align: left;
  }
  h1 {
    margin: 0 0 0.25rem;
    font-size: 1.4rem;
  }
  /* El título de cada sección, que viene de su componente. */
  main.settings :global(h2) {
    margin: 2rem 0 0.75rem;
    padding-bottom: 0.4rem;
    border-bottom: 1px solid var(--border);
    font-size: 1rem;
  }
</style>
