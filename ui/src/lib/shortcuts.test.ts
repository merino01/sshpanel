import { describe, expect, it } from "vitest";
import { actionFor, comboOf } from "./shortcuts";

const key = (key: string, mods: Partial<KeyboardEvent> = {}) => ({
  key,
  ctrlKey: false,
  shiftKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

describe("comboOf", () => {
  it("escribe Ctrl como Mod fuera de macOS y Cmd como Mod en macOS", () => {
    expect(comboOf(key("n", { ctrlKey: true, shiftKey: true }), false)).toBe("Mod+Shift+N");
    expect(comboOf(key("n", { metaKey: true, shiftKey: true }), true)).toBe("Mod+Shift+N");
    expect(comboOf(key("n", { ctrlKey: true }), true)).toBe("Ctrl+N");
    expect(comboOf(key("n", { metaKey: true }), false)).toBe("Meta+N");
  });

  it("deja las teclas con nombre como están", () => {
    expect(comboOf(key("F2"), false)).toBe("F2");
    expect(comboOf(key("ArrowLeft", { altKey: true }), false)).toBe("Alt+ArrowLeft");
  });

  it("no hay combinación si solo se pulsa un modificador", () => {
    expect(comboOf(key("Control", { ctrlKey: true }), false)).toBeNull();
    expect(comboOf(key("Shift", { shiftKey: true }), false)).toBeNull();
  });
});

describe("actionFor", () => {
  it("Ajustes es Ctrl+, fuera de macOS y Cmd+, en macOS", () => {
    expect(actionFor(key(",", { ctrlKey: true }), undefined, false)).toBe("settings");
    expect(actionFor(key(",", { metaKey: true }), undefined, true)).toBe("settings");
    expect(actionFor(key(",", { ctrlKey: true }), undefined, true)).toBeNull();
  });

  it("no lanza nada con otras teclas o modificadores", () => {
    expect(actionFor(key(","), undefined, false)).toBeNull();
    expect(actionFor(key(".", { ctrlKey: true }), undefined, false)).toBeNull();
    expect(actionFor(key(",", { ctrlKey: true, altKey: true }), undefined, false)).toBeNull();
  });

  it("usa los atajos que se le pasan", () => {
    expect(actionFor(key("F9"), { settings: "F9" }, false)).toBe("settings");
  });
});
