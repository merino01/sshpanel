import { describe, expect, it } from "vitest";
import { formatSize } from "./format";

describe("formatSize", () => {
  it("usa la unidad más grande que quede por encima de 1", () => {
    expect(formatSize(0)).toBe("0 B");
    expect(formatSize(1023)).toBe("1023 B");
    expect(formatSize(1536)).toBe("1.5 KB");
    expect(formatSize(5 * 1024 ** 3)).toBe("5.0 GB");
  });
});
