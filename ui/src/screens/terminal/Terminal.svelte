<!--
  Terminales (solo en Modo avanzado): pestañas con «+» para abrir otra y, debajo,
  la Terminal de la pestaña activa. Atajos: Ctrl+Mayús+T abre una nueva desde
  cualquier pantalla; Ctrl+Mayús+W cierra la actual y Ctrl+RePág / Ctrl+AvPág
  cambian de pestaña.
-->
<script lang="ts">
  import Close from "~icons/heroicons/x-mark-16-solid";
  import Plus from "~icons/heroicons/plus-20-solid";
  import { Button } from "../../components/ui/button";
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

<section class="flex min-h-0 min-w-0 flex-col" hidden={!visible}>
  <header class="flex h-11 shrink-0 items-center gap-1 border-b px-2.5">
    <div class="flex min-w-0 gap-1 overflow-x-auto [scrollbar-width:none]" role="tablist" aria-label={t("terminal.tabs")}>
      {#each terminals.tabs as tab (tab.id)}
        {@const label = t("terminal.tab", { number: String(tab.number) })}
        {@const active = tab.id === terminals.active}
        <div class="flex h-7 shrink-0 items-center rounded-lg pr-1 transition-colors {active ? 'bg-foreground/10' : 'hover:bg-foreground/5'}">
          <button
            class="h-full rounded-lg pr-1.5 pl-3 focus-visible:outline-2 focus-visible:outline-ring {active ? 'font-medium' : 'text-muted-foreground'} {tab.status === 'closed' ? 'line-through decoration-muted-foreground/60' : ''}"
            role="tab"
            aria-selected={active}
            aria-controls="terminal-{tab.id}"
            onclick={() => selectTerminal(tab.id)}
          >
            {label}
          </button>
          <button
            class="grid size-5 place-items-center rounded-md text-muted-foreground hover:bg-foreground/10 hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
            aria-label={t("terminal.close", { number: String(tab.number) })}
            title={t("terminal.close", { number: String(tab.number) })}
            onclick={() => closeTerminal(tab.id)}
          >
            <Close class="size-3.5" />
          </button>
        </div>
      {/each}
    </div>
    <Button variant="ghost" size="icon-sm" aria-label={t("terminal.new")} title={t("terminal.new")} onclick={newTerminal}>
      <Plus />
    </Button>
  </header>
  {#each terminals.tabs as tab (tab.id)}
    <TerminalView {tab} visible={visible && tab.id === terminals.active} />
  {/each}
</section>
