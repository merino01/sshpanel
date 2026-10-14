import { describe, expect, it } from "vitest";
import { authorizedKeyLine } from "./authorizedKeys";

const KEY = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl";

describe("authorizedKeyLine", () => {
  it("antepone las opciones de cada modo", () => {
    expect(authorizedKeyLine(KEY, "simple")).toBe(`restrict,command="internal-sftp" ${KEY}`);
    expect(authorizedKeyLine(KEY, "advanced")).toBe(`restrict,pty ${KEY}`);
  });

  it("acepta la clave con espacios alrededor y conserva el comentario", () => {
    expect(authorizedKeyLine(`  ${KEY}  alice@portatil \n`, "advanced")).toBe(`restrict,pty ${KEY} alice@portatil`);
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
