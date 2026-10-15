<!-- Papelera: lo borrado desde la app, con de dónde vino y cuándo se borró. -->
<script lang="ts">
  import { Button } from "../../components/ui/button";
  import { t } from "../../i18n";
  import { isAdvanced } from "../../state/session.svelte";
  import { emptyTrash, trashView } from "../../state/trash.svelte";
  import { EMPTY, HEADER, TRASH_COLUMNS } from "../files";
  import TrashRow from "./TrashRow.svelte";
</script>

<section class="flex min-h-0 min-w-0 flex-col">
  <header class="flex shrink-0 items-center justify-between gap-4 border-b px-4 py-3">
    <div>
      <h1 class="text-base font-semibold">{t("trash.title")}</h1>
      <p class="text-xs text-muted-foreground">{t(isAdvanced() ? "trash.helpAdvanced" : "trash.help")}</p>
    </div>
    {#if isAdvanced()}
      <Button variant="destructive" size="sm" disabled={!trashView.items.length} onclick={emptyTrash}>{t("trash.emptyTrash")}</Button>
    {/if}
  </header>
  <div class="{HEADER} {TRASH_COLUMNS}" aria-hidden="true">
    <span></span>
    <span>{t("explorer.nameColumn")}</span>
    <span>{t("trash.originColumn")}</span>
    <span>{t("trash.deletedColumn")}</span>
    <span>{t("explorer.sizeColumn")}</span>
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto p-1.5">
    {#if trashView.items.length}
      <ul>
        {#each trashView.items as item (item.id)}
          <TrashRow {item} />
        {/each}
      </ul>
    {:else}
      <p class={EMPTY}>{t("trash.empty")}</p>
    {/if}
  </div>
</section>
