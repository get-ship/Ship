import { defineConfig, loadEnv, type Plugin } from "vite";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, unlinkSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
// URL scheme SSOT: generated from apps/shipd/src/routes.rs by its build.rs.
// Never hardcode `/_sui/*.js` here — derive from the one route table. Import the
// generated file directly (not the @ship/sdk barrel, which drags connect deps
// that don't resolve in this node config context).
import { suiRuntimeJs, suiDomJs, suiSdkClientJs } from "../../packages/deck/sdk/src/generated/routes";

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

// Resolve the SUI runtime + its importmap siblings to the daemon's /_sui URLs
// in dev. Vite would otherwise resolve bare `@ship/sui-*` specifiers to the
// workspace SOURCE (served via @fs) — a SECOND runtime instance alongside the
// daemon's bundled /_sui/runtime.js that the importmap-resolved view bundles
// use (split-brain signals; the runtime guard fires "loaded 2 times"). By
// rewriting the import to the same `/_sui/*` URL the importmap targets — served
// through the existing /_sui proxy — shell code and view bundles share ONE
// runtime instance, so dev matches production.
//
// Mirrors the importmap in `index.html`. Returning a servable URL as `external`
// (not the bare specifier — vite can't load that in serve mode) leaves the
// import for the browser, which fetches it through the proxy.
function suiRuntimeToDaemonUrl(): Plugin {
  const rt = suiRuntimeJs();
  const MAP: Record<string, string> = {
    "@ship/sui-runtime": rt,
    // Subpath specifiers resolve to the same single runtime module.
    "@ship/sui-runtime/route": rt,
    "@ship/sui-runtime/template-hooks": rt,
    "@ship/sui-runtime/patch": rt,
    "@ship/sui-dom": suiDomJs(),
    "@ship/sdk-client": suiSdkClientJs(),
  };
  return {
    name: "ship:sui-runtime-to-daemon",
    enforce: "pre",
    resolveId(id) {
      const url = MAP[id];
      return url ? { id: url, external: true } : null;
    },
  };
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

/**
 * Write `<daemon_dir>/vite.pid` while this vite process is alive so
 * `kill_vite()` in `scripts/dev/_lib.sh` (consumed by `just dev-stop` /
 * `just dev-restart`) can reap us. Removes the file on graceful shutdown
 * (closeBundle) and on SIGINT/SIGTERM/exit. No-op when the daemon binary
 * isn't built yet — `daemonDir` is null and there's nothing to coordinate.
 */
function vitePidPlugin(daemonDir: string | null): Plugin {
  const pidFile = daemonDir ? join(daemonDir, "vite.pid") : null;
  let written = false;
  const remove = () => {
    if (!pidFile || !written) return;
    try { unlinkSync(pidFile); } catch { /* already gone */ }
    written = false;
  };
  const write = () => {
    if (!pidFile || written) return;
    try {
      writeFileSync(pidFile, String(process.pid));
      written = true;
    } catch { /* daemon dir vanished — best-effort */ }
  };
  return {
    name: "ship:vite-pid",
    apply: "serve",
    configureServer() {
      write();
      const onSignal = () => { remove(); process.exit(0); };
      process.once("SIGINT", onSignal);
      process.once("SIGTERM", onSignal);
      process.once("exit", remove);
    },
    closeBundle() { remove(); },
  };
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
    plugins: [vitePidPlugin(daemonDir), suiRuntimeToDaemonUrl()],
    // The SUI runtime specifiers resolve to daemon `/_sui/*` URLs (see
    // suiRuntimeToDaemonUrl); keep vite's dep pre-scanner from trying to read
    // them off disk.
    optimizeDeps: {
      exclude: [
        "@ship/sui-runtime",
        "@ship/sui-runtime/route",
        "@ship/sui-runtime/template-hooks",
        "@ship/sui-runtime/patch",
        "@ship/sui-dom",
        "@ship/sdk-client",
      ],
    },
    server: {
      port: vitePort,
      strictPort: true,
      // Proxy keys are path prefixes by default — `/sui` would also catch
      // shell-routed paths like `/sui-demo` (a SUI view served via /sui-demo
      // → shell mounts /_sui/views/sui-demo). Anchor the daemon-only
      // standalone host at `^/sui/` so the prefix doesn't shadow shell views
      // whose name happens to start with `sui`.
      proxy: {
        "/api": proxyForDaemon(),
        "/_sui": proxyForDaemon(),
        "^/sui/.*": proxyForDaemon(),
        "/view-host": proxyForDaemon(),
        // ConnectRPC op surface (`ship.<domain>.<op>()` → POST
        // /ship.ops.v1.<Service>/<Method>). The GUI flip (#72/#116) moved shell
        // data calls off /api/v1 onto these; without proxying them they hit the
        // vite SPA fallback → 404 and no data renders.
        "^/ship\\.ops\\.v1\\..*": proxyForDaemon(),
      },
    },
  };
});
