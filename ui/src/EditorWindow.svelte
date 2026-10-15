<!--
  Una ventana del Editor, con su archivo. Ctrl+S (Cmd+S) guarda: es propio de
  esta ventana, no un Atajo configurable, y se atiende antes que nada (fase de
  captura) para que ni el área de edición ni los Atajos lo vean. Sus avisos y
  preguntas van aparte de los de la ventana principal.
-->
<script lang="ts">
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import Toasts from "./components/Toasts.svelte";
  import { comboOf } from "./lib/shortcuts";
  import Editor from "./screens/editor/Editor.svelte";
  import { confirmation } from "./state/confirm.svelte";
  import { save } from "./state/editor.svelte";

  function onkeydown(event: KeyboardEvent) {
    if (comboOf(event) !== "Mod+S") return;
    event.preventDefault();
    event.stopPropagation();
    if (!confirmation.current) void save();
  }
</script>

<svelte:window onkeydowncapture={onkeydown} />

<Editor />
<ConfirmDialog />
<Toasts />
