# Working on the SUI codebase

Read this when changing `crates/sui-compiler/`, `crates/sui-lint/`, `crates/lsp/`, `packages/ui-runtime/`, `packages/ui-dom/`, the daemon's SUI handlers (`apps/shipd/src/views/sui_*.rs`), or any `views/<name>/`. Authoring views? Use `template-surface.md`. This page is for framework internals.

## Crate map

| Crate | Role |
|---|---|
| `crates/sui-compiler` | Parses `.sui` → IR → emits `<stem>.compiled.js` + `<stem>.sui.d.ts` into a sibling `dist/`. Owns the `check/` phase (build-time hard walls). |
| `crates/sui-lint` | Lints `*.controller.ts`. Same rule walker drives editor diagnostics and the compiler check. |
| `crates/lsp` | Single LSP for both stacks. Dispatches by `Stack` enum — j2 → `lint`, SUI → `sui-lint`. Distributed as `ship lsp`. |
| `packages/ui-runtime` | Signal engine (`signal`/`derived`/`effect`/`untrack`/`batch`), bindings (`bindText`/`bindAttr`/`bindClass`/`bindValue`/`bindHtml`/`bindIf`/`bindFor`), template hooks. |
| `packages/ui-dom` | `attach()` — turns SSR markers into live bindings against the compiler-emitted manifest. |
| `crates/core/lint` | j2 stack lint engine. |
| `crates/core/compiler` | Agent-config compiler — emits provider-specific configs (`.claude/settings.json`, `.claude/lsp.json`, `.codex/config.toml`, …). |

## Adding a SUI lint rule

Rules live in `crates/sui-lint/src/rules/`. The same rule definition powers two consumers: the LSP serves it live to the editor and the agent; the sui-compiler check phase escalates `Severity::Error` rules to build-time hard walls (`Severity::Warn` rules stay LSP-only).

Two slices in the registry:

- `CONTROLLER_RULES` — lints `*.controller.ts` (oxc-parsed JS/TS AST walking).
- `SUI_TEMPLATE_RULES` — lints `*.sui` source (string-walking, char-boundary-safe).

1. **Create the module** at `crates/sui-lint/src/rules/<rule_id>.rs`. Export `RULE_ID: &str` and `check(path: &Path, source: &str) -> Vec<Finding>`. For controller rules: parse with `oxc_parser`, walk via `oxc_ast_visit::Visit`, skip parse errors silently — the compiler reports those. For template rules: walk char boundaries via `source.char_indices()` so multi-byte chars (em-dashes, entity-decoded symbols) don't trip str-slice panics.

2. **Pick severity.** `Error` rejects the build; `Warn` surfaces only in the LSP (and the dispatcher filters by severity before deciding whether to fail compile).

3. **Register** in `crates/sui-lint/src/rules/mod.rs`:

   ```rust
   pub mod <rule_id>;

   // pick the right slice:
   pub const CONTROLLER_RULES: &[(&str, RuleFn)] = &[
       (controller_purity::RULE_ID, controller_purity::check),
       (<rule_id>::RULE_ID, <rule_id>::check),  // append
   ];
   pub const SUI_TEMPLATE_RULES: &[(&str, RuleFn)] = &[
       (css_token::RULE_ID, css_token::check),       // Error
       (for_key::RULE_ID, for_key::check),           // Warn
       (event_name::RULE_ID, event_name::check),     // Warn
       (<rule_id>::RULE_ID, <rule_id>::check),       // append
   ];
   ```

4. **Test** alongside the rule module (`#[cfg(test)] mod tests`) — positive (clean code passes), negative (slop reports), edge cases (multi-byte chars for template rules), and a smoke fired against a real offender in the repo to keep the rule honest.

5. **Verify** with `cargo test -p sui-lint -p sui-compiler -p lsp`. Dispatch is registry-driven; no other call sites need changes.

6. **Document** in the relevant authoring skill if the rule changes the contract. Diagnostic messages always include `suggest:` pointing at what TO do.

## Dev loop — three steps, all required

JS bundle changes only land in the running daemon after:

1. `sui-build` — compiles every `.sui` source in the touched package.
2. `cargo run -p xtask -- build-assets` — rebakes the framework bundles (sui-runtime/dom/sdk-client/components) into `crates/deck/deck-core/assets/` via the in-process Rolldown bundler (#163, #170).
3. `cargo build -p shipd` — picks up the newly baked bundles via `include_str!` in `crates/deck/deck-core/src/assets.rs`.
4. `bash scripts/dev/restart.sh` — cycles the per-worktree daemon.

Skipping any step ships stale bytes.

## Tests — run all four when changing the framework

| Surface | Command |
|---|---|
| Compiler | `cargo test -p sui-compiler` |
| Runtime | `bun run --filter @ship/sui-runtime test` |
| DOM / attach | `bun run --filter @ship/ui-dom test` |
| LSP | `bun run --filter @ship/ui-lsp test` |
| Browser smoke | `node .artifacts/sui-counter-probe.mjs` (Playwright probe; daemon must be up) |

## Generated artifacts (gitignored, emitted under `dist/`)

Don't commit. Don't be surprised by:

- `views/<name>/dist/page.compiled.js` — view emit. The paired controller is `views/<name>/controller.ts`; legacy directories still use `view.sui` / `view.controller.ts` and emit `view.compiled.js`. Vitest resolves `.ts` first via `packages/ui-dom/vitest.config.ts::resolve.extensions`.
- `packages/components/sui/primitives/src/<tag>/dist/<tag>.compiled.js` — component emit; `packages/components/sui/primitives/src/index.ts` side-effect-imports each one to register the custom element.
- `**/dist/*.sui.d.ts` — type declarations the compiler emits alongside the JS module.

## Adding a runtime export to `@ship/sui-runtime`

Each public symbol gets its own file in `packages/ui-runtime/src/<name>.ts` (mirrors `signal.ts`, `state.ts`, `controller.ts`, `view.ts`). Re-export from `index.ts`. Don't bolt onto unrelated files like `controller-resolver.ts` — that's how parallel implementations get created.

## Form-association wiring

`<prop ... form-value />` and `<prop ... form-disabled />` are the public surface; everything below is the implementation.

- **Lower / IR** (`crates/sui-compiler/src/ir/directive.rs`, `lower/directives.rs`): `form_value: bool` / `form_disabled: bool` on the prop IR. Lower validates: only one `form-value` per component, `form-value` rejected on `object` / `array`, `form-disabled` only on `bool`.
- **Emit** (`crates/sui-compiler/src/emit/js.rs`): when any prop carries either flag, the generated `component({...})` call gains `formAssociated: true` plus `formValue: '<prop>'` / `formDisabled: '<prop>'` keys declaring which prop drives each lifecycle.
- **Runtime** (`packages/sui/runtime/src/component-factory.ts`, `component-form.ts`, `component.ts`): `defineComponent` reads those keys, sets `static formAssociated = true` on the generated class, calls `host.attachInternals()` on connect, installs the four platform callbacks (`formResetCallback`, `formStateRestoreCallback`, `formDisabledCallback`, and the per-change `setFormValue` effect), and exposes `internals?: ElementInternals` in setup ctx for callers who need `setValidity()` / `checkValidity()`.
- **Parallel emitters**: schema.json, custom-elements.json, and .sui.d.ts all mirror the form-value / form-disabled markers so docs, agents, and TypeScript consumers see the form-association contract.

When adding new form lifecycle hooks (custom validity, willValidate-aware behavior), extend `component-form.ts` — not the controller. The controller's job stays "react to the prop change"; the runtime owns the platform contract.

## Compile shape contract — don't conflate

- `view(setup)` from `@ship/sui-runtime`: the compiler emits `export default view((ctx) => {...})` for every `.sui` view module. Wraps a setup function as `{ setup }` so `attach()` can call `module.default.setup(ctx)`.
- `controller(def)`: the paired `controller.ts` (or legacy `view.controller.ts`) exports `controller<Scope>({ scope, on, mount, unmount })` for tier-2 logic. Resolved by the compiled view's setup.
- They coexist. `view` ≠ `controller`.

## Marker contract

The SSR walker emits author-syntax markers (preserved attribute names + comment-bracketed text + custom-element boundary wrappers). attach() walks them and wires bindings against the manifest exported alongside `default`. Reference: `template-surface.md` § "SSR + attach pipeline" for the table of constructs → markers.

When adding a new control-flow construct: parse → IR → lower → render walker (emit `<sui-<name> key="bN">` boundary) → bindings manifest entry → attach() boundary wirer dispatches via the runtime helper. Mirror `{#if}` exactly.

## Fixture / test-corpus maintenance

Adding a parser feature means updating `crates/sui-compiler/tests/fixtures/*.sui` and `*.expected.js` snapshots in the same change. The CLI smoke (`tests/cli.rs`) and snapshot tests (`tests/snapshot.rs`) gate the emit shape — keep them current.

## Commit hygiene

Commit at every passing test gate. A multi-hour uncommitted session is a dead session — if anything crashes, work is gone.

## Discipline

When a test fails, fix root cause. Don't paper over with config workarounds. The vitest `resolve.extensions` reorder is an exception — it expresses real project layout (compiled `.js` under `dist/` resolves before source `.ts`), not a workaround.
