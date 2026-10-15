<!--
  «Autorizar clave» (Administrador): de la clave pública que envía un Miembro,
  la línea para su `authorized_keys` con las opciones de su modo. Es lo que hace
  que el Servidor aplique el modo, se edite lo que se edite en el Perfil.
-->
<script lang="ts">
  import Check from "~icons/heroicons/check-16-solid";
  import Copy from "~icons/heroicons/clipboard-document";
  import { Button } from "../components/ui/button";
  import { Input } from "../components/ui/input";
  import * as RadioGroup from "../components/ui/radio-group";
  import { Textarea } from "../components/ui/textarea";
  import { t } from "../i18n";
  import { authorizedKeyLine } from "../lib/authorizedKeys";
  import { createForm } from "../state/createProfile.svelte";

  const authorize = $derived(createForm.authorize);
  const line = $derived(authorizedKeyLine(authorize.key, authorize.mode, authorize.owner));
  let copied = $state("");

  async function copy() {
    if (!line) return;
    try {
      await navigator.clipboard.writeText(line);
      copied = t("ready.copied");
    } catch {
      copied = t("ready.copyFailed");
    }
  }

  $effect(() => {
    void line;
    copied = "";
  });
</script>

<div class="mt-8 flex w-full max-w-[28rem] flex-col gap-3 border-t pt-6 text-left">
  <h2 class="text-base font-semibold">{t("authorize.title")}</h2>
  <p class="text-muted-foreground">{t("authorize.help")}</p>
  <label class="flex flex-col gap-1.5">
    <span class="font-medium">{t("authorize.key")}</span>
    <Textarea class="min-h-20 font-mono text-xs md:text-xs" bind:value={authorize.key} rows={3} spellcheck={false} placeholder="ssh-ed25519 AAAA…" />
  </label>
  <label class="flex flex-col gap-1.5">
    <span class="font-medium">{t("authorize.owner")}</span>
    <Input type="text" bind:value={authorize.owner} placeholder={t("authorize.ownerPlaceholder")} spellcheck={false} />
    <small class="text-xs text-muted-foreground">{t("authorize.ownerHelp")}</small>
  </label>
  <fieldset>
    <legend class="mb-1.5 font-medium">{t("create.mode")}</legend>
    <RadioGroup.Root bind:value={authorize.mode} class="flex gap-5">
      <label class="flex cursor-pointer items-center gap-2"><RadioGroup.Item value="simple" /> {t("create.simple")}</label>
      <label class="flex cursor-pointer items-center gap-2"><RadioGroup.Item value="advanced" /> {t("create.advanced")}</label>
    </RadioGroup.Root>
  </fieldset>
  {#if line}
    <p class="text-muted-foreground">{t("authorize.lineHelp")}</p>
    <code class="block rounded-lg bg-muted px-3 py-2.5 font-mono text-xs break-all select-all">{line}</code>
    <div class="flex items-center justify-end gap-3">
      <span class="text-xs text-muted-foreground" role="status">{copied}</span>
      <Button onclick={copy}>
        {#if copied === t("ready.copied")}<Check />{:else}<Copy />{/if}
        {t("authorize.copy")}
      </Button>
    </div>
  {:else if authorize.key.trim()}
    <p class="text-destructive" role="alert">{t("authorize.invalid")}</p>
  {/if}
</div>
