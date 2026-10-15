<!--
  Barra de título de todas las ventanas, dentro de la interfaz: se arrastra
  desde cualquier punto que no sea un control y el doble clic la maximiza. En
  macOS deja sitio a la izquierda para los semáforos del sistema; en Windows y
  Linux pone a la derecha los botones de la ventana. Lo de dentro lo pone cada
  ventana.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { isMac } from "../lib/shortcuts";
  import WindowControls from "./WindowControls.svelte";

  let { children }: { children?: Snippet } = $props();
</script>

<header data-tauri-drag-region="deep" class="flex h-11 shrink-0 items-center gap-3 select-none {isMac ? 'pr-3 pl-20' : 'pl-3'}">
  <div class="flex min-w-0 flex-1 items-center gap-3">
    {@render children?.()}
  </div>
  {#if !isMac}
    <WindowControls />
  {/if}
</header>
