<!-- Accesos directos fijos: carpeta personal, Carpeta compartida si el Perfil la define, Papelera y, en Modo avanzado, Terminal. -->
<script lang="ts">
  import { t } from "../i18n";
  import { explorer, openFolder } from "../state/explorer.svelte";
  import { isAdvanced, session } from "../state/session.svelte";
  import { openTerminal } from "../state/terminal.svelte";
  import { openTrash } from "../state/trash.svelte";

  const inTrash = $derived(session.screen.name === "trash");
  const inTerminal = $derived(session.screen.name === "terminal");
  const shared = $derived(session.appState?.profile?.shared_folder);
  const folders = $derived([
    { label: t("explorer.home"), path: explorer.home },
    ...(shared ? [{ label: t("explorer.shared"), path: shared }] : []),
  ]);
</script>

<nav>
  {#each folders as folder (folder.path)}
    <button aria-current={session.screen.name === "explorer" && explorer.listing.path === folder.path ? "location" : undefined} onclick={() => openFolder(folder.path)}>
      {folder.label}
    </button>
  {/each}
  <button aria-current={inTrash ? "location" : undefined} onclick={() => openTrash()}>{t("trash.title")}</button>
  {#if isAdvanced()}
    <button aria-current={inTerminal ? "location" : undefined} onclick={openTerminal}>{t("terminal.title")}</button>
  {/if}
</nav>

<style>
  nav {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  button {
    padding: 0.45rem 0.6rem;
    border: 0;
    border-radius: 0.4rem;
    background: none;
    color: inherit;
    text-align: left;
  }
  button:hover {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }
  button[aria-current] {
    background: var(--accent);
    color: var(--on-accent);
  }
</style>
