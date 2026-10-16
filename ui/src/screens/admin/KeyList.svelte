<!--
  Las Claves públicas autorizadas de cada usuario, leídas de
  `/etc/ssh/authorized_keys/`: su nombre (o el final de la clave) y su modo.
  Cada una se puede revocar o cambiar de modo desde su menú; «Personalizada» es
  una clave configurada a mano, que la app solo revoca. Arriba, «Autorizar clave».
-->
<script lang="ts">
  import Key from "~icons/heroicons/key";
  import More from "~icons/heroicons/ellipsis-horizontal";
  import Plus from "~icons/heroicons/plus";
  import { Button } from "../../components/ui/button";
  import { t } from "../../i18n";
  import { parseAuthorizedKey, type AuthorizedKey, type KeyMode } from "../../lib/authorizedKeys";
  import { admin, emptyAuthorizeForm, loadKeys, revokeKey, setKeyMode } from "../../state/admin.svelte";
  import { openMenu, type MenuItem } from "../../state/menu.svelte";
  import { EMPTY, MUTED } from "../files";
  import AuthorizeKey from "../AuthorizeKey.svelte";

  /** Las claves de cada usuario, con su línea tal cual para cambiarla; las líneas que no son una clave se quedan fuera. */
  const users = $derived(
    (admin.keys ?? []).map(({ user, lines }) => ({
      user,
      keys:
        lines?.flatMap((line) => {
          const key = parseAuthorizedKey(line);
          return key ? [{ line, key }] : [];
        }) ?? null,
    })),
  );

  const MODE_LABEL: Record<KeyMode, string> = { simple: "admin.modeSimple", advanced: "admin.modeAdvanced", custom: "admin.modeCustom" };
  const MODE_STYLE: Record<KeyMode, string> = {
    simple: "border-border text-muted-foreground",
    advanced: "border-transparent bg-accent text-accent-text",
    custom: "border-warning/50 text-warning",
  };

  function menuItems(user: string, line: string, key: AuthorizedKey): MenuItem[] {
    const items: MenuItem[] = [];
    if (key.mode === "advanced") items.push({ label: t("admin.toSimple"), action: () => void setKeyMode(user, line, key, "simple") });
    if (key.mode === "simple") items.push({ label: t("admin.toAdvanced"), action: () => void setKeyMode(user, line, key, "advanced") });
    items.push({ label: t("admin.revoke"), destructive: true, action: () => void revokeKey(user, line, key) });
    return items;
  }

  function showMenu(event: MouseEvent, anchor: HTMLElement, user: string, line: string, key: AuthorizedKey) {
    if (admin.busy) return event.preventDefault();
    openMenu(event, anchor, t("admin.menuLabel", { name: key.name ?? t("admin.unnamed"), user }), menuItems(user, line, key));
  }
</script>

{#if admin.error}
  <div class="{EMPTY} flex flex-col items-center gap-3" role="alert">
    <p>{admin.error}</p>
    <Button size="sm" variant="outline" onclick={loadKeys}>{t("admin.reload")}</Button>
  </div>
{:else if !admin.keys}
  <p class={EMPTY}>{t("admin.loading")}</p>
{:else}
  <div class="flex flex-col gap-5 px-4 py-4">
    {#if admin.authorize}
      <div class="max-w-[28rem] self-center rounded-xl border p-5">
        <AuthorizeKey form={admin.authorize} oncancel={() => (admin.authorize = null)} />
      </div>
    {:else}
      <div class="flex justify-end gap-2">
        <Button size="sm" variant="outline" disabled={admin.loading} onclick={loadKeys}>{t("admin.reload")}</Button>
        <Button size="sm" onclick={() => (admin.authorize = emptyAuthorizeForm())}><Plus />{t("authorize.title")}</Button>
      </div>
    {/if}
    {#if !users.length}
      <p class={EMPTY}>{t("admin.noUsers")}</p>
    {/if}
    {#each users as { user, keys } (user)}
      <section aria-labelledby="keys-{user}">
        <h2 id="keys-{user}" class="mb-1 px-2.5 font-mono text-sm font-semibold">{user}</h2>
        {#if keys === null}
          <p class="px-2.5 text-sm text-destructive">{t("admin.unreadable")}</p>
        {:else if !keys.length}
          <p class="px-2.5 text-sm text-muted-foreground">{t("admin.noKeys")}</p>
        {:else}
          <ul>
            {#each keys as { line, key }, i (i)}
              <li
                class="grid h-9 grid-cols-[1.25rem_minmax(0,1fr)_auto_1.75rem] items-center gap-3 rounded-md px-2.5 hover:bg-accent/50"
                oncontextmenu={(event) => showMenu(event, event.currentTarget, user, line, key)}
              >
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
                <button
                  class="grid size-7 place-items-center rounded-md text-muted-foreground hover:bg-foreground/8 hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-50"
                  aria-label={t("admin.menuLabel", { name: key.name ?? t("admin.unnamed"), user })}
                  aria-haspopup="menu"
                  disabled={admin.busy}
                  onclick={(event) => showMenu(event, event.currentTarget, user, line, key)}
                >
                  <More class="size-4" />
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {/each}
  </div>
{/if}
