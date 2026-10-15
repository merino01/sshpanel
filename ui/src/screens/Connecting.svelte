<script lang="ts">
  import Spinner from "~icons/heroicons/arrow-path";
  import { t } from "../i18n";
</script>

<Spinner class="size-8 animate-spin text-primary motion-reduce:animate-none" aria-hidden="true" />
<h1 class="text-2xl font-semibold tracking-tight">{t("connect.title")}</h1>
<p class="text-muted-foreground" role="status">{t("connect.wait")}</p>
