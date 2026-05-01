# Working on the SUI codebase

Read this when changing `crates/sui-compiler/`, `packages/ui-runtime/`, `packages/ui-dom/`, `packages/ui-lsp/`, the daemon's SUI handlers (`apps/shipd/src/views/sui_*.rs`), or any `views/<name>/`. Authoring views? Use `sui-authoring.md`. This page is for framework internals.

## Dev loop — three steps, all required

JS bundle changes only land in the running daemon after:

1. `node scripts/build-bundles.mjs --force` — rebuilds `apps/shipd/assets/sui-*.js` and per-view `.js` from TS sources.
2. `cargo build -p shipd` — Rust `include_str!`s the bundles at compile time, so it must rebuild after step 1.
3. `bash scripts/dev/restart.sh` — cycles the per-worktree daemon.

Skipping any step ships stale bytes. The `include_str!` call sites are `apps/shipd/src/views/sui_view.rs` and `apps/shipd/src/views/sui.rs`.

## Tests — run all four when changing the framework

| Surface | Command |
|---|---|
| Compiler | `cargo test -p sui-compiler` |
| Runtime | `pnpm -F @ship/ui-runtime test` |
| DOM / attach | `pnpm -F @ship/ui-dom test` |
| LSP | `pnpm -F @ship/ui-lsp test` |
| Browser smoke | `node .artifacts/sui-counter-probe.mjs` (Playwright probe; daemon must be up) |

## Generated artifacts (gitignored, sit next to source)

Don't commit. Don't be surprised by:

- `views/<name>/view.js` — sui-compiler emit; the paired controller is `view.ts`. Vitest prefers `.ts` via `packages/ui-dom/vitest.config.ts::resolve.extensions`.
- `packages/ship-set-components/src/<tag>/<tag>.compiled.js` — component emit; the paired `.ts` side-effect-imports it.
- `**/*.sui.d.ts`, `**/*.html.set.d.ts` — type declarations the compiler emits alongside source.

## Adding a runtime export to `@ship/ui-runtime`

Each public symbol gets its own file in `packages/ui-runtime/src/<name>.ts` (mirrors `signal.ts`, `state.ts`, `controller.ts`, `view.ts`). Re-export from `index.ts`. Don't bolt onto unrelated files like `controller-resolver.ts` — that's how parallel implementations get created.

## Compile shape contract — don't conflate

- `view(setup)` from `@ship/ui-runtime`: the compiler emits `export default view((ctx) => {...})` for every `.sui` view module. Wraps a setup function as `{ setup }` so `attach()` can call `module.default.setup(ctx)`.
- `controller(def)`: the paired `view.ts` exports `controller<Scope>({ derived, on, use, mount, unmount })` for tier-2/3 logic. Resolved by the compiled view's setup.
- They coexist. `view` ≠ `controller`.

## Marker contract

The SSR walker emits author-syntax markers (preserved attribute names + comment-bracketed text + custom-element boundary wrappers). attach() walks them and wires bindings against the manifest exported alongside `default`. Reference: `sui-authoring.md` § "SSR + attach pipeline" for the table of constructs → markers.

When adding a new control-flow construct: parse → IR → lower → render walker (emit `<sui-<name> key="bN">` boundary) → bindings manifest entry → attach() boundary wirer dispatches via the runtime helper. Mirror `{#if}` exactly.

## Fixture / test-corpus maintenance

Adding a parser feature means updating `crates/sui-compiler/tests/fixtures/*.sui` and `*.expected.js` snapshots in the same change. The CLI smoke (`tests/cli.rs`) and snapshot tests (`tests/snapshot.rs`) gate the emit shape — keep them current.

## Commit hygiene

Commit at every passing test gate. A multi-hour uncommitted session is a dead session — if anything crashes, work is gone.

## Discipline

When a test fails, fix root cause. Don't paper over with config workarounds. The vitest `resolve.extensions` reorder is an exception — it expresses real project layout (compiled `.js` next to source `.ts`), not a workaround.
