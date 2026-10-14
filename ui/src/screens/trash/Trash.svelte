<!-- Papelera: lo borrado desde la app, con de dónde vino y cuándo se borró. -->
<script lang="ts">
  import { t } from "../../i18n";
  import { isAdvanced } from "../../state/session.svelte";
  import { emptyTrash, trashView } from "../../state/trash.svelte";
  import TrashRow from "./TrashRow.svelte";
</script>

<section>
  <header>
    <div>
      <h1>{t("trash.title")}</h1>
      <p class="help">{t(isAdvanced() ? "trash.helpAdvanced" : "trash.help")}</p>
    </div>
    {#if isAdvanced()}
      <button class="secondary" disabled={!trashView.items.length} onclick={emptyTrash}>{t("trash.emptyTrash")}</button>
    {/if}
  </header>
  <div class="columns" aria-hidden="true">
    <span></span>
    <span>{t("explorer.nameColumn")}</span>
    <span>{t("trash.originColumn")}</span>
    <span>{t("trash.deletedColumn")}</span>
    <span>{t("explorer.sizeColumn")}</span>
  </div>
  <div class="items">
    {#if trashView.items.length}
      <ul class="files">
        {#each trashView.items as item (item.id)}
          <TrashRow {item} />
        {/each}
      </ul>
    {:else}
      <p class="help empty">{t("trash.empty")}</p>
    {/if}
  </div>
</section>

<style>
  section {
    --columns: 1.5rem minmax(0, 1fr) minmax(0, 1fr) 10rem 7rem;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  header {
    display: flex;
    gap: 1rem;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(127, 127, 127, 0.25);
  }
  header :global(button.secondary) {
    flex-shrink: 0;
    padding: 0.35rem 0.8rem;
    font-size: 0.9rem;
  }
  h1 {
    margin: 0 0 0.25rem;
    font-size: 1.2rem;
  }
  .items {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
</style>
