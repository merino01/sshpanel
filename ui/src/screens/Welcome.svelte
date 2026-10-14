<script lang="ts">
  import { branding } from "../branding.generated";
  import { t } from "../i18n";
  import { importProfile } from "../state/session.svelte";

  let { error }: { error?: string } = $props();
</script>

<h1>{t("welcome.title", { name: branding.name })}</h1>
<p>{t("welcome.subtitle")}</p>
<button class="primary" onclick={importProfile}>{t("welcome.import")}</button>
{#if error}
  <p class="error" role="alert">{error}</p>
{/if}
