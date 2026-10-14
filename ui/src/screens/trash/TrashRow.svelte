<!-- Un elemento de la Papelera; «Restaurar» y, en Modo avanzado, «Eliminar definitivamente» están en su menú contextual. -->
<script lang="ts">
  import { t } from "../../i18n";
  import type { Trashed } from "../../lib/api";
  import { formatDate, formatSize } from "../../lib/format";
  import { openMenu } from "../../state/menu.svelte";
  import { isAdvanced } from "../../state/session.svelte";
  import { deleteForever, restore } from "../../state/trash.svelte";

  let { item }: { item: Trashed } = $props();
  let row = $state<HTMLLIElement>();

  function oncontextmenu(event: MouseEvent) {
    if (!row) return;
    openMenu(event, row, t("explorer.menuLabel", { name: item.name }), [
      { label: t("trash.restore"), action: () => restore(item) },
      ...(isAdvanced() ? [{ label: t("trash.deleteForever"), action: () => deleteForever(item) }] : []),
    ]);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<li bind:this={row} class="entry {item.kind}" tabindex="0" {oncontextmenu}>
  <span class="icon" aria-hidden="true">{item.kind === "dir" ? "📁" : "📄"}</span>
  <span class="name" title={item.name}>{item.name}</span>
  <span class="muted" title={item.original_dir}>{item.original_dir}</span>
  <span class="muted">{item.deleted === null ? "" : formatDate(item.deleted)}</span>
  <span class="muted">{item.kind === "file" ? formatSize(item.size) : ""}</span>
</li>
