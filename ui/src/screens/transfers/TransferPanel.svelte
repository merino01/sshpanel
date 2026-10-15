<!-- Panel de Transferencias, fijo en una esquina; sigue a la vista al navegar o al perder la conexión. -->
<script lang="ts">
  import { t } from "../../i18n";
  import { isFinished } from "../../lib/transferQueue.svelte";
  import { transfers } from "../../state/transfers.svelte";
  import TransferRow from "./TransferRow.svelte";
</script>

{#if transfers.items.length}
  <aside aria-label={t("transfers.title")}>
    <header>
      <h2>{t("transfers.title")}</h2>
      {#if transfers.items.some(isFinished)}
        <button class="link" onclick={() => transfers.clearFinished()}>{t("transfers.clear")}</button>
      {/if}
    </header>
    <ul>
      {#each transfers.items as transfer (transfer.id)}
        <TransferRow {transfer} />
      {/each}
    </ul>
  </aside>
{/if}

<style>
  aside {
    position: fixed;
    right: 1rem;
    bottom: 1rem;
    z-index: 5;
    width: min(24rem, calc(100vw - 2rem));
    max-height: 40vh;
    overflow-y: auto;
    padding: 0.6rem 0.8rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: var(--bg);
    box-shadow: 0 6px 20px var(--shadow);
    text-align: left;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  h2 {
    margin: 0;
  }
  ul {
    margin: 0.3rem 0 0;
    padding: 0;
    list-style: none;
  }
</style>
