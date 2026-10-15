<!--
  Avisos de lo que acaba de pasar, arriba a la derecha (el panel de
  Transferencias va abajo). Se monta una sola vez y pinta `state/toasts`. Con el
  ratón o el foco encima, un aviso no se cierra solo.
-->
<script lang="ts">
  import { flip } from "svelte/animate";
  import { prefersReducedMotion } from "svelte/motion";
  import { fade, fly } from "svelte/transition";
  import { t } from "../i18n";
  import { toasts } from "../state/toasts.svelte";

  const duration = $derived(prefersReducedMotion.current ? 0 : 180);
</script>

<section aria-label={t("toast.region")}>
  {#each toasts.items as toast (toast.id)}
    <div
      class="toast {toast.kind}"
      role={toast.kind === "error" ? "alert" : "status"}
      onmouseenter={() => toasts.pause(toast.id)}
      onmouseleave={() => toasts.resume(toast.id)}
      onfocusin={() => toasts.pause(toast.id)}
      onfocusout={() => toasts.resume(toast.id)}
      in:fly={{ x: 24, duration }}
      out:fade={{ duration }}
      animate:flip={{ duration }}
    >
      <p>{toast.text}</p>
      {#if toast.action}
        <button class="action" onclick={() => toasts.act(toast.id)}>{toast.action.label}</button>
      {/if}
      <button class="close" aria-label={t("toast.close")} title={t("toast.close")} onclick={() => toasts.dismiss(toast.id)}>×</button>
    </div>
  {/each}
</section>

<style>
  section {
    position: fixed;
    /* Bajo la cabecera de las pantallas, para no tapar sus botones. */
    top: 4rem;
    right: 1rem;
    z-index: 20;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: min(24rem, calc(100vw - 2rem));
    pointer-events: none;
  }
  /* Los colores del tema, al revés: destaca sin añadir colores nuevos. */
  .toast {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    padding: 0.65rem 0.6rem 0.65rem 0.9rem;
    border-radius: 0.5rem;
    background: var(--fg);
    color: var(--bg);
    box-shadow: 0 6px 20px var(--shadow);
    pointer-events: auto;
    text-align: left;
  }
  .toast.error {
    background: var(--error);
    color: var(--bg);
  }
  p {
    flex: 1;
    min-width: 0;
    margin: 0;
    overflow-wrap: anywhere;
    font-size: 0.92rem;
    line-height: 1.35;
  }
  button {
    flex-shrink: 0;
    border: 0;
    border-radius: 0.35rem;
    background: none;
    color: inherit;
  }
  .action {
    padding: 0.3rem 0.55rem;
    color: color-mix(in srgb, var(--accent) 70%, var(--bg));
    font-weight: 700;
  }
  .close {
    padding: 0.15rem 0.45rem;
    font-size: 1.1rem;
    line-height: 1;
    opacity: 0.7;
  }
  button:hover {
    background: color-mix(in srgb, currentColor 14%, transparent);
    opacity: 1;
  }
  button:focus-visible {
    outline: 2px solid currentColor;
    outline-offset: 1px;
  }
</style>
