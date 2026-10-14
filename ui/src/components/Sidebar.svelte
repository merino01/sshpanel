<!-- Barra lateral de las pantallas de trabajo: el nombre de la app, los accesos directos y, para el Administrador, «Crear perfil». -->
<script lang="ts">
  import { branding } from "../branding.generated";
  import { t } from "../i18n";
  import { openCreateProfile } from "../state/createProfile.svelte";
  import { session } from "../state/session.svelte";
  import Shortcuts from "./Shortcuts.svelte";
</script>

<aside>
  <p class="brand">{branding.name}</p>
  <Shortcuts />
  {#if session.appState?.profile?.admin}
    <button class="admin" aria-current={session.screen.name === "createProfile" ? "location" : undefined} onclick={openCreateProfile}>{t("create.sidebar")}</button>
  {/if}
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem 0.75rem;
    border-right: 1px solid rgba(127, 127, 127, 0.25);
    background: color-mix(in srgb, var(--fg) 4%, var(--bg));
  }
  .admin {
    margin-top: auto;
    padding: 0.45rem 0.6rem;
    border: 1px solid rgba(127, 127, 127, 0.4);
    border-radius: 0.4rem;
    background: none;
    color: inherit;
    text-align: left;
  }
  .admin[aria-current] {
    border-color: var(--accent);
    background: var(--accent);
    color: white;
  }
  .admin:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
  .brand {
    margin: 0 0.5rem;
    color: var(--accent);
    font-weight: 700;
  }
</style>
