<!--
  Menú contextual de la app. Se monta una sola vez y pinta el que se abra con
  `openMenu()` (state/menu.svelte.ts): junto al puntero o, si se abrió con el
  teclado, bajo el elemento. El teclado (flechas, Inicio, Fin, Escape) y el
  cierre al pulsar fuera los pone Bits UI, también en los submenús (flecha
  derecha para entrar, izquierda para salir). Al cancelarlo, el foco vuelve al
  elemento sobre el que se abrió; al elegir una opción, se queda donde la deje.
-->
<script lang="ts">
  import * as DropdownMenu from "./ui/dropdown-menu";
  import { closeMenu, menu, type MenuItem, type OpenMenu } from "../state/menu.svelte";

  // El último abierto: se sigue pintando mientras se cierra.
  let shown = $state.raw<OpenMenu | null>(null);
  let chosen = false;

  $effect.pre(() => {
    if (menu.current) {
      shown = menu.current;
      chosen = false;
    }
  });

  const ITEM = "rounded-md px-2.5 py-1.5 focus:bg-primary focus:text-primary-foreground";
  const PANEL = "w-auto min-w-44 rounded-xl border border-glass-edge bg-popover/90 p-1 shadow-[0_10px_30px_var(--shadow)] ring-0 backdrop-blur-xl";

  // En el puntero, un punto sin tamaño; con el teclado, el propio elemento.
  const anchor = $derived.by(() => {
    if (!shown?.pointer) return shown?.anchor ?? null;
    const { x, y } = shown.pointer;
    return { getBoundingClientRect: () => DOMRect.fromRect({ x, y, width: 0, height: 0 }) };
  });
</script>

<svelte:window onblur={closeMenu} onresize={closeMenu} />

{#snippet items(list: MenuItem[])}
  {#each list as item (item.label)}
    {#if "items" in item}
      <DropdownMenu.Sub>
        <DropdownMenu.SubTrigger class="{ITEM} data-open:bg-primary data-open:text-primary-foreground">{item.label}</DropdownMenu.SubTrigger>
        <DropdownMenu.SubContent class={PANEL}>
          {@render items(item.items)}
        </DropdownMenu.SubContent>
      </DropdownMenu.Sub>
    {:else}
      <DropdownMenu.Item
        variant={item.destructive ? "destructive" : "default"}
        class={ITEM}
        onSelect={() => {
          chosen = true;
          item.action();
        }}
      >
        {item.label}
      </DropdownMenu.Item>
    {/if}
  {/each}
{/snippet}

{#if shown}
  {#key shown}
    <DropdownMenu.Root open={menu.current === shown} onOpenChange={(open) => !open && closeMenu()}>
      <DropdownMenu.Content
        customAnchor={anchor}
        side="bottom"
        align="start"
        sideOffset={shown.pointer ? 0 : 2}
        alignOffset={shown.pointer ? 0 : 16}
        aria-label={shown.label}
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          if (!chosen) shown?.anchor.focus();
        }}
        class={PANEL}
      >
        {@render items(shown.items)}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  {/key}
{/if}
