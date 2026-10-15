<!--
  La ventana principal: la barra superior, la pantalla que toque según
  `session.screen` y, una sola vez fuera de ella, el panel de Transferencias, el aviso de soltar, el menú contextual, el
  diálogo de confirmación y los avisos, para que sigan ahí al cambiar de pantalla.
-->
<script lang="ts">
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import ContextMenu from "./components/ContextMenu.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Toasts from "./components/Toasts.svelte";
  import TopBar from "./components/TopBar.svelte";
  import Connecting from "./screens/Connecting.svelte";
  import CreateProfile from "./screens/CreateProfile.svelte";
  import Explorer from "./screens/explorer/Explorer.svelte";
  import Problem from "./screens/Problem.svelte";
  import TerminalScreen from "./screens/terminal/Terminal.svelte";
  import PublicKey from "./screens/PublicKey.svelte";
  import DropOverlay from "./screens/transfers/DropOverlay.svelte";
  import TransferPanel from "./screens/transfers/TransferPanel.svelte";
  import Trash from "./screens/trash/Trash.svelte";
  import Welcome from "./screens/Welcome.svelte";
  import { inWorkspace, isAdvanced, session } from "./state/session.svelte";

  // Las pantallas de trabajo ocupan toda la ventana, con la barra lateral; el resto va centrado.
  const workspace = $derived(inWorkspace());
</script>

<div class="window" class:workspace>
  <TopBar />
  <main class:workspace>
    {#if workspace}
      <Sidebar />
    {/if}
    {#if session.screen.name === "welcome"}
      <Welcome error={session.screen.error} />
    {:else if session.screen.name === "key"}
      <PublicKey notice={session.screen.notice} />
    {:else if session.screen.name === "connecting"}
      <Connecting />
    {:else if session.screen.name === "problem"}
      <Problem message={session.screen.message} />
    {:else if session.screen.name === "explorer"}
      <Explorer />
    {:else if session.screen.name === "trash"}
      <Trash />
    {:else if session.screen.name === "createProfile"}
      <CreateProfile />
    {/if}
    {#if workspace && isAdvanced()}
      <TerminalScreen visible={session.screen.name === "terminal"} />
    {/if}
  </main>
</div>
<TransferPanel />
<DropOverlay />
<ContextMenu />
<ConfirmDialog />
<Toasts />
