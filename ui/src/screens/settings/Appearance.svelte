<!--
  Ajustes › Apariencia: el Tema y, donde el equipo lo admite, la Transparencia.
  Cada Tema lleva una ventana en miniatura pintada con su paleta (fondo, texto,
  acento y una línea de la Terminal con sus colores); la de Sistema, mitad Claro
  y mitad Oscuro.
-->
<script lang="ts">
  import { branding } from "../../branding.generated";
  import { t } from "../../i18n";
  import { Switch } from "../../components/ui/switch";
  import { accentOf, glassSupported, resolveTheme, SYSTEM_THEME, themeIds, themes, type ThemePalette } from "../../lib/themes";
  import { activeTheme, preferences, setPreference } from "../../state/preferences.svelte";

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
  // Alto contraste no admite el cristal: el interruptor se queda, apagado y sin poder tocarse.
  const glassAllowed = $derived(activeTheme.palette.glass !== false);
</script>

{#snippet preview(palette: ThemePalette, half = false)}
  <span
    class="col-start-1 row-start-1 grid grid-rows-[auto_auto_1fr] content-start gap-1.5 border-t-[7px] border-(--swatch-accent) bg-(--swatch-bg) px-2.5 pt-2 {half
      ? '[clip-path:polygon(62%_0,100%_0,100%_100%,38%_100%)]'
      : ''}"
    style={colors(palette)}
  >
    <span class="h-1.5 rounded-full bg-(--swatch-fg) opacity-55"></span>
    <span class="h-1.5 w-[55%] rounded-full bg-(--swatch-fg) opacity-55"></span>
    <span class="-mx-2.5 flex items-center gap-1 self-end border-t border-(--swatch-border) bg-(--swatch-terminal) px-2.5 py-1">
      {#each ANSI as color (color)}
        <span class="size-1.5 rounded-full" style="background: {palette.terminal[color]}"></span>
      {/each}
    </span>
  </span>
{/snippet}

<section aria-labelledby="appearance">
  <h2 id="appearance" class="text-xl font-semibold tracking-tight">{t("settings.appearance")}</h2>
  <p class="mt-1 mb-5 max-w-[60ch] text-muted-foreground">{t("settings.appearance.help")}</p>
  <fieldset>
    <legend class="mb-2.5 font-medium">{t("settings.theme")}</legend>
    <div class="grid grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-3">
      {#each themeIds as id (id)}
        <label class="group relative cursor-pointer rounded-xl border border-input p-2 transition-colors hover:border-primary has-checked:border-primary has-checked:ring-1 has-checked:ring-primary has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-ring grid gap-2">
          <input
            class="pointer-events-none absolute opacity-0"
            type="radio"
            name="theme"
            value={id}
            checked={preferences.theme === id}
            onchange={() => setPreference("theme", id)}
          />
          <span class="relative grid h-16 overflow-hidden rounded-md border" aria-hidden="true">
            {#if id === SYSTEM_THEME}
              {@render preview(resolveTheme(id, false))}
              {@render preview(resolveTheme(id, true), true)}
            {:else}
              {@render preview(themes[id])}
            {/if}
          </span>
          <span class="px-0.5 text-sm group-has-checked:font-semibold">{label(id)}</span>
        </label>
      {/each}
    </div>
  </fieldset>
  {#if glassSupported}
    <label class="mt-6 flex max-w-[60ch] cursor-pointer items-start justify-between gap-6 rounded-xl border p-4 has-data-disabled:cursor-default">
      <span class="flex flex-col gap-1">
        <span class="font-medium">{t("settings.transparency")}</span>
        <span class="text-muted-foreground">{t(glassAllowed ? "settings.transparency.help" : "settings.transparency.unavailable")}</span>
      </span>
      <Switch
        class="mt-0.5"
        checked={glassAllowed && preferences.transparency}
        disabled={!glassAllowed}
        onCheckedChange={(checked) => setPreference("transparency", checked)}
      />
    </label>
  {/if}
</section>
