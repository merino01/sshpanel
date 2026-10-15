<!-- No se ha podido conectar, o se ha perdido la conexión. -->
<script lang="ts">
  import Warning from "~icons/heroicons/exclamation-triangle";
  import { Button } from "../components/ui/button";
  import { t } from "../i18n";
  import { connect, importProfile } from "../state/session.svelte";

  let { message }: { message: string } = $props();
</script>

<Warning class="size-9 text-destructive" aria-hidden="true" />
<h1 class="text-2xl font-semibold tracking-tight">{t("problem.title")}</h1>
<p class="max-w-[46ch] text-destructive" role="alert">{message}</p>
<Button class="mt-4" onclick={connect}>{t("connect.retry")}</Button>
<Button variant="link" size="sm" onclick={importProfile}>{t("ready.importAnother")}</Button>
