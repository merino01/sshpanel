<!--
  Ventana de Ajustes: las preferencias del Miembro, guardadas en su equipo. Cada
  cambio se guarda al momento y llega a las demás ventanas. Cada grupo es una
  sección; de momento, la Apariencia con el Tema.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import { THEMES } from "../../lib/preferences";
  import { preferences, setPreference } from "../../state/preferences.svelte";
</script>

<main class="settings">
  <h1>{t("settings.title")}</h1>
  <p class="help">{t("settings.help")}</p>

  <section aria-labelledby="appearance">
    <h2 id="appearance">{t("settings.appearance")}</h2>
    <fieldset>
      <legend>{t("settings.theme")}</legend>
      <div class="themes">
        {#each THEMES as theme (theme)}
          <label class="theme">
            <input
              type="radio"
              name="theme"
              value={theme}
              checked={preferences.theme === theme}
              onchange={() => setPreference("theme", theme)}
            />
            <span class="swatch {theme}" aria-hidden="true">
              <span class="line"></span>
              <span class="line short"></span>
            </span>
            <span class="name">{t(`settings.theme.${theme}`)}</span>
          </label>
        {/each}
      </div>
    </fieldset>
  </section>
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
  h2 {
    margin: 2rem 0 0.75rem;
    padding-bottom: 0.4rem;
    border-bottom: 1px solid rgba(127, 127, 127, 0.25);
    font-size: 1rem;
  }
  fieldset {
    margin: 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: 0.6rem;
    font-size: 0.9rem;
    font-weight: 600;
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(8.5rem, 1fr));
    gap: 0.75rem;
  }
  .theme {
    position: relative;
    display: grid;
    gap: 0.5rem;
    padding: 0.5rem;
    border: 1px solid rgba(127, 127, 127, 0.35);
    border-radius: 0.6rem;
    cursor: pointer;
  }
  .theme:hover {
    border-color: var(--accent);
  }
  .theme:has(input:checked) {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .theme:has(input:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  /* Una ventana en miniatura con los colores de cada Tema; Sistema, mitad y mitad. */
  .swatch {
    display: grid;
    align-content: start;
    gap: 0.3rem;
    height: 3.5rem;
    padding: 0.55rem 0.6rem;
    border: 1px solid rgba(127, 127, 127, 0.3);
    border-radius: 0.35rem;
    border-top: 0.45rem solid var(--accent);
  }
  .swatch.light {
    background: #ffffff;
    --line: #1f2937;
  }
  .swatch.dark {
    background: #111827;
    --line: #f3f4f6;
  }
  .swatch.system {
    background: linear-gradient(115deg, #ffffff 50%, #111827 50%);
    --line: #6b7280;
  }
  .line {
    height: 0.3rem;
    border-radius: 0.15rem;
    background: var(--line);
    opacity: 0.55;
  }
  .line.short {
    width: 55%;
  }
  .name {
    padding: 0 0.15rem;
    font-size: 0.9rem;
  }
  .theme:has(input:checked) .name {
    font-weight: 600;
  }
</style>
