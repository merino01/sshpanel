<!-- Accesos directos fijos: carpeta personal, Carpeta compartida si el Perfil la define, Papelera y, en Modo avanzado, Terminal. -->
<script lang="ts">
  import type { Component } from "svelte";
  import Home from "~icons/heroicons/home";
  import Shared from "~icons/heroicons/user-group";
  import Terminal from "~icons/heroicons/command-line";
  import Trash from "~icons/heroicons/trash";
  import { t } from "../i18n";
  import { explorer, openFolder } from "../state/explorer.svelte";
  import { isAdvanced, session } from "../state/session.svelte";
  import { openTerminal } from "../state/terminal.svelte";
  import { openTrash } from "../state/trash.svelte";

  interface Place {
    label: string;
    icon: Component;
    current: boolean;
    open: () => void;
  }

  const shared = $derived(session.appState?.profile?.shared_folder);
  const places = $derived.by(() => {
    const inFolder = (path: string) => session.screen.name === "explorer" && explorer.listing.path === path;
    const list: Place[] = [{ label: t("explorer.home"), icon: Home, current: inFolder(explorer.home), open: () => openFolder(explorer.home) }];
    if (shared) list.push({ label: t("explorer.shared"), icon: Shared, current: inFolder(shared), open: () => openFolder(shared) });
    list.push({ label: t("trash.title"), icon: Trash, current: session.screen.name === "trash", open: () => openTrash() });
    if (isAdvanced()) list.push({ label: t("terminal.title"), icon: Terminal, current: session.screen.name === "terminal", open: openTerminal });
    return list;
  });
</script>

<nav class="flex flex-col gap-0.5">
  {#each places as place (place.label)}
    <button
      class="group flex h-8 items-center gap-2.5 rounded-lg px-2.5 text-left transition-colors hover:bg-foreground/8 focus-visible:outline-2 focus-visible:outline-ring aria-[current]:bg-primary aria-[current]:text-primary-foreground"
      aria-current={place.current ? "location" : undefined}
      onclick={place.open}
    >
      <place.icon class="size-[18px] shrink-0 text-primary group-aria-[current]:text-primary-foreground" />
      <span class="truncate">{place.label}</span>
    </button>
  {/each}
</nav>
