<!-- Minimizar, maximizar y cerrar, en Windows y Linux: allí la ventana no trae barra del sistema. -->
<script lang="ts">
  import Close from "~icons/heroicons/x-mark-20-solid";
  import Minimize from "~icons/heroicons/minus-20-solid";
  import Maximize from "~icons/heroicons/stop";
  import { t } from "../i18n";
  import { api } from "../lib/api";

  const base = "grid w-11 place-items-center text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring";
</script>

<div class="flex self-stretch">
  <button class="{base} hover:bg-foreground/10" aria-label={t("window.minimize")} title={t("window.minimize")} onclick={api.minimizeWindow}>
    <Minimize class="size-4" />
  </button>
  <button class="{base} hover:bg-foreground/10" aria-label={t("window.maximize")} title={t("window.maximize")} onclick={api.toggleMaximizeWindow}>
    <Maximize class="size-3.5" />
  </button>
  <button class="{base} hover:bg-[#e81123] hover:text-white" aria-label={t("window.close")} title={t("window.close")} onclick={api.closeWindow}>
    <Close class="size-4" />
  </button>
</div>
