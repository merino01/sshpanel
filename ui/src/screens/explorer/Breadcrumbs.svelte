<!--
  Ruta de la carpeta en pantalla. Cada parte lleva a esa carpeta; la última es
  la actual. Si no cabe, se ve el final y el principio se desplaza.
-->
<script lang="ts">
  import Separator from "~icons/heroicons/chevron-right-16-solid";
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

<nav class="min-w-0" aria-label={t("explorer.breadcrumbs")}>
  <ol bind:this={list} class="flex items-center overflow-x-auto font-mono text-xs whitespace-nowrap [scrollbar-width:none]">
    {#each parts as part, i (part.path)}
      <li class="flex items-center">
        {#if i > 0}<Separator class="size-3.5 shrink-0 text-muted-foreground/60" aria-hidden="true" />{/if}
        {#if i === parts.length - 1}
          <span class="px-1.5 py-0.5 font-semibold" aria-current="page">{part.name}</span>
        {:else}
          <button
            class="rounded-md px-1.5 py-0.5 text-muted-foreground hover:bg-foreground/8 hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
            onclick={() => openFolder(part.path)}
          >
            {part.name}
          </button>
        {/if}
      </li>
    {/each}
  </ol>
</nav>
