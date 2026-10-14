<!-- Accesos directos fijos: carpeta personal, Carpeta compartida si el Perfil la define y Papelera. -->
<script lang="ts">
  import { t } from "../i18n";
  import { explorer, openFolder } from "../state/explorer.svelte";
  import { session } from "../state/session.svelte";
  import { openTrash } from "../state/trash.svelte";

  const inTrash = $derived(session.screen.name === "trash");
  const shared = $derived(session.appState?.profile?.shared_folder);
  const folders = $derived([
    { label: t("explorer.home"), path: explorer.home },
    ...(shared ? [{ label: t("explorer.shared"), path: shared }] : []),
  ]);
</script>

<nav>
  {#each folders as folder (folder.path)}
    <button aria-current={!inTrash && explorer.listing.path === folder.path ? "location" : undefined} onclick={() => openFolder(folder.path)}>
      {folder.label}
    </button>
  {/each}
  <button aria-current={inTrash ? "location" : undefined} onclick={() => openTrash()}>{t("trash.title")}</button>
</nav>

<style>
  nav {
    display: flex;
    gap: 0.4rem;
  }
  button {
    padding: 0.35rem 0.9rem;
    border: 1px solid rgba(127, 127, 127, 0.4);
    border-radius: 999px;
    background: none;
    color: inherit;
  }
  button[aria-current] {
    border-color: var(--accent);
    background: var(--accent);
    color: white;
  }
</style>
