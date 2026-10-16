<!--
  Panel de Transferencias, fijo en una esquina; sigue a la vista al navegar o
  al perder la conexión. Debajo, los archivos abiertos en el equipo, solo
  mientras se abren, se suben o no se han podido subir.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import { isFinished } from "../../lib/transferQueue.svelte";
  import { localCopies } from "../../state/localCopies.svelte";
  import { transfers } from "../../state/transfers.svelte";
  import LocalCopyRow from "./LocalCopyRow.svelte";
  import TransferRow from "./TransferRow.svelte";
</script>

{#if transfers.visible.length || localCopies.active.length}
  <aside
    class="fixed right-4 bottom-4 z-40 max-h-[40vh] w-[min(24rem,calc(100vw-2rem))] overflow-y-auto rounded-xl border border-glass-edge bg-popover/90 px-3.5 py-2.5 text-left shadow-[0_10px_30px_var(--shadow)] backdrop-blur-xl"
    aria-label={t("transfers.title")}
  >
    <header class="flex items-baseline justify-between">
      <h2 class="text-sm font-semibold">{t("transfers.title")}</h2>
      {#if transfers.visible.some(isFinished)}
        <button class="text-xs text-accent-text hover:underline" onclick={() => transfers.clearFinished()}>{t("transfers.clear")}</button>
      {/if}
    </header>
    <ul class="mt-1 divide-y divide-border">
      {#each transfers.visible as transfer (transfer.id)}
        <TransferRow {transfer} />
      {/each}
      {#each localCopies.active as copy (copy.id)}
        <LocalCopyRow {copy} />
      {/each}
    </ul>
  </aside>
{/if}
