<!--
  Las Claves públicas autorizadas de cada usuario, leídas de
  `/etc/ssh/authorized_keys/`: su nombre (o el final de la clave) y su modo.
  «Personalizada» es una clave configurada a mano, que la app no va a tocar.
-->
<script lang="ts">
  import Key from "~icons/heroicons/key";
  import { Button } from "../../components/ui/button";
  import { t } from "../../i18n";
  import { parseAuthorizedKey, type AuthorizedKey, type KeyMode } from "../../lib/authorizedKeys";
  import { admin, loadKeys } from "../../state/admin.svelte";
  import { EMPTY, MUTED } from "../files";

  /** Las líneas de cada usuario, leídas; las que no son una clave se quedan fuera. */
  const users = $derived(
    (admin.keys ?? []).map(({ user, lines }) => ({
      user,
      keys: lines?.map(parseAuthorizedKey).filter((key): key is AuthorizedKey => key !== null) ?? null,
    })),
  );

  const MODE_LABEL: Record<KeyMode, string> = { simple: "admin.modeSimple", advanced: "admin.modeAdvanced", custom: "admin.modeCustom" };
  const MODE_STYLE: Record<KeyMode, string> = {
    simple: "border-border text-muted-foreground",
    advanced: "border-transparent bg-accent text-accent-text",
    custom: "border-warning/50 text-warning",
  };
</script>

{#if admin.error}
  <div class="{EMPTY} flex flex-col items-center gap-3" role="alert">
    <p>{admin.error}</p>
    <Button size="sm" variant="outline" onclick={loadKeys}>{t("admin.reload")}</Button>
  </div>
{:else if !admin.keys}
  <p class={EMPTY}>{t("admin.loading")}</p>
{:else if !users.length}
  <p class={EMPTY}>{t("admin.noUsers")}</p>
{:else}
  <div class="flex flex-col gap-5 px-4 py-4">
    {#each users as { user, keys } (user)}
      <section aria-labelledby="keys-{user}">
        <h2 id="keys-{user}" class="mb-1 px-2.5 font-mono text-sm font-semibold">{user}</h2>
        {#if keys === null}
          <p class="px-2.5 text-sm text-destructive">{t("admin.unreadable")}</p>
        {:else if !keys.length}
          <p class="px-2.5 text-sm text-muted-foreground">{t("admin.noKeys")}</p>
        {:else}
          <ul>
            {#each keys as key, i (i)}
              <li class="grid h-9 grid-cols-[1.25rem_minmax(0,1fr)_auto] items-center gap-3 rounded-md px-2.5 hover:bg-accent/50">
                <Key class="size-4 text-muted-foreground" aria-hidden="true" />
                <span class="flex min-w-0 items-baseline gap-2">
                  <span class="truncate" title={key.name ?? undefined}>{key.name ?? t("admin.unnamed")}</span>
                  <span class={MUTED}>{key.type} …{key.key.slice(-8)}</span>
                </span>
                <span
                  class="rounded-full border px-2 py-0.5 text-xs font-medium {MODE_STYLE[key.mode]}"
                  title={key.mode === "custom" ? t("admin.modeCustomHelp") : undefined}
                >
                  {t(MODE_LABEL[key.mode])}
                </span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {/each}
    <div class="flex justify-end">
      <Button size="sm" variant="outline" disabled={admin.loading} onclick={loadKeys}>{t("admin.reload")}</Button>
    </div>
  </div>
{/if}
