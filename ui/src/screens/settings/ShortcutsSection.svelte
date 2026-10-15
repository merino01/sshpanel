<!--
  Sección «Atajos» de Ajustes: cada acción con sus combinaciones. «Cambiar»
  graba la siguiente que se pulse (Escape cancela); si ya la usa otra acción,
  lo dice y no la guarda hasta que se pulse otra. «Restablecer» vuelve a los
  de por defecto.
-->
<script lang="ts">
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
  <header>
    <div>
      <h2 id="shortcuts">{t("settings.shortcuts")}</h2>
      <p class="help">{t("settings.shortcuts.help")}</p>
    </div>
    <button class="secondary" disabled={!changed} onclick={resetShortcuts}>{t("settings.shortcuts.reset")}</button>
  </header>
  <ul>
    {#each ACTIONS as action (action)}
      {@const name = t(`shortcuts.action.${action}`)}
      <li class:recording={recording === action}>
        <span class="name">{name}</span>
        <span class="keys">
          {#each shortcuts[action] as combo, i (combo)}
            {#if i > 0}<span class="or">{t("settings.shortcuts.or")}</span>{/if}
            <kbd>{label(combo)}</kbd>
          {/each}
        </span>
        <!-- Mientras graba, la app no aplica Atajos aquí: la tecla es para grabarla. -->
        <button
          class="change"
          data-no-shortcuts
          aria-label={recording === action ? t("settings.shortcuts.recording") : t("settings.shortcuts.changeLabel", { action: name })}
          onclick={() => toggle(action)}
          onkeydown={(event) => onkeydown(event, action)}
          onkeyup={onkeyup}
          onblur={() => recording === action && stop()}
        >
          {recording === action ? t("settings.shortcuts.recording") : t("settings.shortcuts.change")}
        </button>
        {#if recording === action && conflict}
          <p class="conflict" role="alert">{conflictMessage(conflict)}</p>
        {/if}
      </li>
    {/each}
  </ul>
</section>

<style>
  header {
    display: flex;
    gap: 1rem;
    align-items: start;
    justify-content: space-between;
    margin-bottom: 1.1rem;
  }
  header .secondary {
    flex: none;
    padding: 0.4rem 0.9rem;
    font-size: 0.9rem;
  }
  .help {
    font-size: 0.9rem;
  }
  ul {
    margin: 0;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: 0.6rem;
    list-style: none;
  }
  li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 7.5rem;
    gap: 0.75rem;
    align-items: center;
    padding: 0.5rem 0.5rem 0.5rem 0.85rem;
    border-bottom: 1px solid var(--border);
  }
  li:last-child {
    border-bottom: 0;
  }
  li.recording {
    background: color-mix(in srgb, var(--accent) 8%, transparent);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  li:first-child.recording {
    border-radius: 0.6rem 0.6rem 0 0;
  }
  li:last-child.recording {
    border-radius: 0 0 0.6rem 0.6rem;
  }
  .name {
    font-size: 0.9rem;
  }
  .keys {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    align-items: center;
    justify-content: flex-end;
  }
  /* Cada combinación, como una tecla: el borde de abajo más grueso le da relieve. */
  kbd {
    padding: 0.15rem 0.45rem;
    border: 1px solid var(--border-strong);
    border-bottom-width: 2px;
    border-radius: 0.35rem;
    background: var(--subtle);
    font-family: inherit;
    font-size: 0.85rem;
    white-space: nowrap;
  }
  .or {
    font-size: 0.8rem;
    opacity: 0.6;
  }
  .change {
    padding: 0.3rem 0.6rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.4rem;
    background: none;
    color: inherit;
    font-size: 0.85rem;
  }
  .change:hover {
    border-color: var(--accent);
    color: var(--accent-text);
  }
  .change:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
  li.recording .change {
    border: 1px dashed var(--accent);
    color: var(--accent-text);
    font-weight: 600;
  }
  .conflict {
    grid-column: 1 / -1;
    margin: 0;
    color: var(--error);
    font-size: 0.85rem;
  }
</style>
