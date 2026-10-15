<!--
  Avisos de lo que acaba de pasar, arriba a la derecha (el panel de
  Transferencias va abajo). Se monta una sola vez y pinta `state/toasts`. Con el
  ratón o el foco encima, un aviso no se cierra solo.
-->
<script lang="ts">
  import { flip } from "svelte/animate";
  import { prefersReducedMotion } from "svelte/motion";
  import { fade, fly } from "svelte/transition";
  import Close from "~icons/heroicons/x-mark-20-solid";
  import { t } from "../i18n";
  import { toasts } from "../state/toasts.svelte";

  const duration = $derived(prefersReducedMotion.current ? 0 : 180);
</script>

<!-- Bajo la barra de título y la de cada pantalla, para no tapar sus botones. -->
<section class="pointer-events-none fixed top-24 right-4 z-[60] flex w-[min(24rem,calc(100vw-2rem))] flex-col gap-2" aria-label={t("toast.region")}>
  {#each toasts.items as toast (toast.id)}
    <!-- Los colores del Tema, al revés: destaca sin añadir colores nuevos. -->
    <div
      class="pointer-events-auto flex items-center gap-2 rounded-xl py-2 pr-2 pl-3.5 text-left shadow-[0_10px_30px_var(--shadow)] {toast.kind === 'error'
        ? 'bg-destructive text-background'
        : 'bg-foreground/90 text-background backdrop-blur-xl'}"
      role={toast.kind === "error" ? "alert" : "status"}
      onmouseenter={() => toasts.pause(toast.id)}
      onmouseleave={() => toasts.resume(toast.id)}
      onfocusin={() => toasts.pause(toast.id)}
      onfocusout={() => toasts.resume(toast.id)}
      in:fly={{ x: 24, duration }}
      out:fade={{ duration }}
      animate:flip={{ duration }}
    >
      <p class="min-w-0 flex-1 text-sm [overflow-wrap:anywhere]">{toast.text}</p>
      {#if toast.action}
        <button
          class="shrink-0 rounded-md px-2 py-1 text-sm font-semibold text-[color-mix(in_oklab,var(--primary)_70%,var(--background))] hover:bg-current/15 focus-visible:outline-2 focus-visible:outline-current"
          onclick={() => toasts.act(toast.id)}
        >
          {toast.action.label}
        </button>
      {/if}
      <button
        class="grid size-7 shrink-0 place-items-center rounded-md opacity-70 hover:bg-current/15 hover:opacity-100 focus-visible:outline-2 focus-visible:outline-current"
        aria-label={t("toast.close")}
        title={t("toast.close")}
        onclick={() => toasts.dismiss(toast.id)}
      >
        <Close class="size-4" />
      </button>
    </div>
  {/each}
</section>
