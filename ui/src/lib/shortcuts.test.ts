import { describe, expect, it } from "vitest";
import {
  ACTIONS,
  actionFor,
  allowed,
  comboOf,
  conflictOf,
  defaultShortcuts,
  isReload,
  resolveShortcuts,
  shortcutLabel,
  validOverrides,
  withShortcut,
  type ShortcutContext,
} from "./shortcuts";

/** Una pulsación; `code` es la posición de la tecla, por defecto la de su letra. */
const key = (key: string, mods: Partial<KeyboardEvent> = {}) => ({
  key,
  code: /^[a-z]$/i.test(key) ? `Key${key.toUpperCase()}` : key,
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
    expect(comboOf(key(" ", { code: "Space", ctrlKey: true }), false)).toBe("Mod+Space");
  });

  it("los signos van por su posición, para que Shift o la distribución no los cambien", () => {
    // En un teclado español, Shift+, da «;» y la tecla ` es una tecla muerta.
    expect(comboOf(key(";", { code: "Comma", ctrlKey: true, shiftKey: true }), false)).toBe("Mod+Shift+,");
    expect(comboOf(key("Dead", { code: "BracketLeft", ctrlKey: true }), false)).toBe("Mod+[");
    expect(comboOf(key("º", { code: "Backquote", ctrlKey: true }), false)).toBe("Mod+`");
    expect(comboOf(key("!", { code: "Digit1", shiftKey: true }), false)).toBe("Shift+1");
  });

  it("las letras siguen la distribución, también las que no son del inglés", () => {
    expect(comboOf(key("z", { code: "KeyW", ctrlKey: true }), false)).toBe("Mod+Z");
    expect(comboOf(key("ñ", { code: "Semicolon", ctrlKey: true }), false)).toBe("Mod+Ñ");
  });

  it("con Alt en macOS, la letra de la tecla y no el carácter que sale", () => {
    expect(comboOf(key("å", { code: "KeyA", altKey: true }), true)).toBe("Alt+A");
  });

  it("no hay combinación si solo se pulsa un modificador", () => {
    expect(comboOf(key("Control", { ctrlKey: true }), false)).toBeNull();
    expect(comboOf(key("Shift", { shiftKey: true }), false)).toBeNull();
  });
});

describe("Atajos por defecto", () => {
  it("son los de la tabla de la spec, con Ctrl o Cmd según el sistema", () => {
    for (const mac of [false, true]) {
      const shortcuts = defaultShortcuts(mac);
      const press = (k: string, mods: Partial<KeyboardEvent> = {}) => actionFor(key(k, mods), shortcuts, mac);
      const mod = mac ? { metaKey: true } : { ctrlKey: true };
      expect(press("N", { ...mod, shiftKey: true })).toBe("newFolder");
      expect(press("u", mod)).toBe("upload");
      expect(press("F2")).toBe("rename");
      expect(press("Enter")).toBe("open");
      expect(press("ArrowLeft", { altKey: true })).toBe("back");
      expect(press("Backspace")).toBe("back");
      expect(press("F5")).toBe("reload");
      expect(press("C", { ...mod, shiftKey: true })).toBe("copyPath");
      expect(press(",", { ...mod, code: "Comma" })).toBe("settings");
      expect(press("`", { ...mod, code: "Backquote" })).toBe("terminal");
    }
  });

  it("Mover a la Papelera es Supr, y Cmd+Retroceso en macOS", () => {
    expect(actionFor(key("Delete"), defaultShortcuts(false), false)).toBe("trash");
    expect(actionFor(key("Backspace", { metaKey: true }), defaultShortcuts(true), true)).toBe("trash");
    expect(actionFor(key("Delete"), defaultShortcuts(true), true)).toBeNull();
  });

  it("Ctrl en macOS no lanza los Atajos de Cmd", () => {
    expect(actionFor(key(",", { code: "Comma", ctrlKey: true }), defaultShortcuts(true), true)).toBeNull();
  });

  it("ninguna combinación se repite ni choca con las pestañas de la Terminal", () => {
    for (const mac of [false, true]) {
      const shortcuts = defaultShortcuts(mac);
      for (const action of ACTIONS) {
        for (const combo of shortcuts[action]) expect(conflictOf(shortcuts, action, combo, mac)).toBeNull();
      }
    }
  });
});

describe("actionFor", () => {
  it("usa los atajos que se le pasan", () => {
    const shortcuts = resolveShortcuts({ settings: ["F9"] }, false);
    expect(actionFor(key("F9"), shortcuts, false)).toBe("settings");
    expect(actionFor(key(",", { code: "Comma", ctrlKey: true }), shortcuts, false)).toBeNull();
  });

  it("no lanza nada con otras teclas o modificadores", () => {
    const shortcuts = defaultShortcuts(false);
    expect(actionFor(key(",", { code: "Comma" }), shortcuts, false)).toBeNull();
    expect(actionFor(key(",", { code: "Comma", ctrlKey: true, altKey: true }), shortcuts, false)).toBeNull();
  });
});

describe("conflictOf", () => {
  const shortcuts = defaultShortcuts(false);

  it("dice qué acción ya usa la combinación", () => {
    expect(conflictOf(shortcuts, "rename", "F5", false)).toBe("reload");
    expect(conflictOf(shortcuts, "rename", "Backspace", false)).toBe("back");
  });

  it("la propia acción no cuenta", () => {
    expect(conflictOf(shortcuts, "reload", "F5", false)).toBeNull();
    expect(conflictOf(shortcuts, "rename", "F3", false)).toBeNull();
  });

  it("las pestañas de la Terminal usan Ctrl también en macOS", () => {
    expect(conflictOf(shortcuts, "rename", "Mod+Shift+T", false)).toBe("terminalTabs");
    expect(conflictOf(defaultShortcuts(true), "rename", "Ctrl+PageDown", true)).toBe("terminalTabs");
    expect(conflictOf(defaultShortcuts(true), "rename", "Mod+Shift+T", true)).toBeNull();
  });
});

describe("cambios del Miembro", () => {
  it("se ponen encima de los Atajos por defecto", () => {
    const shortcuts = resolveShortcuts({ back: ["Alt+B"] }, false);
    expect(shortcuts.back).toEqual(["Alt+B"]);
    expect(shortcuts.reload).toEqual(["F5"]);
  });

  it("solo se guarda lo que difiere de los por defecto", () => {
    expect(withShortcut({}, "reload", ["F6"], false)).toEqual({ reload: ["F6"] });
    expect(withShortcut({ reload: ["F6"] }, "reload", ["F5"], false)).toEqual({});
    expect(withShortcut({ reload: ["F6"] }, "back", ["Alt+B"], false)).toEqual({ reload: ["F6"], back: ["Alt+B"] });
  });

  it("lo leído del fichero se queda solo con acciones que existen y listas de combinaciones", () => {
    expect(validOverrides(undefined)).toEqual({});
    expect(validOverrides("F5")).toEqual({});
    expect(validOverrides(["F5"])).toEqual({});
    expect(validOverrides({ reload: ["F6"], fly: ["F7"], back: "F8", open: [], rename: [3], upload: [""] })).toEqual({ reload: ["F6"] });
  });
});

describe("shortcutLabel", () => {
  const names = { Shift: "Mayús", Delete: "Supr", Enter: "Intro", Backspace: "Retroceso" };

  it("fuera de macOS, Ctrl y los nombres del idioma", () => {
    expect(shortcutLabel("Mod+Shift+N", names, false)).toBe("Ctrl+Mayús+N");
    expect(shortcutLabel("Delete", names, false)).toBe("Supr");
    expect(shortcutLabel("Alt+ArrowLeft", names, false)).toBe("Alt+←");
    expect(shortcutLabel("Mod+,", names, false)).toBe("Ctrl+,");
    expect(shortcutLabel("F5", names, false)).toBe("F5");
  });

  it("en macOS, los símbolos del sistema en su orden y sin «+»", () => {
    expect(shortcutLabel("Mod+Shift+N", names, true)).toBe("⇧⌘N");
    expect(shortcutLabel("Mod+Backspace", names, true)).toBe("⌘⌫");
    expect(shortcutLabel("Ctrl+Alt+F2", names, true)).toBe("⌃⌥F2");
    expect(shortcutLabel("Mod+,", names, true)).toBe("⌘,");
  });

  it("la tecla «+» no se confunde con el separador", () => {
    expect(shortcutLabel("Mod++", names, false)).toBe("Ctrl++");
    expect(shortcutLabel("+", names, false)).toBe("+");
  });
});

describe("allowed", () => {
  const explorer: ShortcutContext = {
    inTerminal: false,
    inText: false,
    overlay: false,
    explorer: true,
    selected: true,
    workspace: true,
    advanced: true,
  };

  it("en el Explorador, con una fila con el foco, todo vale", () => {
    for (const action of ACTIONS) expect(allowed(action, explorer)).toBe(true);
  });

  it("con el foco en la Terminal no se aplica ningún Atajo", () => {
    for (const action of ACTIONS) expect(allowed(action, { ...explorer, inTerminal: true })).toBe(false);
  });

  it("con un menú o una confirmación abiertos, tampoco", () => {
    for (const action of ACTIONS) expect(allowed(action, { ...explorer, overlay: true })).toBe(false);
  });

  it("en un campo de texto, solo Ajustes y Terminal", () => {
    const inText = { ...explorer, inText: true };
    expect(ACTIONS.filter((action) => allowed(action, inText))).toEqual(["settings", "terminal"]);
  });

  it("sin fila con el foco, nada de lo que actúa sobre una fila", () => {
    const none = { ...explorer, selected: false };
    expect(allowed("rename", none)).toBe(false);
    expect(allowed("trash", none)).toBe(false);
    expect(allowed("open", none)).toBe(false);
    expect(allowed("back", none)).toBe(true);
    expect(allowed("copyPath", none)).toBe(true);
  });

  it("fuera del Explorador, solo Ajustes y Terminal", () => {
    const trash = { ...explorer, explorer: false, selected: false };
    expect(ACTIONS.filter((action) => allowed(action, trash))).toEqual(["settings", "terminal"]);
  });

  it("la Terminal, solo en Modo avanzado y conectado", () => {
    expect(allowed("terminal", { ...explorer, advanced: false })).toBe(false);
    expect(allowed("terminal", { ...explorer, explorer: false, workspace: false })).toBe(false);
    expect(allowed("settings", { ...explorer, explorer: false, workspace: false })).toBe(true);
  });
});

describe("isReload", () => {
  it("F5 y Ctrl/Cmd+R recargarían la webview", () => {
    expect(isReload("F5")).toBe(true);
    expect(isReload("Mod+R")).toBe(true);
    expect(isReload("Mod+Shift+R")).toBe(true);
    expect(isReload("R")).toBe(false);
    expect(isReload(null)).toBe(false);
  });
});
