# SUI tooling observations — running list

Scope: the developer-experience surface of the SUI rendering framework.
Spans both layers:

- **TS layer** — `packages/sui-runtime/`, `packages/sui-dom/`, `packages/sui-lsp/`, `packages/sui-grammar/`, `packages/ui-components/`.
- **Rust layer (ship-sui)** — `crates/sui-compiler/`, `crates/sui-signals/`, `crates/sui-lint/`, `crates/ship-sui/`, `crates/ship-sui-host/`, `crates/ship-sui-server/`, the daemon's SUI-serving routes in `apps/shipd/`, and the MCP tool surface.

Both sides are part of the same authoring DX. A typo in `.sui` flows through the Rust compiler; a wrong `<data source=` fails on ship-sui-server; an agent's quick-fix would need both LSP (TS) and the lint crate (Rust) to agree on rule IDs. Treat tooling as one concern.

This document is **append-only by section**. Add entries with date + context. Don't rewrite history — leave resolved items in place with their resolution noted.

---

## Standing concerns

### Performance instrumentation has no recorded baseline

What exists:
- `crates/sui-signals/benches/notify.rs` — criterion microbench, fanout 1/16/256/4096 subscribers.
- `packages/sui-runtime/fixtures/e2e/perf.mjs` — Playwright cold-load TTI (median of 3 across 8 fixture views) + 1000-cycle mount/unmount soak with heap-delta sampling. 25 KB leak threshold.
- CI bundle-size budget guard (commit `8db3de71`).
- Cross-browser E2E (chromium / firefox / webkit) — commit `2efa6f32`.
- A11y suite via axe-core — commit `f12c1d7d`.

What's missing: **no committed baseline numbers**. The benches run; the output isn't captured. "High-performing" is currently a *claim* backed by a sound architecture, not a *fact* with numbers to point at. Without a baseline, regressions show up as anecdote rather than failure.

Action proposed: commit a `baselines/` artifact (date-stamped JSON or markdown) under `.artifacts/sui-arc/benches/` capturing one run per measurement source. Update on every release. CI can then diff.

### `mount()` is Ship-daemon-coupled by design — modularity consideration

Confirmed during js-framework-benchmark Phase 1. SUI's `mount()` (in `packages/sui-runtime/src/mount.ts`) takes a `ViewPayload` with `html` (SSR-prerendered), `css`, `scope`, and `moduleUrl` — shaped for the daemon's `/_sui/views/<name>` endpoint.

This is **intentional and correct**: SUI is for Ship apps first, not generic web apps. The custom-element path (component() factory + `customElements.define`) is the escape hatch for "standalone, no daemon" — it works perfectly (the benchmark entry uses it).

**Modularity consideration (not a gap):** the current `mount()` signature couples to the daemon's specific payload shape. A future refactor could introduce a **contract** (interface) for "payload provider" so:
- The Ship daemon implements the contract (current behavior)
- A static-file host could implement it
- A test fixture could implement it
- Third-party adapters could too

This keeps SUI-for-Ship as the primary path while making the framework theoretically reusable. Out of scope for now; capture the design idea so it's not lost.

### js-framework-benchmark integration — Phase 1 landed

Implemented at `~/dev/js-framework-benchmark/frameworks/keyed/sui/` (sibling repo, not Ship). Build pipeline:

```
scripts/precompile.mjs   →  invokes Ship's target/debug/sui-compiler on src/*.sui
esbuild src/main.ts      →  bundles + minifies to dist/main.js
```

Total: 6 files, 265 LOC authored. Bundle: 23KB minified.

**Smoke test verified** (Playwright, served via python http.server):
- 6 benchmark buttons found by id
- `#run` creates 1000 rows
- `#swaprows` swaps positions 1 and 998 correctly
- `#clear` empties the table
- Zero page errors

**What this proves**: the SUI custom-element path is a standalone-bundleable framework. Custom elements register via `customElements.define()` when the compiled module is imported. No `mount()`, no SSR, no daemon needed. ~265 LOC of integration code vs Lit's similar size (~140 LOC for the impl + same package.json shell).

**Issue surfaced during Phase 1**: SUI's compile-time scope-identifier check (in `sui-compiler`) caught a real authoring error — handlers must be returned from the `scope:` factory, not bolted on via `Object.assign` in `mount()`. The compiler error pointed at the exact expression. Validates the "templates declare, controllers do" enforcement.

**Next phase (Phase 2)**: run the actual `webdriver-ts` harness for timing numbers vs Lit/Solid/Svelte/Vue. Requires `npm ci && npm run install-local` at the benchmark repo root (heavy install). Out of scope for inline-commander work; either dispatch a separate "run benchmark" job or do it manually.

### js-framework-benchmark — Phase 2 results (no throttling)

First full run completed. **9 of 12 ops passed; 3 failed with real bugs.**

CPU (lower = faster):

| Op | SUI median (ms) |
|---|---|
| 01_run1k | 44.9 |
| 02_replace1k | 56.1 |
| 04_select1k | **3.5** |
| 05_swap1k | 43.7 |
| 08_create1k-after1k_x2 | 56.8 |
| 09_clear1k_x8 | 9.6 |

Memory: 0.62 MB ready, 3.11 MB run, 0.75 MB run-clear (excellent).
Size: 22.4 KB uncompressed, **6.7 KB compressed** (competitive with Solid).
First paint: 59.5 ms.

#### Real bugs surfaced

1. **`bindFor` per-item attribute reactivity is broken** — promoted from "needs investigation" to "confirmed bug." `03_update10th1k_x16` failed because updating row.label on existing rows (same keys, new item object) didn't refresh the cell text. Same root cause as the sui-heatmap key-busting workaround we flagged earlier. Fix lives in `packages/sui-runtime/src/for.ts`.

2. **Click events fire twice on shadow-DOM buttons** — `07_create10k` failed with "at most one mousedown event is expected." Likely shadow-DOM retargeting causing duplicate event delivery. Quick test: re-run with `useShadowRoot: false`.

3. **`pierce/` selector can't click the remove span** — `06_remove-one-1k`. May be shadow-DOM-related (same as #2) or our DOM tree shape differs from Lit's in a subtle way.

4. **swap1k 43.7ms vs Lit ~13ms.** Likely the for reconciler isn't optimized for in-place rearrangement (a smart reconciler uses LIS to compute minimal moves). 3-4× perf opportunity after #1 is fixed.

#### Bench infrastructure issue (cosmetic)

The result filenames contain "sui-vERROR: Not found in package-lock" because the harness's `frameworkVersionFromPackage` lookup can't resolve a version for `@ship/sui-runtime` via the `file:` workspace dep. Fix: add a literal `version: "0.1.0"` (or similar) to `frameworks/keyed/sui/package.json` to bypass the lookup.

### js-framework-benchmark integration — assessment

**Repo**: https://github.com/krausest/js-framework-benchmark

The de-facto cross-framework standard. Compares React / Lit / Vue / Solid / Svelte / Preact / Inferno / etc. on a fixed set of operations (create 1000 rows, append 10000, swap two rows, remove row, clear, partial update, select row) with strict timing methodology (Puppeteer-driven, headless Chrome with throttling).

**Why bother:**
- Empirical, externally credible numbers vs every major framework. Independent of our own benches.
- The benchmark file is tiny (typically 100-200 LOC per framework). It IS the apples-to-apples test.
- Hits SUI's strongest hypotheses: signal-driven granular updates, keyed `{#for}` with stable reconciliation, no VDOM diff overhead.

**What it takes to run:**

1. **Fork the benchmark repo**. Add a `frameworks/keyed/sui/` directory (also a `non-keyed/sui/` for the without-`key=` variant, which will be slower — that's the point of comparison).
2. **Implementation file**: a single `.sui` view (or `<sui-*>` component) implementing the benchmark's contract — buttons with specific IDs (`add`, `run`, `runlots`, `update`, `clear`, `swaprows`, `select`, `delete`) and a `<tbody id="tbody">` of rows.
3. **Build config**: a `package.json` that produces a bundled output the benchmark harness expects. The benchmark uses Vite or webpack per framework; SUI's compiled output ships ESM modules that can be Vite-built. Need to package the runtime + dom + the view as one bundle.
4. **Manifest**: the benchmark wants a `frameworks/<keyed|non-keyed>/<name>/framework.json` declaring framework version, source, build command, type. Trivial to fill.
5. **Run**: `npm run bench keyed/sui`. Produces JSON results that the report tool tabulates.

**Real cost**:
- The implementation itself: 1 day for a clean keyed version, half a day for non-keyed. SUI's `{#for key={…}}` syntax is a direct match for the keyed contract.
- Packaging is the harder part. The harness expects a self-contained bundle; SUI's daemon-served model needs adaptation — either compile the view AOT to a standalone ESM (the runtime's `mount(host, payload)` API supports this), or stand up a tiny static server with the bundle.
- First run will reveal gaps in SUI's standalone (no-daemon) story. That's valuable signal.

**Suggested approach**:
1. Phase 1 — Standalone bundle proof: bundle `mount()` + a tiny benchmark view into one HTML file. Verify the benchmark harness can load it and call the right buttons. Doesn't even need to be checked into the benchmark repo yet — local Playwright run is enough.
2. Phase 2 — Fork the benchmark repo, add the directory, run against the standard matrix locally. Capture numbers.
3. Phase 3 — Decide whether to upstream a PR (after numbers look respectable). The benchmark's maintainer accepts PRs for new frameworks with reasonable acceptance criteria.

**Risks**:
- If numbers are bad, we'll know exactly why (the benchmark's microbench split makes it diagnostic). Better than not knowing.
- The benchmark's Chrome throttling model amplifies any startup-cost overhead. SUI's per-component ESM module loading might show up as noise on the smaller scenarios; package as one bundle to avoid.

### LSP is thin

What exists (`packages/sui-lsp/`):
- 12 files / 65+ tests
- Diagnostics surface for the existing lint rules
- Hover / go-to-definition for some symbols

Known gaps (from `.artifacts/sui-arc/audits/agent-authoring.md`):
- **No snippets** for `{#for}` / `{#if}` / `{#match}` / `{#while-loading}` blocks
- **No CEM-driven prop autocomplete** for `<sui-*>` tags (Phase 6 TODO)
- **No code actions / quick-fixes**. Examples that would be high-value:
  - "Add `key={item.id}` to {#for}" (sui-for-key fix)
  - "Replace `on:Click` with `on:click`" (sui-event-name fix)
  - "Wrap with `<data>` element" (when a `ship.*` call appears outside a `<data>` block)
  - "Convert to bindHtml()" (when innerHTML assignment is detected)
- **No rename refactor** (rename prop, rename custom element across views)
- **No error-recovery test coverage** in the existing 12 test files

### MCP surface for SUI authoring — missing

What exists today: `mcp__ship__ship_view_errors`, `mcp__ship__ship_view_lint`, `mcp__ship__ship_view_scaffold` (visible in this session's tool list). These cover runtime errors, lint diagnostics, and scaffolding — useful but agent-side.

What's missing:
- **`sui.compile(path)`** — run the Rust compiler on a `.sui` file, return diagnostics + emit artifacts. Closes the loop for agents working in the filesystem without an open LSP. ~150 LOC Rust + manifest.
- **`sui.lint(path)`** — lint-only variant for cheaper checks during authoring.
- **`sui.preview(path)`** — render a view to HTML server-side so the agent can verify markup output without booting a browser.
- **`sui.format(path)`** — opinionated formatting for `.sui` files. Avoids the back-and-forth of "is this indented right".

### Lint rule coverage

What's enforced today (`crates/sui-lint/src/rules/`):
- `controller-purity` — Error severity. Rejects `innerHTML`, `outerHTML`, `insertAdjacentHTML` in `*.controller.ts`.
- `css-token` — Error severity. Validates design-token usage in `<style>` blocks.
- `sui-for-key` — Warn severity (new this session). Flags `{#for}` blocks missing `key=`.
- `sui-event-name` — Warn severity (new this session). Flags `on:<event>` typos with Levenshtein suggestions.

What's missing:
- **`composition-validity`** — flag `<sui-rail-item>` (or any compound child) used outside its declared parent. Needs a per-component manifest declaring allowed parents.
- **`data-source-typecheck`** — verify `<data source="ship.X.Y()">` resolves to a real SDK call with matching arg types. Highest-value gap; everything else is friction, this is correctness.
- **`prop-default-type-mismatch`** — `<prop name="x" type="number" default="abc" />` should error at compile, not at runtime decode.
- **`unused-prop`** — declared `<prop>` with no template / controller reference. Warn.
- **`unused-bind-this`** — `bind:this={ref}` with `ref` never read in controller. Warn.
- **`reflect-bool-without-default-false`** — boolean props that reflect but default to true silently produce surprising attribute behavior. Warn.

### Compiler diagnostics quality

Sampled in the prior audit: spans are good. Messages are mixed:
- ✓ Good: "`<state>` is no longer supported; declare reactive state in the controller's `scope:` factory" — names the replacement.
- ✓ Good: "arrow functions are only allowed in `on:` event handlers" — clear contract.
- ✗ Poor: `on:Click={…}` (typo) — **silent**, no diagnostic. (Now caught by sui-event-name lint, but only at LSP time, not compile time.)
- ✗ Poor: `<data source="ship.bogus()" />` — silent until runtime.
- ✗ Poor: "`bind:this` requires a bare identifier" — names what's wrong but not what to do.

Action proposed: an audit pass that adds `suggest:` strings to every diagnostic missing one.

### `emit/js.rs` is over the file-size cap

`crates/sui-compiler/src/emit/js.rs` is 455 LOC — the form-value agent flagged it but did not refactor (out of scope). Real refactor debt; affects every change that touches emit. Split candidates:
- Per-binding emitters (text, attr, class, prop, style)
- Per-block emitters (`{#if}` / `{#for}` / `{#match}`)
- Module-shape header / footer
- The shared `Writer` is already separate; the split should keep referencing it.

### Ship agents → Claude subagents bridge

Confirmed earlier this session: the compile pipeline emits `.claude/agents/<id>.md` from `compiler::AgentProfile` but never loads `.ship/agents/*.jsonc` into that struct (only `ship-platform::AgentProfile`, which is a different type). Result: `.claude/agents/` is empty, dispatch via `Agent` tool with `subagent_type: ship-designer` etc. doesn't work.

Status: user is fixing this elsewhere. Tracked here for completeness.

### Compiler does not emit `<sui-component shadow="false" />` directive

`crates/sui-compiler/src/emit/js.rs:140` hardcodes `shadow: true` for every component. There's no way to opt out from a `.sui` file. Surfaced during js-framework-benchmark Phase 2 — needed to flip shadow off to test whether the `06_remove-one-1k` and `07_create10k` failures were shadow-DOM-related. Worked around with a sed-post-process in the benchmark's `precompile.mjs`.

Fix: add `<sui-component shadow="false" />` directive parsing. Top-level element, sets a flag on the component's IR, emit reads it. ~30-50 LOC compiler change. Until then, components are shadow-by-default with no opt-out.

### `{#for}` per-item attribute reactivity — CONFIRMED runtime+compiler bug

Originally suspected from `sui-heatmap` key-busting workaround. Confirmed twice more:

1. `sui-step-segments` avoided `{#for}` entirely (imperative createElement+appendChild) — likely because the agent hit this same bug and worked around it.
2. js-framework-benchmark `03_update10th1k_x16` fails: updating row.label by replacing the row object (same id, new label) leaves the cell text unchanged.

**Root cause** (now traced):

`packages/sui-runtime/src/for.ts:141-144` correctly calls `existing.update?.(item, i)` on matched keys. But the compiler's `{#for}` emit doesn't return an `update` hook in the render closure:

```js
render: (row, __i) => {
  const __childScope29 = Object.create($scope);
  __childScope29.row = row;          // ← sealed at first render
  __childScope29.__i = __i;
  return /* { nodes: [...] } */;     // ← no update field
}
```

The `update` call is always a no-op. Cached `__childScope29.row` stays the original object; bindings like `bindText(node, () => $scope.row.label)` keep reading the stale value.

**Fix paths** (compiler change in `crates/sui-compiler/src/emit/template_emit.rs` or equivalent):

- **(A) Item-as-signal**: emit `__childScope29.row = signal(row)`. Bindings read as `$scope.row().label`. update() calls `$scope.row.set(newRow)`. Reactivity natural.
- **(B) Item-as-plain + imperative re-run**: emit `update(newRow, newI) => { __childScope29.row = newRow; __childScope29.__i = newI; /* re-run each binding's effect */ }`. More plumbing to track per-item binding effects.

(A) is cleaner; (B) is more local but requires per-binding tracking. Recommend (A).

**Three known consequences** of this single bug:
- `sui-heatmap` controller workaround (embed fill in key)
- `sui-step-segments` imperative-DOM controller (avoided {#for} entirely)
- Benchmark 03_update10th1k_x16 failure

Fixing this single bug unblocks all three.

### Old: `{#for}` per-item reactivity — needs investigation (resolved by root-cause trace above)

Surfaced in sui-heatmap gate: the controller embeds the cell's `fill` string into the per-cell `{#for}` key (`c${ri}-${ci}:${fill}`) with this comment:

> bindFor reuses DOM nodes for matching keys, so a changed fill must produce a new key to force the rect to re-render.

The intent is right: when `data` changes, cell intensities change, and the SVG rect's fill must update. But the workaround says "force destroy+create per cell on every data change" — which means a heatmap with N cells re-creates N DOM nodes on every reactive change, even if only one cell's value moved.

The CORRECT SUI semantic (per how I read the runtime): when `cells()` emits a new array with matching keys, `bindFor` reconciles and per-item reactive bindings (like `style={cell.fill}`) re-evaluate against the new item object. No DOM destroy+create needed; the style attribute updates in place.

Either:
1. The agent's assumption is wrong (workaround is unnecessary churn), OR
2. `bindFor`'s per-item attribute reactivity has a real gap.

Need to either write a minimal repro or read `packages/sui-runtime/src/for.ts` end-to-end to confirm. If runtime is correct: simplify heatmap (key by position only). If runtime is broken: fix `bindFor` to re-evaluate per-item bindings on key-match.

### Lint rule gap — empty `.sui` + imperative-DOM controllers

Surfaced in sui-step-segments gate: the `.sui` body was just `<span bind:this={rowEl}></span>` and the controller built segment elements via `document.createElement` + `appendChild` in an effect. This violates the SUI authoring contract ("Templates declare. Controllers do.") AND the explicit anti-pattern documented in ship-components/SKILL.md:

> A `.sui` body that is just `<div bind:this={ref}></div>` — the template is the renderer

But the `sui-controller-purity` lint rule only flags `innerHTML` / `outerHTML` / `insertAdjacentHTML`. It does NOT flag `createElement` + `appendChild` constructing per-item DOM in a controller. Lint coverage gap.

Two follow-ups:
1. Rewrite sui-step-segments to use a `{#for}` block over a derived signal of parsed segments.
2. Extend `sui-controller-purity` (or add a new `template-is-renderer` rule) to flag empty `.sui` bodies and per-item `createElement` patterns. Detection heuristics: `.sui` template body contains only a single `<bind:this=ref>` element AND the paired controller has `createElement` followed by `appendChild` on that ref.

### dispatch.sh polish items

Surfaced in the three-port gate batch:

1. **`pnpm install --frozen-lockfile` exits non-zero on `IGNORED_BUILDS` warnings.** pnpm 11 prints "Ignored build scripts" then exits with 1 even when the install succeeded for everything else. My dispatch.sh's error branch fires spuriously. Fix: check for actual install failure (missing `node_modules/.modules.yaml` or similar) rather than relying on exit code, OR pass `--config.confirmModulesPurge=false --reporter=ndjson` and parse for real errors.

2. **pnpm 11's auto-prompt writes placeholder text into `pnpm-workspace.yaml`.** All three ports in this batch had identical `allowBuilds: { esbuild: 'set this to true or false', ... }` artifacts. The dispatch's `pnpm install` runs the prompt because pending-build-approvals are present, and pnpm modifies the workspace file to ask. Fix: either pre-allowlist the listed built-deps in `pnpm-workspace.yaml` (esbuild, rs-module-lexer, sharp, tree-sitter-cli, workerd), OR pass `--config.confirmModulesPurge=false` plus `--config.allowedBuildScripts=...`.

### Gate process — discovered gaps

Real gate verification (build + full test suite) caught issues that
inspection alone missed:

1. **Conflict markers can commit silently.** Merging job/sui-file-upload
   left `<<<<<<< HEAD` markers in `packages/ui-components/src/index.ts`.
   The merge commit landed with them in place because my mid-merge Edit
   returned "file modified since read" but I treated it as soft. The
   per-component vitest runs passed (didn't import index.ts), but
   `all-components.test.ts` broke immediately. Fixed in `73f9a492`.
   Lesson: always run the FULL test suite post-merge, not just the new
   component's tests.

2. **Edit tool "file modified since read" is hard, not soft.** When that
   warning surfaces mid-merge, re-read and retry — never assume the
   resolution stuck.

3. **`sui-grammar` tree-sitter native build fails on this WSL2
   environment.** `node-gyp configure error` on every pnpm install,
   blocks the whole install. Workaround: `pnpm install --ignore-scripts`.
   Needs separate investigation — likely missing build dep (Python,
   make, gcc) or node-gyp version incompat with Node 24.

4. **Pre-existing `pnpm-lock.yaml` drift.** `package.json` has `react` +
   `react-dom` in `pnpm.overrides`; the lockfile only had `react`.
   Fresh `pnpm install` regenerated the lock with both. The
   sui-navigation-menu agent's worktree saw this same drift and tried
   to "fix" it by removing the overrides block entirely — wrong
   direction. Lock is now back in sync after the post-gate verification
   ran install.

### Worktree safety — destructive git ops by dispatched agents

Discovered this session (root cause of the lost `apps/shipd/src/routes.rs` work): general-purpose subagents dispatched via Claude Code's `Agent` tool with no permission constraints ran `git stash && git stash drop` against the parent worktree as a "verify pre-existing failure" routine. The proper `dispatch.sh` flow uses fresh worktrees, but agents still had access to the parent through path.

Action proposed: extend the proposed permission constraints in `commander.jsonc`'s `tools_deny` to cover `Bash(git stash drop*)`, `Bash(git checkout --*)`, `Bash(git restore --*)`, `Bash(git reset --hard*)` against any path outside the agent's worktree. Currently relies on agent discipline; should be runtime-enforced.

### Skill currency — see follow-ups for in-flight work

The skill refresh dispatch (`job/sui-skill-refresh`) is in flight as of this writing. When it lands, this doc's "Skills" sub-section can be dropped — it's no longer a standing concern.

---

## Resolved

(Entries move here when fixed. Keep them with a one-line resolution.)

- F-D1 (general-purpose subagent dispatch) — resolved by switching to `dispatch.sh` + ship-designer for SUI work, this session.
- F-C5 / `form-value` directive — landed via `job/form-value` in `crates/sui-compiler/` + `packages/sui-runtime/` + 5 form components retrofitted.

---

## Open questions (for the human)

1. **js-framework-benchmark**: Phase 1 (standalone bundle proof) — do this in-tree under `.artifacts/sui-arc/bench-jsfb/`, or in a side checkout? In-tree gives commit history of attempts; side checkout keeps the main branch clean.
2. **MCP `sui.compile()` tool**: implement as a ship-sui-server MCP method, or as a separate `sui-mcp` crate sibling to `crates/sui-lint`? The former matches today's "everything goes through shipd" pattern; the latter would let agents call it without spinning up the daemon.
3. **Compiler emit/js.rs split**: ship-designer's lane or rust-lane's? It's Rust code authoring `.sui` output — falls between the two by intent. Lean rust-lane since the file's correctness model is Rust-shaped.
4. **Lint severity escalation**: keep `sui-for-key` and `sui-event-name` at Warn (LSP-only), or escalate to Error (blocks compile) after the existing codebase is clean? Warn keeps existing views buildable; Error makes the lint a hard wall.
