<!--
  El Editor de un archivo: arriba, qué archivo es, si está guardado en el
  Servidor y «Guardar»; en medio, el texto; abajo, el lenguaje, la posición del
  cursor y los finales de línea. Si se cae la conexión, una banda lo explica y
  el texto se queda.
-->
<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import TitleBar from "../../components/TitleBar.svelte";
  import { Button } from "../../components/ui/button";
  import { locale, t } from "../../i18n";
  import { formatSize } from "../../lib/format";
  import { parentOf } from "../../lib/paths";
  import { lineSeparator } from "../../lib/textFile";
  import { attachText, changed, editor, fileName, load, save, startEditor } from "../../state/editor.svelte";
  import { activeTheme } from "../../state/preferences.svelte";
  import { label } from "../../state/shortcuts.svelte";
  import { createEditor, type EditorHandle } from "./codemirror";

  let host = $state<HTMLDivElement>();
  let handle: EditorHandle | undefined;
  let language = $state(t("editor.plainText"));
  let cursor = $state({ line: 1, column: 1 });
  let newline = $state<"LF" | "CRLF">("LF");

  /** El estado del archivo respecto al Servidor: es lo que hay que mirar antes de cerrar. */
  const status = $derived(editor.offline ? "offline" : editor.saving ? "saving" : editor.unsaved ? "unsaved" : "saved");

  void startEditor().then(show);

  /** Pinta el texto leído en el área de edición. */
  async function show(file: Awaited<ReturnType<typeof load>>) {
    if (!file) return;
    // El área se monta cuando la fase pasa a «ready».
    await tick();
    if (!host) return;
    newline = lineSeparator(file.text) === "\r\n" ? "CRLF" : "LF";
    handle?.destroy();
    handle = createEditor(host, {
      text: file.text,
      name: fileName(),
      palette: activeTheme.palette,
      onChange: changed,
      onCursor: (line, column) => (cursor = { line, column }),
      onLanguage: (name) => (language = name),
    });
    attachText(handle.text);
    handle.focus();
  }

  async function retry() {
    await show(await load());
  }

  // Con el Tema cambia también el resaltado.
  $effect(() => {
    const palette = activeTheme.palette;
    handle?.setPalette(palette);
  });

  onDestroy(() => handle?.destroy());
</script>

<div class="grid h-screen grid-rows-[auto_minmax(0,1fr)] bg-surface glass:bg-glass">
  <TitleBar>
    <p class="flex min-w-0 flex-1 items-baseline gap-2.5 whitespace-nowrap">
      <span class="font-semibold">{fileName()}</span>
      <span class="truncate font-mono text-xs text-muted-foreground">{parentOf(editor.path) ?? ""}</span>
    </p>
    {#if editor.phase.name === "ready"}
      <p
        class="flex items-center gap-2 whitespace-nowrap {status === 'unsaved' ? 'text-accent-text' : status === 'offline' ? 'text-warning' : 'text-muted-foreground'}"
        role="status"
      >
        <span
          class="size-2 rounded-full border-[1.5px] border-current {status === 'saved' ? 'border-success bg-success' : ''} {status === 'saving' ? 'border-dashed' : ''}"
          aria-hidden="true"
        ></span>
        {t(`editor.${status}`)}
      </p>
      <Button size="sm" variant={editor.unsaved ? "default" : "outline"} disabled={editor.saving} onclick={() => void save()}>
        {t("editor.save")}
        <kbd class="rounded bg-current/15 px-1.5 font-sans text-xs font-normal">{label("Mod+S")}</kbd>
      </Button>
    {/if}
  </TitleBar>

  <div class="mr-2 mb-2 grid min-h-0 grid-rows-[auto_minmax(0,1fr)_auto] overflow-hidden rounded-xl border border-glass-edge bg-terminal shadow-[0_1px_3px_var(--shadow)]">
    <div>
      {#if editor.offline}
        <p class="border-b border-warning/50 bg-[color-mix(in_oklab,var(--warning)_14%,var(--background))] px-4 py-2" role="alert">{t("editor.offlineHelp")}</p>
      {/if}
    </div>

    <div class="min-h-0">
      {#if editor.phase.name === "ready"}
        <div class="h-full" bind:this={host}></div>
      {:else if editor.phase.name === "loading"}
        <p class="grid h-full place-content-center p-4 text-muted-foreground">{editor.path ? t("editor.loading", { name: fileName() }) : ""}</p>
      {:else}
        <div class="grid h-full place-content-center justify-items-center gap-3 p-4 text-center">
          {#if editor.path}<h2 class="text-base font-semibold">{t("editor.loadFailed", { name: fileName() })}</h2>{/if}
          <p class="text-muted-foreground">{editor.phase.message}</p>
          {#if editor.phase.retry}
            <Button onclick={() => void retry()}>{t("editor.retry")}</Button>
          {/if}
        </div>
      {/if}
    </div>

    {#if editor.phase.name === "ready"}
      <footer class="flex gap-5 border-t bg-background px-4 py-1 text-xs text-muted-foreground tabular-nums" aria-label={t("editor.statusLabel")}>
        <span class="flex-1">{language}</span>
        <span>{t("editor.position", { line: String(cursor.line), column: String(cursor.column) })}</span>
        <span>{newline}</span>
        {#if editor.stamp}<span>{formatSize(editor.stamp.size, locale.current)}</span>{/if}
      </footer>
    {:else}
      <div></div>
    {/if}
  </div>
</div>
