<script lang="ts">
  import Download from "~icons/heroicons/arrow-down-tray-16-solid";
  import Upload from "~icons/heroicons/arrow-up-tray-16-solid";
  import { Progress } from "../../components/ui/progress";
  import { locale, t } from "../../i18n";
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
    return t("transfers.progress", { done: formatSize(Math.min(progress.done, progress.total), locale.current), total: formatSize(progress.total, locale.current) });
  });

  const retryLabel = $derived(t("transfers.retryLabel", { name: transfer.name }));
  const cancelLabel = $derived(t("transfers.cancelLabel", { name: transfer.name }));
</script>

<li class="grid grid-cols-[1.25rem_minmax(0,1fr)_auto] items-center gap-x-2 gap-y-1 py-2">
  <span class={transfer.status === "done" ? "text-accent-text" : "text-muted-foreground"} title={t(isUpload ? "transfers.upload" : "transfers.download")}>
    {#if isUpload}<Upload class="size-4" />{:else}<Download class="size-4" />{/if}
  </span>
  <span class="truncate" title={transfer.name}>{transfer.name}</span>
  <span class="flex gap-3 text-xs">
    {#if transfer.status === "failed"}
      <button class="text-accent-text hover:underline" aria-label={retryLabel} title={retryLabel} onclick={() => transfers.retry(transfer)}>{t("transfers.retry")}</button>
    {/if}
    {#if transfer.status !== "done" && transfer.status !== "cancelled"}
      <button
        class="text-accent-text hover:underline disabled:opacity-40"
        aria-label={cancelLabel}
        title={cancelLabel}
        disabled={transfer.cancelling}
        onclick={() => transfers.cancel(transfer)}
      >
        {t("transfers.cancel")}
      </button>
    {/if}
  </span>
  <Progress class="col-span-2 col-start-2 h-1.5" max={1} value={value ?? null} />
  <span class="col-span-2 col-start-2 truncate text-xs tabular-nums {transfer.status === 'failed' ? 'text-destructive' : 'text-muted-foreground'}" title={detail}>{detail}</span>
</li>
