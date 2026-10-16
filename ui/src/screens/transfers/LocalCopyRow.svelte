<!--
  Un archivo abierto en el equipo mientras pasa algo que conviene ver: se está
  abriendo (con su avance), se está subiendo un guardado o no se ha podido subir.
-->
<script lang="ts">
  import Download from "~icons/heroicons/arrow-down-tray-16-solid";
  import Upload from "~icons/heroicons/arrow-up-tray-16-solid";
  import { Progress } from "../../components/ui/progress";
  import { locale, t } from "../../i18n";
  import { formatSize } from "../../lib/format";
  import type { LocalCopy } from "../../lib/localCopies.svelte";

  let { copy }: { copy: LocalCopy } = $props();

  const progress = $derived(copy.transfer?.progress ?? null);

  /** Avance de 0 a 1 al abrir; `null` deja la barra indeterminada mientras se prepara. */
  const value = $derived(progress ? (progress.total ? Math.min(1, progress.done / progress.total) : 1) : null);

  const detail = $derived.by(() => {
    if (copy.status === "uploading") return t("localCopies.uploading");
    if (copy.status === "failed") return t("localCopies.failed");
    if (!progress) return t("localCopies.preparing");
    return t("localCopies.opening", {
      done: formatSize(Math.min(progress.done, progress.total), locale.current),
      total: formatSize(progress.total, locale.current),
    });
  });
</script>

<li class="grid grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-x-2 gap-y-1 py-2">
  <span class="text-muted-foreground" aria-hidden="true">
    {#if copy.status === "opening"}<Download class="size-4" />{:else}<Upload class="size-4" />{/if}
  </span>
  <span class="truncate" title={copy.name}>{copy.name}</span>
  {#if copy.status === "opening"}
    <Progress class="col-start-2 h-1.5" max={1} {value} />
  {/if}
  <span class="col-start-2 truncate text-xs tabular-nums {copy.status === 'failed' ? 'text-destructive' : 'text-muted-foreground'}" title={detail}>{detail}</span>
</li>
