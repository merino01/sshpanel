<!--
  Ajustes › Apariencia: el Tema. Cada opción lleva una ventana en miniatura
  pintada con la paleta de su Tema (fondo, texto, acento y una línea de la
  Terminal con sus colores); la de Sistema, mitad Claro y mitad Oscuro.
-->
<script lang="ts">
  import { branding } from "../../branding.generated";
  import { t } from "../../i18n";
  import { accentOf, resolveTheme, SYSTEM_THEME, themeIds, themes, type ThemePalette } from "../../lib/themes";
  import { preferences, setPreference } from "../../state/preferences.svelte";

  /** Los colores de la Terminal que enseña la miniatura. */
  const ANSI = ["red", "green", "yellow", "blue", "magenta", "cyan"] as const;

  /** El nombre del Tema: el de la interfaz si lo tiene y, si no, el de su fichero. */
  function label(id: string): string {
    const key = `settings.theme.${id}`;
    const text = t(key);
    return text !== key ? text : (themes[id]?.name ?? id);
  }

  function colors(palette: ThemePalette): string {
    const { colors, terminal } = palette;
    return [
      `--swatch-bg: ${colors.bg}`,
      `--swatch-fg: ${colors.fg}`,
      `--swatch-border: ${colors.border}`,
      `--swatch-accent: ${accentOf(palette, branding.accentColor)}`,
      `--swatch-terminal: ${terminal.background}`,
    ].join("; ");
  }
</script>

{#snippet preview(palette: ThemePalette, half = false)}
  <span class="preview" class:half style={colors(palette)}>
    <span class="line"></span>
    <span class="line short"></span>
    <span class="terminal">
      {#each ANSI as color (color)}
        <span class="dot" style="background: {palette.terminal[color]}"></span>
      {/each}
    </span>
  </span>
{/snippet}

<section aria-labelledby="appearance">
  <h2 id="appearance">{t("settings.appearance")}</h2>
  <fieldset>
    <legend>{t("settings.theme")}</legend>
    <div class="themes">
      {#each themeIds as id (id)}
        <label class="theme">
          <input
            type="radio"
            name="theme"
            value={id}
            checked={preferences.theme === id}
            onchange={() => setPreference("theme", id)}
          />
          <span class="swatch" aria-hidden="true">
            {#if id === SYSTEM_THEME}
              {@render preview(resolveTheme(id, false))}
              {@render preview(resolveTheme(id, true), true)}
            {:else}
              {@render preview(themes[id])}
            {/if}
          </span>
          <span class="name">{label(id)}</span>
        </label>
      {/each}
    </div>
  </fieldset>
</section>

<style>
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
    border: 1px solid var(--border-strong);
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
  /* Una ventana en miniatura con los colores de cada Tema; Sistema, una encima de otra cortada en diagonal. */
  .swatch {
    position: relative;
    display: grid;
    height: 4rem;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 0.35rem;
  }
  .preview {
    grid-area: 1 / 1;
    display: grid;
    grid-template-rows: auto auto 1fr;
    align-content: start;
    gap: 0.3rem;
    padding: 0.5rem 0.6rem 0;
    border-top: 0.45rem solid var(--swatch-accent);
    background: var(--swatch-bg);
  }
  .preview.half {
    clip-path: polygon(62% 0, 100% 0, 100% 100%, 38% 100%);
  }
  .line {
    height: 0.3rem;
    border-radius: 0.15rem;
    background: var(--swatch-fg);
    opacity: 0.55;
  }
  .line.short {
    width: 55%;
  }
  .terminal {
    display: flex;
    gap: 0.2rem;
    align-items: center;
    align-self: end;
    margin: 0 -0.6rem;
    padding: 0.25rem 0.6rem;
    border-top: 1px solid var(--swatch-border);
    background: var(--swatch-terminal);
  }
  .dot {
    width: 0.35rem;
    height: 0.35rem;
    border-radius: 50%;
  }
  .name {
    padding: 0 0.15rem;
    font-size: 0.9rem;
  }
  .theme:has(input:checked) .name {
    font-weight: 600;
  }
</style>
