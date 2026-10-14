<!--
  La app: la pantalla que toque según `session.screen` y, una sola vez fuera de
  ella, el panel de Transferencias, el aviso de soltar, el menú contextual y el
  diálogo de confirmación, para que sigan ahí al cambiar de pantalla.
-->
<script lang="ts">
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import ContextMenu from "./components/ContextMenu.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Connecting from "./screens/Connecting.svelte";
  import Explorer from "./screens/explorer/Explorer.svelte";
  import Problem from "./screens/Problem.svelte";
  import TerminalScreen from "./screens/terminal/Terminal.svelte";
  import PublicKey from "./screens/PublicKey.svelte";
  import DropOverlay from "./screens/transfers/DropOverlay.svelte";
  import TransferPanel from "./screens/transfers/TransferPanel.svelte";
  import Trash from "./screens/trash/Trash.svelte";
  import Welcome from "./screens/Welcome.svelte";
  import { isAdvanced, session } from "./state/session.svelte";

  // El Explorador, la Papelera y la Terminal ocupan toda la ventana, con la barra lateral; el resto va centrado.
  const workspace = $derived(["explorer", "trash", "terminal"].includes(session.screen.name));
</script>

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
  {/if}
  {#if workspace && isAdvanced()}
    <TerminalScreen visible={session.screen.name === "terminal"} />
  {/if}
</main>
<TransferPanel />
<DropOverlay />
<ContextMenu />
<ConfirmDialog />
