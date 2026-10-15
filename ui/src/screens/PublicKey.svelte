<!-- El Servidor todavía no acepta la clave del Miembro: se la muestra para que se la envíe al Administrador. -->
<script lang="ts">
  import { t } from "../i18n";
  import { connect, importProfile, session } from "../state/session.svelte";

  let { notice }: { notice: string } = $props();
  let copied = $state("");

  const profile = $derived(session.appState?.profile);
  const publicKey = $derived(session.appState?.public_key ?? "");

  async function copy() {
    try {
      await navigator.clipboard.writeText(publicKey);
      copied = t("ready.copied");
    } catch {
      copied = t("ready.copyFailed");
    }
  }
</script>

<h1>{t("unauthorized.title")}</h1>
{#if profile}
  <p>{t("ready.subtitle", { username: profile.username, host: profile.host })}</p>
{/if}
<p class="notice" role="alert">{notice}</p>
<h2>{t("ready.keyHeading")}</h2>
<p class="help">{t("ready.keyHelp")}</p>
<code class="key">{publicKey}</code>
<button class="primary" onclick={copy}>{t("ready.copyKey")}</button>
<p class="feedback" role="status">{copied}</p>
<button class="secondary" onclick={connect}>{t("connect.retry")}</button>
<button class="link" onclick={importProfile}>{t("ready.importAnother")}</button>

<style>
  .notice {
    padding: 0.6rem 0.8rem;
    border-radius: 0.4rem;
    background: color-mix(in srgb, var(--warning) 15%, transparent);
  }
  .key {
    display: block;
    max-width: 100%;
    padding: 0.6rem;
    border-radius: 0.4rem;
    background: var(--subtle);
    overflow-wrap: anywhere;
    user-select: all;
    font-size: 0.8rem;
  }
</style>
