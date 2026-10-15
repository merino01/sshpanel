import { describe, expect, it } from "vitest";
import { DEFAULT_PREFERENCES, withDefaults } from "./preferences";
import { themeIds } from "./themes";

describe("withDefaults", () => {
  it("sin fichero o con uno que no es un objeto, usa los valores por defecto", () => {
    expect(withDefaults(null)).toEqual(DEFAULT_PREFERENCES);
    expect(withDefaults("dark")).toEqual(DEFAULT_PREFERENCES);
    expect(withDefaults([])).toEqual(DEFAULT_PREFERENCES);
  });

  it("completa las preferencias que faltan", () => {
    expect(withDefaults({})).toEqual(DEFAULT_PREFERENCES);
    expect(withDefaults({ other: 1 })).toEqual(DEFAULT_PREFERENCES);
  });

  it("cambia por el valor por defecto lo que no vale", () => {
    expect(withDefaults({ theme: "purple" }).theme).toBe("system");
    expect(withDefaults({ theme: 3 }).theme).toBe("system");
    expect(withDefaults({ theme: "toString" }).theme).toBe("system");
  });

  it("mantiene lo guardado", () => {
    expect(withDefaults({ theme: "system" }).theme).toBe("system");
    expect(withDefaults({ theme: "dark" }).theme).toBe("dark");
  });

  it("acepta cualquier Tema descubierto", () => {
    for (const id of themeIds) expect(withDefaults({ theme: id }).theme).toBe(id);
  });

  it("se queda con los Atajos cambiados que valen", () => {
    expect(withDefaults({ shortcuts: { reload: ["F6"], fly: ["F7"] } }).shortcuts).toEqual({ reload: ["F6"] });
    expect(withDefaults({ shortcuts: "F6" }).shortcuts).toEqual({});
  });
});
