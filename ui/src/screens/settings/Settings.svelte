<!--
  Ventana de Ajustes: las preferencias del Miembro, guardadas en su equipo. Cada
  cambio se guarda al momento y llega a las demás ventanas. A la izquierda, el
  menú de secciones; a la derecha, la elegida, cada una en su propio componente.
  Para añadir una sección, se añade a `SECTIONS`.
-->
<script lang="ts">
  import type { Component } from "svelte";
  import Swatch from "~icons/heroicons/swatch";
  import Keys from "~icons/heroicons/command-line";
  import Language from "~icons/heroicons/language";
  import { t } from "../../i18n";
  import Appearance from "./Appearance.svelte";
  import LanguageSection from "./LanguageSection.svelte";
  import ShortcutsSection from "./ShortcutsSection.svelte";

  /** Cada sección: su texto, su icono y su componente. */
  const SECTIONS: { id: string; label: string; icon: Component; component: Component }[] = [
    { id: "appearance", label: "settings.appearance", icon: Swatch, component: Appearance },
    { id: "shortcuts", label: "settings.shortcuts", icon: Keys, component: ShortcutsSection },
    { id: "language", label: "settings.language", icon: Language, component: LanguageSection },
  ];

  let current = $state(SECTIONS[0]);
</script>

<main class="grid min-h-0 grid-cols-[12rem_minmax(0,1fr)]">
  <nav class="flex min-h-0 flex-col gap-0.5 px-2 pt-1 pb-3" aria-label={t("settings.title")}>
    {#each SECTIONS as section (section.id)}
      <button
        class="group flex h-8 items-center gap-2.5 rounded-lg px-2.5 text-left transition-colors hover:bg-foreground/8 focus-visible:outline-2 focus-visible:outline-ring aria-[current]:bg-primary aria-[current]:text-primary-foreground"
        aria-current={current.id === section.id ? "page" : undefined}
        onclick={() => (current = section)}
      >
        <section.icon class="size-[18px] shrink-0 text-primary group-aria-[current]:text-primary-foreground" aria-hidden="true" />
        {t(section.label)}
      </button>
    {/each}
    <p class="mt-auto px-2.5 text-xs text-muted-foreground">{t("settings.help")}</p>
  </nav>
  <div class="mr-2 mb-2 min-h-0 overflow-y-auto rounded-xl border border-glass-edge bg-background px-7 py-6 shadow-[0_1px_3px_var(--shadow)]">
    <current.component />
  </div>
</main>
