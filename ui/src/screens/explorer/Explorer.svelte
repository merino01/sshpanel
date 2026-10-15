<!--
  Explorador: barra con «Carpeta anterior», «Carpeta superior», la ruta y
  «Subir archivos», y debajo el contenido de la carpeta a todo el ancho.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import { isWithin, parentOf } from "../../lib/paths";
  import { allowedPlaces, explorer, goBack, openFolder, startCreating } from "../../state/explorer.svelte";
  import { openMenu } from "../../state/menu.svelte";
  import { pickUploads } from "../../state/transfers.svelte";
  import Breadcrumbs from "./Breadcrumbs.svelte";
  import EntryRow from "./EntryRow.svelte";
  import NameForm from "./NameForm.svelte";

  // En Modo simple, no se sube más allá de la carpeta personal o la Carpeta compartida.
  const parent = $derived.by(() => {
    const path = parentOf(explorer.listing.path);
    const places = allowedPlaces();
    return path !== null && (!places || places.some((place) => isWithin(path, place.path))) ? path : null;
  });
  const editing = $derived(explorer.editing);

  // Clic derecho en el espacio libre de la carpeta, fuera de cualquier fila; con
  // el teclado, tecla Menú o Shift+F10 con el espacio libre enfocado. La barra y
  // la cabecera de columnas quedan fuera.
  function onFolderMenu(event: MouseEvent) {
    if ((event.target as Element).closest("li")) return;
    openMenu(event, event.currentTarget as HTMLElement, t("explorer.menuFolderLabel"), [
      { label: t("explorer.newFolder"), action: startCreating },
      { label: t("explorer.upload"), action: () => pickUploads(explorer.listing.path) },
    ]);
  }
</script>

<section>
  <header>
    <button class="icon" aria-label={t("explorer.back")} title={t("explorer.back")} disabled={!explorer.history.length} onclick={goBack}>
      ←
    </button>
    <button
      class="icon"
      aria-label={t("explorer.up")}
      title={t("explorer.up")}
      disabled={parent === null}
      onclick={() => parent !== null && openFolder(parent)}
    >
      ↑
    </button>
    <Breadcrumbs />
    <button class="secondary" onclick={() => pickUploads(explorer.listing.path)}>{t("explorer.upload")}</button>
  </header>
  <div class="columns" aria-hidden="true">
    <span></span>
    <span>{t("explorer.nameColumn")}</span>
    <span>{t("explorer.sizeColumn")}</span>
    <span>{t("explorer.modifiedColumn")}</span>
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div class="folder" role="region" tabindex="0" aria-label={t("explorer.folderLabel")} oncontextmenu={onFolderMenu}>
    {#if editing || explorer.listing.entries.length}
      <ul class="files">
        {#if editing?.kind === "create"}
          <NameForm {editing} />
        {/if}
        {#each explorer.listing.entries as entry (entry.name)}
          {#if editing?.kind === "rename" && editing.name === entry.name}
            <NameForm {editing} {entry} />
          {:else}
            <EntryRow {entry} />
          {/if}
        {/each}
      </ul>
    {:else}
      <p class="help empty">{t("files.empty")}</p>
    {/if}
  </div>
</section>

<style>
  section {
    --columns: 1.5rem minmax(0, 1fr) 7rem 10rem;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  header {
    display: grid;
    grid-template-columns: auto auto minmax(0, 1fr) auto;
    gap: 0.4rem;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
  }
  header :global(button.secondary) {
    padding: 0.35rem 0.8rem;
    font-size: 0.9rem;
  }
  .icon {
    width: 2rem;
    height: 2rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.4rem;
    background: none;
    color: inherit;
    font-size: 1rem;
    line-height: 1;
  }
  .icon:not(:disabled):hover {
    border-color: var(--accent);
    color: var(--accent-text);
  }
  .folder {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  .folder:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
