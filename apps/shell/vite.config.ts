import { defineConfig, loadEnv } from "vite";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join, resolve } from "node:path";

/**
 * Vite plugin: treat `.svg` imports as raw text strings.
 *
 * Mirrors esbuild's `--loader:.svg=text` (set in
 * `packages/ship-set/package.json` `bundle:elements` script). Without
 * this, Vite resolves `import claudeSvg from '...claude.svg'` to an
 * asset URL — `<ship-provider-blob>` then renders the URL instead of
 * the inline SVG markup, so brand glyphs disappear.
 *
 * Applies to all .svg imports during dev + build. Excludes SVGs
 * imported with explicit `?url` (lets callers opt out if they want
 * the URL form).
 */
function svgAsText() {
  return {
    name: "svg-as-text",
    enforce: "pre" as const,
    transform(_code: string, id: string) {
      const [path, query = ""] = id.split("?");
      if (!path.endsWith(".svg")) return null;
      if (query.includes("url")) return null;
      const text = readFileSync(path, "utf8");
      return { code: `export default ${JSON.stringify(text)};`, map: null };
    },
  };
}

/**
 * Walk up from the shell's vite-config dir to find the repo root —
 * the directory containing `Cargo.toml` with a `[workspace]` table.
 * Used to locate the worktree's freshly-built `ship` binary so we
 * stay in sync with whatever the daemon was built from.
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

/**
 * Locate the per-worktree daemon dir (`~/.ship/machine/daemons/<hash>/`).
 *
 * Each worktree gets its own daemon (devx-04). The dir is keyed on a
 * sha256 of the project root path so two worktrees never collide on
 * port, token, pid, or log file. The `ship daemon dir` command knows
 * how to compute it; we shell out so the JS side stays in sync with
 * the Rust runtime's path-hash without reimplementing it. The worktree
 * binary wins so a freshly-built `ship` with new logic isn't shadowed
 * by a stale `~/.local/bin/ship` on $PATH.
 */
function resolveDaemonDir(): string | null {
  const root = repoRoot();
  const candidates = [
    join(root, "target", "debug", "ship"),
    join(root, "target", "release", "ship"),
  ];
  for (const bin of candidates) {
    if (!existsSync(bin)) continue;
    try {
      return execFileSync(bin, ["daemon", "dir"], { encoding: "utf8" }).trim();
    } catch {
      continue;
    }
  }
  return null;
}

function readSmall(path: string): string {
  try {
    return readFileSync(path, "utf8").trim();
  } catch {
    return "";
  }
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");

  // Per-worktree daemon: read port + token from `<daemon_dir>/{port,token}`.
  // Explicit env-var overrides win for tests / CI / one-off forwarding.
  const daemonDir = resolveDaemonDir();
  const daemonPortFromDir = daemonDir ? readSmall(join(daemonDir, "port")) : "";
  const daemonPort =
    env.SHIP_DAEMON_PORT ||
    process.env.SHIP_DAEMON_PORT ||
    daemonPortFromDir ||
    "9315";
  const daemonPortNum = Number(daemonPort);
  const daemon = env.VITE_DAEMON_URL ?? `http://127.0.0.1:${daemonPort}`;

  // Vite dev port follows daemon port so two worktrees run side-by-side:
  // daemon 9315 → vite 3100; daemon 9385 → vite 3170; etc. Guard against
  // a bad daemonPort read (would yield a negative number) by falling back
  // to 3100 when the offset doesn't make sense.
  const offset = daemonPortNum >= 9315 ? daemonPortNum - 9315 : 0;
  const vitePort = Number(env.VITE_PORT ?? process.env.VITE_PORT ?? 3100 + offset);

  // Token resolved per-request so daemon restarts (which mint a new token)
  // don't require a Vite restart. Precedence: env override → daemon-dir
  // file (canonical) → legacy `~/.ship/daemon.token` for back-compat.
  function currentToken(): string {
    const explicit = env.VITE_DAEMON_TOKEN ?? process.env.VITE_DAEMON_TOKEN;
    if (explicit && explicit.length > 0) return explicit;
    if (daemonDir) {
      const t = readSmall(join(daemonDir, "token"));
      if (t) return t;
    }
    return readSmall(join(homedir(), ".ship", "daemon.token"));
  }
  // Embedded production assets are served at /shell/ by shipd
  // (apps/shipd/src/shell_assets.rs). Dev mode runs at the Vite root and
  // proxies /api → daemon, so base must stay `/` there.
  const isDev = mode === "development";
  return {
    base: isDev ? "/" : "/shell/",
    plugins: [svgAsText()],
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
        // SUI framework — daemon serves the runtime/dom/sdk-client
        // bundles, view templates, the embeddable fragment endpoint
        // (`/_sui/fragment/<name>`), the JSON island data feed, and
        // mutations under /_sui/*. Listed before /_ship so the more-
        // specific prefix wins.
        "/_sui": {
          target: daemon,
          changeOrigin: true,
          configure: (proxy) => {
            proxy.on("proxyReq", (proxyReq) => {
              const t = currentToken();
              if (t) proxyReq.setHeader("Authorization", `Bearer ${t}`);
            });
          },
        },
        // SUI full-page views — file-system routed at /sui/<name>.
        "/sui": {
          target: daemon,
          changeOrigin: true,
          configure: (proxy) => {
            proxy.on("proxyReq", (proxyReq) => {
              const t = currentToken();
              if (t) proxyReq.setHeader("Authorization", `Bearer ${t}`);
            });
          },
        },
        // SFC components — daemon serves compiled .ship files at
        // /_ship/components/<tag>.js. Same auth + same origin as /api.
        "/_ship": {
          target: daemon,
          changeOrigin: true,
          configure: (proxy) => {
            proxy.on("proxyReq", (proxyReq) => {
              const t = currentToken();
              if (t) proxyReq.setHeader("Authorization", `Bearer ${t}`);
            });
          },
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
