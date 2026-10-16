<!--
  La ventana principal: la barra superior, la pantalla que toque según
  `session.screen` y, una sola vez fuera de ella, el panel de Transferencias, el aviso de soltar, el menú contextual, el
  diálogo de confirmación y los avisos, para que sigan ahí al cambiar de pantalla.
-->
<script lang="ts">
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import SudoDialog from "./components/SudoDialog.svelte";
  import ContextMenu from "./components/ContextMenu.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Toasts from "./components/Toasts.svelte";
  import TopBar from "./components/TopBar.svelte";
  import Connecting from "./screens/Connecting.svelte";
  import Admin from "./screens/admin/Admin.svelte";
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

<!--
  El marco (barra de título y barra lateral) es de cristal si la Transparencia
  está activada; el trabajo va en una hoja opaca encima. Las pantallas de trabajo
  ocupan la hoja entera; el resto, una hoja centrada.
-->
<div class="grid h-screen grid-rows-[auto_minmax(0,1fr)] bg-surface glass:bg-glass">
  <TopBar />
  {#if workspace}
    <main class="grid min-h-0 grid-cols-[13rem_minmax(0,1fr)]">
      <Sidebar />
      <div class="mr-2 mb-2 grid min-h-0 min-w-0 grid-rows-[minmax(0,1fr)] overflow-hidden rounded-xl border border-glass-edge bg-background shadow-[0_1px_3px_var(--shadow)]">
        {#if session.screen.name === "explorer"}
          <Explorer />
        {:else if session.screen.name === "trash"}
          <Trash />
        {:else if session.screen.name === "admin"}
          <Admin />
        {:else if session.screen.name === "createProfile"}
          <div class="min-h-0 overflow-y-auto px-8 py-10">
            <CreateProfile />
          </div>
        {/if}
        {#if isAdvanced()}
          <TerminalScreen visible={session.screen.name === "terminal"} />
        {/if}
      </div>
    </main>
  {:else}
    <main class="grid min-h-0 overflow-y-auto px-4 pb-4">
      <div class="m-auto flex w-full max-w-[34rem] flex-col items-center gap-2 rounded-xl border border-glass-edge bg-background px-8 py-10 text-center shadow-[0_8px_30px_var(--shadow)]">
        {#if session.screen.name === "welcome"}
          <Welcome error={session.screen.error} />
        {:else if session.screen.name === "key"}
          <PublicKey notice={session.screen.notice} />
        {:else if session.screen.name === "connecting"}
          <Connecting />
        {:else if session.screen.name === "problem"}
          <Problem message={session.screen.message} />
        {:else if session.screen.name === "createProfile"}
          <CreateProfile />
        {/if}
      </div>
    </main>
  {/if}
</div>
<TransferPanel />
<DropOverlay />
<ContextMenu />
<ConfirmDialog />
<SudoDialog />
<Toasts />
