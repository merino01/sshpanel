import { describe, expect, it } from "vitest";
import en from "./en.json";
import es from "./es.json";

const LANGUAGES: Record<string, Record<string, string>> = { es, en };

/** Las `{variables}` de un texto, ordenadas y sin repetir. */
function variables(text: string): string[] {
  return [...new Set([...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]))].sort();
}

describe("textos de la interfaz", () => {
  it("el español y el inglés tienen las mismas claves", () => {
    const missingInEn = Object.keys(es).filter((key) => !(key in en));
    const missingInEs = Object.keys(en).filter((key) => !(key in es));
    expect({ missingInEn, missingInEs }).toEqual({ missingInEn: [], missingInEs: [] });
  });

  it("cada texto usa las mismas {variables} en los dos idiomas", () => {
    const different = Object.keys(es)
      .filter((key) => key in en)
      .filter((key) => variables(es[key as keyof typeof es]).join() !== variables(en[key as keyof typeof en]).join());
    expect(different).toEqual([]);
  });

  it("ningún texto está vacío", () => {
    for (const [language, messages] of Object.entries(LANGUAGES)) {
      const empty = Object.entries(messages).filter(([, text]) => text.trim() === "").map(([key]) => key);
      expect({ language, empty }).toEqual({ language, empty: [] });
    }
  });
});
