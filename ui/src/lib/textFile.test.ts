import { describe, expect, it } from "vitest";
import { fileTitle, isLarge, isText, LARGE_FILE, languageHint, lineSeparator } from "./textFile";

const bytes = (text: string) => new TextEncoder().encode(text);

describe("isText", () => {
  it("es texto lo que no tiene bytes nulos, en cualquier codificación", () => {
    expect(isText(bytes("server {\n  listen 80;\n}\n"))).toBe(true);
    expect(isText(bytes("mañana, acción, 日本語, 🙂"))).toBe(true);
    expect(isText(new Uint8Array([0x6d, 0x61, 0xf1, 0x61]))).toBe(true);
  });

  it("un archivo vacío es texto", () => {
    expect(isText(new Uint8Array())).toBe(true);
  });

  it("un byte nulo en cualquier sitio del primer trozo lo hace binario", () => {
    expect(isText(new Uint8Array([0, 1, 2]))).toBe(false);
    expect(isText(new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0x0d]))).toBe(false);
    expect(isText(new Uint8Array([...bytes("hola"), 0]))).toBe(false);
  });

  it("el texto en UTF-16 lleva bytes nulos y no cuenta como texto", () => {
    expect(isText(new Uint8Array([0xff, 0xfe, 0x68, 0, 0x6f, 0]))).toBe(false);
  });
});

describe("isLarge", () => {
  it("avisa solo por encima de 5 MiB", () => {
    expect(LARGE_FILE).toBe(5 * 1024 * 1024);
    expect(isLarge(0)).toBe(false);
    expect(isLarge(LARGE_FILE)).toBe(false);
    expect(isLarge(LARGE_FILE + 1)).toBe(true);
  });
});

describe("lineSeparator", () => {
  it("sigue a la primera línea", () => {
    expect(lineSeparator("uno\ndos\n")).toBe("\n");
    expect(lineSeparator("uno\r\ndos\r\n")).toBe("\r\n");
    expect(lineSeparator("uno\r\ndos\ntres")).toBe("\r\n");
  });

  it("sin saltos de línea, LF", () => {
    expect(lineSeparator("")).toBe("\n");
    expect(lineSeparator("una sola línea")).toBe("\n");
    expect(lineSeparator("\n")).toBe("\n");
  });
});

describe("languageHint", () => {
  it("reconoce los ficheros de configuración por su nombre", () => {
    expect(languageHint(".bashrc")).toBe("shell");
    expect(languageHint(".profile")).toBe("shell");
    expect(languageHint(".env")).toBe("properties");
    expect(languageHint(".gitconfig")).toBe("properties");
  });

  it("y las extensiones de configuración", () => {
    expect(languageHint("my.cnf")).toBe("properties");
    expect(languageHint("App.CONF")).toBe("properties");
    expect(languageHint("produccion.env")).toBe("properties");
  });

  it("deja el resto al Editor", () => {
    expect(languageHint("script.py")).toBeNull();
    expect(languageHint("Dockerfile")).toBeNull();
    expect(languageHint(".conf.bak")).toBeNull();
  });
});

describe("fileTitle", () => {
  it("da el nombre y la carpeta, con «•» si hay cambios sin guardar", () => {
    expect(fileTitle("/home/alice/notas.txt")).toBe("notas.txt — /home/alice");
    expect(fileTitle("/etc/hosts", true)).toBe("• hosts — /etc");
    expect(fileTitle("/raiz.txt")).toBe("raiz.txt — /");
  });
});
