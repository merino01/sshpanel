<!--
  Explorador: barra con «Carpeta anterior», «Carpeta superior», la ruta y
  «Subir archivos», y debajo el contenido de la carpeta a todo el ancho.
-->
<script lang="ts">
  import Back from "~icons/heroicons/arrow-left-20-solid";
  import Up from "~icons/heroicons/arrow-up-20-solid";
  import Upload from "~icons/heroicons/arrow-up-tray-16-solid";
  import { Button } from "../../components/ui/button";
  import { t } from "../../i18n";
  import { isWithin, parentOf } from "../../lib/paths";
  import { allowedPlaces, explorer, goBack, openFolder, startCreating } from "../../state/explorer.svelte";
  import { openMenu } from "../../state/menu.svelte";
  import { pickUploads } from "../../state/transfers.svelte";
  import Breadcrumbs from "./Breadcrumbs.svelte";
  import EntryRow from "./EntryRow.svelte";
  import NameForm from "./NameForm.svelte";
  import { EMPTY, EXPLORER_COLUMNS, HEADER } from "../files";

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

<section class="flex min-h-0 min-w-0 flex-col">
  <header class="grid h-12 shrink-0 grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-1 border-b px-2.5">
    <Button variant="ghost" size="icon-sm" aria-label={t("explorer.back")} title={t("explorer.back")} disabled={!explorer.history.length} onclick={goBack}>
      <Back />
    </Button>
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label={t("explorer.up")}
      title={t("explorer.up")}
      disabled={parent === null}
      onclick={() => parent !== null && openFolder(parent)}
    >
      <Up />
    </Button>
    <Breadcrumbs />
    <Button variant="outline" size="sm" onclick={() => pickUploads(explorer.listing.path)}>
      <Upload />
      {t("explorer.upload")}
    </Button>
  </header>
  <div class="{HEADER} {EXPLORER_COLUMNS}" aria-hidden="true">
    <span></span>
    <span>{t("explorer.nameColumn")}</span>
    <span>{t("explorer.sizeColumn")}</span>
    <span>{t("explorer.modifiedColumn")}</span>
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="min-h-0 flex-1 overflow-y-auto p-1.5 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring"
    role="region"
    tabindex="0"
    aria-label={t("explorer.folderLabel")}
    oncontextmenu={onFolderMenu}
  >
    {#if editing || explorer.listing.entries.length}
      <ul>
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
      <p class={EMPTY}>{t("files.empty")}</p>
    {/if}
  </div>
</section>
