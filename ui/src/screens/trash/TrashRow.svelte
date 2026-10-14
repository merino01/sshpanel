<!-- Un elemento de la Papelera; «Restaurar» está en su menú contextual. -->
<script lang="ts">
  import { t } from "../../i18n";
  import type { Trashed } from "../../lib/api";
  import { formatDate, formatSize } from "../../lib/format";
  import { openMenu } from "../../state/menu.svelte";
  import { restore } from "../../state/trash.svelte";

  let { item }: { item: Trashed } = $props();
  let row = $state<HTMLLIElement>();

  const origin = $derived(
    item.deleted === null
      ? t("trash.from", { dir: item.original_dir })
      : t("trash.fromOn", { dir: item.original_dir, date: formatDate(item.deleted) }),
  );

  function oncontextmenu(event: MouseEvent) {
    if (row) openMenu(event, row, t("explorer.menuLabel", { name: item.name }), [{ label: t("trash.restore"), action: () => restore(item) }]);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<li bind:this={row} class="entry {item.kind}" tabindex="0" {oncontextmenu}>
  <span class="icon" aria-hidden="true">{item.kind === "dir" ? "📁" : "📄"}</span>
  <span class="name" title={item.name}>{item.name}</span>
  <span class="size">{item.kind === "file" ? formatSize(item.size) : ""}</span>
  <span class="origin" title={origin}>{origin}</span>
</li>

<style>
  .origin {
    grid-column: 2 / -1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.6;
    font-size: 0.8rem;
  }
</style>
