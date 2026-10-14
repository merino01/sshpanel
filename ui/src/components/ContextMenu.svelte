<!--
  Menú contextual de la app. Se monta una sola vez y pinta el que se abra con
  `openMenu()` (state/menu.svelte.ts). Se maneja con el teclado: flechas, Inicio
  y Fin para moverse; Escape o Tab para cerrarlo y devolver el foco al elemento
  sobre el que se abrió.
-->
<script lang="ts">
  import { closeMenu, menu, type MenuItem } from "../state/menu.svelte";

  let list = $state<HTMLUListElement>();

  function buttons(): HTMLButtonElement[] {
    return [...(list?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])];
  }

  // Junto al puntero o, si se abrió con el teclado, bajo el elemento; sin salirse de la ventana.
  // Con el ratón no se marca ninguna opción; con el teclado, la primera.
  $effect(() => {
    const current = menu.current;
    if (!current || !list) return;
    const rect = current.anchor.getBoundingClientRect();
    const x = current.pointer?.x ?? rect.left + 16;
    const y = current.pointer?.y ?? rect.bottom;
    list.style.left = `${Math.max(4, Math.min(x, innerWidth - list.offsetWidth - 4))}px`;
    list.style.top = `${Math.max(4, Math.min(y, innerHeight - list.offsetHeight - 4))}px`;
    if (current.pointer) list.focus();
    else buttons()[0]?.focus();
  });

  function choose(item: MenuItem) {
    closeMenu();
    item.action();
  }

  function onkeydown(event: KeyboardEvent) {
    const all = buttons();
    const focused = all.indexOf(document.activeElement as HTMLButtonElement);
    // Sin ninguna marcada (abierto con el ratón), ↓ va a la primera y ↑ a la última.
    const up = focused === -1 ? all.length - 1 : focused - 1;
    const target = { ArrowDown: focused + 1, ArrowUp: up, Home: 0, End: all.length - 1 }[event.key];
    if (target !== undefined) {
      event.preventDefault();
      all[(target + all.length) % all.length].focus();
    } else if (event.key === "Escape" || event.key === "Tab") {
      event.preventDefault();
      const anchor = menu.current?.anchor;
      closeMenu();
      anchor?.focus();
    }
  }

  function onpointerdown(event: PointerEvent) {
    if (menu.current && !list?.contains(event.target as Node)) closeMenu();
  }
</script>

<svelte:document onpointerdowncapture={onpointerdown} />
<svelte:window onblur={closeMenu} onresize={closeMenu} />

{#if menu.current}
  <ul bind:this={list} class="context-menu" role="menu" tabindex="-1" aria-label={menu.current.label} {onkeydown}>
    {#each menu.current.items as item (item.label)}
      <li role="none">
        <button role="menuitem" tabindex="-1" onclick={() => choose(item)}>{item.label}</button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .context-menu {
    position: fixed;
    z-index: 10;
    min-width: 11rem;
    margin: 0;
    padding: 0.3rem;
    border: 1px solid rgba(127, 127, 127, 0.3);
    border-radius: 0.5rem;
    background: var(--bg);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.25);
    list-style: none;
    text-align: left;
  }
  .context-menu:focus {
    outline: none;
  }
  [role="menuitem"] {
    display: block;
    width: 100%;
    padding: 0.4rem 0.7rem;
    border: 0;
    border-radius: 0.3rem;
    background: none;
    color: inherit;
    text-align: left;
  }
  [role="menuitem"]:hover,
  [role="menuitem"]:focus {
    outline: none;
    background: var(--accent);
    color: white;
  }
</style>
