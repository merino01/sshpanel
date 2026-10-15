import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import Icons from "unplugin-icons/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [tailwindcss(), svelte({ configFile: false }), Icons({ compiler: "svelte" })],
  // `@/…` es `ui/src/…`, como espera shadcn-svelte.
  resolve: { alias: { "@": "/src" } },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true, target: "es2022" },
  test: { include: ["src/**/*.test.ts"] },
});
