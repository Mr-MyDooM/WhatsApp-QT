import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// One HTML entry per window. Keep the port equal to build.devUrl in tauri.conf.json.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    rollupOptions: {
      input: {
        index: "index.html",
        settings: "settings.html",
        rename: "rename.html",
        error: "error.html",
      },
    },
  },
});
