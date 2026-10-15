<!--
  Barra superior, en todas las pantallas. A la izquierda, con qué usuario y
  Servidor se está (el punto se rellena con la conexión abierta); a la derecha,
  «Crear perfil» para el Administrador y «Ajustes».
-->
<script lang="ts">
  import { t } from "../i18n";
  import { openCreateProfile } from "../state/createProfile.svelte";
  import { openSettings } from "../state/preferences.svelte";
  import { inWorkspace, session } from "../state/session.svelte";

  const profile = $derived(session.appState?.profile);
  const connected = $derived(inWorkspace());
</script>

<header>
  {#if profile}
    <p class="identity" title={t(connected ? "topbar.connected" : "topbar.disconnected")}>
      <span class="dot" class:connected aria-hidden="true"></span>
      <span class="user">{profile.username}</span>
      <span class="at">@</span>
      <span class="host">{profile.host}</span>
    </p>
  {/if}
  <nav aria-label={t("topbar.label")}>
    {#if profile?.admin}
      <button aria-current={session.screen.name === "createProfile" ? "location" : undefined} onclick={openCreateProfile}>
        {t("topbar.createProfile")}
      </button>
    {/if}
    <button onclick={openSettings}>{t("topbar.settings")}</button>
  </nav>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-height: 2.5rem;
    padding: 0 0.75rem 0 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    font-size: 0.875rem;
  }
  .identity {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
    min-width: 0;
    margin: 0;
    white-space: nowrap;
  }
  .dot {
    flex: none;
    align-self: center;
    width: 0.5rem;
    height: 0.5rem;
    margin-right: 0.3rem;
    border: 1.5px solid var(--accent);
    border-radius: 50%;
    box-sizing: border-box;
  }
  .dot.connected {
    background: var(--accent);
  }
  .user {
    font-weight: 600;
  }
  .at {
    opacity: 0.5;
  }
  .host {
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0.8;
  }
  nav {
    display: flex;
    gap: 0.25rem;
    margin-left: auto;
  }
  button {
    padding: 0.3rem 0.65rem;
    border: 0;
    border-radius: 0.4rem;
    background: none;
    color: inherit;
  }
  button:hover {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
  button[aria-current] {
    background: var(--accent);
    color: var(--on-accent);
  }
</style>
