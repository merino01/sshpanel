<!--
  «Autorizar clave» (Administrador): de la clave pública que envía un Miembro,
  la línea para su `authorized_keys` con las opciones de su modo. Es lo que hace
  que el Servidor aplique el modo, se edite lo que se edite en el Perfil.
-->
<script lang="ts">
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

<div class="authorize">
  <h2>{t("authorize.title")}</h2>
  <p class="help">{t("authorize.help")}</p>
  <label>
    <span>{t("authorize.key")}</span>
    <textarea bind:value={authorize.key} rows="3" spellcheck={false} placeholder="ssh-ed25519 AAAA…"></textarea>
  </label>
  <label>
    <span>{t("authorize.owner")}</span>
    <input type="text" bind:value={authorize.owner} placeholder={t("authorize.ownerPlaceholder")} spellcheck={false} />
    <small class="help">{t("authorize.ownerHelp")}</small>
  </label>
  <fieldset>
    <legend>{t("create.mode")}</legend>
    <label class="choice"><input type="radio" bind:group={authorize.mode} value="simple" /> {t("create.simple")}</label>
    <label class="choice"><input type="radio" bind:group={authorize.mode} value="advanced" /> {t("create.advanced")}</label>
  </fieldset>
  {#if line}
    <p class="help">{t("authorize.lineHelp")}</p>
    <code class="line">{line}</code>
    <div class="actions">
      <span class="feedback" role="status">{copied}</span>
      <button class="primary" onclick={copy}>{t("authorize.copy")}</button>
    </div>
  {:else if authorize.key.trim()}
    <p class="error" role="alert">{t("authorize.invalid")}</p>
  {/if}
</div>

<style>
  .authorize {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    width: min(28rem, 100%);
    margin-top: 1.5rem;
    padding-top: 1.2rem;
    border-top: 1px solid var(--border);
    text-align: left;
  }
  h2 {
    margin: 0;
    color: var(--fg);
    font-size: 1.05rem;
  }
  label:not(.choice) {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  label > span {
    font-weight: 600;
  }
  textarea,
  input[type="text"] {
    padding: 0.45rem 0.6rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.4rem;
    background: var(--bg);
    color: var(--fg);
    font: inherit;
    font-size: 0.85rem;
    resize: vertical;
  }
  textarea:focus,
  input[type="text"]:focus {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }
  fieldset {
    display: flex;
    gap: 1rem;
    margin: 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: 0.3rem;
    padding: 0;
    font-weight: 600;
  }
  .line {
    padding: 0.5rem 0.7rem;
    border-radius: 0.4rem;
    background: var(--subtle);
    overflow-wrap: anywhere;
    user-select: all;
    font-size: 0.8rem;
  }
  .actions {
    display: flex;
    gap: 0.8rem;
    align-items: center;
    justify-content: flex-end;
  }
  .error {
    margin: 0;
  }
</style>
