/**
 * «Autorizar clave» (Administrador): la línea de `authorized_keys` con la que
 * el Servidor aplica el modo del Miembro (ADR 0006). La app solo enseña o no
 * la Terminal; quien la permite o la niega es el Servidor, con estas opciones.
 */
import type { Profile } from "./api";

/** Opciones de cada modo: Modo simple solo puede usar SFTP; el avanzado, además, una consola. Ninguno abre túneles. */
export const KEY_OPTIONS: Record<Profile["mode"], string> = {
  simple: 'restrict,command="internal-sftp"',
  advanced: "restrict,pty",
};

const KEY_TYPES = /^(ssh-ed25519|ssh-rsa|ecdsa-sha2-nistp(256|384|521)|sk-ssh-ed25519@openssh\.com|sk-ecdsa-sha2-nistp256@openssh\.com)$/;
const BASE64 = /^[A-Za-z0-9+/]+={0,3}$/;

/**
 * La línea para `/etc/ssh/authorized_keys/<usuario>`, o `null` si `key` no es una
 * única clave pública tal como la copia la app (`tipo base64 [comentario]`).
 * Una línea que ya trae opciones no vale: las pone el modo.
 *
 * `owner` («De quién es») sustituye al comentario de la clave, que en las que
 * genera la app es siempre `sshpanel`, para reconocer cada línea cuando varias
 * claves comparten usuario. Los espacios se cambian por `-`.
 */
export function authorizedKeyLine(key: string, mode: Profile["mode"], owner = ""): string | null {
  const line = key.trim();
  if (!line || /[\r\n]/.test(line)) return null;
  const [type, data, ...comment] = line.split(/\s+/);
  if (!KEY_TYPES.test(type) || !data || !BASE64.test(data)) return null;
  const name = owner.trim().replace(/\s+/g, "-");
  return [KEY_OPTIONS[mode], type, data, ...(name ? [name] : comment)].join(" ");
}
