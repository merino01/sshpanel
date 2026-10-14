<script lang="ts">
  import { t } from "../../i18n";
  import { formatSize } from "../../lib/format";
  import type { QueuedTransfer } from "../../lib/transferQueue.svelte";
  import { transfers } from "../../state/transfers.svelte";

  let { transfer }: { transfer: QueuedTransfer } = $props();

  const isUpload = $derived(transfer.direction === "upload");

  /** Avance de 0 a 1; `undefined` deja la barra indeterminada mientras se prepara. */
  const value = $derived.by(() => {
    const { status, progress } = transfer;
    if (status === "done") return 1;
    if (progress) return progress.total ? Math.min(1, progress.done / progress.total) : 1;
    return status === "running" ? undefined : 0;
  });

  const detail = $derived.by(() => {
    const { status, progress } = transfer;
    if (status !== "running") return transfer.outcome ?? t("transfers.waiting");
    if (!progress) return t("transfers.preparing");
    return t("transfers.progress", { done: formatSize(Math.min(progress.done, progress.total)), total: formatSize(progress.total) });
  });

  const retryLabel = $derived(t("transfers.retryLabel", { name: transfer.name }));
  const cancelLabel = $derived(t("transfers.cancelLabel", { name: transfer.name }));
</script>

<li class={transfer.status}>
  <span class="icon" class:done={transfer.status === "done"} title={t(isUpload ? "transfers.upload" : "transfers.download")}>
    {isUpload ? "↑" : "↓"}
  </span>
  <span class="name" title={transfer.name}>{transfer.name}</span>
  <span class="actions">
    {#if transfer.status === "failed"}
      <button class="link" aria-label={retryLabel} title={retryLabel} onclick={() => transfers.retry(transfer)}>{t("transfers.retry")}</button>
    {/if}
    {#if transfer.status !== "done" && transfer.status !== "cancelled"}
      <button class="link" aria-label={cancelLabel} title={cancelLabel} disabled={transfer.cancelling} onclick={() => transfers.cancel(transfer)}>
        {t("transfers.cancel")}
      </button>
    {/if}
  </span>
  {#if value === undefined}
    <progress max="1"></progress>
  {:else}
    <progress max="1" {value}></progress>
  {/if}
  <span class="detail" class:error={transfer.status === "failed"} title={detail}>{detail}</span>
</li>

<style>
  li {
    display: grid;
    grid-template-columns: 1.2rem 1fr auto;
    gap: 0.15rem 0.4rem;
    padding: 0.4rem 0;
    border-bottom: 1px solid rgba(127, 127, 127, 0.2);
    font-size: 0.9rem;
  }
  li:last-child {
    border-bottom: 0;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  progress,
  .detail {
    grid-column: 2 / -1;
  }
  .actions {
    display: flex;
    gap: 0.6rem;
    font-size: 0.8rem;
  }
  progress {
    width: 100%;
    height: 0.5rem;
    accent-color: var(--accent);
  }
  .detail {
    font-size: 0.8rem;
    opacity: 0.8;
  }
  .detail.error {
    opacity: 1;
  }
  .icon.done {
    color: var(--accent);
  }
</style>
