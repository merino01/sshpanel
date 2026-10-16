/** «Crear perfil» (Administrador): el formulario, guardar el fichero, el resultado y «Autorizar clave». */
import { save } from "@tauri-apps/plugin-dialog";
import { branding } from "../branding.generated";
import { errorMessage, t } from "../i18n";
import { api, type Profile } from "../lib/api";
import { parseProfileForm, type ProfileField, type ProfileForm } from "../lib/profileForm";
import { importProfileFrom, session, show } from "./session.svelte";

export const createForm = $state<{
  form: ProfileForm;
  invalid: ProfileField[];
  creating: boolean;
  /** Sin Perfil importado: es el primero, el del propio Administrador. */
  own: boolean;
  error?: string;
  /** El último Perfil creado y dónde se guardó. */
  created?: { profile: Profile; path: string };
  /** «Autorizar clave»: la clave pública que ha enviado un Miembro, su modo y de quién es. */
  authorize: { key: string; mode: Profile["mode"]; owner: string };
}>({ form: emptyForm(), invalid: [], creating: false, own: false, authorize: { key: "", mode: "simple", owner: "" } });

/**
 * Formulario nuevo. Con un Perfil importado, es para un Miembro y propone su
 * mismo Servidor y Carpeta compartida; sin él, es el del propio Administrador.
 */
function emptyForm(): ProfileForm {
  const current = session.appState?.profile;
  return {
    host: current?.host ?? "",
    port: String(current?.port ?? 22),
    username: "",
    mode: current ? "simple" : "advanced",
    sharedFolder: current?.shared_folder ?? "",
    admin: !current,
  };
}

/** Abre «Crear perfil» suelto (desde la Bienvenida); al salir se vuelve a la pantalla actual. */
export function openCreateProfile() {
  resetCreateForm();
  const back = session.screen.name === "createProfile" ? session.screen.back : session.screen;
  show({ name: "createProfile", back });
}

/** Deja «Crear perfil» como recién abierto. */
export function resetCreateForm() {
  Object.assign(createForm, {
    form: emptyForm(),
    invalid: [],
    creating: false,
    own: !session.appState?.profile,
    error: undefined,
    created: undefined,
    authorize: { key: "", mode: "simple", owner: "" },
  });
}

/** Sale de «Crear perfil»: suelto, a la pantalla de antes; en la Administración, a las Claves. */
export function leaveCreateProfile() {
  if (session.screen.name === "createProfile") show(session.screen.back);
  else if (session.screen.name === "admin") {
    resetCreateForm();
    show({ name: "admin", section: "keys" });
  }
}

/** Otro Perfil para otro Miembro, con el mismo Servidor. */
export function createAnother() {
  const { host, port, sharedFolder } = createForm.form;
  Object.assign(createForm, { form: { ...emptyForm(), host, port, sharedFolder }, created: undefined, error: undefined });
}

/** Comprueba el formulario, pide dónde guardar y crea el Perfil, calculando la Huella. */
export async function submitProfile() {
  const parsed = parseProfileForm(createForm.form);
  createForm.error = undefined;
  if ("invalid" in parsed) {
    createForm.invalid = parsed.invalid;
    return;
  }
  createForm.invalid = [];
  const path = await save({
    title: t("create.saveDialog"),
    defaultPath: `${parsed.profile.username}.${branding.profileExtension}`,
    filters: [{ name: t("welcome.profileFiles", { name: branding.name }), extensions: [branding.profileExtension] }],
  });
  if (!path) return;
  createForm.creating = true;
  try {
    const profile = await api.createProfile(parsed.profile, path);
    createForm.created = { profile, path };
    // Cuando el Miembro mande su clave, se autoriza con el mismo modo.
    createForm.authorize = { key: "", mode: profile.mode, owner: "" };
  } catch (code) {
    createForm.error = errorMessage(code);
  } finally {
    createForm.creating = false;
  }
}

/** Importa el Perfil recién creado, el del propio Administrador, y entra con él. */
export async function importCreated() {
  if (createForm.created) await importProfileFrom(createForm.created.path);
}
