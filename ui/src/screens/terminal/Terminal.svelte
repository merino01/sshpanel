<!--
  Terminal (solo en Modo avanzado): xterm.js conectado a una consola del
  Servidor. Sigue montada, oculta, mientras se usan el Explorador o la Papelera,
  para no perder la sesión ni lo que se ve en ella.
-->
<script lang="ts">
  import { FitAddon } from "@xterm/addon-fit";
  import { Terminal } from "@xterm/xterm";
  import "@xterm/xterm/css/xterm.css";
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { reopenTerminal, resize, sendInput, setOutput, startTerminal, terminal } from "../../state/terminal.svelte";

  let { visible }: { visible: boolean } = $props();
  let container = $state<HTMLDivElement>();
  let xterm = $state<Terminal>();
  const fit = new FitAddon();

  onMount(() => {
    const accent = getComputedStyle(document.documentElement).getPropertyValue("--accent").trim();
    const term = new Terminal({
      cursorBlink: true,
      fontFamily: "ui-monospace, 'Cascadia Mono', 'DejaVu Sans Mono', monospace",
      fontSize: 14,
      theme: { background: "#1e1e1e", foreground: "#e5e5e5", cursor: accent || "#e5e5e5", selectionBackground: "#ffffff40" },
    });
    term.loadAddon(fit);
    term.open(container!);
    setOutput((data) => term.write(data));
    const input = term.onData(sendInput);
    const resized = term.onResize(({ cols, rows }) => resize(cols, rows));
    // Oculta mide 0: solo se ajusta cuando se ve.
    const observer = new ResizeObserver(() => container?.offsetParent && fit.fit());
    observer.observe(container!);
    xterm = term;
    return () => {
      observer.disconnect();
      input.dispose();
      resized.dispose();
      setOutput(null);
      term.dispose();
    };
  });

  // Al verse: se ajusta al hueco, se abre la sesión si no hay ninguna y recibe el foco.
  $effect(() => {
    if (!visible || !xterm) return;
    fit.fit();
    if (terminal.status === "idle") {
      xterm.reset();
      void startTerminal(xterm.cols, xterm.rows);
    }
    xterm.focus();
  });
</script>

<section hidden={!visible}>
  <header>
    <h1>{t("terminal.title")}</h1>
    {#if terminal.status === "opening"}
      <p class="help">{t("terminal.opening")}</p>
    {/if}
  </header>
  {#if terminal.status === "closed"}
    <div class="closed" role="status">
      <p>{terminal.message ?? t("terminal.closed")}</p>
      <button class="primary" onclick={reopenTerminal}>{t("terminal.reopen")}</button>
    </div>
  {/if}
  <div class="screen" role="region" aria-label={t("terminal.label")} bind:this={container}></div>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  section[hidden] {
    display: none;
  }
  header {
    display: flex;
    gap: 1rem;
    align-items: baseline;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(127, 127, 127, 0.25);
  }
  h1 {
    margin: 0;
    font-size: 1.2rem;
  }
  header .help {
    margin: 0;
  }
  .closed {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid rgba(127, 127, 127, 0.25);
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
