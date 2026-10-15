<!--
  El Editor de un archivo: arriba, qué archivo es, si está guardado en el
  Servidor y «Guardar»; en medio, el texto; abajo, el lenguaje, la posición del
  cursor y los finales de línea. Si se cae la conexión, una banda lo explica y
  el texto se queda.
-->
<script lang="ts">
  import { onDestroy, tick } from "svelte";
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

<div class="editor">
  <header>
    <p class="file">
      <span class="name">{fileName()}</span>
      <span class="dir">{parentOf(editor.path) ?? ""}</span>
    </p>
    {#if editor.phase.name === "ready"}
      <p class="state {status}" role="status">
        <span class="dot" aria-hidden="true"></span>
        {t(`editor.${status}`)}
      </p>
      <button class="save" class:pending={editor.unsaved} disabled={editor.saving} onclick={() => void save()}>
        {t("editor.save")}
        <kbd>{label("Mod+S")}</kbd>
      </button>
    {/if}
  </header>

  {#if editor.offline}
    <p class="offline" role="alert">{t("editor.offlineHelp")}</p>
  {/if}

  <div class="body">
    {#if editor.phase.name === "ready"}
      <div class="code" bind:this={host}></div>
    {:else if editor.phase.name === "loading"}
      <p class="message help">{editor.path ? t("editor.loading", { name: fileName() }) : ""}</p>
    {:else}
      <div class="message">
        {#if editor.path}<h2>{t("editor.loadFailed", { name: fileName() })}</h2>{/if}
        <p class="help">{editor.phase.message}</p>
        {#if editor.phase.retry}
          <button class="primary" onclick={() => void retry()}>{t("editor.retry")}</button>
        {/if}
      </div>
    {/if}
  </div>

  {#if editor.phase.name === "ready"}
    <footer aria-label={t("editor.statusLabel")}>
      <span>{language}</span>
      <span>{t("editor.position", { line: String(cursor.line), column: String(cursor.column) })}</span>
      <span>{newline}</span>
      {#if editor.stamp}<span>{formatSize(editor.stamp.size, locale.current)}</span>{/if}
    </footer>
  {/if}
</div>

<style>
  .editor {
    display: grid;
    grid-template-rows: auto auto 1fr auto;
    height: 100vh;
  }

  header {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-height: 2.75rem;
    padding: 0 0.6rem 0 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    font-size: 0.875rem;
  }
  .file {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    flex: 1;
    min-width: 0;
    margin: 0;
    white-space: nowrap;
  }
  .name {
    font-weight: 600;
  }
  .dir {
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0.6;
    font-family: ui-monospace, monospace;
    font-size: 0.8rem;
  }

  /* El estado del archivo respecto al Servidor: es lo que hay que mirar antes de cerrar. */
  .state {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin: 0;
    white-space: nowrap;
    opacity: 0.85;
  }
  .dot {
    width: 0.55rem;
    height: 0.55rem;
    border: 1.5px solid currentColor;
    border-radius: 50%;
    box-sizing: border-box;
  }
  .state.saved .dot {
    border-color: var(--success);
    background: var(--success);
  }
  .state.unsaved {
    opacity: 1;
    color: var(--accent-text);
  }
  .state.saving .dot {
    border-style: dashed;
  }
  .state.offline {
    opacity: 1;
    color: var(--warning);
  }

  .save {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.35rem 0.5rem 0.35rem 0.85rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.45rem;
    background: var(--bg);
    color: var(--fg);
    font-weight: 600;
  }
  .save.pending {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent);
  }
  .save:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  kbd {
    padding: 0.05rem 0.35rem;
    border-radius: 0.3rem;
    background: color-mix(in srgb, currentColor 14%, transparent);
    font: inherit;
    font-size: 0.75rem;
    font-weight: 400;
  }

  .offline {
    margin: 0;
    padding: 0.55rem 1rem;
    border-bottom: 1px solid color-mix(in srgb, var(--warning) 50%, transparent);
    background: color-mix(in srgb, var(--warning) 14%, var(--bg));
    font-size: 0.875rem;
  }

  .body {
    min-height: 0;
    background: var(--terminal-bg);
  }
  .code {
    height: 100%;
  }
  .message {
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 0.75rem;
    height: 100%;
    margin: 0;
    padding: 1rem;
    text-align: center;
  }
  .message h2 {
    margin: 0;
  }

  footer {
    display: flex;
    gap: 1.25rem;
    padding: 0.3rem 1rem;
    border-top: 1px solid var(--border);
    background: var(--surface);
    font-size: 0.78rem;
    opacity: 0.85;
  }
  footer span:first-child {
    flex: 1;
  }
</style>
