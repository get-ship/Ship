# @ship/vite-plugin-sui

Vite plugin that compiles `.sui` files to JS via the `sui-compiler` binary. Replaces `scripts/build-bundles.mjs` (production bundling) and `crates/ship-ui-framework/src/dev_assets.rs` (HMR watcher).

## Install

The plugin is a workspace package; consumers in this repo depend on it via `pnpm`'s workspace protocol. The `sui-compiler` binary is resolved at runtime from (in order):

1. `~/.ship/runtime/sui-compiler` — the toolchain layout produced by `ship runtime install`.
2. `<workspace>/target/release/sui-compiler` — for in-repo dev (`cargo build -p sui-compiler --release`).
3. `<workspace>/target/debug/sui-compiler` — for in-repo dev (debug build).

Pass `Options.compilerPath` to override.

## Usage

```ts
import { defineConfig } from 'vite'
import suiPlugin from '@ship/vite-plugin-sui'

export default defineConfig({
  plugins: [suiPlugin()],
})
```

## Options

| Option | Default | Description |
|---|---|---|
| `compilerPath` | resolved as above | Absolute path to the `sui-compiler` binary. |
| `detect` | `null` | `(absPath: string) => ViteFileMode \| null` — override component-vs-view detection. Return `null` to fall back to the default heuristic. |
| `noDts` | `true` | Suppress emitting `.sui.d.ts` declarations during transform. The package consumer supplies types via its own declaration sources. |

## Detection

Default heuristic: a `.sui` file is a **component** when its filename stem equals the parent directory name (e.g. `packages/ui-components/src/sui-button/sui-button.sui` → `<sui-button>`). Every other `.sui` is treated as a **view**. Override via `Options.detect` for one-off cases.

## Source maps

The current sui-compiler does not emit per-AST-node position info, so the plugin returns a low-resolution stub map: every line of emitted JS points at the start of the source `.sui` file. This is enough for DevTools "open source" navigation. When the compiler grows real position emit, the plugin's `compile.ts` `stubSourceMap` gets replaced wholesale with no API change.

## Runtime

The plugin is **Bun-only at runtime**. The hot path uses `Bun.spawn` (subprocess) and `Bun.file` (read emitted JS) — the cold-setup paths still go through `node:fs/promises`, `node:os`, `node:path` because Bun has no native equivalents for `mkdtemp` / `tmpdir` / `path.join` and Bun's own first-party code routes those through node-compat too. So:

- **Where it counts (per-transform spawn + read):** Bun-native.
- **Where Bun has no native primitive:** `node:` imports via Bun's compat layer.

Vite invoked via `bun run vite` is the supported entry point. Running the plugin under bare Node will throw at the first `Bun.spawn` call.

## Testing

```
pnpm --filter @ship/vite-plugin-sui test
```

Tests use **`bun test`** rather than vitest. Vitest 4's worker pool spawns Node subprocesses — even when invoked via `bun run vitest` — so `Bun.spawn` / `Bun.file` aren't defined inside test workers. `bun test` is API-compatible with vitest assertions (`describe`/`it`/`expect`) and keeps the Bun runtime end-to-end.

The transform tests skip when no `sui-compiler` binary is found on disk (which is the right behavior on a clean workspace before `cargo build`). Detection tests are pure-logic and always run.
