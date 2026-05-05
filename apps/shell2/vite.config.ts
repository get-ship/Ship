import { defineConfig, loadEnv } from "vite";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

/**
 * Walk up from this config to find the workspace root (the dir
 * containing the cargo workspace's `Cargo.toml`). Used to locate the
 * freshly-built `ship` binary so we ask it for the per-worktree daemon
 * dir — which is keyed on a sha256 of the project path.
 */
function repoRoot(): string {
  let dir = __dirname;
  for (let i = 0; i < 10; i++) {
    const cargo = join(dir, "Cargo.toml");
    if (existsSync(cargo)) {
      try {
        if (readFileSync(cargo, "utf8").includes("[workspace]")) return dir;
      } catch { /* continue */ }
    }
    const parent = join(dir, "..");
    if (parent === dir) break;
    dir = parent;
  }
  return process.cwd();
}

function resolveDaemonDir(): string | null {
  const root = repoRoot();
  const candidates = [join(root, "target", "debug", "ship"), join(root, "target", "release", "ship")];
  for (const bin of candidates) {
    if (!existsSync(bin)) continue;
    try {
      return execFileSync(bin, ["daemon", "dir"], { encoding: "utf8" }).trim();
    } catch { continue; }
  }
  return null;
}

function readSmall(path: string): string {
  try { return readFileSync(path, "utf8").trim(); } catch { return ""; }
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const daemonDir = resolveDaemonDir();
  const daemonPort =
    env.SHIP_DAEMON_PORT ||
    process.env.SHIP_DAEMON_PORT ||
    (daemonDir ? readSmall(join(daemonDir, "port")) : "") ||
    "9315";
  const daemonPortNum = Number(daemonPort);

  // Shell2 runs at vite-port = daemon-port - 9315 + 3200 (so daemon 9315
  // → vite 3200; daemon 9385 → vite 3270). Stays out of shell's 3100+
  // range so both shells can run side-by-side during the migration.
  const offset = daemonPortNum >= 9315 ? daemonPortNum - 9315 : 0;
  const vitePort = Number(env.VITE_PORT ?? process.env.VITE_PORT ?? 3200 + offset);

  function currentDaemonUrl(): string {
    const explicit = env.VITE_DAEMON_URL ?? process.env.VITE_DAEMON_URL;
    if (explicit) return explicit;
    const portEnv = env.SHIP_DAEMON_PORT ?? process.env.SHIP_DAEMON_PORT;
    if (portEnv) return `http://127.0.0.1:${portEnv}`;
    const port = daemonDir ? readSmall(join(daemonDir, "port")) : "";
    return `http://127.0.0.1:${port || "9315"}`;
  }

  function currentToken(): string {
    const explicit = env.VITE_DAEMON_TOKEN ?? process.env.VITE_DAEMON_TOKEN;
    if (explicit && explicit.length > 0) return explicit;
    if (daemonDir) {
      const t = readSmall(join(daemonDir, "token"));
      if (t) return t;
    }
    return readSmall(join(homedir(), ".ship", "daemon.token"));
  }

  const proxyForDaemon = (rewrite?: (p: string) => string) => ({
    target: currentDaemonUrl(),
    router: () => currentDaemonUrl(),
    changeOrigin: true,
    ...(rewrite ? { rewrite } : {}),
    configure: (proxy: { on: (event: string, cb: (req: { setHeader: (k: string, v: string) => void }) => void) => void }) => {
      proxy.on("proxyReq", (proxyReq) => {
        const t = currentToken();
        if (t) proxyReq.setHeader("Authorization", `Bearer ${t}`);
      });
    },
  });

  return {
    base: "/",
    server: {
      port: vitePort,
      strictPort: true,
      // Proxy keys are path prefixes by default — `/sui` would also catch
      // shell-routed paths like `/sui-demo` (a SUI view served via /sui-demo
      // → shell2 mounts /_sui/views/sui-demo). Anchor the daemon-only
      // standalone host at `^/sui/` so the prefix doesn't shadow shell views
      // whose name happens to start with `sui`.
      proxy: {
        "/api": proxyForDaemon(),
        "/_sui": proxyForDaemon(),
        "^/sui/.*": proxyForDaemon(),
        "/view-host": proxyForDaemon(),
      },
    },
  };
});
