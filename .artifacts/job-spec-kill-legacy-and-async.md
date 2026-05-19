# Job Spec: kill legacy JS template compiler + add `{#async}` directive

**Branch:** `job/kill-legacy-async`
**Agent:** rust-lane
**Mode:** autonomous
**Estimated effort:** ~1.5-2 engineering days

## Goal

Eliminate the parallel JS-side template compiler in `packages/sui/dom/src/template/` and the legacy hydrate path in `packages/sui/dom/src/legacy/`. Route the 4 test packages (sui-dom, primitives, charts, ui-components, plus apps/shell) through the WASM-built Rust compiler so tests parse `.sui` files the same way production does. Then add the `{#async}` block directive to the Rust compiler so views can replace the repetitive `{#while-loading}` + `{#while-error}` + `{#if X().kind === 'ok'}` triple with a single block.

## Background

The Rust `sui-compiler` (in `crates/sui/compiler/`) is the production source of truth. It emits `.compiled.js` files via `sui-build` ahead of time; the client runtime never sees `.sui` source. The `legacy/hydrate.ts` path exists only to support tests that import `.sui` as raw strings and parse at test time — every directive currently has to be implemented in both the TS template module *and* the Rust compiler, and the TS implementation is never used in production. The user has explicitly directed that we remove this duplication: *"if this does not exist in rust then it does not exist at all… testing parallel impl is insane!"*

The WASM build at `packages/sui/compiler-wasm/pkg/` already exists and exports `compile(source, opts_json): { Ok: { js, css, dts, ssr_html } | Err: { message } }`. It was rebuilt fresh on this branch (see `just sui-wasm`).

## File scope

**Phase A — kill legacy:**
- `packages/sui/dom/src/legacy/` — delete entirely (hydrate.ts, parse.ts, directives/{component,data,state}.ts, index.ts)
- `packages/sui/dom/src/template/` — delete entirely (tokenize.ts, compile.ts, compile-blocks.ts, render.ts, render-for.ts, render-while.ts, render-attrs.ts, expr.ts)
- `packages/sui/dom/src/index.ts` — drop legacy re-exports
- New: `packages/sui/dom/src/test-utils/compile-sui.ts` — wasm-backed test helper
- Update vitest configs in: `apps/shell/`, `packages/components/{primitives,charts,ui-components}/` — replace the `compiled-js-shim` plugin's legacy-hydrate output with wasm compile output
- Migrate the 5 remaining tests in `packages/sui/dom/test/` that import legacy hydrate: `reactive-data.test.ts`, `template-hooks.test.ts`, `hydrate.test.ts`, `git-view.test.ts`, `controller-hydrate.test.ts`

**Phase B — `{#async}` in Rust:**
- `crates/sui/compiler/src/parse/template/tokenize.rs` — add `OpenAsync { src }`, `AsyncLoading`, `AsyncError { src }`, `AsyncOk`, `CloseAsync` variants to `TokenKind` + `Discriminant` + labels + `classify_brace` cases
- `crates/sui/compiler/src/parse/template/blocks.rs` — add `parse_async` + `parse_async_header` mirroring the TS impl (see commit `6c211a822` for reference, since reverted). Optional `{:loading}` / `{:error}` branches fall back to defaults: `<sui-spinner size="md" centered>` / `<sui-error-state>{err.message}</…>`. `{:ok}` is required.
- `crates/sui/compiler/src/parse/template/mod.rs` — wire `OpenAsync` to `parse_async`. Splice the returned multi-instr vec into the parent.
- `crates/sui/compiler/src/ir/template.rs` — add `Let(LetInstr { name: String, value: Expr, body: Vec<Instr>, span: Span })` variant for the `as v` scope binding inside `{:ok}`.
- `crates/sui/compiler/src/check/scope_bound.rs` + `check/mod.rs` — walk `Let` body with `v` added to in-scope identifiers.
- `crates/sui/compiler/src/render/control_flow.rs` (or new `render/render_let.rs`) — extend `walker.scope` with the let binding before walking the body.
- `crates/sui/compiler/src/render/mod.rs` — dispatch `Instr::Let` to the new renderer.
- `crates/sui/compiler/src/emit/template_emit.rs` — emit `Let` as a `{ const v = (<value-expr>); ...body... }` JS block, ensuring `v` references inside the body resolve to the JS const. Note: the binding is a **snapshot at render time**, not a reactive getter — the surrounding `{#if X().kind === 'ok'}` already gates re-render when the signal state changes, so refreshing `v` per access isn't needed.
- New fixtures: `crates/sui/compiler/tests/fixtures/async-basic.sui`, `async-with-as.sui`, `async-defaults.sui`. Tests assert correct JS emit + SSR HTML.

**Phase C — migrate views:**
- 22 occurrences across 20 view files (see `grep -rln "while-loading" views/`).
- The rewrite is mechanical: each triple
  ```
  {#while-loading of X}…{/while-loading}
  {#while-error err of X}…{/while-error}
  {#if X().kind === 'ok'}…{/if}
  ```
  becomes
  ```
  {#async X as v}
    {:loading}…
    {:error err}…
    {:ok}…  (with X().value.field rewritten to v.field)
  {/async}
  ```
- Where the loading/error bodies are pure defaults, drop those branches entirely.

## What to change (per phase, in order)

### Phase A.1 — wasm test helper

Build `packages/sui/dom/src/test-utils/compile-sui.ts` exporting:

```ts
export interface CompiledSui {
  /** The compiled JS module body (what production .compiled.js contains). */
  js: string
  /** Pre-rendered SSR HTML (optional, when emit_ssr opt is set). */
  ssr_html: string | null
}

/** Synchronously compile a .sui source via the embedded WASM compiler. */
export function compileSui(source: string, opts?: { mode?: 'view' | 'component' | 'auto'; componentName?: string }): CompiledSui

/** Evaluate compiled JS in a sandboxed scope and return its exports. */
export function loadCompiledModule(js: string): { default?: unknown; setup?: Function; [key: string]: unknown }
```

Implementation: load `@ship/sui-compiler-wasm` once (use `initSync` with the .wasm bundled via `?url` resolver, or read with `fs.readFileSync` in Node). Cache the init promise/result on module load. `loadCompiledModule` uses dynamic import via a Blob URL (browser-safe in jsdom) or `new Function(...)` for Node.

### Phase A.2 — vitest plugin rewrite

In each of the 4 vitest configs, the `compiled-js-shim` plugin currently does:
```ts
load(id: string) {
  // ...
  return `import { hydrate } from '@ship/sui-dom/legacy'
          import template from ${JSON.stringify(htmlSet)}
          if (typeof document !== 'undefined') {
            const __c = document.createElement('div')
            hydrate(template, undefined, { container: __c })
          }`
}
```

Replace with:
```ts
load(id: string) {
  // ...
  const suiPath = `${base}.sui`
  const source = fs.readFileSync(suiPath, 'utf8')
  const { compileSui } = require('@ship/sui-dom/test-utils')
  const compiled = compileSui(source)
  return compiled.js
}
```

The returned `js` is the production-equivalent compiled module — running it fires `customElements.define` and any other module-level side effects, identical to production.

### Phase A.3 — dom/ test migration

The 5 tests pass raw template strings to `hydrate(template, setup, opts)`. Migrate by:
- Building each test's template into a `.sui` fixture file in `test/fixtures/`
- Replacing `hydrate(template, setup, opts)` with `compileSui(source).js` + `loadCompiledModule(js)` + invoking the rendered module's exports
- Where setup was an inline closure, move it to a paired `.controller.ts` fixture

Alternatively (simpler): provide a synchronous `compileAndMount(source, opts)` test-utility that does compile + eval + render-into-container in one call, mirroring the legacy `hydrate` shape so test bodies don't need to restructure.

### Phase A.4 — delete legacy

```
rm -r packages/sui/dom/src/legacy/
rm -r packages/sui/dom/src/template/
```

Update `packages/sui/dom/src/index.ts` to drop the legacy subpath re-export. Verify nothing imports `@ship/sui-dom/legacy` outside tests.

### Phase A.5 — verify

```
cd packages/sui/dom && bun run test       # was 93, now should be 93 minus my just-deleted async tests
cd packages/components/primitives && bun run test     # 1055
cd packages/components/charts && bun run test          # 172
cd packages/components/ui-components && bun run test  # 118
cd apps/shell && bun run test                          # (whatever shell tests there are)
```

Plus the e2e visual suite: `bun run test:e2e:visual`.

### Phase B — `{#async}` in Rust

The TS-side reference impl is in this branch's reverted commit `6c211a822` (`git show 6c211a822`). Mirror the structure in Rust:

1. Add tokens to `tokenize.rs` — see the existing `OpenWhileLoading` pattern.
2. Add `parse_async` to `blocks.rs` — header parser handles `<signal>[ as <var>]`; body parser switches on `AsyncLoading` / `AsyncError { src }` / `AsyncOk` markers up to `CloseAsync`. Required: `{:ok}`. Optional with defaults: the others.
3. Wire `OpenAsync` in `mod.rs` to call `parse_async` and splice the resulting Vec<Instr> into the parent stream.
4. Add `Instr::Let` to `ir/template.rs` for the `as v` binding inside `{:ok}` only.
5. Update `check/scope_bound.rs` walker — when visiting `Let`, extend the in-scope identifier set with `name` for the body.
6. Render: in `render/control_flow.rs`, add `render_let` that updates `walker.scope` with the let binding before walking the body. (For SSR, treat the binding as a snapshot — `walker.scope` is a flat map and existing `if/match` already do scope evaluation top-to-bottom.)
7. Emit: in `template_emit.rs`, emit `Let` as a JS block:
   ```js
   ;{ const <name> = (<value-source>); /* …body emit… */ }
   ```
   Need to confirm body emit references to identifiers don't get rewritten incorrectly (the existing `rewrite_for_scope` logic handles for-loop bindings; let bindings need similar treatment if the emit references them).

Tests: at least 3 fixtures under `crates/sui/compiler/tests/fixtures/`:
- `async-basic.sui` — only `{:ok}`, no `as`
- `async-with-as.sui` — `as v`, references `v.field` inside `{:ok}`
- `async-defaults.sui` — omits `{:loading}` and `{:error}`, asserts default emit

### Phase C — view migration

The 20 view files containing the triple pattern (see `grep -rln "while-loading" views/`). Per file:
1. Replace the three-block sequence with `{#async X as v}{:loading}…{:error err}…{:ok}…{/async}`.
2. Inside the `:ok` body, rewrite every `X().value.field` to `v.field`.
3. Where `{:loading}` or `{:error}` bodies are pure default content (`<sui-spinner centered>` / `<sui-error-state>{err.message}</…>`), omit them.

Run `bun packages/deck/bundler/index.ts --all` after each batch to catch syntax errors early. Full e2e visual suite at the end to confirm rendering parity.

## Architectural context

- Active ADRs: charter at `docs/ship-architecture/references/docs/charter.md` (no specific ADRs for this).
- Key constraints from CLAUDE.md:
  - "One way to do things. If a solution exists, use it. No parallel systems." — exactly what this job addresses.
  - "Rust logic in runtime. CLI/MCP are thin transport." — the JS template compiler violates this.
  - 300-line file cap; current legacy + template dirs sit at ~1500 LoC total — deletion is the path.

## Acceptance criteria

1. `packages/sui/dom/src/legacy/` and `packages/sui/dom/src/template/` directories are deleted.
2. `git grep "@ship/sui-dom/legacy"` returns zero matches outside test-helper internals.
3. All test suites pass without the legacy hydrate path:
   - `bun run test` in `packages/sui/dom/` — 93 tests (or current count minus the deleted async ones)
   - `packages/components/primitives/` — 1055
   - `packages/components/charts/` — 172
   - `packages/components/ui-components/` — 118
   - `apps/shell/` test suite
   - `bun run test:e2e:visual` passes
4. `{#async}` directive works in Rust: `crates/sui/compiler/tests/` has 3 new fixtures + assertions, all pass under `cargo test -p sui-compiler`.
5. `bun packages/deck/bundler/index.ts --all` succeeds after view migration.
6. The 22 view occurrences are migrated; `grep -rln "while-loading" views/` returns ≤ 0 matches.
7. Final visual QA artifact under `.artifacts/web-qa/<surface>/<run>/` proves the migrated views render correctly across all 7 themes.

## Out of scope

- Reactivity refactor (the `as v` binding is a snapshot, not a reactive getter — the surrounding `{#if .kind === 'ok'}` re-renders on state change, which is enough).
- New runtime helpers (everything desugars to existing `bindWhileLoading` / `bindWhileError` / `bindIf` runtime calls).
- Performance optimization of wasm initialization in tests (one cold cost per worker process is acceptable; we can sync-init from a bundled `.wasm` buffer).

---

> Execute this job yourself. Do not delegate to a subagent — subagents are read-only research helpers, not implementers for cross-cutting refactors.
>
> Commit incrementally — one commit per Phase A sub-step, one for Phase B (or 2-3 if it's natural), one batch per ~5 views in Phase C. The commit graph should let a reviewer pause at any phase boundary.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
