<!--
  «Crear perfil» (Administrador): servidor, usuario, modo y Carpeta compartida
  de un Miembro. La Huella no se escribe: la app la calcula conectándose al
  Servidor. Al terminar dice dónde ha quedado el fichero para enviárselo. Debajo,
  «Autorizar clave» da la línea de `authorized_keys` de cada Miembro. Sin Perfil
  importado, el primero es el del propio Administrador: se importa al momento y
  no hay claves de otros que autorizar.
-->
<script lang="ts">
  import { Button } from "../components/ui/button";
  import { Checkbox } from "../components/ui/checkbox";
  import { Input } from "../components/ui/input";
  import * as RadioGroup from "../components/ui/radio-group";
  import { t } from "../i18n";
  import type { ProfileField } from "../lib/profileForm";
  import AuthorizeKey from "./AuthorizeKey.svelte";
  import { createAnother, createForm, importCreated, leaveCreateProfile, submitProfile } from "../state/createProfile.svelte";

  const form = $derived(createForm.form);
  const created = $derived(createForm.created);
  const isInvalid = (field: ProfileField) => createForm.invalid.includes(field);

  function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    void submitProfile();
  }
</script>

<section class="flex w-full flex-col items-center gap-2 text-center">
  <h1 class="text-2xl font-semibold tracking-tight">{t("create.title")}</h1>
  {#if created}
    <p role="status">{t("create.done", { username: created.profile.username, path: created.path })}</p>
    {#if createForm.own}
      <p>{t("create.nextStepOwn")}</p>
    {:else}
      <p>{t("create.nextStep", { username: created.profile.username })}</p>
    {/if}
    <p class="mt-2 text-muted-foreground">{t("create.fingerprintHelp")}</p>
    <code class="block w-full rounded-lg bg-muted px-3 py-2.5 font-mono text-xs break-all select-all">{created.profile.fingerprint}</code>
    <div class="mt-3 flex w-full justify-end gap-2">
      <Button variant="outline" onclick={leaveCreateProfile}>{t("create.back")}</Button>
      {#if createForm.own}
        <Button onclick={importCreated}>{t("create.importOwn")}</Button>
      {:else}
        <Button onclick={createAnother}>{t("create.another")}</Button>
      {/if}
    </div>
  {:else}
    <p class="max-w-[46ch] text-muted-foreground">{t(createForm.own ? "create.helpOwn" : "create.help")}</p>
    <form class="mt-3 flex w-full max-w-[28rem] flex-col gap-4 text-left" {onsubmit} novalidate>
      <div class="grid grid-cols-[minmax(0,1fr)_6rem] gap-3">
        <label class="flex flex-col gap-1.5">
          <span class="font-medium">{t("create.host")}</span>
          <Input type="text" bind:value={form.host} placeholder="vps.example.com" spellcheck={false} aria-invalid={isInvalid("host")} />
          {#if isInvalid("host")}<small class="text-xs text-destructive">{t("create.invalid.host")}</small>{/if}
        </label>
        <label class="flex flex-col gap-1.5">
          <span class="font-medium">{t("create.port")}</span>
          <Input type="text" inputmode="numeric" class="tabular-nums" bind:value={form.port} aria-invalid={isInvalid("port")} />
          {#if isInvalid("port")}<small class="text-xs text-destructive">{t("create.invalid.port")}</small>{/if}
        </label>
      </div>
      <label class="flex flex-col gap-1.5">
        <span class="font-medium">{t("create.username")}</span>
        <Input type="text" bind:value={form.username} placeholder="alice" spellcheck={false} autocapitalize="off" aria-invalid={isInvalid("username")} />
        <small class="text-xs text-muted-foreground">{t("create.usernameHelp")}</small>
        {#if isInvalid("username")}<small class="text-xs text-destructive">{t("create.invalid.username")}</small>{/if}
      </label>
      <fieldset class="flex flex-col gap-1.5">
        <legend class="mb-1.5 font-medium">{t("create.mode")}</legend>
        <RadioGroup.Root bind:value={form.mode} class="grid grid-cols-2 gap-2">
          {#each [["simple", "create.simple", "create.simpleHelp"], ["advanced", "create.advanced", "create.advancedHelp"]] as [value, name, help] (value)}
            <label
              class="flex cursor-pointer gap-2.5 rounded-lg border border-input p-3 transition-colors hover:border-primary has-data-checked:border-primary has-data-checked:bg-accent"
            >
              <RadioGroup.Item {value} class="mt-0.5" />
              <span class="flex flex-col gap-0.5">
                <strong class="font-medium">{t(name)}</strong>
                <small class="text-xs text-muted-foreground">{t(help)}</small>
              </span>
            </label>
          {/each}
        </RadioGroup.Root>
      </fieldset>
      <label class="flex flex-col gap-1.5">
        <span class="font-medium">{t("create.sharedFolder")}</span>
        <Input type="text" class="font-mono" bind:value={form.sharedFolder} placeholder="/srv/compartida" spellcheck={false} aria-invalid={isInvalid("sharedFolder")} />
        <small class="text-xs text-muted-foreground">{t("create.sharedFolderHelp")}</small>
        {#if isInvalid("sharedFolder")}<small class="text-xs text-destructive">{t("create.invalid.sharedFolder")}</small>{/if}
      </label>
      <label class="flex cursor-pointer gap-2.5">
        <Checkbox bind:checked={form.admin} class="mt-0.5" />
        <span class="flex flex-col gap-0.5">
          <strong class="font-medium">{t("create.admin")}</strong>
          <small class="text-xs text-muted-foreground">{t("create.adminHelp")}</small>
        </span>
      </label>
      {#if createForm.error}
        <p class="text-destructive" role="alert">{createForm.error}</p>
      {/if}
      <div class="mt-1 flex justify-end gap-2">
        <Button type="button" variant="outline" onclick={leaveCreateProfile} disabled={createForm.creating}>{t("explorer.cancel")}</Button>
        <Button type="submit" disabled={createForm.creating}>
          {createForm.creating ? t("create.creating") : t("create.submit")}
        </Button>
      </div>
    </form>
  {/if}
  {#if !createForm.own}
    <AuthorizeKey />
  {/if}
</section>
