<!--
  Una pestaña de Terminal: xterm.js conectado a una consola del Servidor. Sigue
  montada, oculta, mientras se ve otra pestaña u otra pantalla, para no perder
  la sesión ni lo que se ve en ella.
-->
<script lang="ts">
  import { FitAddon } from "@xterm/addon-fit";
  import { Terminal } from "@xterm/xterm";
  import "@xterm/xterm/css/xterm.css";
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { shortcutFor } from "../../lib/terminalTabs";
  import { reopenTerminal, resize, sendInput, setOutput, startTerminal, type TerminalTab } from "../../state/terminal.svelte";

  let { tab, visible }: { tab: TerminalTab; visible: boolean } = $props();
  let container = $state<HTMLDivElement>();
  let xterm = $state<Terminal>();
  const fit = new FitAddon();

  onMount(() => {
    const id = tab.id;
    const accent = getComputedStyle(document.documentElement).getPropertyValue("--accent").trim();
    const term = new Terminal({
      cursorBlink: true,
      fontFamily: "ui-monospace, 'Cascadia Mono', 'DejaVu Sans Mono', monospace",
      fontSize: 14,
      theme: { background: "#1e1e1e", foreground: "#e5e5e5", cursor: accent || "#e5e5e5", selectionBackground: "#ffffff40" },
    });
    term.loadAddon(fit);
    term.open(container!);
    // Los atajos de las pestañas no llegan al Servidor; los atiende la pantalla.
    term.attachCustomKeyEventHandler((event) => shortcutFor(event) === null);
    setOutput(id, (data) => term.write(data));
    const input = term.onData((data) => sendInput(id, data));
    const resized = term.onResize(({ cols, rows }) => resize(id, cols, rows));
    // Oculta mide 0: solo se ajusta cuando se ve.
    const observer = new ResizeObserver(() => container?.offsetParent && fit.fit());
    observer.observe(container!);
    xterm = term;
    return () => {
      observer.disconnect();
      input.dispose();
      resized.dispose();
      setOutput(id, null);
      term.dispose();
    };
  });

  // Al verse: se ajusta al hueco, se abre la sesión si no hay ninguna y recibe el foco.
  $effect(() => {
    if (!visible || !xterm) return;
    fit.fit();
    if (tab.status === "idle") {
      xterm.reset();
      void startTerminal(tab.id, xterm.cols, xterm.rows);
    }
    xterm.focus();
  });
</script>

<div class="view" role="tabpanel" id="terminal-{tab.id}" hidden={!visible}>
  {#if tab.status === "opening"}
    <p class="help notice">{t("terminal.opening")}</p>
  {:else if tab.status === "closed"}
    <div class="notice closed" role="status">
      <p>{tab.message ?? t("terminal.closed")}</p>
      <button class="primary" onclick={() => reopenTerminal(tab.id)}>{t("terminal.reopen")}</button>
    </div>
  {/if}
  <div class="screen" role="region" aria-label={t("terminal.label", { number: String(tab.number) })} bind:this={container}></div>
</div>

<style>
  .view {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
  }
  .view[hidden] {
    display: none;
  }
  .notice {
    margin: 0;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid rgba(127, 127, 127, 0.25);
  }
  .closed {
    display: flex;
    gap: 1rem;
    align-items: center;
  }
  .closed p {
    margin: 0;
  }
  .screen {
    flex: 1;
    min-height: 0;
    padding: 0.5rem;
    background: #1e1e1e;
  }
</style>
