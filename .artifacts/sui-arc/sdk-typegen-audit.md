# SDK + Scope type-gen audit — 2026-05-13

Branch: `job/sdk-typegen-audit`. Read-only audit of the post-`scope-typing` (#89) /
post-`cast-removal-sweep` (#92) state.

## TL;DR

The Rust emitter for `.sui.d.ts` (scope-typing) exists and works on every spot-checked
view/component, **but no `.sui.d.ts` files exist in the repo** because every build-pipeline
call site hard-codes `--no-dts` / `emit_dts: false`. The job spec's premise — "every `.sui`
has a corresponding `.sui.d.ts`" — is false against the current tree.

Scope-typing itself is largely correct on the surface it covers. Two scope-factory
inference bugs are visible in the spot-check corpus (string-literal generics fall back to
`unknown`; `live(...)` is not recognised). The `@ship/sdk-client` runtime surface declares
only 3 of the 11 `ship.<domain>.<op>()` shapes actually referenced by views / controllers
— 8 are server-resolved string exprs in `<data live source=…>` (intentional) but one
controller (`views/overview/controller.ts`) escapes the gap via `(ship as any)`.

Renderable-subset and scope-typing do not disagree at template-emit time; they don't share
state at all — scope-typing only feeds editor/LSP.

## Numbers

- `.sui` files under `packages/ui-components/src` + `views`: **138**
- `.sui.d.ts` files committed: **0** (gitignored — `.gitignore:171: **/*.sui.d.ts`)
- `.sui.d.ts` files emitted by a fresh `pnpm sui-build`: **0** (every build path forces
  `--no-dts`)
- Recognised SDK domains in `@ship/sdk-client`: **2** (`agents`, `git`)
- Unique `ship.<domain>.<op>()` references in `.sui` + `.controller.ts` corpus: **11**

## Root finding: `.sui.d.ts` is plumbed but never written

The compiler library defaults `emit_dts: true`
(`crates/sui-compiler/src/lib.rs:97`) and the `sui-compiler` binary defaults emission
on (`--no-dts` is opt-out, `crates/sui-compiler/src/bin/sui-compiler.rs:80-82, 279`). The
emitter writes `<stem>.sui.d.ts` next to the source
(`crates/sui-compiler/src/bin/sui-compiler.rs:327-333`).

Every production call site forces it off:

| Call site                                                | Setting                       |
|----------------------------------------------------------|-------------------------------|
| `packages/sui-build/src/runner.ts:73-74`                 | passes `--no-dts`             |
| `crates/sui-compiler/src/ffi.rs:96`                      | `emit_dts: false`             |
| `crates/ship-sui-server/src/view.rs:85`                  | `emit_dts: false`             |

`packages/sui-lsp/src/setup.ts:72-80` *reads* a sibling `<stem>.sui.d.ts` when present
(used to enrich symbol resolution), so the LSP path is already wired to consume what the
build pipeline never produces. End-to-end the typegen feature is dark.

To verify the emitter is correct, the spot-check below was performed by invoking
`./target/debug/sui-compiler build <file> --component-name <stem>` directly (which leaves
`--no-dts` at its default of false). All artifacts were deleted after sampling — the
worktree is unchanged.

## Scope shape spot-check (10 representative files)

Each entry: file → emitted Scope members → verdict. All paths relative to repo root.

### 1. `packages/ui-components/src/sui-button/sui-button.sui` — ✅ clean
```
tone: Signal<'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'>
variant: Signal<'solid' | 'soft' | 'outline' | 'ghost' | 'default'>
size: Signal<'sm' | 'md' | 'lg'>
type: Signal<'button' | 'submit' | 'reset'>
disabled: Signal<boolean>
loading: Signal<boolean>
label: Signal<string>
pressed: Signal<string>
expanded: Signal<string>
```
Union-prop kinds emit verbatim. `pressed` / `expanded` are aria-mirror props declared
as plain `<prop name="pressed">` strings in source; that's a `.sui` authoring concern,
not a typegen one.

### 2. `packages/ui-components/src/sui-error-boundary/sui-error-boundary.sui` — ✅ clean
```
fallbackText: Signal<string>
// From controller scope: factory
caughtError: Signal<Error | null>
hasFallbackSlot: Signal<boolean>
reset: () => void
```
Typed-generic `signal<Error | null>` resolves correctly. Arrow methods get
`() => void` from explicit annotation. Header comment separates template-derived from
factory-derived members.

### 3. `packages/ui-components/src/sui-portal/sui-portal.sui` — ✅ clean
```
target: Signal<string>
portalSlot: Signal<HTMLSlotElement | null>
```
`bind:this` on a `<slot>` resolves to `HTMLSlotElement` via the tag→TS-element map
(`dts.rs:223`).

### 4. `packages/ui-components/src/sui-rail/sui-rail.sui` — ✅ clean
```
collapsed: Signal<boolean>
// From controller scope: factory
projectOpen: Signal<boolean>
projects: Signal<Project[]>
activeProjectId: Signal<string | null>
activeView: Signal<string>
sessionCount: Signal<number>
projectInitial: () => string
popoverEl: Signal<HTMLElement | null>
projectBadgeEl: Signal<HTMLElement | null>
launcherBtnEl: Signal<HTMLElement | null>
createBtnEl: Signal<HTMLElement | null>
toggleBtn: Signal<HTMLElement | null>
missionEl: Signal<HTMLElement | null>
navSlot: Signal<HTMLSlotElement | null>
```
The `bind:this` refs on custom elements (`<sui-popover>`, `<sui-button>`, …) fall
through to `HTMLElement` (correct — `dts.rs:175-176`). Method `projectInitial` keeps its
declared return type.

### 5. `packages/ui-components/src/sui-table/sui-table.sui` — ⚠️ one inference miss
```
variant: Signal<'slot' | 'native' | 'data'>
density: Signal<'compact' | 'default' | 'comfortable'>
dense: Signal<boolean>
hoverable: Signal<boolean>
striped: Signal<boolean>
stickyHeader: Signal<boolean>
stickyFirstCol: Signal<boolean>
gutter: Signal<boolean>
label: Signal<string>
// From controller scope: factory
rows: Signal<Row[]>
columns: Signal<Column[]>
sortKey: Signal<string | null>
sortDir: unknown
selected: Signal<string[]>
sortedRows: Signal<unknown>
```
`sortDir` in the controller is `signal<'asc' | 'desc'>('asc')`
(`sui-table.controller.ts:34`) but the emitted dts is the bare `unknown`. Root cause is
documented under **Scope-factory inference gaps** below.

### 6. `packages/ui-components/src/sui-tree-view/sui-tree-view.sui` — ✅ clean
```
// From controller scope: factory
nodes: Signal<TreeNode[]>
```
Single-field factory; `nodes: signal<TreeNode[]>([])` survives unchanged.

### 7. `views/home/page.sui` — ✅ clean
```
git: Signal<LiveState<unknown>>
// From controller scope: factory
seen: Signal<number>
```
`<data live>` correctly resolves to `Signal<LiveState<unknown>>` and triggers the
`LiveState` import. Controller-derived `seen` is fine.

### 8. `views/agents/page.sui` — ✅ clean
```
agents: Signal<LiveState<unknown>>
```
Single live data; correct.

### 9. `views/overview/page.sui` — ⚠️ two inference misses
```
// From controller scope: factory
systems: unknown
depsJson: Signal<unknown>
search: Signal<string>
filterStatus: Signal<string>
filterLayer: Signal<string>
drawerOpen: Signal<boolean>
drawerId: Signal<string | null>
drawerTitle: Signal<string>
fldName: Signal<string>
fldLayer: Signal<string>
fldStatus: Signal<string>
fldSummary: Signal<string>
fldDocSkill: Signal<string>
fldGaps: Signal<string[]>
fldCodePaths: Signal<string[]>
fldDependsOn: Signal<string[]>
wrapRef: Signal<HTMLElement | null>
canvasRef: Signal<HTMLElement | null>
edgesRef: Signal<SVGElement | null>
```
- `systems: unknown` — the controller has `live(() => ...)`
  (`views/overview/controller.ts:7`). `live` isn't on the scope-factory recogniser's
  call-name list (only `signal`/`ref`/`derived`/`computed`/`memo`), so the field falls
  back to bare `unknown`.
- `edgesRef: Signal<SVGElement | null>` — fine, `ref<SVGElement>(null)` is handled.

The template only does `bindFor` over reads (`state.systems()` is a zero-arg call), so
the runtime works fine; the typing surface for editors is poor here.

### 10. `views/ship-ui-studio/[name]/page.sui` — ⚠️ two inference misses
```
// From controller scope: factory
allCatalog: Signal<CatalogEntry[]>
loading: Signal<boolean>
propValues: unknown
_dispose: unknown
filterKind: Signal<StudioFilterKind>
filterText: Signal<string>
copyFeedback: Signal<boolean>
```
- `propValues: unknown` and `_dispose: unknown` — bound to expressions the factory
  recogniser doesn't model (a `Map`/`new Map`-shaped value and a function reference,
  respectively).

These two underscore-prefixed / opaque fields are technically internal and may be
acceptable as `unknown`, but `propValues` is an interactive map and gets read by the
template — same usability concern as `systems` above.

## Scope-factory inference gaps

### Bug 1 — string-literal generics bail to `unknown`

`crates/sui-compiler/src/emit/scope_factory.rs:475-495` (`match_angle`) walks generic
args bracket-by-bracket but bails on encountering `'`, `"`, or `` ` ``:

```rust
b'"' | b'\'' | b'`' => return start, // string in type args — bail.
```

That triggers for `signal<'asc' | 'desc'>(...)` (sui-table) and any other typed signal
whose generic contains a string-literal union. The caller treats `start` as "no
generic", falls through, and emits the bare `unknown`. Fix is a ~6-line addition to
`match_angle` to track an `in_str` state (mirroring `skip_balanced`'s loop). Within the
≤30-LOC budget but **outside this audit's allow-list** (`scope_factory.rs` is not in the
allow-list); see Recommendations.

### Bug 2 — `live(...)` produces no `Signal<…>` wrapper

`infer_ts_type` recognises `signal`, `ref`, `derived`, `computed`, `memo`
(`scope_factory.rs:392-403`). It does not recognise `live` (the runtime helper that
returns a `LiveSource<T>`-wrapped signal). Result: any `const x = live(() => …)`
binding ends up as `unknown`. Two cases in the spot-check (`views/overview` →
`systems`; same idiom elsewhere). Fix would add a case symmetric to `derived`:
`"live" => Signal<LiveState<unknown>>` (or `LiveState<T>` when a generic is supplied).
Again `scope_factory.rs` is outside the allow-list.

### Non-bugs worth noting

- Camel-casing of kebab prop names is in `dts.rs:32`. No corpus member tripped it.
- `bind:this` element-tag resolution is correct for the SVG case (`edgesRef`) only
  because the source declares `ref<SVGElement>(null)` explicitly; the tag map in
  `dts.rs:178-239` is HTML-only and would return `HTMLElement` for an SVG tag.
  Authors who use `bind:this` on an SVG element without a controller annotation get
  the wrong type. No corpus hit, but worth flagging.

## SDK reference vs declaration

Unique `ship.<domain>.<op>` symbols referenced across `views/` and `packages/ui-components/src/`
(via `grep -rEh "ship\.[a-z_]+\.[a-z_]+\(" | sort -u`):

```
ship.agents.create_ctx
ship.agents.summaries
ship.canvas.list
ship.collections.items
ship.collections.list
ship.dashboard.state
ship.git.annotations
ship.git.diff
ship.git.overview
ship.mesh.snapshot
ship.settings.state
```

### What the @ship/sdk-client surface actually declares

`packages/ship-sdk-client/src/index.ts:42-64` exposes a typed `ShipSDK` interface with
exactly three methods:

```
agents.summaries(): LiveSource<AgentsSummariesResponse>
git.overview(): LiveSource<GitOverview>
git.diff(args: GitDiffArgs): LiveSource<GitDiff>
```

There is **no** `packages/sdk/src/generated/routes.ts` file (referenced in the job
spec) — neither the `generated/` directory nor any generated routes table exists.
`packages/sdk/src/` is hand-authored: `client.ts`, `transport.ts`, `fetch.ts`,
`types.ts`, plus `view-mount/`. The `client.ts` surface is the generic
`get/post/put/del` + a small set of named REST helpers (jobs, collections, pair-init).
It is **not** the same surface as `@ship/sdk-client` — these are two distinct
packages that share a name only by accident.

### Defined / orphaned / undefined

| Reference                | Defined in `@ship/sdk-client`? | Notes |
|--------------------------|---------------------------------|-------|
| `ship.agents.summaries`  | ✓                               | live source |
| `ship.git.overview`      | ✓                               | live source |
| `ship.git.diff`          | ✓                               | live source |
| `ship.agents.create_ctx` | ✗                               | only as `<data source="ship.agents.create_ctx()" />` — server-resolved |
| `ship.canvas.list`       | ✗                               | `<data live source="…" />` |
| `ship.collections.items` | ✗                               | both `<data>` *and* `live(() => (ship as any).collections.items(...))` in `views/overview/controller.ts:7` — controller bypasses the type via `as any` |
| `ship.collections.list`  | ✗                               | `<data live>` only |
| `ship.dashboard.state`   | ✗                               | `<data live>` |
| `ship.git.annotations`   | ✗                               | mentioned in a comment in `views/git/controller.ts`; no live call site |
| `ship.mesh.snapshot`     | ✗                               | `<data live>` |
| `ship.settings.state`    | ✗                               | `<data live>` |

Orphaned routes (defined in `@ship/sdk-client` but unused in this corpus): **none**.

The intended split, as far as can be reconstructed from the code: `<data live
source="ship.x.y()">` is a *string expression* compiled to a `data({ subscribe:
'/_sui/data?expr=…' })` SSE handle — the daemon resolves the expr; the client doesn't
need a typed entry. Controller-side `live(() => ship.x.y())` calls go through the
imported `ShipSDK` value and therefore *do* need typed entries. Today there's exactly
one bypass (`views/overview/controller.ts:7` casts through `(ship as any)` to reach
`collections.items`). That cast is the only thing keeping the build green.

## Renderable-subset vs scope-typing consistency

`crates/sui-compiler/src/check/expr_subset.rs:22-24` defines the method allowlist:

```
"slice", "map", "filter", "includes", "toString", "length", "keys", "values", "entries"
```

The subset checker rejects:
- bare keywords `new`, `await`, `yield`, `this`, `function`, `delete`, `void`;
- arrow functions, tagged template literals, regex literals;
- assignment and update operators;
- calls with arguments **on dotted member chains** outside the allowlist;
- spread outside array / object literals.

It **allows**:
- identifier reads, literals, member access (`?.` included);
- ternary, nullish coalescing, logical / arithmetic / comparison / unary `!`;
- template literals (interpolations recurse);
- zero-arg calls on idents / chains — i.e. `state.foo()` for any signal read.

This is orthogonal to scope-typing. The compiler does not consult the emitted
`.sui.d.ts` while emitting templates; it only validates each `{expr}` against the
subset. So there's no "Scope claims a member but the template rejects it" failure
mode. The only consistency edge to call out:

- Scope-factory may emit `field: () => Promise<…>` for an async method
  (`scope_factory.rs:334-358`). The renderable subset disallows `await` in
  expressions, so a template can read the method as `state.field` (allowed) and pass it
  as a handler, but cannot call it with `await` inside an interpolation. This is
  consistent with the design — `await` belongs in controller code — and not a typing
  bug.

No recommended changes for the renderable-subset checker.

## What works

- `dts.rs` and `scope_factory.rs` compile to a sound, mostly-correct Scope shape on
  every spot-checked file.
- Cast removal post-#92 is verified clean on the three referenced controllers
  (`sui-rail.controller.ts`, `sui-table.controller.ts`, `sui-tree-view.controller.ts`);
  there are no remaining `state['x'] as unknown` accesses in those files.
- The renderable-subset checker is internally consistent and does not interact with
  scope-typing.
- The Rust unit tests in `scope_factory.rs:643-781` cover the happy paths
  (`signal<T>`, `ref<T>`, untyped literal inference, block-body factories).

## What's misaligned

1. **The feature is never user-visible.** All three build paths force `emit_dts: false`,
   so no `.sui.d.ts` ever lands on disk after a normal build. The Rust tests pass; the
   end-to-end output is dark.
2. **`packages/sdk/src/generated/routes.ts` does not exist.** The job spec assumes a
   generated SDK route shape; the actual `packages/sdk` is hand-authored REST helpers
   with no generated layer.
3. **`@ship/sdk-client` exposes 3 of 11 referenced `ship.*` shapes.** Eight live
   exclusively in `<data live source="…">` strings (intentional, server-resolved);
   one (`collections.items`) is bypassed via `(ship as any)` in a controller.
4. **`scope_factory.rs::match_angle` bails on string-literal generics.** `sortDir`
   in `sui-table.controller.ts` is a typed `signal<'asc' | 'desc'>(…)` that becomes
   bare `unknown`.
5. **`live(...)` is not recognised by the scope-factory infer pass.** `systems` in
   `views/overview/controller.ts` and any analogous binding end up as bare `unknown`.

## What's fragile

- The `(ship as any)` cast in `views/overview/controller.ts:7` is the only thing
  keeping a `ship.collections.items(...)` call site type-checkable. If the cast is
  ever removed without first adding `collections` to `ShipSDK`, the build breaks.
- `bind:this` on SVG / MathML elements returns `HTMLElement` from the dts emitter
  (the tag map in `dts.rs:178-239` is HTML-only). No corpus hit today, but adding an
  SVG `bind:this` without an explicit `ref<SVGElement>` annotation would silently
  emit the wrong type.
- The build pipeline's `--no-dts` is duplicated in three places
  (`runner.ts`, `ffi.rs`, `view.rs`). If one is flipped without flipping the others,
  partial emission will mask the gap.

## Recommendations (for future dispatch specs)

1. **Flip emission on once globally.** Open a follow-up to remove `--no-dts` from
   `packages/sui-build/src/runner.ts:73-74` and set `emit_dts: true` in both
   `crates/sui-compiler/src/ffi.rs:96` and `crates/ship-sui-server/src/view.rs:85`.
   Combine with a brief audit that the LSP path
   (`packages/sui-lsp/src/setup.ts:72-80`) actually consumes the emitted files.
2. **Fix `match_angle` string-literal bail.** Replace the current bail with an
   in-string skip loop (~6 LOC in `scope_factory.rs`). Test corpus already includes a
   typed-generic case (`typed_signal_generic_uses_generic_arg`); add a sibling case for
   `signal<'asc' | 'desc'>(...)`.
3. **Add `live` to the scope-factory recogniser.** Extend `infer_ts_type` so that
   `live` produces `Signal<LiveState<T>>` (with `T = unknown` when no generic is
   supplied). Add a test case mirroring the existing `derived` test.
4. **Reconcile the two "SDK client" packages.** `packages/sdk` and
   `packages/ship-sdk-client` share neither a surface nor a purpose. Either rename one,
   or merge `ship-sdk-client`'s `ShipSDK` definition into `packages/sdk` and generate it
   from a single descriptor. Update the job-spec wording: there is no
   `packages/sdk/src/generated/routes.ts`.
5. **Type `ship.collections.*` end-to-end.** Promote `ship.collections.items`,
   `ship.collections.list` (and any other server-resolved shape with controller call
   sites) to the `ShipSDK` interface so `views/overview/controller.ts` can drop its
   `(ship as any)` cast.
6. **Add SVG / MathML tags to `element_type_for_tag`.** Trivial dts.rs extension; in
   the allow-list for this dispatch but skipped here since no corpus member needed it.
7. **Lint rule.** Once `.sui.d.ts` files actually exist, add a CI check that
   `find packages/ui-components/src views -name "*.sui" -newer <dist-stamp>` is empty
   *iff* every paired `.sui.d.ts` is also up to date — the regenerated-artifact
   contract.

## Fixes applied in this dispatch

None. The two clear bugs (`match_angle` string bail, `live` recogniser) live in
`crates/sui-compiler/src/emit/scope_factory.rs`, which is outside this audit's
allow-list. The build-pipeline `--no-dts` is in `packages/sui-build/src/runner.ts`,
also outside. Filed as recommendations above so a future dispatch can pick them up
together.

## Verification

```
$ find packages/ui-components/src views -name "*.sui" | wc -l
138
$ find packages/ui-components/src views -name "*.sui.d.ts" | wc -l
0
$ cargo build -p sui-compiler  # clean
$ ./target/debug/sui-compiler build <file> --component-name <stem>  # emits dts next to source
```

Spot-check artifacts were generated locally, inspected, and deleted; the worktree state
matches the start of the dispatch.
