<!-- Aviso de adónde se subirá lo que se está arrastrando sobre la ventana. -->
<script lang="ts">
  import { t } from "../../i18n";
  import { drop } from "../../state/transfers.svelte";
</script>

{#if drop.target !== null}
  <div>{t("transfers.dropHere", { path: drop.target })}</div>
{/if}

<style>
  div {
    position: fixed;
    inset: 0.5rem;
    z-index: 20;
    display: grid;
    place-content: center;
    border: 3px dashed var(--accent);
    border-radius: 0.8rem;
    background: color-mix(in srgb, var(--bg) 85%, transparent);
    color: var(--accent);
    font-size: 1.3rem;
    font-weight: 600;
    pointer-events: none;
  }
</style>
