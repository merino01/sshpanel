<!-- Papelera: lo borrado desde la app, con de dónde vino y cuándo se borró. -->
<script lang="ts">
  import Feedback from "../../components/Feedback.svelte";
  import Shortcuts from "../../components/Shortcuts.svelte";
  import { t } from "../../i18n";
  import { trashView } from "../../state/trash.svelte";
  import TrashRow from "./TrashRow.svelte";
</script>

<h1>{t("trash.title")}</h1>
<Shortcuts />
<p class="help">{t("trash.help")}</p>
<Feedback notice={trashView.notice} message={trashView.message} />
{#if trashView.items.length}
  <ul class="files">
    {#each trashView.items as item (item.id)}
      <TrashRow {item} />
    {/each}
  </ul>
{:else}
  <p class="help empty">{t("trash.empty")}</p>
{/if}
