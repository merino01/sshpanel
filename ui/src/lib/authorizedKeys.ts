/**
 * Las líneas de `authorized_keys` (Administrador): generar la que aplica el
 * modo del Miembro (ADR 0006) y leer las que ya hay. La app solo enseña o no
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

/** El modo de una clave autorizada; `custom` («Personalizada»): sus opciones no son las de ningún modo. */
export type KeyMode = Profile["mode"] | "custom";

/** Una línea de `authorized_keys` leída. */
export interface AuthorizedKey {
  mode: KeyMode;
  type: string;
  /** La clave en base64. */
  key: string;
  /** El comentario de la clave (en las que autoriza la app, de quién es); `null` si no tiene. */
  name: string | null;
}

/**
 * Lee una línea de `authorized_keys`: `[opciones] tipo base64 [comentario]`.
 * El modo sale de las opciones, sin importar su orden; sin opciones, o con
 * otras, es `custom`. `null` si la línea no es una clave.
 */
export function parseAuthorizedKey(line: string): AuthorizedKey | null {
  const text = line.trim();
  let options = "";
  let rest = text;
  if (!KEY_TYPES.test(text.split(/\s+/)[0] ?? "")) {
    const end = unquoted(text, /\s/);
    if (end < 0) return null;
    options = text.slice(0, end);
    rest = text.slice(end).trim();
  }
  const [type, key, ...comment] = rest.split(/\s+/);
  if (!type || !KEY_TYPES.test(type) || !key || !BASE64.test(key)) return null;
  const mode = (Object.keys(KEY_OPTIONS) as Profile["mode"][]).find((m) => sameOptions(options, KEY_OPTIONS[m])) ?? "custom";
  return { mode, type, key, name: comment.length ? comment.join(" ") : null };
}

/** Si dos listas de opciones son las mismas, en cualquier orden. */
function sameOptions(a: string, b: string): boolean {
  const left = splitOptions(a);
  const right = splitOptions(b);
  return left.length > 0 && left.length === right.length && left.every((option) => right.includes(option));
}

/** Las opciones separadas por las comas que no van entre comillas. */
function splitOptions(options: string): string[] {
  const parts: string[] = [];
  let rest = options;
  while (rest) {
    const comma = unquoted(rest, /,/);
    if (comma < 0) {
      parts.push(rest);
      break;
    }
    parts.push(rest.slice(0, comma));
    rest = rest.slice(comma + 1);
  }
  return parts.filter(Boolean);
}

/** Posición del primer carácter de `text` que cumple `match` fuera de comillas dobles, o -1. */
function unquoted(text: string, match: RegExp): number {
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    if (text[i] === "\\" && quoted) i++;
    else if (text[i] === '"') quoted = !quoted;
    else if (!quoted && match.test(text[i])) return i;
  }
  return -1;
}

/** La línea de `key` con las opciones de `mode`, conservando su nombre. */
export function withMode(key: AuthorizedKey, mode: Profile["mode"]): string {
  return authorizedKeyLine([key.type, key.key, ...(key.name ? [key.name] : [])].join(" "), mode)!;
}

/** Si `key`, de `user`, es la clave con la que se está conectado: el mismo usuario y la misma `publicKey`. */
export function isOwnKey(user: string, key: AuthorizedKey, username: string, publicKey: string | null): boolean {
  return user === username && publicKey?.trim().split(/\s+/)[1] === key.key;
}
