import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const daemon = env.VITE_DAEMON_URL ?? "http://127.0.0.1:51742";
  return {
    server: {
      port: 3100,
      strictPort: true,
      proxy: {
        // Optional proxy: lets SSE/EventSource hit same-origin and inherit cookies.
        "/daemon": {
          target: daemon,
          changeOrigin: true,
          rewrite: (p) => p.replace(/^\/daemon/, ""),
        },
      },
    },
    build: {
      target: "es2022",
      outDir: "dist",
    },
  };
});
