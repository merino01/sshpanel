<script lang="ts">
  import { branding } from "../branding.generated";
  import { Button } from "../components/ui/button";
  import { t } from "../i18n";
  import { openCreateProfile } from "../state/createProfile.svelte";
  import { importProfile } from "../state/session.svelte";

  let { error }: { error?: string } = $props();
</script>

<h1 class="text-2xl font-semibold tracking-tight">{t("welcome.title", { name: branding.name })}</h1>
<p class="max-w-[46ch] text-muted-foreground">{t("welcome.subtitle")}</p>
<Button class="mt-4" size="lg" onclick={importProfile}>{t("welcome.import")}</Button>
<Button class="mt-6 text-muted-foreground" variant="link" size="sm" onclick={openCreateProfile}>{t("welcome.admin")}</Button>
{#if error}
  <p class="text-destructive" role="alert">{error}</p>
{/if}
