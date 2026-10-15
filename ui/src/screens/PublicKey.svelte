<!-- El Servidor todavía no acepta la clave del Miembro: se la muestra para que se la envíe al Administrador. -->
<script lang="ts">
  import Check from "~icons/heroicons/check-16-solid";
  import Copy from "~icons/heroicons/clipboard-document";
  import { Button } from "../components/ui/button";
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

<h1 class="text-2xl font-semibold tracking-tight">{t("unauthorized.title")}</h1>
{#if profile}
  <p class="text-muted-foreground">{t("ready.subtitle", { username: profile.username, host: profile.host })}</p>
{/if}
<p class="mt-2 w-full rounded-lg bg-warning/15 px-3 py-2.5 text-left" role="alert">{notice}</p>
<h2 class="mt-4 text-base font-semibold">{t("ready.keyHeading")}</h2>
<p class="text-muted-foreground">{t("ready.keyHelp")}</p>
<code class="block w-full rounded-lg bg-muted px-3 py-2.5 text-left font-mono text-xs break-all select-all">{publicKey}</code>
<Button class="mt-2" onclick={copy}>
  {#if copied === t("ready.copied")}<Check />{:else}<Copy />{/if}
  {t("ready.copyKey")}
</Button>
<p class="min-h-5 text-xs text-muted-foreground" role="status">{copied}</p>
<Button variant="outline" onclick={connect}>{t("connect.retry")}</Button>
<Button variant="link" size="sm" onclick={importProfile}>{t("ready.importAnother")}</Button>
