<!-- No se ha podido conectar, o se ha perdido la conexión. -->
<script lang="ts">
  import { t } from "../i18n";
  import { connect, importProfile } from "../state/session.svelte";

  let { message }: { message: string } = $props();
</script>

<h1>{t("problem.title")}</h1>
<p class="error" role="alert">{message}</p>
<button class="primary" onclick={connect}>{t("connect.retry")}</button>
<button class="link" onclick={importProfile}>{t("ready.importAnother")}</button>
