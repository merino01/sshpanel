/**
 * La Administración: sus secciones y las Claves públicas autorizadas de cada
 * usuario, leídas del Servidor y cambiadas con sudo (ADR 0007).
 */
import { errorMessage, t } from "../i18n";
import { api, type Profile, type UserKeys } from "../lib/api";
import { isOwnKey, withMode, type AuthorizedKey } from "../lib/authorizedKeys";
import { isConnectionError } from "../lib/errors";
import { confirm } from "./confirm.svelte";
import { resetCreateForm } from "./createProfile.svelte";
import { session, show, showConnectionLost } from "./session.svelte";
import { askSudoPassword } from "./sudo.svelte";
import { toasts } from "./toasts.svelte";

/** «Autorizar clave»: la clave pública que ha enviado un Miembro, para qué usuario, su modo y de quién es. */
export interface AuthorizeForm {
  key: string;
  user: string;
  mode: Profile["mode"];
  owner: string;
}

export const admin = $state<{
  keys: UserKeys[] | null;
  /** Los usuarios del Servidor, para elegir a quién autorizar una clave. */
  users: string[];
  loading: boolean;
  /** Se está cambiando una clave. */
  busy: boolean;
  error?: string;
  /** El formulario de «Autorizar clave» de la pestaña Claves, si está abierto. */
  authorize: AuthorizeForm | null;
}>({ keys: null, users: [], loading: false, busy: false, authorize: null });

export function emptyAuthorizeForm(user = "", mode: Profile["mode"] = "simple"): AuthorizeForm {
  return { key: "", user, mode, owner: "" };
}

/** Abre la Administración en las Claves, leyéndolas de nuevo. */
export function openAdmin() {
  resetCreateForm();
  admin.authorize = null;
  show({ name: "admin", section: "keys" });
  void loadKeys();
}

export function showSection(section: "keys" | "createProfile") {
  if (session.screen.name === "admin") show({ name: "admin", section });
}

/** Lee las claves de cada usuario y los usuarios del Servidor. */
export async function loadKeys() {
  admin.loading = true;
  admin.error = undefined;
  try {
    const [keys, users] = await Promise.all([api.authorizedKeys(), api.serverUsers().catch(() => [])]);
    admin.keys = keys;
    admin.users = users;
  } catch (code) {
    if (isConnectionError(code)) return showConnectionLost(code);
    admin.error = errorMessage(code);
  } finally {
    admin.loading = false;
  }
}

/** Autoriza `line` para `user`. Devuelve si se ha hecho. */
export async function authorizeKey(user: string, line: string): Promise<boolean> {
  return changeKeys(() => api.authorizeKey(user, line), t("admin.authorized", { user }));
}

/** Revoca la clave `line` de `user`, confirmándolo antes; si es la propia, explicando que se perderá el acceso. */
export async function revokeKey(user: string, line: string, key: AuthorizedKey) {
  const own = isOwn(user, key);
  const name = key.name ?? t("admin.unnamed");
  const ok = await confirm({
    title: t(own ? "admin.revokeOwnTitle" : "admin.revokeTitle", { name, user }),
    body: t(own ? "admin.revokeOwnBody" : "admin.revokeBody"),
    confirmLabel: t(own ? "admin.revokeOwnConfirm" : "admin.revoke"),
    destructive: true,
  });
  if (ok) await changeKeys(() => api.revokeKey(user, line), t("admin.revoked", { name, user }));
}

/** Pasa la clave `line` de `user` a `mode`; si es la propia y pasa a simple, explica que se perderá la Administración. */
export async function setKeyMode(user: string, line: string, key: AuthorizedKey, mode: Profile["mode"]) {
  if (mode === "simple" && isOwn(user, key)) {
    const ok = await confirm({
      title: t("admin.ownToSimpleTitle"),
      body: t("admin.ownToSimpleBody"),
      confirmLabel: t("admin.toSimple"),
      destructive: true,
    });
    if (!ok) return;
  }
  const label = t(mode === "simple" ? "admin.modeSimple" : "admin.modeAdvanced");
  await changeKeys(() => api.replaceKey(user, line, withMode(key, mode)), t("admin.modeChanged", { name: key.name ?? t("admin.unnamed"), mode: label }));
}

function isOwn(user: string, key: AuthorizedKey): boolean {
  const profile = session.appState?.profile;
  return !!profile && isOwnKey(user, key, profile.username, session.appState?.public_key ?? null);
}

/**
 * Hace un cambio de claves y vuelve a leerlas. Si sudo necesita la contraseña
 * (o la recordada ya no vale), la pide y lo reintenta; cancelar no cambia nada.
 */
async function changeKeys(run: () => Promise<void>, done: string): Promise<boolean> {
  admin.busy = true;
  try {
    for (;;) {
      try {
        await run();
        toasts.success(done);
        return true;
      } catch (code) {
        if (code === "admin.sudo_password_needed" || code === "admin.sudo_wrong_password") {
          if (await askSudoPassword(code === "admin.sudo_wrong_password")) continue;
          return false;
        }
        if (isConnectionError(code)) {
          showConnectionLost(code);
          return false;
        }
        toasts.error(errorMessage(code));
        return false;
      }
    }
  } finally {
    admin.busy = false;
    if (session.screen.name === "admin") void loadKeys();
  }
}
