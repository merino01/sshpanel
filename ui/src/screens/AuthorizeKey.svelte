<!--
  «Autorizar clave» (Administrador): escribe en el Servidor, con sudo, la clave
  pública que envía un Miembro para su usuario, con las opciones de su modo. Es
  lo que hace que el Servidor aplique el modo, se edite lo que se edite en el
  Perfil. Está en la pestaña Claves y tras crear un Perfil, ya con su usuario.
-->
<script lang="ts">
  import { Button } from "../components/ui/button";
  import { Input } from "../components/ui/input";
  import * as RadioGroup from "../components/ui/radio-group";
  import * as Select from "../components/ui/select";
  import { Textarea } from "../components/ui/textarea";
  import { t } from "../i18n";
  import { authorizedKeyLine } from "../lib/authorizedKeys";
  import { admin, authorizeKey, type AuthorizeForm } from "../state/admin.svelte";

  let { form, oncancel }: { form: AuthorizeForm; oncancel?: () => void } = $props();

  const line = $derived(authorizedKeyLine(form.key, form.mode, form.owner));
  /** Los usuarios del Servidor, los que ya tienen claves y el elegido. */
  const users = $derived(
    [...new Set([...admin.users, ...(admin.keys ?? []).map((keys) => keys.user), ...(form.user ? [form.user] : [])])].sort(),
  );

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    if (!line || !form.user) return;
    if (await authorizeKey(form.user, line)) {
      form.key = "";
      form.owner = "";
    }
  }
</script>

<form class="flex w-full flex-col gap-3 text-left" {onsubmit}>
  <h2 class="text-base font-semibold">{t("authorize.title")}</h2>
  <p class="text-muted-foreground">{t("authorize.help")}</p>
  <label class="flex flex-col gap-1.5">
    <span class="font-medium">{t("authorize.key")}</span>
    <Textarea class="min-h-20 font-mono text-xs md:text-xs" bind:value={form.key} rows={3} spellcheck={false} placeholder="ssh-ed25519 AAAA…" />
    {#if form.key.trim() && !line}
      <small class="text-xs text-destructive" role="alert">{t("authorize.invalid")}</small>
    {/if}
  </label>
  <div class="flex flex-col gap-1.5">
    <span class="font-medium" id="authorize-user">{t("authorize.user")}</span>
    <Select.Root type="single" bind:value={form.user}>
      <Select.Trigger class="w-full font-mono" aria-labelledby="authorize-user">
        {form.user || t("authorize.userPlaceholder")}
      </Select.Trigger>
      <Select.Content>
        {#each users as user (user)}
          <Select.Item value={user} label={user} class="font-mono" />
        {/each}
      </Select.Content>
    </Select.Root>
  </div>
  <label class="flex flex-col gap-1.5">
    <span class="font-medium">{t("authorize.owner")}</span>
    <Input type="text" bind:value={form.owner} placeholder={t("authorize.ownerPlaceholder")} spellcheck={false} />
    <small class="text-xs text-muted-foreground">{t("authorize.ownerHelp")}</small>
  </label>
  <fieldset>
    <legend class="mb-1.5 font-medium">{t("create.mode")}</legend>
    <RadioGroup.Root bind:value={form.mode} class="flex gap-5">
      <label class="flex cursor-pointer items-center gap-2"><RadioGroup.Item value="simple" /> {t("create.simple")}</label>
      <label class="flex cursor-pointer items-center gap-2"><RadioGroup.Item value="advanced" /> {t("create.advanced")}</label>
    </RadioGroup.Root>
  </fieldset>
  <div class="flex justify-end gap-2">
    {#if oncancel}
      <Button type="button" variant="outline" onclick={oncancel} disabled={admin.busy}>{t("explorer.cancel")}</Button>
    {/if}
    <Button type="submit" disabled={!line || !form.user || admin.busy}>{admin.busy ? t("authorize.authorizing") : t("authorize.submit")}</Button>
  </div>
</form>
