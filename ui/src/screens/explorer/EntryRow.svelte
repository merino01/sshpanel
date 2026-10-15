<!--
  Un archivo o carpeta del Explorador. Un clic la selecciona (recibe el foco y
  se marca) y el doble clic la abre: una carpeta, en el Explorador; un archivo
  de texto, en el Editor. Con el foco, los Atajos (Intro, Supr, F2…) actúan
  sobre ella; el menú contextual tiene el resto.
-->
<script lang="ts">
  import { locale, t } from "../../i18n";
  import type { Entry } from "../../lib/api";
  import { formatDate, formatSize } from "../../lib/format";
  import { joinPath } from "../../lib/paths";
  import { openInEditor } from "../../state/editorWindows.svelte";
  import { askToTrash, explorer, openEntry, select, startRenaming } from "../../state/explorer.svelte";
  import { openMenu, type MenuItem } from "../../state/menu.svelte";
  import { pickDownload } from "../../state/transfers.svelte";

  let { entry }: { entry: Entry } = $props();
  let row = $state<HTMLLIElement>();

  const isDir = $derived(entry.kind === "dir");
  const isFile = $derived(entry.kind === "file");

  function open() {
    openEntry(entry);
  }

  /** Pide confirmación y la mueve a la Papelera; si se cancela, el foco vuelve a la fila. */
  async function trash() {
    if (!(await askToTrash(entry.name))) row?.focus();
  }

  function menuItems(): MenuItem[] {
    const items: MenuItem[] = [];
    if (isDir) items.push({ label: t("explorer.openShort"), action: open });
    if (isFile) items.push({ label: t("editor.openWith"), action: () => void openInEditor(joinPath(explorer.listing.path, entry.name)) });
    // Un enlace roto no tiene nada que descargar.
    if (entry.kind !== "symlink") items.push({ label: t("explorer.download"), action: () => pickDownload(explorer.listing.path, entry.name) });
    items.push(
      { label: t("explorer.rename"), action: () => startRenaming(entry) },
      { label: t("explorer.delete"), action: trash },
    );
    return items;
  }

  function oncontextmenu(event: MouseEvent) {
    if (row) openMenu(event, row, t("explorer.menuLabel", { name: entry.name }), menuItems());
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
<li
  bind:this={row}
  class="entry {entry.kind}"
  class:selected={explorer.selected === entry.name}
  tabindex="0"
  title={isDir || isFile ? t("explorer.open", { name: entry.name }) : entry.name}
  ondblclick={isDir || isFile ? open : undefined}
  onfocus={() => select(entry.name)}
  onblur={() => select(undefined)}
  {oncontextmenu}
>
  <span class="icon" aria-hidden="true">{isDir ? "📁" : "📄"}</span>
  <span class="name">{entry.name}</span>
  <span class="muted">{entry.kind === "file" ? formatSize(entry.size, locale.current) : ""}</span>
  <span class="muted">{entry.modified === null ? "" : formatDate(entry.modified, locale.current)}</span>
</li>

<style>
  li {
    cursor: default;
    user-select: none;
  }
  li:hover {
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }
  li.selected {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
  }
</style>
