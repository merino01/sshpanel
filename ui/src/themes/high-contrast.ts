/**
 * Alto contraste: negro y blanco puros, bordes visibles y colores vivos, con
 * todo el texto en WCAG AAA (7:1). El acento del Branding se queda en botones y
 * marcas; el texto que iría en acento va en blanco, y el de los botones en
 * blanco o negro, el que más contraste.
 */
import type { ThemePalette } from "../lib/themes";

export default {
  scheme: "dark",
  colors: {
    bg: "#000000",
    fg: "#ffffff",
    surface: "#0d0d0d",
    border: "#a3a3a3",
    borderStrong: "#ffffff",
    subtle: "#1f1f1f",
    shadow: "rgba(0, 0, 0, 0)",
    backdrop: "rgba(0, 0, 0, 0.8)",
    accentText: "#ffffff",
    error: "#ff8080",
    success: "#7dff9b",
    warning: "#ffd700",
  },
  terminal: {
    background: "#000000",
    foreground: "#ffffff",
    cursor: "#ffffff",
    selection: "rgba(255, 255, 255, 0.4)",
    black: "#000000",
    red: "#ff6b6b",
    green: "#5cff5c",
    yellow: "#ffff55",
    blue: "#8cb4ff",
    magenta: "#ff8cff",
    cyan: "#5cffff",
    white: "#e0e0e0",
    brightBlack: "#a0a0a0",
    brightRed: "#ff9e9e",
    brightGreen: "#a6ffa6",
    brightYellow: "#ffffa6",
    brightBlue: "#b8d0ff",
    brightMagenta: "#ffb8ff",
    brightCyan: "#a6ffff",
    brightWhite: "#ffffff",
  },
} satisfies ThemePalette;
