<!-- Un elemento de la Papelera; «Restaurar» y, en Modo avanzado, «Eliminar definitivamente» están en su menú contextual. -->
<script lang="ts">
  import FileIcon from "../../components/FileIcon.svelte";
  import { locale, t } from "../../i18n";
  import type { Trashed } from "../../lib/api";
  import { formatDate, formatSize } from "../../lib/format";
  import { openMenu } from "../../state/menu.svelte";
  import { isAdvanced } from "../../state/session.svelte";
  import { deleteForever, restore } from "../../state/trash.svelte";
  import { MUTED, ROW, TRASH_COLUMNS } from "../files";

  let { item }: { item: Trashed } = $props();
  let row = $state<HTMLLIElement>();

  function oncontextmenu(event: MouseEvent) {
    if (!row) return;
    openMenu(event, row, t("explorer.menuLabel", { name: item.name }), [
      { label: t("trash.restore"), action: () => restore(item) },
      ...(isAdvanced() ? [{ label: t("trash.deleteForever"), action: () => deleteForever(item), destructive: true }] : []),
    ]);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<li bind:this={row} class="{ROW} {TRASH_COLUMNS}" tabindex="0" {oncontextmenu}>
  <FileIcon entry={item} />
  <span class="truncate" title={item.name}>{item.name}</span>
  <span class="{MUTED} font-mono" title={item.original_dir}>{item.original_dir}</span>
  <span class={MUTED}>{item.deleted === null ? "" : formatDate(item.deleted, locale.current)}</span>
  <span class="{MUTED} text-right">{item.kind === "file" ? formatSize(item.size, locale.current) : ""}</span>
</li>
