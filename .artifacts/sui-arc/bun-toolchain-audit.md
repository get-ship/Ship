# Bun Toolchain Audit — Dev vs Prod Stories for SUI

Branch: `job/bun-toolchain-audit`. Scope: SUI/web build pipeline only — `packages/sui-build/`, `runtime-assets/bundler.ts`, `Justfile`, the dev scripts, and the shipd asset/HMR boot.

## TL;DR

The dev and prod stories share zero code today. They are two independent bundlers that happen to produce overlapping outputs:

| Pipeline | Implementation | Role | Invoked by |
|---|---|---|---|
| **`@ship/sui-build` CLI** | `packages/sui-build/` (sui-build.mjs → bun adapter or esbuild adapter) | Prod / CI build of `ui-components` + `ship-set` | `pnpm -F @ship/ui-components build`, `pnpm -F @ship/set build`, `just cem`, `smoke-deploy.sh` |
| **`runtime-assets/bundler.ts`** | Bun worker spawned by shipd over JSON-line stdin/stdout | Dev build + file watcher + HMR source for everything (islands, views, runtime, ship-set) | `shipd` daemon at start (`apps/shipd/src/lib.rs:214-261`); standalone via `bun runtime-assets/bundler.ts --all` |

The standalone bundler-worker (dev) is the one a user actually exercises during `just dev`. The `@ship/sui-build` CLI runs only when you explicitly invoke a `pnpm build` for one of the two packages that wire it up, or via `smoke-deploy.sh`. **Neither writes anywhere `shipd` reads at runtime that the other doesn't also write** — the bundler-worker is the source of truth for what the daemon serves.

`@ship/sui-build`'s two bundler adapters (Bun, esbuild) are a non-issue in practice: the path is rarely traversed in the inner loop and the Bun-vs-esbuild choice is invisible to the rest of the system.

The interesting friction is **not** between Bun and esbuild — it is between the **two parallel bundlers**, plus a few stale outputs and a documented dev flow that no recipe codifies.

---

## 1. Dev path — what `just dev-restart` actually does

`just dev-restart` resolves to `scripts/dev/restart.sh` → `scripts/dev/start.sh`:

1. `cargo build -p shipd -p ship-cli` (`start.sh:12`).
2. `stash_binary shipd` + `stash_binary ship` — atomic-copies the freshly built binaries to `$daemon_dir/.bin/`. Daemon dir is `~/.ship/machine/daemons/<sha256(repo_root)|head-c8>/`, so each worktree gets its own (`scripts/dev/_lib.sh:27-61`).
3. Spawn `$daemon_dir/.bin/shipd --dev --workspace <repo>` (`start.sh:35`). Port auto-derives; the daemon writes `$daemon_dir/port` once HTTP is bound.

Inside shipd's boot (`apps/shipd/src/lib.rs:214-261`):

4. `Bundler::start(home, repo_root)` (`crates/core/ship-application/src/bundler/mod.rs:32`) supervises a Bun child process that runs `~/.ship/runtime/bundler.ts`. The worker speaks JSON-line over stdin/stdout per `crates/core/ship-application/src/bundler/proto.rs`.
5. `bundler.start_watching()` is called. The worker installs `fs.watch` on `packages/` and `views/` (`runtime-assets/bundler.ts:356-398`).
6. Two SSE channels are wired in dev mode only (`apps/shipd/src/views/sui_hmr.rs:274-284`):
   - `/_sui/hmr` — ship-set HMR keyed off `notify`-watched `~/.ship/sui/*.set` writes.
   - `/_sui/dev/hmr` — bundle HMR. `ModuleChanged` / `CompileError` events from the bundler worker → `publish_bundle_event` → broadcast SSE.
7. Browsers fetch `/_sui/hmr-client.js` (inline in `sui_hmr.rs:205-243`), which subscribes to both channels and triggers a cache-busted reload of `/_sui/views/<name>/view.js`.

Asset serving (`crates/ship-sui-server/src/assets.rs:14-95`): every `read_asset(rel)` first reads `crates/ship-sui-server/assets/<rel>` off disk; falls back to the `include_str!` constant if absent. **So in dev, a bundler-worker rebuild becomes visible without restarting shipd.** Same try-disk-then-include pattern for `view-host.html` (`view_host.rs:75-89`).

`apps/shell2/vite.config.ts` is a thin outer-shell vite that proxies `/api`, `/_sui`, `^/sui/.*`, `/view-host` to the daemon (`vite.config.ts:104-109`). It runs only if a user invokes `pnpm --filter shell2 dev` (the `scripts/dev/shell2-start.sh` helper exists, but no Justfile recipe wires it up). `just dev` alone does **not** start vite — shipd serves a full UI on its own port.

Net dev loop: `.sui` edit → `fs.watch` fires → bundler worker re-runs `Bun.build` with `sui-compiler` subprocess transform → writes to `crates/ship-sui-server/assets/...` → emits `ModuleChanged` over stdout → shipd rebroadcasts on SSE → HMR client reloads.

---

## 2. The two parallel bundlers

### 2a. `@ship/sui-build` CLI — the prod story

- Entry: `packages/sui-build/bin/sui-build.mjs` → spawns `bun dist/cli.js` (the `#!/usr/bin/env node` shebang is misleading; `bin/sui-build.mjs:7` always invokes bun, and `loadConfig` aborts if not under bun: `src/cli.ts:83-87`).
- Reads `<package>/sui.config.ts` (default export — `src/config.ts:SuiBuildConfig`).
- Per-file compile via the Rust `sui-compiler`. Prefers FFI through `bun:ffi` against `target/debug/libsui_compiler.{so,dylib,dll}` (`adapters/bun.ts:14-57`); falls back to `execFile(target/debug/sui-compiler …)` or `cargo run -p sui-compiler` (`runner.ts:84-97`). FFI shaves the subprocess startup per `.sui` file — meaningful with N>50 files. Disable with `SUI_BUILD_DISABLE_FFI=1`.
- Then runs **one** of the two bundler adapters:
  - `bun.ts` — `Bun.build()`, supports the Bun loader set (`BUN_LOADERS`, line 59-61). `process.env.NODE_ENV` is injected via `define` (`bun.ts:97-100`) so Lit picks the production export condition without polluting the host env.
  - `esbuild.ts` — `esbuild.build()`, supports the esbuild loader set. Sets `target: 'es2020'` (Bun adapter leaves target as default 'browser').
- Adapter selection (`cli.ts:67-80`):
  1. `--bundler bun|esbuild` flag wins.
  2. Else `SUI_BUILD_BUNDLER` env wins.
  3. Else: bun if `process.versions.bun` is truthy.
  4. Else: warn + fall back to esbuild.
  Since the wrapper always spawns bun, step 3 always succeeds in practice — **esbuild is the dead branch in normal use**, picked up only when `--bundler esbuild` or `SUI_BUILD_BUNDLER=esbuild` is set, or when something other than `bin/sui-build.mjs` re-invokes `cli.ts` from a Node process.

Wired-up consumers:
- `packages/ui-components/package.json` — `"build"` and `"pretest"` both run `sui-build`. `sui.config.ts` outdir: `crates/ship-sui-server/assets`. CEM dest: `crates/ship-sui/assets/sui-custom-elements.json` (correct — that's what `apps/shipd/src/views/assets.rs:42` `include_str!`s).
- `packages/ship-set/package.json` — `"build"` runs `sui-build`. Outdir: `apps/shipd/assets`.

Invoked from:
- `just cem` — only `pnpm -F @ship/ui-components build`.
- `scripts/dev/smoke-deploy.sh:33-37` — `pnpm -F @ship/set build && pnpm -F @ship/ui-components build` before daemon restart. The local CI gate.

Not invoked during `just dev` / `just dev-restart`.

### 2b. `runtime-assets/bundler.ts` — the dev story (and what shipd actually depends on)

- 526-line Bun script. Installed to `~/.ship/runtime/bundler.ts` by `ship runtime install` (copy-from `runtime-assets/bundler.ts`, sources resolved by walking up from cwd: `crates/core/ship-application/src/toolchain/install.rs:139-152`).
- Spawned by shipd's supervised worker (`crates/core/ship-application/src/bundler/worker.rs:143-206`). Auto-restarts with exponential backoff. If `~/.ship/runtime/bun` or `bundler.ts` is missing the supervisor logs once and stays idle (`worker.rs:155-163`) — no spawn loop.
- Hard-codes a target list (`bundler.ts:123-138`) plus discovery for `packages/ui-components/src/islands/*` and `views/<name>/page.sui`. Every target is a `Bun.build()` with a custom `suiPlugin` that loads `.sui` files by spawning `sui-compiler --no-dts`.
- Writes outputs to:
  - `crates/ship-sui-server/assets/sui-*.js` (sui-components, sui-runtime, sui-dom, sui-sdk-client, lazy island bundles, view bundles).
  - `apps/shipd/assets/*.js` (ship-elements + ship-* island bundles).
- Watches `packages/` and `views/` recursively; 150ms-per-target debounce; emits `module_changed` / `compile_error` events on the SSE channel.
- Has a `--all` CLI flag (`bundler.ts:420-439`) for one-shot rebuilds. **Not currently invoked by any Justfile recipe.**

### 2c. The actual overlap

Both pipelines target the same output trees but with subtly different paths:

| What | `sui-build` writes | `runtime/bundler.ts` writes | Served? |
|---|---|---|---|
| `sui-components` main bundle | `crates/ship-sui-server/assets/sui-components.js` | same | yes |
| Islands (sui-terminal, sui-code-editor, …) | `crates/ship-sui-server/assets/sui-<name>.js` (flat — per `sui.config.ts lazyBundles`) | `crates/ship-sui-server/assets/sui-islands/sui-<name>.js` | **only `sui-islands/<name>.js`** (`crates/ship-sui-server/src/assets.rs:26-32`). The top-level files emitted by sui-build are **dead artifacts**. |
| `sui-runtime`, `sui-dom`, `sui-sdk-client` | (not configured) | `crates/ship-sui-server/assets/sui-{runtime,dom,sdk-client}.js` | yes |
| View bundles (`views/<name>/view.js`) | (not configured) | `crates/ship-sui-server/assets/views/<name>/view.js` | yes (dev disk, prod include_str! for agents + git only) |
| `ship-elements` + island bundles | `apps/shipd/assets/ship-*.js` | same dir, different naming for islands | partial overlap |
| Aggregated CEM | `crates/ship-sui/assets/sui-custom-elements.json` (canonical — included by shipd) | `crates/ship-ui-framework/assets/sui-custom-elements.json` — **target crate does not exist**; write fails or hits a stray path. Stale post-rename. (`bundler.ts:217`) |

`git status` on the audit branch confirms the duplication: every island has both `crates/ship-sui-server/assets/sui-<name>.js` (untracked, written by sui-build) and `crates/ship-sui-server/assets/sui-islands/sui-<name>.js` (modified, written by bundler.ts). Only the second one is served.

### 2d. Bun vs esbuild adapter — the original audit question

In practice the choice is invisible:

- **Speed**: never measured in this repo (no benchmark, no CI gate). Anecdotally Bun.build is faster on a cold start in Bun than spinning up esbuild via Node; with the wrapper always spawning bun the contest is fair. Per-file dominates either way — `sui-compiler` runs N times.
- **Feature parity**: both adapters share `mirrorResolverPlugin` (`adapters/import-rewrite.ts`) which rewrites `.compiled.js` imports to the mirror tree under `<repo>/.ship/sui-out/` and re-anchors `@ship/*` to workspace sources. Loader sets differ (esbuild has more: `copy`, `file`, `empty`, `global-css`, `local-css`) but neither sui.config.ts uses anything outside the common subset. Source-map handling is `inline | none` in both.
- **Bun-only difference**: NODE_ENV injection via `define` so Lit picks its production export condition. esbuild ignores this — its `minify: true` mode does drop dead branches, but if you ever ship a non-minified production build via esbuild you get the Lit dev path. Not a current failure mode because nothing builds with esbuild in CI.
- **Real-world divergence between adapters**: zero. Both bundles passed through smoke-deploy if you set the env. The dual-adapter abstraction exists for fallback when Bun is unavailable — a defensible insurance policy, not a routine choice.

The actual fan-out of bundler implementations the team maintains is **not bun-vs-esbuild — it's `@ship/sui-build` vs `runtime-assets/bundler.ts`**. The two `sui-build` adapters are noise compared to that.

---

## 3. Prod build — `pnpm build` + `cargo build --release`

There is no single `pnpm build` at the repo root. The closest end-to-end production path is `scripts/dev/smoke-deploy.sh`:

```
cargo build                                # workspace, unstable features
cargo run -p xtask -- gen-types            # specta → packages/*/src/generated.ts
pnpm install --frozen-lockfile
pnpm -F @ship/set icons                    # generate lucide-icons.generated.ts
pnpm -F @ship/set build                    # sui-build → apps/shipd/assets/
pnpm -F @ship/ui-components build          # sui-build → crates/ship-sui-server/assets/
just dev-restart                           # cargo build -p shipd -p ship-cli + stash + spawn
node scripts/dev/smoke-views.mjs --port    # HTTP-200 every view via daemon
pnpm test:e2e:visual                       # playwright vs PNG baselines
```

`just build-release` produces stable release binaries:

```
cargo build --release -p ship-cli -p mcp -p shipd
```

The release binaries `include_str!` the assets bundled in `crates/ship-sui-server/assets/` and `apps/shipd/assets/` at compile time. **Production correctness depends entirely on those committed files being up to date when `cargo build --release` runs.** There is no production-mode bundler step. The dev-only bundler worker is what keeps those files current on developer machines; pre-merge that work falls to `smoke-deploy.sh` (or the manual `just cem`).

This is the actual prod story: **commit the bundles, ship the binary**. It works because the bundles live in `crates/` and are read via `include_str!`. It is fragile because nobody runs `pnpm -F @ship/ui-components build` in CI — `smoke-deploy.sh` is a contributor-local gate.

---

## 4. Fresh-clone bootstrap (clone → `just dev`)

Today the steps are spread across the README, `docs/ship-contributing/references/docs/development.md` (out of date — describes a retired TanStack Start / D1 stack), and the Justfile. Consolidated reality:

```sh
# Prereqs on host: rust toolchain (rust-toolchain.toml pins channel),
# bun (curl -fsSL https://bun.sh/install | bash), pnpm, just.

git clone <repo> && cd ship

# Generated TS bindings the workspace needs to resolve at install time:
cargo run -p xtask -- gen-types

# Generated lucide icon registry the ship-set build needs:
cargo build -p ship-cli                    # bootstrap target/debug/ship
pnpm install
pnpm -F @ship/set icons

# Bundle assets so include_str! has something to ingest on first cargo build:
pnpm -F @ship/set build
pnpm -F @ship/ui-components build

# Populate ~/.ship/runtime/ — bun, sui-compiler (release), bundler.ts:
just dev-runtime-install

# Now you can start the dev daemon:
just dev
```

The `dev-runtime-install` recipe (`Justfile:47-58`) currently builds `sui-compiler --release` and `ship-cli --debug`, then runs `ship runtime install --bun $BUN_PATH --sui-compiler target/release/sui-compiler`. `bundler.ts` is auto-located by walking up to `runtime-assets/bundler.ts` (`toolchain/install.rs:139-152`).

Pain points a new contributor will hit:
1. **No single bootstrap command.** Five separate ordered steps, easy to skip one and discover the breakage at build time.
2. **`pnpm -F @ship/set icons` is a hidden prereq.** Without it `@ship/set build` fails on missing `lucide-icons.generated.ts`. Documented inline in `smoke-deploy.sh:24-28` but not in any user-facing doc.
3. **`just dev` only starts shipd.** The docs (`docs/ship-sui/references/tooling.md:174`) claim it brings up "daemon + bundler + shell2 in watch mode" — false. shipd embeds the bundler worker; vite (`shell2`) is a separate `scripts/dev/shell2-start.sh` that no Justfile recipe surfaces.
4. **`runtime-assets/bundler.ts` aggregateCem destination is broken** (`bundler.ts:217` → `crates/ship-ui-framework/assets/sui-custom-elements.json`, crate does not exist). Dev never refreshes the served CEM; you only see CEM updates after a `pnpm -F @ship/ui-components build`. Out of scope to fix here, but documenting it for the next pass.
5. **No prod-time bundler invocation.** `cargo build --release` succeeds with whatever bundles are committed. There is no CI gate ensuring `crates/ship-sui-server/assets/*.js` is the result of a clean rebuild against current sources.

---

## 5. Dead code / stale artifacts

| Item | Location | Note |
|---|---|---|
| `targetsForFile`'s `precompile` predicate | `runtime-assets/bundler.ts:347` | `BundleTarget` has no `precompile` field. Always falsy; dead. |
| `aggregateCem` destination | `runtime-assets/bundler.ts:217` | Writes to `crates/ship-ui-framework/assets/` — crate does not exist. Stale post-rename to `ship-sui`. |
| Top-level island bundles | `crates/ship-sui-server/assets/sui-<name>.js` | Emitted by `@ship/sui-build` via `packages/ui-components/sui.config.ts`'s flat `lazyBundles`. Not referenced by `assets.rs`; only `sui-islands/<name>.js` (written by `runtime/bundler.ts`) is served. Dead artifact path. |
| esbuild adapter | `packages/sui-build/src/adapters/esbuild.ts` | Reachable only via `--bundler esbuild` or `SUI_BUILD_BUNDLER=esbuild`. Never selected automatically because the wrapper spawns bun. Carrying its weight as a fallback only. |
| `docs/ship-contributing/references/docs/development.md` | whole file | Describes TanStack Start + D1 Drizzle + Cloudflare Worker shell — none of which match current reality (shell2 + vite + shipd). Out of scope for this branch. |
| `docs/ship-sui/references/tooling.md` end-to-end dev-loop block | `tooling.md:172-183` | Claims `just dev` starts "daemon + bundler + shell2"; actually only the daemon (which embeds the bundler worker). Fixed in this pass. |

---

## 6. Recommendations

Ordered by leverage.

1. **Pick one bundler.** The repo doesn't actually need both `@ship/sui-build` and `runtime-assets/bundler.ts`. The dev-time worker is the source of truth for what shipd serves; the standalone CLI ships the same outputs only in a CI-adjacent path. Two reasonable directions:
   - **Drop `@ship/sui-build`**, fold the loader/CEM-aggregate handling into `runtime-assets/bundler.ts`, and call `bun runtime-assets/bundler.ts --all` from `smoke-deploy.sh` and a new `just assets-build` recipe. -1k LOC; one mental model for both paths. (Out of scope here — touches `packages/ui-components/package.json` and others.)
   - **Drop `runtime-assets/bundler.ts`**, replace it with `sui-build --watch` over the union of all targets (sui-runtime, sui-dom, islands, views), and have shipd spawn `sui-build --watch` instead of a custom Bun worker. Keeps the dual-adapter abstraction. Requires extending `sui-build` to know about views/* and watch HMR events.
   The first option is the smaller refactor and the better fit for the "edit your own apps" user story — third parties never want a daemon-internal worker; they want a CLI they can invoke.

2. **Make `--all` reachable from `just`.** `runtime-assets/bundler.ts --all` already exists. A `just dev-build-all` recipe gives users a way to refresh every served asset without poking the SSE channel via edits. Done in this pass.

3. **Wire shell2 into `just`.** `scripts/dev/shell2-start.sh` exists but no Justfile recipe surfaces it. Added `just shell2-dev` in this pass.

4. **Add a `just bootstrap` recipe** that does the fresh-clone sequence in one shot. Added in this pass.

5. **Fix the `aggregateCem` path in `runtime-assets/bundler.ts:217`.** Out of allowed scope on this branch (file isn't in the allow-list), but the one-line `'crates/ship-ui-framework/'` → `'crates/ship-sui/'` change should land next.

6. **Stop emitting flat-layout island bundles from `sui-build`.** Either prefix the `lazyBundles` keys in `packages/ui-components/sui.config.ts` with `sui-islands/`, or delete `lazyBundles` and rely on `runtime/bundler.ts`'s island discovery. Out of allowed scope here.

7. **Add a CI gate** that runs `pnpm -F @ship/set build && pnpm -F @ship/ui-components build && git diff --exit-code crates/ship-sui-server/assets apps/shipd/assets` to catch committed-bundle drift. Out of scope; needs `.github/workflows/` touch.

---

## 7. Targeted fixes landed on this branch

Both fixes are inside the spec's allow-list (`Justfile`, `docs/ship-sui/references/tooling.md`, `README.md`):

1. **`Justfile`** — three new daily recipes:
   - `bootstrap` — one-shot fresh-clone setup.
   - `shell2-dev` — wraps `scripts/dev/shell2-start.sh`.
   - `dev-build-all` — runs `bun runtime-assets/bundler.ts --all` to rebuild every served asset from source.

2. **`docs/ship-sui/references/tooling.md`** — folded in the dev-flow correction (just dev starts shipd-with-embedded-bundler, not shell2) and a short pointer to `@ship/sui-build`'s actual role (the CI/prod-side bundler, not the dev-side one).

`README.md` — bootstrap section already covers the user-facing install. The contributor-side bootstrap is added in the Justfile recipe rather than the README to keep the README as the user-facing install doc.

No code changes to `packages/sui-build/`. The audit found no allow-list-scoped code change with positive ROI; the high-leverage fixes are all gated by the `crates/` / `packages/ui-components/` / `runtime-assets/` boundaries this branch is forbidden from touching, and are listed under Recommendations §5 / §6 for a follow-up.
