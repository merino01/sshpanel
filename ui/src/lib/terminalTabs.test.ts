import { describe, expect, it } from "vitest";
import { activeAfterClose, cycle, nextNumber, shortcutFor } from "./terminalTabs";

const tabs = [
  { id: 4, number: 1 },
  { id: 7, number: 3 },
  { id: 9, number: 2 },
];

describe("nextNumber", () => {
  it("usa el número más bajo libre", () => {
    expect(nextNumber([])).toBe(1);
    expect(nextNumber(tabs)).toBe(4);
    expect(nextNumber([{ id: 1, number: 2 }])).toBe(1);
  });
});

describe("activeAfterClose", () => {
  it("pasa a la de la derecha o, si era la última, a la de la izquierda", () => {
    expect(activeAfterClose(tabs, 4, 4)).toBe(7);
    expect(activeAfterClose(tabs, 9, 9)).toBe(7);
    expect(activeAfterClose([tabs[0]], 4, 4)).toBeNull();
  });

  it("no cambia si se cierra otra", () => {
    expect(activeAfterClose(tabs, 7, 4)).toBe(4);
  });
});

describe("cycle", () => {
  it("avanza y retrocede dando la vuelta", () => {
    expect(cycle(tabs, 4, 1)).toBe(7);
    expect(cycle(tabs, 9, 1)).toBe(4);
    expect(cycle(tabs, 4, -1)).toBe(9);
    expect(cycle([], null, 1)).toBeNull();
  });
});

describe("shortcutFor", () => {
  const key = (key: string, mods: Partial<KeyboardEvent> = {}) =>
    shortcutFor({ key, ctrlKey: true, shiftKey: false, altKey: false, metaKey: false, ...mods });

  it("reconoce los atajos de las pestañas", () => {
    expect(key("T", { shiftKey: true })).toBe("new");
    expect(key("w", { shiftKey: true })).toBe("close");
    expect(key("PageDown")).toBe("next");
    expect(key("PageUp")).toBe("previous");
  });

  it("deja pasar a la Terminal lo demás, como Ctrl+C o Ctrl+W", () => {
    expect(key("c")).toBeNull();
    expect(key("w")).toBeNull();
    expect(key("t")).toBeNull();
    expect(key("T", { shiftKey: true, altKey: true })).toBeNull();
    expect(shortcutFor({ key: "T", ctrlKey: false, shiftKey: true, altKey: false, metaKey: false })).toBeNull();
  });
});
