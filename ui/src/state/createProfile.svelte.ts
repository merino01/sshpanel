/** «Crear perfil» (Administrador): el formulario, guardar el fichero, el resultado y «Autorizar clave». */
import { save } from "@tauri-apps/plugin-dialog";
import { branding } from "../branding.generated";
import { errorMessage, t } from "../i18n";
import { api, type Profile } from "../lib/api";
import { parseProfileForm, type ProfileField, type ProfileForm } from "../lib/profileForm";
import { session, show } from "./session.svelte";

export const createForm = $state<{
  form: ProfileForm;
  invalid: ProfileField[];
  creating: boolean;
  error?: string;
  /** El último Perfil creado y dónde se guardó. */
  created?: { profile: Profile; path: string };
  /** «Autorizar clave»: la clave pública que ha enviado un Miembro, su modo y de quién es. */
  authorize: { key: string; mode: Profile["mode"]; owner: string };
}>({ form: emptyForm(), invalid: [], creating: false, authorize: { key: "", mode: "simple", owner: "" } });

/** Formulario nuevo; con un Perfil importado, propone su mismo Servidor y Carpeta compartida. */
function emptyForm(): ProfileForm {
  const current = session.appState?.profile;
  return {
    host: current?.host ?? "",
    port: String(current?.port ?? 22),
    username: "",
    mode: "simple",
    sharedFolder: current?.shared_folder ?? "",
    admin: false,
  };
}

/** Abre «Crear perfil»; al salir se vuelve a la pantalla actual. */
export function openCreateProfile() {
  Object.assign(createForm, {
    form: emptyForm(),
    invalid: [],
    creating: false,
    error: undefined,
    created: undefined,
    authorize: { key: "", mode: "simple", owner: "" },
  });
  const back = session.screen.name === "createProfile" ? session.screen.back : session.screen;
  show({ name: "createProfile", back });
}

export function leaveCreateProfile() {
  if (session.screen.name === "createProfile") show(session.screen.back);
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
