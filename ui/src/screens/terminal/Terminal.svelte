<!--
  Terminales (solo en Modo avanzado): pestañas con «+» para abrir otra y, debajo,
  la Terminal de la pestaña activa. Atajos: Ctrl+Mayús+T abre una nueva desde
  cualquier pantalla; Ctrl+Mayús+W cierra la actual y Ctrl+RePág / Ctrl+AvPág
  cambian de pestaña.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import { shortcutFor } from "../../lib/terminalTabs";
  import { closeTerminal, cycleTerminal, newTerminal, selectTerminal, terminals } from "../../state/terminal.svelte";
  import TerminalView from "./TerminalView.svelte";

  let { visible }: { visible: boolean } = $props();

  function onkeydown(event: KeyboardEvent) {
    const shortcut = shortcutFor(event);
    if (!shortcut || (shortcut !== "new" && !visible)) return;
    event.preventDefault();
    if (shortcut === "new") newTerminal();
    else if (shortcut === "close" && terminals.active !== null) closeTerminal(terminals.active);
    else if (shortcut === "next") cycleTerminal(1);
    else if (shortcut === "previous") cycleTerminal(-1);
  }
</script>

<svelte:window {onkeydown} />

<section hidden={!visible}>
  <header>
    <div class="tabs" role="tablist" aria-label={t("terminal.tabs")}>
      {#each terminals.tabs as tab (tab.id)}
        {@const label = t("terminal.tab", { number: String(tab.number) })}
        <div class="tab" class:active={tab.id === terminals.active} class:ended={tab.status === "closed"}>
          <button
            role="tab"
            aria-selected={tab.id === terminals.active}
            aria-controls="terminal-{tab.id}"
            onclick={() => selectTerminal(tab.id)}
          >
            {label}
          </button>
          <button
            class="close"
            aria-label={t("terminal.close", { number: String(tab.number) })}
            title={t("terminal.close", { number: String(tab.number) })}
            onclick={() => closeTerminal(tab.id)}
          >
            ×
          </button>
        </div>
      {/each}
    </div>
    <button class="new" aria-label={t("terminal.new")} title={t("terminal.new")} onclick={newTerminal}>+</button>
  </header>
  {#each terminals.tabs as tab (tab.id)}
    <TerminalView {tab} visible={visible && tab.id === terminals.active} />
  {/each}
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
    gap: 0.4rem;
    align-items: center;
    padding: 0.5rem 1rem 0;
    border-bottom: 1px solid rgba(127, 127, 127, 0.25);
  }
  .tabs {
    display: flex;
    gap: 0.25rem;
    min-width: 0;
    overflow-x: auto;
  }
  .tab {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    border: 1px solid transparent;
    border-bottom: 0;
    border-radius: 0.4rem 0.4rem 0 0;
  }
  .tab.active {
    border-color: rgba(127, 127, 127, 0.3);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }
  .tab.ended button[role="tab"] {
    opacity: 0.6;
  }
  .tab button {
    border: 0;
    background: none;
    color: inherit;
  }
  button[role="tab"] {
    padding: 0.4rem 0.3rem 0.4rem 0.7rem;
  }
  .tab.active button[role="tab"] {
    font-weight: 600;
  }
  .close {
    padding: 0.2rem 0.5rem;
    border-radius: 0.3rem;
    line-height: 1;
  }
  .close:hover,
  .new:hover {
    color: var(--accent);
  }
  .new {
    flex-shrink: 0;
    width: 1.8rem;
    height: 1.8rem;
    margin-bottom: 0.2rem;
    border: 1px solid rgba(127, 127, 127, 0.4);
    border-radius: 0.4rem;
    background: none;
    color: inherit;
    font-size: 1.1rem;
    line-height: 1;
  }
</style>
