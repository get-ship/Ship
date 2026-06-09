import { defineConfig, type Plugin } from "vite";
import { join } from "node:path";
import { DECK_PREFIX } from "@ship/sdk-client";
function suiRuntimeToDaemonUrl(daemonUrl: string): Plugin {
  return {
    name: "ship:sui-runtime-to-daemon-spaces",
    enforce: "pre",
    resolveId(id) {
      if (id === "@ship/sui-runtime" || id.startsWith("@ship/sui-runtime/") || id === "@ship/sui-dom" || id === "@ship/sdk-client") {
        // Spaces only uses @ship/sui-runtime and @ship/sdk-client.
        const path = id === "@ship/sdk-client" ? `${DECK_PREFIX}/sdk-client.js` : id === "@ship/sui-dom" ? `${DECK_PREFIX}/dom.js` : `${DECK_PREFIX}/runtime.js`;
        return { id: daemonUrl + path, external: true };
      }
      return null;
    },
  };
}

export default defineConfig(() => ({
  base: "/",
  plugins: [suiRuntimeToDaemonUrl("http://127.0.0.1:9330")],
  optimizeDeps: {
    exclude: [
      "@ship/sui-runtime",
      "@ship/sui-runtime/route",
      "@ship/sui-runtime/template-hooks",
      "@ship/sui-runtime/patch",
      "@ship/sui-dom",
      "@ship/sdk-client",
      `${DECK_PREFIX}/runtime.js`,
      `${DECK_PREFIX}/dom.js`,
      `${DECK_PREFIX}/sdk-client.js`,
    ],
  },
  server: {
    port: 5174,
    strictPort: true,
    proxy: {
      [DECK_PREFIX]: { target: "http://127.0.0.1:9330", changeOrigin: true },
      "/api": { target: "http://127.0.0.1:9330", changeOrigin: true },
      "/ship\\.ops\\.v1\\..*": { target: "http://127.0.0.1:9330", changeOrigin: true },
    },
  },
}));
