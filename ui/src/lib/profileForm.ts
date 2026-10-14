/**
 * Formulario «Crear perfil»: lo que escribe el Administrador y qué campo está
 * mal, con las mismas reglas que el núcleo (`crates/core/src/profile.rs`), para
 * explicarlo campo a campo antes de conectar con el Servidor.
 */
import type { NewProfile } from "./api";

export interface ProfileForm {
  host: string;
  port: string;
  username: string;
  mode: NewProfile["mode"];
  sharedFolder: string;
  admin: boolean;
}

export type ProfileField = "host" | "port" | "username" | "sharedFolder";

/** El formulario como lo recibe el núcleo, o los campos que están mal. */
export function parseProfileForm(form: ProfileForm): { profile: NewProfile } | { invalid: ProfileField[] } {
  const host = form.host.trim();
  const portText = form.port.trim();
  const port = Number(portText);
  const username = form.username.trim();
  const sharedFolder = form.sharedFolder.trim();

  const invalid: ProfileField[] = [];
  if (!host || /[\s/]/.test(host)) invalid.push("host");
  if (!/^\d+$/.test(portText) || port < 1 || port > 65535) invalid.push("port");
  if (!/^[a-z_][a-z0-9_-]*$/.test(username)) invalid.push("username");
  if (sharedFolder && !sharedFolder.startsWith("/")) invalid.push("sharedFolder");
  if (invalid.length) return { invalid };

  return { profile: { host, port, username, mode: form.mode, shared_folder: sharedFolder || null, admin: form.admin } };
}
