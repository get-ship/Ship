# bundler-hygiene — assessment

**Branch:** job/bundler-hygiene · **Base:** optionB-routes-unify (`b3250c3d0`)
**Outcome:** root-cause fix landed; `CwdBundleGuard` deleted; both targeted
suites green; manual build-assets leaves a clean tree.

## Root cause

`BundleLayout::assets_dir_for` routed every consumer-pack target (anything
not a `DECK_OWN_BUNDLES` singleton, `views/*`, or `sui-components/*`) to
`host_assets()` — `out_root.join(".")` — which on the standard bake is
the cwd / repo root. `deck build-assets` therefore wrote
`sui-marketing.js` / `sui-arcade.js` / `sui-pdf.js` /
`ship-code-editor.js` into the repo root on every run. The deck-cli
behavior smoke tests worked around this with a `CwdBundleGuard` drop
that removed those four files after the test ran, keeping the tree
clean at the cost of hiding the regression.

The doc comment on `assets_dir_for` already claimed the bake's contract
is "the bundler always writes under `<out_root>/.deck`" (`deck_out`
doc, `targets.rs:89-98`), and the ADR `in-process-rolldown-bundler`
(pinned the `bundler owns its output layout` invariant) — the consumer
fallthrough to `host_assets` was an old pre-dissolution carry-over from
when consumer packs were configured in a central `deck.jsonc`.

## Fix

`crates/deck/deck-bundler/src/targets.rs` — `assets_dir_for` now
returns `self.deck_out()` for everything that is not a
`DECK_OWN_BUNDLES` singleton. Framework singletons keep their
`.deck/framework/` staging path (the build.rs path that copies them
into `crates/deck/deck-core/assets/` for `include_str!`); all other
targets — `views/*`, `sui-components/*`, and the discovered consumer
packs — land directly under `.deck/`. The `host_assets()` helper is
gone (no other callers) and the `views/` / `sui-components/` prefix
check is dropped because it's no longer a fork — the rule "every
build artifact under `.deck/`" is uniform.

## Tests

**`assets_dir_routes_by_name`** — updated to assert
`/repo/.deck` (not `/repo`) for `sui-marketing`, `sui-arcade`, and a
generic consumer name. The new doc captures the contract.

**`emits_target_with_chunk_closure`** (`crates/deck/deck-bundler/src/emit.rs`) —
the test seeded `sui-marketing.js` and `Chunk-X.js` directly in the
fake repo root to simulate a prior bake. After the fix, the bake
places them in `<repo>/.deck/`, so the test seeds them there instead.
Asserts on the `deck emit` out-dir output are unchanged.

**`cli_smoke.rs`** — `CwdBundleGuard` (struct + `Drop` impl + the
section divider + two `let _cwd_guard = CwdBundleGuard::new();`
lines) deleted. Tests are otherwise untouched.

## Verification

| Step | Result |
|---|---|
| `cargo check -p deck-bundler` | OK |
| `cargo check -p deck-cli --tests` | OK |
| `cargo test -p deck-bundler --lib` (post-fix) | 56 passed |
| `cargo test -p deck-bundler --lib` (stashed baseline) | 56 passed — no new failures, no regression |
| `cargo test -p deck-cli` (no guard) | 7 passed |
| `cargo run -p deck-cli -- build-assets` | framework + consumer packs baked; **0 stray `*.js` in cwd** |
| Post-test cwd | `ls *.js` → not found |
| `cargo clippy -p deck-bundler -p deck-cli --all-targets` | no new warnings in touched files (pre-existing `routes.rs` / `sui_plugin.rs` / `cli_smoke.rs` fmt+clippy drift unchanged) |

Build artifacts after `deck build-assets`:

```
.deck/framework/sui-runtime.js
.deck/framework/sui-dom.js
.deck/framework/sui-sdk-client.js
.deck/sui-marketing.js          189 KB
.deck/sui-arcade.js             708 KB
.deck/sui-pdf.js               1.6 MB
.deck/ship-code-editor.js       889 KB
.deck/views/{artifacts,authority,create-space,eval,git,showcase}/view.js
```

Cwd is empty of stray `*.js` (the `CwdBundleGuard` is gone and the
tree stays clean by construction — proof the root fix works, not the
guard).

## Out-of-scope (pre-existing, not introduced by this job)

- `gen-contract-check` reports every contract doc as STALE on this
  worktree. Verified identical on `git stash` of my changes — pre-existing.
- `check-raw-ship-paths` lists 19 violations, none in my changed files.
  Pre-existing.
- `cargo clippy --workspace … -D warnings` fails on pre-existing warnings
  in `crates/sui/lint/src/rules/font_literal.rs` and
  `crates/deck/deck-bundler/src/{routes.rs,sui_plugin.rs}`. Pre-existing.
- `cargo fmt --check` reports wide drift in `deck-bundler/src/*.rs` and
  the cli smoke test. The asserts I touched are in the same style as
  the file's surrounding asserts (verified by stashing my changes and
  re-running).

None of these are touched by this branch; the spec's acceptance
(`cargo test -p deck-bundler`, `cargo test -p deck-cli`, no stray
`*.js`, no `CwdBundleGuard`) is met.
