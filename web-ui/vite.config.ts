import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// Dev server proxies the API + WebSockets to a locally-running vmlab-web so
// `npm run dev` gives hot-reload against the real backend.
export default defineConfig({
  plugins: [solid()],
  server: {
    proxy: {
      "/api": { target: "http://127.0.0.1:7878", ws: true },
      "/vnc": { target: "http://127.0.0.1:7878", ws: true },
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    // noVNC ships top-level await (WebCodecs feature detection).
    target: "esnext",
  },
});
