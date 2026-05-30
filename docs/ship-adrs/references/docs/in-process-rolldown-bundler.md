---
title: "In-Process Rolldown Bundler"
description: "ADR: the daemon bundles SUI assets in-process via Rolldown (oxc) with a native Rust resolve/load plugin, replacing the spawned bun worker. Pins Rust to 1.95+. bun leaves the deployment runtime; it remains a dev/build-time tool."
group: ADRs
order: 50
audience: internal
status: accepted
date: 2026-05-27
supersedes: []
tags: [bundler, rolldown, bun, toolchain, deployment, hmr]
---

# In-Process Rolldown Bundler

## Context

The daemon's asset bundler was a **long-lived `bun` worker** (`packages/deck/
bundler/index.ts`, installed to the runtime dir as `bundler.ts`) that `shipd`
spawned and drove over a JSON-line stdin/stdout protocol, supervised with
exponential-backoff restarts. It used `Bun.build` + a `Bun.plugin` that compiled
`.sui` on the fly (via `sui-compiler`).

Two problems:

1. **Deployment.** Every end-user received a ~100 MB `bun` binary in
   `~/.ship/runtime/` purely to run the worker. bun in the shipped bundle is
   weight + a non-Rust runtime dependency in the daemon's hot path.
2. **The bundler should be a port, not a chatty subprocess.** The JSON-line
   protocol + supervisor were machinery around a build step.

The hard requirement that shaped the choice: bundling needs **custom resolve +
load hooks** (redirect `./X.compiled.js` → compile the `.sui`), callable from
Rust, without hosting a JS runtime. That single need disqualified most options.

### Alternatives evaluated (all spiked)

| Option | Rust plugins | Builds on our stable | Maintained | Verdict |
|---|---|---|---|---|
| **esbuild** (Go bin) | ❌ CLI has no plugins | ✅ | ✅ | umbrellas need in-place compile (source-tree smell) |
| **swc_bundler** | ✅ Load/Resolve | ✅ | ❌ deprecated | proved the model; dead-end to ship on |
| **Farm** | ✅ | ❌ edition-2024/let-chains break on our rustc | ✅ | unbuildable here |
| **Rolldown** | ✅ best (Vite's engine) | ✅ on **1.95** | ✅ | **chosen** |

The umbrella aggregates (`sui-components`, 119 transitive `./X.compiled.js`
imports + cross-package `@ship/primitives`) require a real resolver — proven by
bundling them through swc_bundler. Rolldown gives that resolver as a native Rust
plugin. Its only catch — "nightly-blocked" — was a **toolchain-version lag**:
oxc uses `if let` guards, stabilized between our pin (1.93.1) and current stable
(1.95.0). No nightly, no `RUSTC_BOOTSTRAP`; just bump the pin.

## Decision

The daemon bundles **in-process via Rolldown**. The bundler is a deck crate
(`crates/deck/deck-bundler/`, #167 — deck is the framework and owns serve +
bundle; it originally landed in `ship-platform` and was moved):

- A native Rust Rolldown `Plugin` (`rolldown_bundle.rs`): `resolve_id`
  externalizes the framework singletons, redirects `.compiled.js` siblings, and
  re-anchors `@ship/<pkg>` to workspace `src/`; `load` spawns `sui-compiler` to
  compile `.sui` → JS. Rolldown handles TS transpile + node resolution natively.
- `Bundler::{build, build_all, start_watching, subscribe_events, shutdown}` —
  async, in-process. `start_watching` uses `notify` and emits `BundlerEvent`s
  for HMR. Output to `assets_dir_for` (sui-out/dist); no source-tree writes.
- **`rust-toolchain.toml` pinned to 1.95.0+** (Rolldown/oxc requirement).
- The bun worker + `bundler.ts` + the JSON-line protocol + the esbuild
  exploration are deleted. The runtime dir holds only `sui-compiler`.

Rolldown can live in-tree because it uses **oxc**, not swc, so it avoids the
`unicode-width`/`ratatui` conflict that would have forced swc into a separate
binary.

## Consequences

- **bun leaves the deployment bundle.** The shipped daemon + runtime dir are
  bun-free. The CI asset bake also moved off bun: `cargo xtask build-assets`
  drives the same in-process Rolldown bundler, so `bundler/index.ts` is deleted.
  bun remains **dev/build-time only** for the `apps/shell` SPA build (via
  `shell-app.ts`), the `@ship/sui-build` package builds, and the lucide
  icon-registry gen. vite is dev-only (shell HMR) and never ships.
- **HMR** for daemon-served surfaces is all-Rust: in-process notify watch →
  `BundlerEvent` → `sui_hmr` SSE (`/_sui/dev/hmr`) → `live-reload.js`.
- **No subprocess / protocol / supervisor** — a build is an async call.
- **Toolchain obligation:** the workspace now requires Rust 1.95+. Future oxc
  bumps may move this floor.
- **Follow-on (done):** the build-time asset bake was ported to
  `cargo xtask build-assets` (calls `Bundler::build_all`), and `bundler/index.ts`
  (the bun bundler, a parallel SSOT to `targets.rs`) is deleted. The bake and the
  daemon now share one bundle-target definition and one engine.
- **Follow-on (open):** end-to-end daemon serve + HMR verification (was blocked
  by a dev-db migration crash).

Full exploration log: `.artifacts/bundler-seam.md` (working notes; this ADR is
the durable record).
