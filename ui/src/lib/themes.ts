/**
 * Temas: cada fichero de `themes/` es una paleta completa y su nombre es el id
 * del Tema. Se descubren al compilar, así que añadir un Tema es crear un fichero.
 * Sistema no tiene fichero: usa Claro u Oscuro según el modo del equipo.
 */
import type { ITheme } from "@xterm/xterm";

/** Los colores de la Terminal: fondo, texto, cursor, selección y los 16 ANSI. */
export interface TerminalPalette {
  background: string;
  foreground: string;
  /** Si falta, el acento. */
  cursor?: string;
  selection: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  brightBlack: string;
  brightRed: string;
  brightGreen: string;
  brightYellow: string;
  brightBlue: string;
  brightMagenta: string;
  brightCyan: string;
  brightWhite: string;
}

export interface ThemePalette {
  /** Claro u oscuro: los controles nativos y el orden en Ajustes. */
  scheme: "light" | "dark";
  /** Nombre propio (Kanagawa Dragon…); los Temas básicos lo traen de i18n (`settings.theme.<id>`). */
  name?: string;
  /** Acento propio; si falta, el del Branding. */
  accent?: string;
  colors: {
    bg: string;
    fg: string;
    /** Barras (superior y lateral), un poco apartadas del fondo. */
    surface: string;
    /** Separadores y bordes de paneles. */
    border: string;
    /** Bordes de campos y botones. */
    borderStrong: string;
    /** Fondo de bloques de texto como una clave o una huella. */
    subtle: string;
    shadow: string;
    /** Lo que queda detrás de un diálogo. */
    backdrop: string;
    /** Texto en color de acento (enlaces, títulos); si falta, el acento. */
    accentText?: string;
    /** Texto sobre el acento; si falta, blanco o negro, el que más contraste. */
    onAccent?: string;
    error: string;
    success: string;
    warning: string;
  };
  terminal: TerminalPalette;
}

/** El Tema por defecto: sigue el modo claro u oscuro del equipo. */
export const SYSTEM_THEME = "system";
const SYSTEM_LIGHT = "light";
const SYSTEM_DARK = "dark";

const files = import.meta.glob<ThemePalette>("../themes/*.ts", { eager: true, import: "default" });

/** Los Temas por id: primero los claros, luego los oscuros, y por id. */
export const themes: Record<string, ThemePalette> = Object.fromEntries(
  Object.entries(files)
    .map(([path, palette]) => [path.slice(path.lastIndexOf("/") + 1, -".ts".length), palette] as const)
    .sort(([a, pa], [b, pb]) => Number(pa.scheme === "dark") - Number(pb.scheme === "dark") || a.localeCompare(b)),
);

/** Lo que se puede elegir en Ajustes: Sistema y cada Tema. */
export const themeIds = [SYSTEM_THEME, ...Object.keys(themes)];

export function isTheme(id: unknown): id is string {
  return typeof id === "string" && themeIds.includes(id);
}

/** La paleta que toca para `id`; Sistema, o un Tema que ya no existe, da Claro u Oscuro según el equipo. */
export function resolveTheme(id: string, prefersDark: boolean): ThemePalette {
  if (id !== SYSTEM_THEME && Object.hasOwn(themes, id)) return themes[id];
  return themes[prefersDark ? SYSTEM_DARK : SYSTEM_LIGHT];
}

/** El acento del Tema, o el del Branding si no trae uno. */
export function accentOf(palette: ThemePalette, brandingAccent: string): string {
  return palette.accent ?? brandingAccent;
}

/** Las variables CSS del Tema para `:root`. */
export function cssVariables(palette: ThemePalette, brandingAccent: string): Record<string, string> {
  const { colors } = palette;
  const accent = accentOf(palette, brandingAccent);
  return {
    "--bg": colors.bg,
    "--fg": colors.fg,
    "--surface": colors.surface,
    "--border": colors.border,
    "--border-strong": colors.borderStrong,
    "--subtle": colors.subtle,
    "--shadow": colors.shadow,
    "--backdrop": colors.backdrop,
    "--accent": accent,
    "--accent-text": colors.accentText ?? accent,
    "--on-accent": colors.onAccent ?? readableOn(accent),
    "--error": colors.error,
    "--success": colors.success,
    "--warning": colors.warning,
    "--terminal-bg": palette.terminal.background,
  };
}

/** Los colores de xterm.js para el Tema. */
export function xtermTheme(palette: ThemePalette, brandingAccent: string): ITheme {
  const { background, foreground, cursor, selection, ...ansi } = palette.terminal;
  return {
    ...ansi,
    background,
    foreground,
    cursor: cursor ?? accentOf(palette, brandingAccent),
    cursorAccent: background,
    selectionBackground: selection,
  };
}

/** Blanco o negro, el que más contraste con `color` (`#rrggbb`). */
export function readableOn(color: string): string {
  return contrast(color, "#ffffff") >= contrast(color, "#000000") ? "#ffffff" : "#000000";
}

/** Contraste WCAG entre dos colores `#rgb` o `#rrggbb` (de 1 a 21). */
export function contrast(a: string, b: string): number {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
}

function luminance(hex: string): number {
  const digits = hex.slice(1);
  const full = digits.length === 3 ? [...digits].map((d) => d + d).join("") : digits.slice(0, 6);
  const [r, g, b] = [0, 2, 4].map((i) => {
    const channel = parseInt(full.slice(i, i + 2), 16) / 255;
    return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
