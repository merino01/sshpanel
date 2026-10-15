<!--
  Sección «Atajos» de Ajustes: cada acción con sus combinaciones. «Cambiar»
  graba la siguiente que se pulse (Escape cancela); si ya la usa otra acción,
  lo dice y no la guarda hasta que se pulse otra. «Restablecer» vuelve a los
  de por defecto.
-->
<script lang="ts">
  import { Button } from "../../components/ui/button";
  import { t } from "../../i18n";
  import { ACTIONS, comboOf, conflictOf, type Action, type Conflict } from "../../lib/shortcuts";
  import { preferences } from "../../state/preferences.svelte";
  import { activeShortcuts, label, resetShortcuts, setShortcut } from "../../state/shortcuts.svelte";

  const shortcuts = $derived(activeShortcuts());
  const changed = $derived(Object.keys(preferences.shortcuts).length > 0);
  /** La acción cuya combinación se está grabando. */
  let recording = $state<Action | null>(null);
  /** La última combinación rechazada al grabar, y quién la usa. */
  let conflict = $state<{ combo: string; by: Conflict } | null>(null);
  /** Una tecla grabada: al soltarla no debe pulsar el botón (Espacio lo haría). */
  let swallowKeyup = false;

  function toggle(action: Action) {
    recording = recording === action ? null : action;
    conflict = null;
  }

  function stop() {
    recording = null;
    conflict = null;
  }

  function onkeydown(event: KeyboardEvent, action: Action) {
    if (recording !== action) return;
    const plain = !event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey;
    // Tab sigue moviendo el foco: no se graba.
    if (event.key === "Tab" && plain) return stop();
    event.preventDefault();
    swallowKeyup = true;
    if (event.key === "Escape" && plain) return stop();
    const combo = comboOf(event);
    if (!combo) return;
    const by = conflictOf(shortcuts, action, combo);
    if (by) {
      conflict = { combo, by };
      return;
    }
    setShortcut(action, combo);
    stop();
  }

  function onkeyup(event: KeyboardEvent) {
    if (!swallowKeyup) return;
    event.preventDefault();
    swallowKeyup = false;
  }

  function conflictMessage({ combo, by }: { combo: string; by: Conflict }): string {
    if (by === "terminalTabs") return t("settings.shortcuts.conflictTerminal", { combo: label(combo) });
    return t("settings.shortcuts.conflict", { combo: label(combo), action: t(`shortcuts.action.${by}`) });
  }
</script>

<section aria-labelledby="shortcuts">
  <header class="mb-5 flex items-start justify-between gap-4">
    <div>
      <h2 id="shortcuts" class="text-xl font-semibold tracking-tight">{t("settings.shortcuts")}</h2>
      <p class="mt-1 max-w-[60ch] text-muted-foreground">{t("settings.shortcuts.help")}</p>
    </div>
    <Button variant="outline" size="sm" disabled={!changed} onclick={resetShortcuts}>{t("settings.shortcuts.reset")}</Button>
  </header>
  <ul class="divide-y overflow-hidden rounded-xl border">
    {#each ACTIONS as action (action)}
      {@const name = t(`shortcuts.action.${action}`)}
      {@const isRecording = recording === action}
      <li class="grid grid-cols-[minmax(0,1fr)_auto_7.5rem] items-center gap-3 py-2 pr-2 pl-3.5 {isRecording ? 'bg-accent shadow-[inset_3px_0_0_var(--primary)]' : ''}">
        <span>{name}</span>
        <span class="flex flex-wrap items-center justify-end gap-1.5">
          {#each shortcuts[action] as combo, i (combo)}
            {#if i > 0}<span class="text-xs text-muted-foreground">{t("settings.shortcuts.or")}</span>{/if}
            <kbd class="rounded-md border border-b-2 border-input bg-muted px-1.5 py-0.5 font-mono text-xs whitespace-nowrap">{label(combo)}</kbd>
          {/each}
        </span>
        <!-- Mientras graba, la app no aplica Atajos aquí: la tecla es para grabarla. -->
        <Button
          variant="outline"
          size="sm"
          class={isRecording ? "border-dashed border-primary font-semibold text-accent-text" : ""}
          data-no-shortcuts
          aria-label={isRecording ? t("settings.shortcuts.recording") : t("settings.shortcuts.changeLabel", { action: name })}
          onclick={() => toggle(action)}
          onkeydown={(event: KeyboardEvent) => onkeydown(event, action)}
          onkeyup={onkeyup}
          onblur={() => isRecording && stop()}
        >
          {isRecording ? t("settings.shortcuts.recording") : t("settings.shortcuts.change")}
        </Button>
        {#if isRecording && conflict}
          <p class="col-span-full text-xs text-destructive" role="alert">{conflictMessage(conflict)}</p>
        {/if}
      </li>
    {/each}
  </ul>
</section>
