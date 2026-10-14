<!--
  Ruta de la carpeta en pantalla. Cada parte lleva a esa carpeta; la última es
  la actual. Si no cabe, se ve el final y el principio se desplaza.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import { breadcrumbs, placeOf } from "../../lib/paths";
  import { allowedPlaces, explorer, openFolder } from "../../state/explorer.svelte";

  let list = $state<HTMLOListElement>();
  // En Modo simple, la ruta empieza en «Mi carpeta» o «Carpeta compartida».
  const parts = $derived.by(() => {
    const path = explorer.listing.path;
    const places = allowedPlaces();
    return breadcrumbs(path, places ? placeOf(path, places) : undefined);
  });

  // Al cambiar de carpeta, que se vea la actual aunque la ruta sea larga.
  $effect(() => {
    void parts;
    if (list) list.scrollLeft = list.scrollWidth;
  });
</script>

<nav aria-label={t("explorer.breadcrumbs")}>
  <ol bind:this={list}>
    {#each parts as part, i (part.path)}
      <li>
        {#if i === parts.length - 1}
          <span aria-current="page">{part.name}</span>
        {:else}
          <button onclick={() => openFolder(part.path)}>{part.name}</button>
        {/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
  nav {
    min-width: 0;
  }
  ol {
    display: flex;
    align-items: center;
    margin: 0;
    padding: 0;
    overflow-x: auto;
    scrollbar-width: none;
    list-style: none;
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
    white-space: nowrap;
  }
  li + li::before {
    content: "›";
    padding: 0 0.3rem;
    opacity: 0.5;
  }
  button {
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    opacity: 0.7;
  }
  button:hover {
    color: var(--accent);
    opacity: 1;
    text-decoration: underline;
  }
  [aria-current] {
    font-weight: 600;
  }
</style>
