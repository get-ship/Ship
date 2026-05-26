import { defineConfig, loadEnv, type Plugin } from "vite";
import { existsSync, readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";
// URL-scheme SSOT — the daemon `/_sui/*` builders (generated from routes.rs).
import { suiRuntimeJs, suiDomJs, suiSdkClientJs } from "../../packages/deck/sdk/src/generated/routes";

// Dev server for the space scaffold. Mirrors the hub's daemon proxy + SUI
// runtime externalization so a space window in dev loads the ONE runtime from
// the daemon (single-runtime) and reaches `/api` + `/_sui` + Connect.
function repoRoot(): string {
  let dir = __dirname;
  for (let i = 0; i < 10; i++) {
    const cargo = join(dir, "Cargo.toml");
    if (existsSync(cargo) && readFileSync(cargo, "utf8").includes("[workspace]")) return dir;
    const parent = join(dir, "..");
    if (parent === dir) break;
    dir = parent;
  }
  return process.cwd();
}

function resolveDaemonDir(): string | null {
  const root = repoRoot();
  for (const bin of [join(root, "target", "debug", "ship"), join(root, "target", "release", "ship")]) {
    if (!existsSync(bin)) continue;
    try { return execFileSync(bin, ["daemon", "dir"], { encoding: "utf8" }).trim(); } catch { /* next */ }
  }
  return null;
}
const readSmall = (p: string): string => { try { return readFileSync(p, "utf8").trim(); } catch { return ""; } };

// Rewrite bare @ship/sui-* to the daemon's /_sui URLs (the importmap mirror) so
// dev matches prod: one runtime instance, no second copy off the source tree.
function suiRuntimeToDaemonUrl(): Plugin {
  const rt = suiRuntimeJs();
  const MAP: Record<string, string> = {
    "@ship/sui-runtime": rt,
    "@ship/sui-runtime/route": rt,
    "@ship/sui-runtime/template-hooks": rt,
    "@ship/sui-runtime/patch": rt,
    "@ship/sui-dom": suiDomJs(),
    "@ship/sdk-client": suiSdkClientJs(),
  };
  return { name: "ship:sui-runtime-to-daemon", enforce: "pre", resolveId: (id) => (MAP[id] ? { id: MAP[id], external: true } : null) };
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const daemonDir = resolveDaemonDir();
  const daemonPort = env.SHIP_DAEMON_PORT || process.env.SHIP_DAEMON_PORT || (daemonDir ? readSmall(join(daemonDir, "port")) : "") || "9315";
  const daemonUrl = `http://127.0.0.1:${daemonPort}`;
  const token = (daemonDir ? readSmall(join(daemonDir, "token")) : "") || readSmall(join(process.env.HOME ?? "", ".ship", "daemon.token"));

  const proxyForDaemon = () => ({
    target: daemonUrl,
    changeOrigin: true,
    configure: (proxy: { on: (e: string, cb: (r: { setHeader: (k: string, v: string) => void }) => void) => void }) => {
      proxy.on("proxyReq", (r) => { if (token) r.setHeader("Authorization", `Bearer ${token}`); });
    },
  });
  const daemonProxy = {
    "/api": proxyForDaemon(),
    "/_sui": proxyForDaemon(),
    "^/ship\\.ops\\.v1\\..*": proxyForDaemon(),
  };

  return {
    base: "/",
    plugins: [suiRuntimeToDaemonUrl()],
    optimizeDeps: {
      exclude: ["@ship/sui-runtime", "@ship/sui-runtime/route", "@ship/sui-runtime/template-hooks", "@ship/sui-runtime/patch", "@ship/sui-dom", "@ship/sdk-client"],
    },
    server: { port: 3201, strictPort: true, proxy: daemonProxy },
    preview: { port: 3201, strictPort: true, proxy: daemonProxy },
  };
});
