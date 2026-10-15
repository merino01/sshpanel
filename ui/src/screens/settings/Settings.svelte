<!--
  Ventana de Ajustes: las preferencias del Miembro, guardadas en su equipo. Cada
  cambio se guarda al momento y llega a las demás ventanas. A la izquierda, el
  menú de secciones; a la derecha, la elegida, cada una en su propio componente.
  Para añadir una sección, se añade a `SECTIONS`.
-->
<script lang="ts">
  import type { Component } from "svelte";
  import { t } from "../../i18n";
  import Appearance from "./Appearance.svelte";
  import LanguageSection from "./LanguageSection.svelte";
  import ShortcutsSection from "./ShortcutsSection.svelte";

  /** Cada sección: su texto, su icono (trazos de 24×24) y su componente. */
  const SECTIONS: { id: string; label: string; icon: string; component: Component }[] = [
    {
      id: "appearance",
      label: "settings.appearance",
      icon: "M12 3a9 9 0 1 0 0 18 2.5 2.5 0 0 0 0-5h1.5a2.5 2.5 0 0 0 2.5-2.5C21 8 17 3 12 3ZM7.5 11h.01M10 7.5h.01M14.5 7.5h.01",
      component: Appearance,
    },
    {
      id: "shortcuts",
      label: "settings.shortcuts",
      icon: "M4.5 6h15a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-15a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2ZM6.5 10h.01M10 10h.01M14 10h.01M17.5 10h.01M7.5 14h9",
      component: ShortcutsSection,
    },
    {
      id: "language",
      label: "settings.language",
      icon: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18ZM3.5 9h17M3.5 15h17M12 3c-2.5 2.6-3.6 5.6-3.6 9s1.1 6.4 3.6 9M12 3c2.5 2.6 3.6 5.6 3.6 9s-1.1 6.4-3.6 9",
      component: LanguageSection,
    },
  ];

  let current = $state(SECTIONS[0]);
</script>

<main class="settings">
  <nav aria-label={t("settings.title")}>
    <p class="title">{t("settings.title")}</p>
    {#each SECTIONS as section (section.id)}
      <button aria-current={current.id === section.id ? "page" : undefined} onclick={() => (current = section)}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d={section.icon} /></svg>
        {t(section.label)}
      </button>
    {/each}
    <p class="help">{t("settings.help")}</p>
  </nav>
  <div class="content">
    <current.component />
  </div>
</main>

<style>
  main.settings {
    display: grid;
    grid-template-columns: 11rem minmax(0, 1fr);
    place-content: stretch;
    justify-items: stretch;
    gap: 0;
    max-width: none;
    height: 100vh;
    min-height: 0;
    margin: 0;
    padding: 0;
    text-align: left;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 1rem 0.6rem;
    border-right: 1px solid var(--border);
    background: var(--surface);
  }
  .title {
    margin: 0 0.5rem 0.75rem;
    font-weight: 700;
  }
  nav button {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    padding: 0.45rem 0.6rem;
    border: 0;
    border-radius: 0.4rem;
    background: none;
    color: inherit;
    text-align: left;
  }
  nav button:hover {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }
  nav button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
  nav button[aria-current] {
    background: var(--accent);
    color: var(--on-accent);
  }
  svg {
    flex: none;
    width: 1.1rem;
    height: 1.1rem;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  nav .help {
    margin: auto 0.5rem 0;
    font-size: 0.8rem;
  }
  .content {
    min-height: 0;
    overflow-y: auto;
    padding: 1.4rem 1.75rem 2rem;
  }
  /* El título de cada sección, que viene de su componente. */
  .content :global(h2) {
    margin: 0 0 0.25rem;
    font-size: 1.3rem;
    color: var(--accent-text);
  }
</style>
