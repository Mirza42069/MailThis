import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

// Tauri loads the dev server from a fixed port (src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**"] } },
});
