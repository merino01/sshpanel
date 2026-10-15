import { describe, expect, it } from "vitest";
import { isLanguagePreference, resolveLanguage } from "./language";

describe("resolveLanguage", () => {
  it("con Sistema, usa el español si el equipo está en cualquier variante de español", () => {
    expect(resolveLanguage("system", "es")).toBe("es");
    expect(resolveLanguage("system", "es-ES")).toBe("es");
    expect(resolveLanguage("system", "es-419")).toBe("es");
    expect(resolveLanguage("system", "ES-mx")).toBe("es");
  });

  it("con Sistema, usa el inglés si el equipo está en inglés", () => {
    expect(resolveLanguage("system", "en")).toBe("en");
    expect(resolveLanguage("system", "en-GB")).toBe("en");
  });

  it("con Sistema, usa el inglés si el equipo no está ni en español ni en inglés", () => {
    expect(resolveLanguage("system", "fr-FR")).toBe("en");
    expect(resolveLanguage("system", "")).toBe("en");
    expect(resolveLanguage("system", undefined)).toBe("en");
  });

  it("un idioma elegido gana al del equipo", () => {
    expect(resolveLanguage("es", "en-US")).toBe("es");
    expect(resolveLanguage("en", "es-ES")).toBe("en");
  });
});

describe("isLanguagePreference", () => {
  it("vale Sistema y cada idioma de la app, nada más", () => {
    for (const value of ["system", "es", "en"]) expect(isLanguagePreference(value)).toBe(true);
    for (const value of ["fr", "es-ES", "", "toString", 3, null, undefined]) expect(isLanguagePreference(value)).toBe(false);
  });
});
