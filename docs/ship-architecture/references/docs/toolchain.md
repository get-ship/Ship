# Toolchain — `~/.ship/runtime/`

Ship ships with a bundled JS toolchain at `~/.ship/runtime/`. It mirrors what `rustup` does for Rust: one well-known directory contains every binary and script Ship's build / dev pipeline needs, pinned to the Ship version that installed it.

## Layout

```
~/.ship/runtime/
  bun                 # Bun static binary (~100 MB)
  sui-compiler        # Rust release build of crates/sui/sui-compiler
  bundler.ts          # Bun worker script — the asset bundler
```

No `node_modules/`, no `package.json`, no lockfile. `bundler.ts` runs under Bun and calls Bun's built-in APIs (`Bun.build`, `Bun.plugin`) directly.

The directory is owned by `crates/core/ship-substrate/src/home/ship_home.rs` — every accessor lives there, no other code constructs paths under `~/.ship/runtime/` directly.

## Why Bun-native bundling

- **No node_modules tree.** The previous Vite-based pipeline required `npm install`-ing `vite` and `@ship/vite-plugin-sui` into `~/.ship/runtime/`. Now the runtime dir is three files.
- **In-process API.** `Bun.build()` and `Bun.plugin()` are part of the Bun binary. There is nothing to install and nothing to resolve.
- **Worker protocol.** `bundler.ts` is a long-lived subprocess supervised by `shipd`. Requests arrive on stdin as JSON lines; responses and push events go back on stdout. The daemon owns the process lifecycle — the bundler is not a server.
- **Supervised restarts.** The Rust side (`crates/core/ship-application/src/bundler/`) wraps the worker with exponential-backoff restarts (1 s → 30 s, max 20 restarts). A crashed worker does not take the daemon with it.

## Components

### `bun`

Executes `bundler.ts`. Also used for ad-hoc dev scripts. Pinned to the Ship version that installed the toolchain. Updated via `ship runtime update`.

### `sui-compiler`

The Rust ahead-of-time compiler for `.sui` files. Invoked by `bundler.ts` via `Bun.spawn` to transform `.sui` source to JavaScript. Same binary as `crates/sui/sui-compiler` ships under `target/release/sui-compiler` for in-repo dev.

### `bundler.ts`

The worker script installed from `runtime/bundler.ts` in the repo. Receives JSON-line requests from the daemon (`build`, `build_all`, `watch_start`, `watch_stop`, `shutdown`) and replies with JSON-line responses (`bundle`, `ack`, `error`) or push events (`module_changed`, `compile_error`, `ready`).

See `crates/core/ship-application/src/bundler/proto.rs` for the Rust-side protocol types.

## Worker protocol

```
stdin  → { "id": 1, "kind": "build", "target": "counter" }
stdout ← { "id": 1, "kind": "bundle", "target": "counter", "bytes": 4096, "out_path": "..." }

stdin  → { "id": 2, "kind": "watch_start" }
stdout ← { "id": 2, "kind": "ack" }
stdout ← { "kind": "module_changed", "target": "counter", "out_path": "...", "bytes": 4200 }

stdin  → { "id": 3, "kind": "shutdown" }
```

Every request carries a numeric `id`; every response echoes it. Push events (`module_changed`, `compile_error`, `ready`) carry no `id`.

## CLI

```sh
ship runtime install       # populate ~/.ship/runtime/ from local sources
ship runtime update        # wipe + reinstall
ship runtime doctor        # verify every required component is present
just dev-build-all         # run one-shot build via bundler.ts
```

`runtime install` resolves `bun` from `--bun <path>` or `which bun` on `PATH`; `sui-compiler` from `--sui-compiler <path>` or a walk-up search for `target/release|debug/sui-compiler` from cwd; `bundler.ts` from `--bundler-ts <path>` or a walk-up search for `runtime/bundler.ts` from cwd. `update` wipes the runtime dir then reinstalls.

`runtime doctor` checks: runtime dir exists → bun present and executable → sui-compiler present and executable → bundler.ts present.

## Errors

Every error in `ship_application::toolchain` carries a recovery hint in its `Display` impl:

- **"Ship toolchain not installed at …"** → `ship runtime install`
- **"Ship toolchain is incomplete: missing …"** → `ship runtime install` (idempotent; repairs partials)
- **"Ship toolchain binary … is not executable"** → `ship runtime install` (resets +x bits)
- **"bundler.ts not found at …"** → `ship runtime install` or `ship runtime update`

## See also

- `docs/ship-sui/SKILL.md` — the `.sui` edit → browser update path via the Bun worker is documented under the Tooling section there.
- `crates/core/ship-application/src/toolchain/` — the Rust orchestration code.
- `crates/core/ship-application/src/bundler/` — the worker supervisor and IPC layer.
- `runtime/bundler.ts` — the worker source.
