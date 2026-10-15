import { describe, expect, it } from "vitest";
import { formatDate, formatSize } from "./format";

describe("formatSize", () => {
  it("usa la unidad más grande que quede por encima de 1", () => {
    expect(formatSize(0, "en")).toBe("0 B");
    expect(formatSize(1023, "es")).toBe("1023 B");
    expect(formatSize(1536, "en")).toBe("1.5 KB");
    expect(formatSize(5 * 1024 ** 3, "en")).toBe("5.0 GB");
  });

  it("escribe los decimales como el idioma activo", () => {
    expect(formatSize(1536, "es")).toBe("1,5 KB");
    expect(formatSize(1536, "en")).toBe("1.5 KB");
    expect(formatSize(5 * 1024 ** 3, "es")).toBe("5,0 GB");
  });
});

describe("formatDate", () => {
  // 31 de diciembre de 2024, 18:05 en UTC: el día y el mes no se confunden.
  const lastDay = Date.UTC(2024, 11, 31, 18, 5) / 1000;

  it("en español, día antes que mes y reloj de 24 horas", () => {
    const text = formatDate(lastDay, "es", "UTC");
    expect(text).toContain("31/12/24");
    expect(text).toContain("18:05");
  });

  it("en inglés, mes antes que día y reloj de 12 horas", () => {
    const text = formatDate(lastDay, "en", "UTC");
    expect(text).toContain("12/31/24");
    expect(text).toMatch(/6:05\sPM/);
  });
});
