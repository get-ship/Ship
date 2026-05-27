# Toolchain — `~/.ship/runtime/`

Ship's asset bundler runs **in-process inside the daemon** (Rolldown, compiled
into `shipd`). The only thing the runtime dir needs is the `sui-compiler` binary
the bundler spawns to compile `.sui` files. It mirrors what `rustup` does for
Rust: one well-known dir, pinned to the Ship version that installed it.

> History (#163): this used to be a bundled `bun` binary + a long-lived
> `bundler.ts` worker the daemon spawned and drove over a JSON-line stdin/stdout
> protocol. That worker is **retired** — the daemon now bundles in-process via
> Rolldown, so there is no child process, no protocol, and no bun in the
> deployment bundle.

## Layout

```
~/.ship/runtime/
  sui-compiler        # Rust release build of crates/sui/compiler
```

The directory is owned by `crates/core/ship-substrate/src/home/ship_home.rs` —
every accessor (`runtime_dir`, `runtime_sui_compiler`) lives there; no other code
constructs paths under the runtime dir directly.

## How bundling works now

The daemon's `Bundler` (`crates/core/ship-platform/src/bundler/`) is in-process:

- **Engine:** Rolldown (oxc-based, Rust-native), embedded in `ship-platform`.
  Requires the pinned toolchain to be Rust **1.95+** (oxc uses `if let` guards,
  stabilized in 1.95) — see `rust-toolchain.toml`.
- **SUI plugin** (`rolldown_bundle.rs`): a native Rust Rolldown `Plugin`.
  `resolve_id` redirects `./X.compiled.js` siblings, externalizes the framework
  singletons (`@ship/sui-runtime`/`-dom`/`sdk-client`), and re-anchors
  `@ship/<pkg>` to workspace `src/`. `load` spawns `sui-compiler` to compile
  `.sui` → JS. Rolldown handles TS transpile + node resolution natively.
- **Output:** `assets_dir_for` — `crates/deck/deck-core/assets` for the baked
  framework bundles, `.deck/` for views/components/spaces. No source-tree writes.
- **API** (`bundler/mod.rs`): `Bundler::start` → `build(target)` / `build_all`
  (+ catalog aggregation) / `start_watching` / `subscribe_events` / `shutdown`,
  all async, in-process.

## HMR

`start_watching` runs a `notify` watcher over the source dirs; on a `.sui`/
`.ts`/`.js` change it re-bundles the affected targets and emits a
`BundlerEvent` (`ModuleChanged` / `CompileError`) on a broadcast channel. The
daemon (`apps/shipd/src/lib.rs`) forwards these to `views::sui_hmr` →
`/_sui/dev/hmr` SSE → the `live-reload.js` client baked into
`crates/deck/deck-core/assets`. All Rust, no bun, no vite.

(The shell SPA's *dev* server uses vite for its own HMR — `apps/shell` `bun run
dev`. That's dev-only; the prod shell ships as a pre-built static bundle. vite
never ships in the daemon or the shell bundle.)

## CLI

```sh
ship runtime install   # populate ~/.ship/runtime/ (sui-compiler) from local sources
ship runtime update    # wipe + reinstall
ship runtime doctor    # verify sui-compiler is present + executable
```

`runtime install` resolves `sui-compiler` from `--sui-compiler <path>` or a
walk-up search for `target/{release,debug}/sui-compiler` from cwd. `update`
wipes the runtime dir then reinstalls.

## Errors

Every `ship_platform::toolchain` error carries a recovery hint in `Display`:

- **"Ship toolchain not installed at …"** → `ship runtime install`
- **"Ship toolchain is incomplete: missing sui-compiler"** → `ship runtime install`
- **"… is not executable"** → `ship runtime install` (resets +x)

## bun, today

bun is **dev/build-time tooling only**, never shipped:
- `apps/shell` prod build (`sui-build && bun shell-app.ts`) and dev server (vite).
- the `@ship/sui-build` CLI (`packages/deck/build`) + `packages/deck/bundler/
  index.ts --all` for baking the framework assets at CI time. (Porting that bake
  to `Bundler::build_all` via an xtask is the remaining follow-on to drop bun
  from the contributor build too.)

## See also

- `crates/core/ship-platform/src/bundler/` — the in-process Rolldown bundler.
- `crates/core/ship-platform/src/toolchain/` — runtime-dir install/ensure.
- `docs/ship-adrs/references/docs/in-process-rolldown-bundler.md` — the ADR
  (decision + alternatives + consequences).
