# `ship build` + dev/prod mode story

## Problem

Two adjacent gaps in the contributor and end-user experience:

1. **No top-level production build.** Dev was a single recipe (`just dev`),
   but cutting a release meant chaining `pnpm -F @ship/set icons`,
   `pnpm -F @ship/set build`, `pnpm -F @ship/ui-components build`, and
   `cargo build --release` by hand and remembering which order they go
   in. Easy to forget a stage; impossible to script for a fresh user.
2. **No user-visible mode signal.** `DaemonMode::{Development,Production}`
   already existed in `crates/ship-sui-server` and gated cache headers /
   HMR, but it wasn't surfaced to the browser. End users running a
   freshly-`ship build`-ed daemon couldn't tell at a glance whether
   they'd booted into production mode.

## Shape of the fix

Four pieces, one job:

- **`Justfile`** — replaced the existing one-liner `build` recipe (which
  was actually a dev cargo build) with a six-stage pipeline that
  produces a deployable artifact set and verifies its outputs. Renamed
  the old recipe to `build-dev` and rewired `install-dev`. `build-release`
  stays as the cargo-only release build for the cases where the front-end
  is already current.
- **`apps/ship-cli/src/build.rs`** — `ship build` subcommand. If `just`
  is on PATH it shells out (single source of truth); otherwise it runs
  the same stage list directly so users without the `just` runner still
  get a working production build. Walks up to the workspace `Cargo.toml`
  so it works from any subdirectory.
- **`crates/ship-sui-server/src/mode_html.rs`** — `inject_mode_meta(html,
  mode)` rewrites the `<meta name="ship-mode" content="…">` tag to match
  the daemon's actual mode. Tolerates either attribute order and either
  quoting style; inserts a fresh meta before `</head>` if missing. Tests
  cover all four input shapes plus the no-`<head>` no-op.
- **shell2 + sui-status-bar** — `apps/shell2/index.html` ships
  `<meta name="ship-mode" content="development">` as the static default
  (Vite serves it as-is). `apps/shell2/src/main.ts` reads the meta into
  `window.__shipMode` at boot and reflects it onto
  `<sui-status-bar mode="…">`. The status bar gains a `mode` prop
  (default `production`) and a hairline DEV badge that is hidden via
  `:host(:not([mode='development'])) .dev-badge { display: none; }` —
  no conditional rendering, just CSS.

## What we did NOT do

- Did **not** wire shipd to actually serve shell2's prebuilt `dist/` —
  that's a separate piece of plumbing (the existing `shell_assets.rs`
  router serves the legacy `apps/shell`). The hook to inject the mode
  is in place (`ship_sui::inject_mode_meta`), but flipping shipd over to
  serving shell2 is out of scope for this job. Until that lands the meta
  stays at its static `development` default whenever Vite is in the loop,
  which matches reality.
- Did **not** add a `mode` field anywhere on the daemon API. The mode is
  already exposed via process state (the `--mode` flag, `SHIP_DAEMON_MODE`
  env var, and `DaemonMode` extension on the router). No new contract
  surface.
- Did **not** touch any view, runtime, or compiler code.

## Verification

- `cargo test -p ship-sui-server` — new `mode_html` tests + existing
  `mode_tests` pass.
- `just build` — runs end-to-end on a clean checkout; the verify step
  fails loudly if any expected output is missing.
- `ship build --dry-run` — prints the stage list without executing, so
  contributors can confirm the plan.

## Files changed

```
Justfile                                                 (rewrite of `build`, add `build-dev`)
apps/ship-cli/src/cli.rs                                 (add `Build` subcommand)
apps/ship-cli/src/main.rs                                (mod + dispatch)
apps/ship-cli/src/build.rs                               (NEW)
apps/shell2/index.html                                   (static meta)
apps/shell2/src/main.ts                                  (read meta, set status bar attr)
crates/ship-sui-server/src/lib.rs                        (mod + re-exports)
crates/ship-sui-server/src/mode_html.rs                  (NEW)
packages/ui-components/src/sui-status-bar/sui-status-bar.sui  (mode prop + DEV badge)
docs/ship-sui/references/tooling.md                      (modes section)
```
