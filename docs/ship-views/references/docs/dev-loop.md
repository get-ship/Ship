# `.sui` Dev Loop

What happens between you saving a `.sui` file and the browser reflecting the change.

## End-to-end path

```
        ┌──────────────────────────────────────────────────────────────┐
        │  edit  views/counter/page.sui  (in your editor)              │
        └───────────────────────────┬──────────────────────────────────┘
                                    │ fs notify
                                    ▼
   ┌────────────────────────────────────────────────────────────────┐
   │ Vite dev server (`bun run vite`)                               │
   │   • watcher fires for `.sui` extension                         │
   │   • @ship/vite-plugin-sui handleHotUpdate() returns ctx.modules│
   │   • transform() re-runs:                                       │
   │       Bun.spawn(['sui-compiler', 'build', <file>, '--out', T]) │
   │       Bun.file(`${T}/page.compiled.js`).text() → JS source     │
   │   • Vite re-bundles the dirty module + its dependents          │
   │   • pushes hot update over the WebSocket                       │
   └───────────────────────────┬────────────────────────────────────┘
                               │ WS message
                               ▼
   ┌────────────────────────────────────────────────────────────────┐
   │ Browser (open at http://127.0.0.1:3270 — the worktree's vite   │
   │   port; daemon-port − 9315 + 3200)                             │
   │   • Vite's HMR client receives the update                      │
   │   • new module re-runs; the SUI runtime accepts it via         │
   │     `import.meta.hot.accept()` in the view's hydration shim    │
   │   • DOM rebinds without full reload                            │
   └────────────────────────────────────────────────────────────────┘
```

Median latency on edit → browser update: **~50 ms** for a typical view (per Step 1's spike).

## What's NOT in the loop anymore

- `crates/ship-ui-framework/src/dev_assets.rs` (deleted in Step 7). The hand-rolled file watcher that previously rebuilt bundles into a per-bundle override map and stuffed them into the daemon's serve handlers. Vite's HMR replaces every part of it.
- `scripts/build-bundles.mjs` (deleted in Step 7). The esbuild build script. Vite + `@ship/vite-plugin-sui` does its job through `scripts/build-bundles-vite.mjs` for the production-build path. There's no production-equivalent in the dev loop — Vite serves modules directly out of memory.

## What you still see

- The daemon (`shipd`) continues serving `/api/...`, `/_sui/...`, the SSE stream, etc. Vite proxies these paths through to the daemon (configured in `apps/shell2/vite.config.ts`).
- `ship dev` (or the existing `just dev`) starts both the daemon and the Vite dev server side by side. They're separate processes; killing Vite leaves the daemon running, and vice versa.
- Production bundles still get baked into the daemon binary via `include_str!` in `crates/ship-ui-framework/src/view.rs`. That's the path you hit when running a release build — Vite produces those bundles via `ship build` (Step 6).

## Triggers for a full reload

Most edits hot-reload. A full reload happens when:

- You edit a file Vite can't isolate as a module (e.g. `index.html`, `vite.config.ts`).
- You add or delete a `.sui` file (Vite re-scans the module graph).
- You change the plugin code itself (`packages/vite-plugin-sui/src/...`) — Vite's config is loaded once at boot, so plugin edits require a Vite restart, not just a reload.

## Debugging stuck HMR

1. Check the Vite dev server's console output. `[vite] (client) hmr update /<path>` lines confirm the watcher fired and the transform re-ran.
2. Open DevTools → Console. Vite logs `[vite] hot updated: /<path>` when the browser successfully accepts the update.
3. If neither line shows, the watcher likely missed the change — common in container-mounted volumes. Set `server.watch.usePolling: true` in `apps/shell2/vite.config.ts` as a workaround.
4. If the dev-server line shows but the browser doesn't update, check whether the touched module exposes `import.meta.hot.accept()`. Without it, the parent module — usually `main.ts` — is the HMR boundary; verify it's marked accepted.

## See also

- `docs/ship-architecture/references/docs/toolchain.md` — what `~/.ship/runtime/` contains and why Bun + Vite specifically.
- `apps/shell2/vite.config.ts` — the live dev-server config that wires `@ship/vite-plugin-sui` and proxies the daemon.
- `packages/vite-plugin-sui/src/index.ts` — `handleHotUpdate` + `transform` hook implementation.
