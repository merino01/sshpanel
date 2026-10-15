<!-- Aviso de adónde se subirá lo que se está arrastrando sobre la ventana. -->
<script lang="ts">
  import Upload from "~icons/heroicons/cloud-arrow-up";
  import { t } from "../../i18n";
  import { drop } from "../../state/transfers.svelte";
</script>

{#if drop.target !== null}
  <div
    class="pointer-events-none fixed inset-2 z-[60] grid place-content-center justify-items-center gap-3 rounded-2xl border-2 border-dashed border-primary bg-background/85 text-xl font-semibold text-accent-text backdrop-blur-sm"
  >
    <Upload class="size-10" />
    {t("transfers.dropHere", { path: drop.target })}
  </div>
{/if}
