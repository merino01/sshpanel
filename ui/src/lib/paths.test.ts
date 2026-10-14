import { describe, expect, it } from "vitest";
import { baseName, breadcrumbs, isWithin, joinPath, parentOf, placeOf } from "./paths";

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

describe("breadcrumbs", () => {
  it("da cada carpeta desde la raíz con su ruta", () => {
    expect(breadcrumbs("/home/alice/fotos")).toEqual([
      { name: "/", path: "/" },
      { name: "home", path: "/home" },
      { name: "alice", path: "/home/alice" },
      { name: "fotos", path: "/home/alice/fotos" },
    ]);
    expect(breadcrumbs("/srv/compartida/")).toEqual([
      { name: "/", path: "/" },
      { name: "srv", path: "/srv" },
      { name: "compartida", path: "/srv/compartida" },
    ]);
    expect(breadcrumbs("/")).toEqual([{ name: "/", path: "/" }]);
  });

  it("empieza en `base` con su nombre si la contiene", () => {
    const home = { name: "Mi carpeta", path: "/home/alice" };
    expect(breadcrumbs("/home/alice/fotos/2026", home)).toEqual([
      home,
      { name: "fotos", path: "/home/alice/fotos" },
      { name: "2026", path: "/home/alice/fotos/2026" },
    ]);
    expect(breadcrumbs("/home/alice", home)).toEqual([home]);
    expect(breadcrumbs("/srv", home)).toEqual([
      { name: "/", path: "/" },
      { name: "srv", path: "/srv" },
    ]);
  });
});

describe("isWithin y placeOf", () => {
  it("sabe qué está dentro de una carpeta, sin dejarse engañar por `..`", () => {
    expect(isWithin("/home/alice", "/home/alice")).toBe(true);
    expect(isWithin("/home/alice/fotos", "/home/alice/")).toBe(true);
    expect(isWithin("/home/alicia", "/home/alice")).toBe(false);
    expect(isWithin("/home", "/home/alice")).toBe(false);
    expect(isWithin("/home/alice/../bob", "/home/alice")).toBe(false);
  });

  it("elige la carpeta más interna", () => {
    const places = [
      { name: "Mi carpeta", path: "/home/alice" },
      { name: "Carpeta compartida", path: "/home/alice/equipo" },
    ];
    expect(placeOf("/home/alice/equipo/actas", places)?.name).toBe("Carpeta compartida");
    expect(placeOf("/home/alice/fotos", places)?.name).toBe("Mi carpeta");
    expect(placeOf("/srv", places)).toBeUndefined();
  });
});

describe("baseName", () => {
  it("entiende rutas de Linux, macOS y Windows", () => {
    expect(baseName("/home/alice/foto.jpg")).toBe("foto.jpg");
    expect(baseName("C:\\Users\\alice\\Fotos\\")).toBe("Fotos");
  });
});
