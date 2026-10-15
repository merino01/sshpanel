import { describe, expect, it } from "vitest";
import {
  contrast,
  cssVariables,
  isTheme,
  readableOn,
  resolveTheme,
  themeIds,
  themes,
  xtermTheme,
  type ThemePalette,
} from "./themes";

const BRANDING = "#3b82f6";
const HEX = /^#[0-9a-f]{6}$/i;
const COLOR = /^(#[0-9a-f]{6}|rgba\(\d+, \d+, \d+, [\d.]+\))$/i;
const ANSI = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];

describe("themes", () => {
  it("descubre los ficheros de Tema, claros primero, con Sistema delante", () => {
    expect(themeIds).toEqual(["system", "light", "dark", "high-contrast"]);
  });

  it("cada Tema trae la paleta completa", () => {
    for (const palette of Object.values(themes)) {
      for (const key of ["bg", "fg", "surface", "border", "borderStrong", "subtle", "error", "success", "warning"] as const) {
        expect(palette.colors[key]).toMatch(HEX);
      }
      for (const key of ["shadow", "backdrop"] as const) expect(palette.colors[key]).toMatch(COLOR);
      const { terminal } = palette;
      expect(terminal.background).toMatch(HEX);
      expect(terminal.foreground).toMatch(HEX);
      expect(terminal.selection).toMatch(COLOR);
      for (const name of ANSI) {
        expect(terminal[name as keyof typeof terminal]).toMatch(HEX);
        expect(terminal[`bright${name[0].toUpperCase()}${name.slice(1)}` as keyof typeof terminal]).toMatch(HEX);
      }
    }
  });

  it("Claro y Oscuro se leen bien (AA)", () => {
    for (const id of ["light", "dark"]) {
      const { colors, terminal } = themes[id];
      expect(contrast(colors.fg, colors.bg)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(colors.error, colors.bg)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(terminal.foreground, terminal.background)).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("Alto contraste llega a AAA en todo el texto", () => {
    const { colors, terminal } = themes["high-contrast"];
    for (const text of [colors.fg, colors.accentText!, colors.error, colors.success, colors.warning]) {
      expect(contrast(text, colors.bg)).toBeGreaterThanOrEqual(7);
    }
    const { background, cursor, selection, black, ...text } = terminal;
    for (const color of Object.values(text)) expect(contrast(color, background)).toBeGreaterThanOrEqual(7);
  });
});

describe("isTheme", () => {
  it("vale Sistema y cada Tema descubierto, nada más", () => {
    for (const id of themeIds) expect(isTheme(id)).toBe(true);
    expect(isTheme("purple")).toBe(false);
    expect(isTheme("constructor")).toBe(false);
    expect(isTheme(undefined)).toBe(false);
  });
});

describe("resolveTheme", () => {
  it("Sistema usa Claro u Oscuro según el equipo", () => {
    expect(resolveTheme("system", false)).toBe(themes.light);
    expect(resolveTheme("system", true)).toBe(themes.dark);
  });

  it("un Tema elegido no depende del equipo", () => {
    expect(resolveTheme("light", true)).toBe(themes.light);
    expect(resolveTheme("high-contrast", false)).toBe(themes["high-contrast"]);
  });

  it("un Tema desconocido se trata como Sistema", () => {
    expect(resolveTheme("purple", true)).toBe(themes.dark);
    expect(resolveTheme("toString", false)).toBe(themes.light);
  });
});

describe("cssVariables", () => {
  it("los Temas básicos usan el acento del Branding", () => {
    for (const id of ["light", "dark", "high-contrast"]) {
      expect(cssVariables(themes[id], BRANDING)["--accent"]).toBe(BRANDING);
    }
  });

  it("un Tema con acento propio lo usa en vez del del Branding", () => {
    const own: ThemePalette = { ...themes.dark, accent: "#c4746e" };
    const vars = cssVariables(own, BRANDING);
    expect(vars["--accent"]).toBe("#c4746e");
    expect(vars["--accent-text"]).toBe("#c4746e");
    expect(xtermTheme(own, BRANDING).cursor).toBe("#c4746e");
  });

  it("sin texto sobre acento propio, elige blanco o negro según el acento", () => {
    expect(cssVariables(themes["high-contrast"], "#1e3a8a")["--on-accent"]).toBe("#ffffff");
    expect(cssVariables(themes["high-contrast"], "#fde047")["--on-accent"]).toBe("#000000");
    expect(cssVariables(themes.light, "#fde047")["--on-accent"]).toBe("#ffffff");
  });

  it("Alto contraste pone el texto en acento en el color del texto", () => {
    expect(cssVariables(themes["high-contrast"], BRANDING)["--accent-text"]).toBe("#ffffff");
  });
});

describe("xtermTheme", () => {
  it("lleva los colores de la Terminal del Tema", () => {
    const theme = xtermTheme(themes.dark, BRANDING);
    expect(theme.background).toBe(themes.dark.terminal.background);
    expect(theme.selectionBackground).toBe(themes.dark.terminal.selection);
    expect(theme.brightCyan).toBe(themes.dark.terminal.brightCyan);
    expect(theme.cursor).toBe(BRANDING);
    expect(xtermTheme(themes["high-contrast"], BRANDING).cursor).toBe("#ffffff");
  });
});

describe("contrast", () => {
  it("sigue la fórmula de WCAG", () => {
    expect(contrast("#ffffff", "#000000")).toBeCloseTo(21);
    expect(contrast("#000", "#fff")).toBeCloseTo(21);
    expect(contrast("#777777", "#777777")).toBe(1);
    expect(readableOn("#000000")).toBe("#ffffff");
  });
});
