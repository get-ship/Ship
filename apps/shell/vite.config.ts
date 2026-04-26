import { defineConfig, loadEnv } from "vite";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join, resolve } from "node:path";

/**
 * Resolve the daemon bearer token. Precedence:
 *   1. VITE_DAEMON_TOKEN env var
 *   2. ~/.ship/daemon.token file (auto, so `pnpm dev` Just Works)
 *
 * Only used for the cross-origin dev path (vite on :3100 → daemon on :9315).
 * When the daemon serves the shell same-origin (cookie auth), no token is read
 * at runtime — the client code drops the Authorization header entirely.
 */
function resolveToken(envToken: string | undefined): string {
  if (envToken && envToken.length > 0) return envToken;
  try {
    return readFileSync(join(homedir(), ".ship", "daemon.token"), "utf8").trim();
  } catch {
    return "";
  }
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  // Daemon target — explicit URL wins; otherwise fall back to SHIP_DAEMON_PORT
  // (set by scripts/dev-daemon.sh when binding a non-default port for
  // multi-worktree dev), then to 127.0.0.1:9315.
  const daemonPort = env.SHIP_DAEMON_PORT ?? process.env.SHIP_DAEMON_PORT ?? "9315";
  const daemon = env.VITE_DAEMON_URL ?? `http://127.0.0.1:${daemonPort}`;
  // Vite dev server port follows the daemon's offset so two worktrees
  // can run side-by-side: daemon :9315 → vite :3100; daemon :9316 → vite :3101, etc.
  const vitePort = Number(env.VITE_PORT ?? process.env.VITE_PORT ?? (3100 + (Number(daemonPort) - 9315)));
  // Token resolved per-request in the proxy below so daemon restarts (which
  // mint a new token) don't require a Vite restart.
  function currentToken(): string {
    return resolveToken(env.VITE_DAEMON_TOKEN);
  }
  // Embedded production assets are served at /shell/ by shipd
  // (apps/shipd/src/shell_assets.rs). Dev mode runs at the Vite root and
  // proxies /api → daemon, so base must stay `/` there.
  const isDev = mode === "development";
  return {
    base: isDev ? "/" : "/shell/",
    resolve: {
      // Single Lit copy at runtime — both shell chrome (Vite) and
      // @ship/set web-components (Vite) resolve to the same module
      // instance, so Lit's "Multiple versions" warning stays quiet.
      dedupe: ["lit"],
    },
    define: {
      // Client sees empty string → relative URLs → Vite proxy forwards to
      // daemon with fresh token. The `daemon` var above is the proxy target
      // only (server-side), never exposed to the browser.
      "import.meta.env.VITE_DAEMON_URL": JSON.stringify(""),
      "import.meta.env.VITE_DAEMON_TOKEN": JSON.stringify(""),
    },
    // Daemon-served assets are proxied at runtime, not local files.
    // Tell Vite not to try pre-transforming them.
    optimizeDeps: {
      exclude: [
        "/api/assets/ship-elements.js",
        "/api/assets/ship-code-editor.js",
        "/api/assets/ship-markdown-editor.js",
        "/api/assets/ship-terminal.js",
        "/api/assets/datastar.js",
      ],
    },
    server: {
      port: vitePort,
      strictPort: true,
      proxy: {
        // Forward daemon API + view endpoints so the browser sees everything
        // as same-origin on :3100. Bearer token injected by the proxy so the
        // client code can run with credentials: "include" + cookie only.
        "/api": {
          target: daemon,
          changeOrigin: true,
          configure: (proxy) => {
            proxy.on("proxyReq", (proxyReq) => {
              const t = currentToken();
              if (t) proxyReq.setHeader("Authorization", `Bearer ${t}`);
            });
          },
        },
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
      rollupOptions: {
        input: {
          main: resolve(__dirname, "index.html"),
        },
      },
    },
  };
});
