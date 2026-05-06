# spike-bun-vite

Step 1 of the **vite-plugin-bun** job. This app validates that **Bun + Vite + a custom plugin** compose end-to-end before we commit to migrating away from `scripts/build-bundles.mjs` and `crates/ship-ui-framework/src/dev_assets.rs`.

It is **not** a deliverable — it lives outside the pnpm workspace and is gitignored from sharing node_modules with the rest of the monorepo. Once the spike passes, the plugin code moves to `packages/vite-plugin-sui/` (Step 2) and this directory is deleted.

## Layout

```
.local/bun                # Bun binary (~100 MB, gitignored)
vite.config.ts            # wires the plugin + a third-party plugin (legacy)
vite-plugin-sui.ts        # spike-local plugin: shells to sui-compiler per .sui file
app.sui                   # smallest possible .sui input
main.ts                   # imports app.sui, mounts it
index.html                # vite entry
package.json              # vite ^7, no workspace deps
```

## Running

Prereq: `cargo build -p sui-compiler --release` (consumed via `target/release/sui-compiler`).

```sh
cd apps/spike-bun-vite
.local/bun install
.local/bun run vite           # dev server
.local/bun run vite build     # production bundle to dist/
```

## What the spike validates

1. `bun install` resolves the Vite dependency graph.
2. `bun run vite` boots the dev server.
3. The `.sui` transform fires (i.e. our plugin's `transform` hook is invoked under Bun).
4. The compiled `.sui` output's `@ship/ui-runtime` / `@ship/ui-dom` imports resolve via `resolve.alias`.
5. Editing `app.sui` triggers `handleHotUpdate` and the browser reflects the change without full reload.
6. `vite build` emits a static bundle.
7. A non-trivial third-party plugin (`@vitejs/plugin-legacy`) composes cleanly alongside ours.

Findings get recorded in `.artifacts/spike-bun-vite-result.md`.
