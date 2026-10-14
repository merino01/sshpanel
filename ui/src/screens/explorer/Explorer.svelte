<!-- Explorador: barra de herramientas y contenido de la carpeta en pantalla. -->
<script lang="ts">
  import Feedback from "../../components/Feedback.svelte";
  import Shortcuts from "../../components/Shortcuts.svelte";
  import { t } from "../../i18n";
  import { parentOf } from "../../lib/paths";
  import { explorer, openFolder, startCreating } from "../../state/explorer.svelte";
  import { openMenu } from "../../state/menu.svelte";
  import { pickUploads } from "../../state/transfers.svelte";
  import EntryRow from "./EntryRow.svelte";
  import NameForm from "./NameForm.svelte";

  const parent = $derived(parentOf(explorer.listing.path));
  const editing = $derived(explorer.editing);

  // Clic derecho en el hueco de la carpeta, fuera de cualquier fila; con el
  // teclado, tecla Menú o Shift+F10 con el hueco enfocado.
  function onFolderMenu(event: MouseEvent) {
    if ((event.target as Element).closest("li")) return;
    openMenu(event, event.currentTarget as HTMLElement, t("explorer.menuFolderLabel"), [
      { label: t("explorer.newFolder"), action: startCreating },
      { label: t("explorer.upload"), action: () => pickUploads(explorer.listing.path) },
    ]);
  }
</script>

<div class="explorer">
  <h1>{t("files.title")}</h1>
  <Shortcuts />
  <div class="toolbar">
    <button class="secondary" disabled={parent === null} onclick={() => parent !== null && openFolder(parent)}>{t("explorer.up")}</button>
    <p class="path">{explorer.listing.path}</p>
    <button class="secondary" onclick={() => pickUploads(explorer.listing.path)}>{t("explorer.upload")}</button>
  </div>
  <Feedback notice={explorer.notice} message={explorer.message} />
  <!-- El hueco ocupa el resto de la ventana, para que siempre haya dónde abrir su menú. Se
       enfoca con Tab para abrirlo también con el teclado (tecla Menú o Shift+F10). -->
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
</div>

<style>
  /* Arriba en vez de centrado, con el hueco de la carpeta hasta el final de la ventana. */
  .explorer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    width: min(30rem, 90vw);
    min-height: calc(100vh - 2rem);
  }
  .explorer h1 {
    margin-bottom: 0;
  }
  .folder {
    flex: 1;
    width: 100%;
    border-radius: 0.4rem;
  }
  .folder:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .toolbar {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 0.5rem;
    align-items: center;
    width: 100%;
  }
  .path {
    margin: 0;
    overflow: hidden;
    opacity: 0.7;
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .toolbar :global(button.secondary) {
    padding: 0.35rem 0.8rem;
    font-size: 0.9rem;
  }
</style>
