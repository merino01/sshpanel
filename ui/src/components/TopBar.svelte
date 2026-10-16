<!--
  Barra superior de la ventana principal, en todas las pantallas. A la
  izquierda, la cápsula con el usuario y el Servidor (el punto se rellena con la
  conexión abierta y late una vez al conectar); a la derecha, «Administración»
  para el Administrador y «Ajustes».
-->
<script lang="ts">
  import Settings from "~icons/heroicons/cog-6-tooth";
  import Shield from "~icons/heroicons/shield-check";
  import { t } from "../i18n";
  import { openAdmin } from "../state/admin.svelte";
  import { openSettings } from "../state/preferences.svelte";
  import { inWorkspace, session } from "../state/session.svelte";
  import TitleBar from "./TitleBar.svelte";

  const profile = $derived(session.appState?.profile);
  const connected = $derived(inWorkspace());
  const administering = $derived(session.screen.name === "admin");

  const action = "flex h-7 items-center gap-1.5 rounded-lg px-2 text-muted-foreground transition-colors hover:bg-foreground/8 hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring aria-[current]:bg-accent aria-[current]:text-accent-text";
</script>

<TitleBar>
  {#if profile}
    <p
      class="flex h-7 min-w-0 items-center gap-2 rounded-full border border-glass-edge bg-foreground/5 px-3 font-mono text-xs whitespace-nowrap"
      title={t(connected ? "topbar.connected" : "topbar.disconnected")}
    >
      <span class="relative grid size-2 shrink-0 place-items-center" aria-hidden="true">
        {#if connected}
          <span class="absolute inset-0 animate-[ping_0.9s_cubic-bezier(0,0,0.2,1)_1] rounded-full bg-primary motion-reduce:animate-none"></span>
        {/if}
        <span class="size-2 rounded-full border-[1.5px] border-primary {connected ? 'bg-primary' : ''}"></span>
      </span>
      <span class="font-semibold">{profile.username}</span><span class="-mx-1.5 text-muted-foreground">@</span><span class="truncate text-muted-foreground">{profile.host}</span>
    </p>
  {/if}
  <nav class="ml-auto flex items-center gap-0.5" aria-label={t("topbar.label")}>
    {#if profile?.admin && connected}
      <button class={action} aria-current={administering ? "location" : undefined} onclick={openAdmin}>
        <Shield class="size-4" />
        {t("topbar.admin")}
      </button>
    {/if}
    <button class={action} onclick={openSettings}>
      <Settings class="size-4" />
      {t("topbar.settings")}
    </button>
  </nav>
</TitleBar>
