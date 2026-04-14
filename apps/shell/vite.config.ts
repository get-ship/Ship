import { defineConfig, loadEnv } from "vite";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

/**
 * Resolve the daemon bearer token. Precedence:
 *   1. VITE_DAEMON_TOKEN env var
 *   2. ~/.ship/daemon.token file (auto, so `pnpm dev` Just Works)
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
  const daemon = env.VITE_DAEMON_URL ?? "http://127.0.0.1:9316";
  const token = resolveToken(env.VITE_DAEMON_TOKEN);
  if (!token) {
    console.warn(
      "[shell] no daemon token — set VITE_DAEMON_TOKEN or create ~/.ship/daemon.token",
    );
  }
  return {
    define: {
      "import.meta.env.VITE_DAEMON_URL": JSON.stringify(daemon),
      "import.meta.env.VITE_DAEMON_TOKEN": JSON.stringify(token),
    },
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
