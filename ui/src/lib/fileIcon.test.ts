import { describe, expect, it } from "vitest";
import { fileIcon } from "./fileIcon";

describe("fileIcon", () => {
  it("las carpetas y los enlaces van por su tipo, se llamen como se llamen", () => {
    expect(fileIcon({ name: "fotos.png", kind: "dir" })).toBe("folder");
    expect(fileIcon({ name: "web", kind: "symlink" })).toBe("link");
  });

  it("los archivos, por su extensión, sin importar mayúsculas", () => {
    expect(fileIcon({ name: "Foto.JPG", kind: "file" })).toBe("image");
    expect(fileIcon({ name: "index.ts", kind: "file" })).toBe("code");
    expect(fileIcon({ name: "copia.tar.gz", kind: "file" })).toBe("archive");
    expect(fileIcon({ name: "notas.md", kind: "file" })).toBe("text");
  });

  it("sin extensión o con una que no conoce, archivo a secas", () => {
    expect(fileIcon({ name: "Makefile", kind: "file" })).toBe("file");
    expect(fileIcon({ name: ".bashrc", kind: "file" })).toBe("file");
    expect(fileIcon({ name: "datos.xyz", kind: "file" })).toBe("file");
  });
});
