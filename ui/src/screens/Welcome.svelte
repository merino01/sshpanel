<script lang="ts">
  import { branding } from "../branding.generated";
  import { t } from "../i18n";
  import { openCreateProfile } from "../state/createProfile.svelte";
  import { importProfile } from "../state/session.svelte";

  let { error }: { error?: string } = $props();
</script>

<h1>{t("welcome.title", { name: branding.name })}</h1>
<p>{t("welcome.subtitle")}</p>
<button class="primary" onclick={importProfile}>{t("welcome.import")}</button>
<button class="link admin" onclick={openCreateProfile}>{t("welcome.admin")}</button>
{#if error}
  <p class="error" role="alert">{error}</p>
{/if}

<style>
  .admin {
    margin-top: 1.5rem;
    font-size: 0.9rem;
    opacity: 0.8;
  }
</style>
