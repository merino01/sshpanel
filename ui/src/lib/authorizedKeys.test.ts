import { describe, expect, it } from "vitest";
import { authorizedKeyLine, isOwnKey, parseAuthorizedKey, withMode } from "./authorizedKeys";

const KEY = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEjEmPlOdEcLaVeNoReAlXxXxXxXxXxXxXxXxXxXxXxX";

describe("authorizedKeyLine", () => {
  it("antepone las opciones de cada modo", () => {
    expect(authorizedKeyLine(KEY, "simple")).toBe(`restrict,command="internal-sftp" ${KEY}`);
    expect(authorizedKeyLine(KEY, "advanced")).toBe(`restrict,pty ${KEY}`);
  });

  it("acepta la clave con espacios alrededor y conserva el comentario", () => {
    expect(authorizedKeyLine(`  ${KEY}  alice@portatil \n`, "advanced")).toBe(`restrict,pty ${KEY} alice@portatil`);
  });

  it("pone de quién es en lugar del comentario de la clave, sin espacios", () => {
    expect(authorizedKeyLine(`${KEY} sshpanel`, "simple", "alice-portatil")).toBe(`restrict,command="internal-sftp" ${KEY} alice-portatil`);
    expect(authorizedKeyLine(KEY, "advanced", "  Bob   de casa ")).toBe(`restrict,pty ${KEY} Bob-de-casa`);
    expect(authorizedKeyLine(`${KEY} sshpanel`, "advanced", "   ")).toBe(`restrict,pty ${KEY} sshpanel`);
  });

  it("rechaza lo que no es una única clave pública", () => {
    expect(authorizedKeyLine("", "simple")).toBeNull();
    expect(authorizedKeyLine("hola, esta es mi clave", "simple")).toBeNull();
    expect(authorizedKeyLine("ssh-ed25519", "simple")).toBeNull();
    expect(authorizedKeyLine("ssh-ed25519 no*es*base64", "simple")).toBeNull();
    expect(authorizedKeyLine(`${KEY}\n${KEY}`, "simple")).toBeNull();
    expect(authorizedKeyLine("-----BEGIN OPENSSH PRIVATE KEY-----", "simple")).toBeNull();
  });

  it("no deja colar otras opciones", () => {
    expect(authorizedKeyLine(`no-pty ${KEY}`, "advanced")).toBeNull();
    expect(authorizedKeyLine(`command="/bin/sh" ${KEY}`, "simple")).toBeNull();
  });
});

describe("parseAuthorizedKey", () => {
  const DATA = KEY.split(" ")[1];

  it("deduce el modo de las opciones, en cualquier orden", () => {
    expect(parseAuthorizedKey(`restrict,command="internal-sftp" ${KEY} bob-casa`)).toEqual({ mode: "simple", type: "ssh-ed25519", key: DATA, name: "bob-casa" });
    expect(parseAuthorizedKey(`command="internal-sftp",restrict ${KEY}`)?.mode).toBe("simple");
    expect(parseAuthorizedKey(`restrict,pty ${KEY}`)?.mode).toBe("advanced");
    expect(parseAuthorizedKey(`pty,restrict ${KEY}`)?.mode).toBe("advanced");
  });

  it("sin opciones, o con otras, es Personalizada", () => {
    expect(parseAuthorizedKey(KEY)?.mode).toBe("custom");
    expect(parseAuthorizedKey(`restrict ${KEY}`)?.mode).toBe("custom");
    expect(parseAuthorizedKey(`restrict,pty,port-forwarding ${KEY}`)?.mode).toBe("custom");
    expect(parseAuthorizedKey(`from="10.0.0.1",restrict,pty ${KEY}`)?.mode).toBe("custom");
  });

  it("las comillas pueden llevar espacios y comas", () => {
    const key = parseAuthorizedKey(`command="echo hola, mundo",restrict ${KEY} de alguien`);
    expect(key).toEqual({ mode: "custom", type: "ssh-ed25519", key: DATA, name: "de alguien" });
  });

  it("el nombre es el comentario, con sus espacios; sin él, null", () => {
    expect(parseAuthorizedKey(`restrict,pty ${KEY} Bob de casa`)?.name).toBe("Bob de casa");
    expect(parseAuthorizedKey(`restrict,pty ${KEY}`)?.name).toBeNull();
    expect(parseAuthorizedKey(`  ${KEY}  `)?.name).toBeNull();
  });

  it("lo que no es una clave da null", () => {
    expect(parseAuthorizedKey("")).toBeNull();
    expect(parseAuthorizedKey("hola mundo")).toBeNull();
    expect(parseAuthorizedKey("restrict,pty")).toBeNull();
    expect(parseAuthorizedKey('command="sin cerrar ssh-ed25519 AAAA')).toBeNull();
    expect(parseAuthorizedKey("restrict,pty ssh-ed25519 no*base64")).toBeNull();
  });
});

describe("withMode", () => {
  it("cambia las opciones y conserva el nombre, también con espacios", () => {
    const key = parseAuthorizedKey(`restrict,command="internal-sftp" ${KEY} Bob de casa`)!;
    expect(withMode(key, "advanced")).toBe(`restrict,pty ${KEY} Bob de casa`);
    expect(parseAuthorizedKey(withMode(key, "advanced"))).toEqual({ ...key, mode: "advanced" });
  });

  it("sin nombre, queda sin nombre", () => {
    expect(withMode(parseAuthorizedKey(`restrict,pty ${KEY}`)!, "simple")).toBe(`restrict,command="internal-sftp" ${KEY}`);
  });
});

describe("isOwnKey", () => {
  const key = parseAuthorizedKey(`restrict,pty ${KEY} alice-portatil`)!;

  it("es la propia si coinciden el usuario y la clave, sin mirar el comentario", () => {
    expect(isOwnKey("alice", key, "alice", `${KEY} sshpanel`)).toBe(true);
  });

  it("no lo es con otro usuario, otra clave o sin clave pública", () => {
    expect(isOwnKey("bob", key, "alice", `${KEY} sshpanel`)).toBe(false);
    expect(isOwnKey("alice", key, "alice", "ssh-ed25519 AAAAotra sshpanel")).toBe(false);
    expect(isOwnKey("alice", key, "alice", null)).toBe(false);
  });
});
