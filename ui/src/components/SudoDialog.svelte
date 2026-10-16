<!--
  La contraseña de sudo, la primera vez que el Administrador cambia una clave
  (state/sudo.svelte.ts). Se queda abierto con el motivo si no vale.
-->
<script lang="ts">
  import * as AlertDialog from "./ui/alert-dialog";
  import { Button } from "./ui/button";
  import { Input } from "./ui/input";
  import { t } from "../i18n";
  import { cancelSudoPassword, submitSudoPassword, sudoPrompt } from "../state/sudo.svelte";

  let password = $state("");

  $effect.pre(() => {
    if (!sudoPrompt.open) password = "";
  });

  function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    if (password) void submitSudoPassword(password);
  }
</script>

<AlertDialog.Root open={sudoPrompt.open} onOpenChange={(open) => !open && cancelSudoPassword()}>
  <AlertDialog.Content class="max-w-sm gap-5 rounded-xl border border-glass-edge p-5 shadow-[0_20px_50px_var(--shadow)] ring-0 sm:max-w-sm">
    <form class="flex flex-col gap-4" {onsubmit}>
      <AlertDialog.Header class="place-items-start text-left">
        <AlertDialog.Title class="text-base font-semibold">{t("sudo.title")}</AlertDialog.Title>
        <AlertDialog.Description>{t("sudo.help")}</AlertDialog.Description>
      </AlertDialog.Header>
      <label class="flex flex-col gap-1.5">
        <span class="font-medium">{t("sudo.password")}</span>
        <!-- svelte-ignore a11y_autofocus -->
        <Input type="password" autocomplete="off" autofocus bind:value={password} disabled={sudoPrompt.busy} aria-invalid={!!sudoPrompt.error} />
        {#if sudoPrompt.error}<small class="text-xs text-destructive" role="alert">{sudoPrompt.error}</small>{/if}
      </label>
      <div class="flex justify-end gap-2">
        <Button type="button" variant="outline" onclick={cancelSudoPassword}>{t("explorer.cancel")}</Button>
        <Button type="submit" disabled={!password || sudoPrompt.busy}>{sudoPrompt.busy ? t("sudo.checking") : t("sudo.submit")}</Button>
      </div>
    </form>
  </AlertDialog.Content>
</AlertDialog.Root>
