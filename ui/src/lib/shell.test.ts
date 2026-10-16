import { describe, expect, it } from "vitest";
import { shellQuote } from "./shell";

describe("shellQuote", () => {
  it("deja una ruta normal entre comillas simples", () => {
    expect(shellQuote("/home/alice/notas.txt")).toBe("'/home/alice/notas.txt'");
  });

  it("los espacios y lo que la consola interpretaría quedan literales", () => {
    expect(shellQuote("/srv/acta final $(rm -rf ~) *;`x`!.txt")).toBe("'/srv/acta final $(rm -rf ~) *;`x`!.txt'");
  });

  it("cierra y reabre las comillas en cada comilla simple", () => {
    expect(shellQuote("/srv/acta 'final'.txt")).toBe(`'/srv/acta '\\''final'\\''.txt'`);
  });
});
