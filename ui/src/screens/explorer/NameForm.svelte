<!-- Fila con un campo para escribir el nombre de una carpeta nueva o el nuevo nombre de `entry`. -->
<script lang="ts">
  import { onMount } from "svelte";
  import FileIcon from "../../components/FileIcon.svelte";
  import { Button } from "../../components/ui/button";
  import { Input } from "../../components/ui/input";
  import { t } from "../../i18n";
  import type { Entry } from "../../lib/api";
  import { stopEditing, submitName, type Editing } from "../../state/explorer.svelte";

  let { editing, entry }: { editing: Editing; entry?: Entry } = $props();
  let input = $state<HTMLInputElement | null>(null);
  let submitting = $state(false);

  const label = $derived(editing.kind === "create" ? t("explorer.newFolderLabel") : t("explorer.renameLabel", { name: editing.name }));

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

<li class="grid grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-3 rounded-md bg-accent px-2.5 py-1">
  <FileIcon entry={entry ?? { name: "", kind: "dir" }} />
  <form class="flex items-center gap-1.5" {onsubmit}>
    <Input
      bind:ref={input}
      class="h-7 min-w-0 flex-1"
      type="text"
      value={editing.value}
      aria-label={label}
      spellcheck={false}
      onkeydown={(event) => event.key === "Escape" && stopEditing()}
    />
    <Button type="submit" size="sm" disabled={submitting}>
      {editing.kind === "create" ? t("explorer.create") : t("explorer.save")}
    </Button>
    <Button type="button" variant="ghost" size="sm" onclick={stopEditing}>{t("explorer.cancel")}</Button>
  </form>
</li>
