<!--
  Un archivo o carpeta del Explorador. La fila recibe el foco y, si es una
  carpeta, se abre con un clic en cualquier parte o con Intro. Supr la mueve a
  la Papelera y el menú contextual tiene el resto.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import type { Entry } from "../../lib/api";
  import { formatSize } from "../../lib/format";
  import { joinPath } from "../../lib/paths";
  import { confirm } from "../../state/confirm.svelte";
  import { explorer, openFolder, startCreating, startRenaming, trashEntry } from "../../state/explorer.svelte";
  import { openMenu, type MenuItem } from "../../state/menu.svelte";
  import { pickDownload } from "../../state/transfers.svelte";

  let { entry }: { entry: Entry } = $props();
  let row = $state<HTMLLIElement>();

  const isDir = $derived(entry.kind === "dir");

  function open() {
    void openFolder(joinPath(explorer.listing.path, entry.name));
  }

  /** Pide confirmación y la mueve a la Papelera; si se cancela, el foco vuelve a la fila. */
  async function askToTrash() {
    const confirmed = await confirm({
      title: t("trash.confirmTitle", { name: entry.name }),
      body: t("trash.confirmHelp"),
      confirmLabel: t("trash.confirm"),
    });
    if (confirmed) void trashEntry(entry.name);
    else row?.focus();
  }

  function menuItems(): MenuItem[] {
    const items: MenuItem[] = [];
    if (isDir) items.push({ label: t("explorer.openShort"), action: open });
    // Un enlace roto no tiene nada que descargar.
    if (entry.kind !== "symlink") items.push({ label: t("explorer.download"), action: () => pickDownload(explorer.listing.path, entry.name) });
    items.push(
      { label: t("explorer.rename"), action: () => startRenaming(entry) },
      { label: t("explorer.newFolder"), action: startCreating },
      { label: t("explorer.delete"), action: askToTrash },
    );
    return items;
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.target !== row) return;
    if (event.key === "Enter" && isDir) open();
    if (event.key === "Delete") void askToTrash();
  }

  function oncontextmenu(event: MouseEvent) {
    if (row) openMenu(event, row, t("explorer.menuLabel", { name: entry.name }), menuItems());
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
<li
  bind:this={row}
  class="entry {entry.kind}"
  tabindex="0"
  title={isDir ? t("explorer.open", { name: entry.name }) : entry.name}
  onclick={isDir ? open : undefined}
  {onkeydown}
  {oncontextmenu}
>
  <span class="icon" aria-hidden="true">{isDir ? "📁" : "📄"}</span>
  <span class="name">{entry.name}</span>
  <span class="size">{entry.kind === "file" ? formatSize(entry.size) : ""}</span>
</li>

<style>
  li {
    cursor: pointer;
  }
  li:hover .name {
    color: var(--accent);
    text-decoration: underline;
  }
</style>
