<!--
  Diálogo modal de confirmación. Se monta una sola vez y muestra lo que se pida
  con `confirm()` (state/confirm.svelte.ts). El foco empieza en «Cancelar» y
  Escape cancela.
-->
<script lang="ts">
  import * as AlertDialog from "./ui/alert-dialog";
  import { t } from "../i18n";
  import { confirmation, type Confirmation } from "../state/confirm.svelte";

  // La última pregunta: se sigue pintando mientras el diálogo se cierra.
  let shown = $state.raw<Confirmation | null>(null);
  let cancel = $state<HTMLButtonElement | null>(null);

  $effect.pre(() => {
    if (confirmation.current) shown = confirmation.current;
  });

  function finish(confirmed: boolean) {
    const pending = confirmation.current;
    confirmation.current = null;
    pending?.resolve(confirmed);
  }
</script>

{#if shown}
  <AlertDialog.Root open={confirmation.current !== null} onOpenChange={(open) => !open && finish(false)}>
    <AlertDialog.Content
      class="max-w-sm gap-5 rounded-xl border border-glass-edge p-5 shadow-[0_20px_50px_var(--shadow)] ring-0 sm:max-w-sm"
      onOpenAutoFocus={(event) => {
        event.preventDefault();
        cancel?.focus();
      }}
    >
      <AlertDialog.Header class="place-items-start text-left">
        <AlertDialog.Title class="text-base font-semibold">{shown.title}</AlertDialog.Title>
        <AlertDialog.Description>{shown.body}</AlertDialog.Description>
      </AlertDialog.Header>
      <div class="flex justify-end gap-2">
        <AlertDialog.Cancel bind:ref={cancel} onclick={() => finish(false)}>{t("explorer.cancel")}</AlertDialog.Cancel>
        <AlertDialog.Action onclick={() => finish(true)}>{shown.confirmLabel}</AlertDialog.Action>
      </div>
    </AlertDialog.Content>
  </AlertDialog.Root>
{/if}
