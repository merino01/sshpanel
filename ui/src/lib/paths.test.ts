import { describe, expect, it } from "vitest";
import { baseName, joinPath, parentOf } from "./paths";

describe("parentOf", () => {
  it("sube una carpeta, hasta la raíz", () => {
    expect(parentOf("/home/alice/fotos")).toBe("/home/alice");
    expect(parentOf("/home/alice/")).toBe("/home");
    expect(parentOf("/home")).toBe("/");
    expect(parentOf("/")).toBeNull();
  });
});

describe("joinPath", () => {
  it("une sin duplicar la barra", () => {
    expect(joinPath("/home/alice", "notas.txt")).toBe("/home/alice/notas.txt");
    expect(joinPath("/", "srv")).toBe("/srv");
  });
});

describe("baseName", () => {
  it("entiende rutas de Linux, macOS y Windows", () => {
    expect(baseName("/home/alice/foto.jpg")).toBe("foto.jpg");
    expect(baseName("C:\\Users\\alice\\Fotos\\")).toBe("Fotos");
  });
});
