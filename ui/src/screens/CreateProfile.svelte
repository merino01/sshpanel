<!--
  «Crear perfil» (Administrador): servidor, usuario, modo y Carpeta compartida
  de un Miembro. La Huella no se escribe: la app la calcula conectándose al
  Servidor. Al terminar dice dónde ha quedado el fichero para enviárselo. Debajo,
  «Autorizar clave» da la línea de `authorized_keys` de cada Miembro.
-->
<script lang="ts">
  import { t } from "../i18n";
  import type { ProfileField } from "../lib/profileForm";
  import AuthorizeKey from "./AuthorizeKey.svelte";
  import { createAnother, createForm, leaveCreateProfile, submitProfile } from "../state/createProfile.svelte";

  const form = $derived(createForm.form);
  const created = $derived(createForm.created);
  const isInvalid = (field: ProfileField) => createForm.invalid.includes(field);

  function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    void submitProfile();
  }
</script>

<section>
  <h1>{t("create.title")}</h1>
  {#if created}
    <p role="status">{t("create.done", { username: created.profile.username, path: created.path })}</p>
    <p class="help">{t("create.fingerprintHelp")}</p>
    <code class="fingerprint">{created.profile.fingerprint}</code>
    <p class="help">{t("create.nextStep", { username: created.profile.username })}</p>
    <div class="actions">
      <button class="secondary" onclick={leaveCreateProfile}>{t("create.back")}</button>
      <button class="primary" onclick={createAnother}>{t("create.another")}</button>
    </div>
  {:else}
    <p class="help">{t("create.help")}</p>
    <form {onsubmit} novalidate>
      <label class:invalid={isInvalid("host")}>
        <span>{t("create.host")}</span>
        <input type="text" bind:value={form.host} placeholder="vps.ejemplo.com" spellcheck={false} aria-invalid={isInvalid("host")} />
        {#if isInvalid("host")}<small class="error">{t("create.invalid.host")}</small>{/if}
      </label>
      <label class:invalid={isInvalid("port")}>
        <span>{t("create.port")}</span>
        <input type="text" inputmode="numeric" bind:value={form.port} aria-invalid={isInvalid("port")} />
        {#if isInvalid("port")}<small class="error">{t("create.invalid.port")}</small>{/if}
      </label>
      <label class:invalid={isInvalid("username")}>
        <span>{t("create.username")}</span>
        <input type="text" bind:value={form.username} placeholder="alice" spellcheck={false} autocapitalize="off" aria-invalid={isInvalid("username")} />
        <small class="help">{t("create.usernameHelp")}</small>
        {#if isInvalid("username")}<small class="error">{t("create.invalid.username")}</small>{/if}
      </label>
      <fieldset>
        <legend>{t("create.mode")}</legend>
        <label class="choice">
          <input type="radio" bind:group={form.mode} value="simple" />
          <span><strong>{t("create.simple")}</strong> <small class="help">{t("create.simpleHelp")}</small></span>
        </label>
        <label class="choice">
          <input type="radio" bind:group={form.mode} value="advanced" />
          <span><strong>{t("create.advanced")}</strong> <small class="help">{t("create.advancedHelp")}</small></span>
        </label>
      </fieldset>
      <label class:invalid={isInvalid("sharedFolder")}>
        <span>{t("create.sharedFolder")}</span>
        <input type="text" bind:value={form.sharedFolder} placeholder="/srv/compartida" spellcheck={false} aria-invalid={isInvalid("sharedFolder")} />
        <small class="help">{t("create.sharedFolderHelp")}</small>
        {#if isInvalid("sharedFolder")}<small class="error">{t("create.invalid.sharedFolder")}</small>{/if}
      </label>
      <label class="choice">
        <input type="checkbox" bind:checked={form.admin} />
        <span><strong>{t("create.admin")}</strong> <small class="help">{t("create.adminHelp")}</small></span>
      </label>
      {#if createForm.error}
        <p class="error" role="alert">{createForm.error}</p>
      {/if}
      <div class="actions">
        <button type="button" class="secondary" onclick={leaveCreateProfile} disabled={createForm.creating}>{t("explorer.cancel")}</button>
        <button type="submit" class="primary" disabled={createForm.creating}>
          {createForm.creating ? t("create.creating") : t("create.submit")}
        </button>
      </div>
    </form>
  {/if}
  <AuthorizeKey />
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    align-items: center;
    min-height: 0;
    padding: 1rem;
    overflow-y: auto;
    text-align: center;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    width: min(28rem, 100%);
    margin-top: 0.5rem;
    text-align: left;
  }
  label:not(.choice) {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  label > span {
    font-weight: 600;
  }
  input[type="text"] {
    padding: 0.45rem 0.6rem;
    border: 1px solid rgba(127, 127, 127, 0.5);
    border-radius: 0.4rem;
    background: var(--bg);
    color: var(--fg);
    font: inherit;
  }
  input[type="text"]:focus {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }
  .invalid input[type="text"] {
    border-color: #dc2626;
  }
  fieldset {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin: 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: 0.3rem;
    padding: 0;
    font-weight: 600;
  }
  .choice {
    display: flex;
    gap: 0.5rem;
    align-items: baseline;
  }
  .choice span > strong {
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    margin-top: 0.4rem;
  }
  .fingerprint {
    padding: 0.5rem 0.7rem;
    border-radius: 0.4rem;
    background: rgba(127, 127, 127, 0.15);
    overflow-wrap: anywhere;
    user-select: all;
    font-size: 0.85rem;
  }
</style>
