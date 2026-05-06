# Toolchain — `~/.ship/runtime/`

Ship ships with a bundled JS toolchain at `~/.ship/runtime/`. It mirrors what `rustup` does for Rust: one well-known directory contains every binary and dependency Ship's build / dev pipeline needs, pinned to the Ship version that installed it.

## Layout

```
~/.ship/runtime/
  bun                 # Bun static binary (~100 MB)
  sui-compiler        # Rust release build of crates/sui-compiler
  node_modules/       # vite + peer deps; @ship/vite-plugin-sui is copied
                      # in from the workspace at build time, not resolved
                      # from a registry (it isn't published)
  package.json        # pinned versions for the toolchain
  bun.lock            # bun lockfile
```

The directory is owned by `crates/core/runtime/src/home/ship_home.rs` — every accessor lives there, no other code constructs paths under `~/.ship/runtime/` directly.

## Why Bun + Vite

- **Single-binary toolchain.** Bun replaces Node + npm + pnpm with one static executable. The whole runtime dir on disk is two binaries and a node_modules tree.
- **Vite for HMR.** Vite's hot-reload story is the gold standard for module-graph-aware updates. The hand-rolled `dev_assets.rs` watcher that previously did this is gone — Vite's WebSocket pushes module updates straight to the browser.
- **Plugin maturity.** Vite's plugin ecosystem covers the things we don't want to write (legacy bundles, source-map plumbing, dependency pre-bundling). `@ship/vite-plugin-sui` slots in alongside them.
- **`bun-as-Node-substitute`.** Vite is built for Node; running it under Bun via Bun's Node compat keeps every Vite plugin working without rewrites. If Bun's compat ever falls short for our case, the plugin is portable to plain Node + Vite without code changes.

We deliberately did **not** adopt `Bun.plugin` / `Bun.build` directly. Bun's bundler + dev server are first-class but younger than Vite's; switching is an option once the team has Bun mileage and Bun's HMR story matches Vite's. The current plugin in `packages/vite-plugin-sui/src/compile.ts` would be ~100 lines of mechanical change to port if/when we make the switch.

## Components

### `bun`

Drives every JS pipeline action: `vite build`, `bun install`, ad-hoc scripts. Pinned to the Ship version that installed the toolchain. Updated via `ship runtime update`.

### `sui-compiler`

The Rust ahead-of-time compiler for `.sui` files. Same binary as `crates/sui-compiler` ships under `target/release/sui-compiler` for in-repo dev. The toolchain's copy is consumed by `@ship/vite-plugin-sui` via subprocess (`Bun.spawn`).

### `node_modules/`

Currently holds `vite` and its transitive deps. `@ship/vite-plugin-sui` is **not** a registry dep — it's a workspace package consumed by `ship build` directly, and copied into `node_modules/@ship/vite-plugin-sui/` at install time so Vite can resolve `import suiPlugin from '@ship/vite-plugin-sui'` in user-supplied configs. (Step 4 of the migration deferred the copy step; consumers in this repo currently import the plugin via workspace path.)

### `package.json`

Pinned versions for the JS toolchain. Bumping these is a deliberate Ship-version event — the daemon and the toolchain ship together so consumers don't end up with mismatched plugin / runtime pairs.

## CLI

```sh
ship runtime install       # populate ~/.ship/runtime/ from local sources
ship runtime update        # wipe + reinstall
ship runtime doctor        # verify every required component is present
ship build [--target X]    # run the full Vite pipeline (or a single target)
```

`runtime install` resolves `bun` from `--bun <path>` or `which bun` on `PATH`; `sui-compiler` from `--sui-compiler <path>` or a walk-up search for `target/release|debug/sui-compiler` from cwd. `update` is `install` with `--clean` (wipes the runtime dir before reinstalling).

`runtime doctor` runs `runtime::toolchain::ensure_present`, checks each piece in order (runtime dir exists → bun present and executable → sui-compiler present and executable → node_modules populated → package.json present), and exits with the first failure.

## Errors

Every error variant in `runtime::toolchain` carries a recovery hint in its `Display` impl. If you see one:

- **"Ship toolchain not installed at …"** → `ship runtime install`
- **"Ship toolchain is incomplete: missing …"** → `ship runtime install` (idempotent; repairs partials)
- **"Ship toolchain binary … is not executable"** → `ship runtime install` (resets +x bits)
- **"build-bundles-vite.mjs not found at …"** → you're running `ship build` outside the workspace; `cd` to the repo root.
- **"`bun install` failed in …"** → captured stderr is appended verbatim; usually a network or registry issue.

## See also

- `docs/ship-views/references/docs/dev-loop.md` — what the `.sui` edit → browser update path looks like under Vite HMR.
- `crates/core/runtime/src/toolchain/` — the Rust orchestration code.
- `packages/vite-plugin-sui/` — the plugin source.
- `.artifacts/spike-bun-vite-result.md` — Step 1 spike report (validation that this whole approach works).
