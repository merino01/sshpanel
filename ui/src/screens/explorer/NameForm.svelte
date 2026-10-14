<!-- Fila con un campo para escribir el nombre de una carpeta nueva o el nuevo nombre de `entry`. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import type { Entry } from "../../lib/api";
  import { stopEditing, submitName, type Editing } from "../../state/explorer.svelte";

  let { editing, entry }: { editing: Editing; entry?: Entry } = $props();
  let input = $state<HTMLInputElement>();
  let submitting = $state(false);

  const label = $derived(editing.kind === "create" ? t("explorer.newFolderLabel") : t("explorer.renameLabel", { name: editing.name }));
  const isFile = $derived(entry !== undefined && entry.kind !== "dir");

  // Al renombrar, selecciona el nombre sin la extensión.
  onMount(() => {
    input?.focus();
    if (editing.kind !== "rename") return;
    const dot = editing.value.lastIndexOf(".");
    input?.setSelectionRange(0, dot > 0 && entry?.kind !== "dir" ? dot : editing.value.length);
  });

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    submitting = true;
    await submitName(input?.value ?? "");
    // Si ha fallado, el formulario sigue ahí con el error encima.
    submitting = false;
    input?.focus();
  }
</script>

<li class="entry editing">
  <span class="icon" aria-hidden="true">{isFile ? "📄" : "📁"}</span>
  <form {onsubmit}>
    <input
      bind:this={input}
      type="text"
      value={editing.value}
      aria-label={label}
      spellcheck={false}
      onkeydown={(event) => event.key === "Escape" && stopEditing()}
    />
    <button type="submit" class="primary small" disabled={submitting}>
      {editing.kind === "create" ? t("explorer.create") : t("explorer.save")}
    </button>
    <button type="button" class="link" onclick={stopEditing}>{t("explorer.cancel")}</button>
  </form>
</li>

<style>
  .entry.editing {
    grid-template-columns: 1.5rem 1fr;
  }
  form {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  input {
    flex: 1;
    min-width: 0;
    padding: 0.35rem 0.5rem;
    border: 1px solid rgba(127, 127, 127, 0.5);
    border-radius: 0.4rem;
    background: var(--bg);
    color: var(--fg);
    font: inherit;
  }
  .primary.small {
    padding: 0.35rem 0.8rem;
  }
</style>
