<!--
  Administración (solo con Perfil de administrador): las Claves públicas
  autorizadas de cada usuario y «Crear perfil», en dos pestañas.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import { showSection } from "../../state/admin.svelte";
  import { session } from "../../state/session.svelte";
  import CreateProfile from "../CreateProfile.svelte";
  import KeyList from "./KeyList.svelte";

  const section = $derived(session.screen.name === "admin" ? session.screen.section : "keys");
  const SECTIONS = [
    ["keys", "admin.keys"],
    ["createProfile", "admin.createProfile"],
  ] as const;
</script>

<section class="flex min-h-0 min-w-0 flex-col">
  <header class="flex shrink-0 items-center justify-between gap-4 border-b px-4 py-3">
    <div>
      <h1 class="text-base font-semibold">{t("admin.title")}</h1>
      <p class="text-xs text-muted-foreground">{t(section === "keys" ? "admin.keysHelp" : "admin.createProfileHelp")}</p>
    </div>
    <div class="flex rounded-lg bg-muted p-0.5" role="tablist" aria-label={t("admin.title")}>
      {#each SECTIONS as [value, label] (value)}
        <button
          role="tab"
          aria-selected={section === value}
          class="rounded-md px-3 py-1 text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring aria-selected:bg-background aria-selected:text-foreground aria-selected:shadow-[0_1px_2px_var(--shadow)]"
          onclick={() => showSection(value)}
        >
          {t(label)}
        </button>
      {/each}
    </div>
  </header>
  <div class="min-h-0 flex-1 overflow-y-auto" role="tabpanel">
    {#if section === "keys"}
      <KeyList />
    {:else}
      <div class="px-8 py-8">
        <CreateProfile embedded />
      </div>
    {/if}
  </div>
</section>
