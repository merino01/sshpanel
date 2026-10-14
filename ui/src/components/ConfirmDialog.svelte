<!--
  Diálogo modal de confirmación. Se monta una sola vez y muestra lo que se pida
  con `confirm()` (state/confirm.svelte.ts). El foco empieza en «Cancelar» y
  Escape cancela.
-->
<script lang="ts">
  import { t } from "../i18n";
  import { confirmation } from "../state/confirm.svelte";

  let dialog = $state<HTMLDialogElement>();
  let cancelButton = $state<HTMLButtonElement>();
  let confirmed = false;

  $effect(() => {
    if (!confirmation.current || !dialog || dialog.open) return;
    confirmed = false;
    dialog.showModal();
    cancelButton?.focus();
  });

  // Se cierra con los botones o con Escape.
  function onclose() {
    const pending = confirmation.current;
    confirmation.current = null;
    pending?.resolve(confirmed);
  }

  function accept() {
    confirmed = true;
    dialog?.close();
  }
</script>

{#if confirmation.current}
  <dialog bind:this={dialog} {onclose}>
    <h2>{confirmation.current.title}</h2>
    <p class="help">{confirmation.current.body}</p>
    <div class="actions">
      <button bind:this={cancelButton} class="secondary" onclick={() => dialog?.close()}>{t("explorer.cancel")}</button>
      <button class="primary" onclick={accept}>{confirmation.current.confirmLabel}</button>
    </div>
  </dialog>
{/if}

<style>
  dialog {
    max-width: min(24rem, 90vw);
    padding: 1.2rem 1.4rem;
    border: 1px solid rgba(127, 127, 127, 0.3);
    border-radius: 0.6rem;
    background: var(--bg);
    color: var(--fg);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.4);
  }
  h2 {
    margin: 0 0 0.5rem;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
</style>
