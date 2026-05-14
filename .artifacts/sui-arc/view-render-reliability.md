# View render reliability — framework fixes + before/after probe

**Branch:** `job/view-render-reliability`
**Probe target:** shell2 at `http://127.0.0.1:3291` (this worktree's vite, daemon 9406)
**Before screenshots:** `.artifacts/sui-arc/probe-shots/` (from shell2-fundamentals)
**After screenshots:** `.artifacts/sui-arc/probe-shots-after/`

Three reliability bugs in the SUI framework — surfacing as five distinct
probe symptoms — fixed at the compiler / host / runtime boundary. No
view source touched, no `packages/ui-components/src/sui-*/` source
touched.

## Diagnoses

### 1. `/git` rail-click → `Mount failed: Failed to resolve module specifier "@ship/sui-..."`

**Root cause.** The compiler emits subpath imports of
`@ship/sui-runtime` — `@ship/sui-runtime/template-hooks` for views
containing `<use-*>` (every view that uses `<use-kbd>` etc.), and
`@ship/sui-runtime/route` for views that reference `route` in
expressions. The daemon's `/view-host` page maps both subpaths to
`/_sui/runtime.js` in its inline importmap (see
`crates/ship-sui-server/assets/view-host.html:13-23`), so `/sui/<view>`
direct goto works. shell2's importmap (`apps/shell2/index.html:32-40`)
only mapped the bare `@ship/sui-runtime` specifier, so any view
imported via shell2's rail-click navigation that touched a subpath
threw at module parse time.

The git view (`views/git/page.sui:1-5`) starts with `<use-kbd>`, which
triggers the `template-hooks` subpath import — caught by the probe as
the canonical mount failure.

**Fix.** Widen shell2's importmap to match view-host's. Both subpaths
point at the same runtime bundle URL — re-resolving to the same
module-record so the runtime universe stays one-instance. See
`apps/shell2/index.html`.

### 2. `r.headerEl is not a function` / `r.bodyEl is not a function`

**Root cause.** A `bind:this={var}` target inside a conditional /
loop branch only has its scope slot installed by `attachRef($scope,
"var", el)` (compiler emit in
`crates/sui-compiler/src/emit/template_emit/dom_emit.rs:117-135`) WHEN
the branch renders. `sui-dialog` (`packages/ui-components/src/sui-dialog/sui-dialog.sui:165-176`)
puts `bind:this={headerEl}` and `bind:this={bodyEl}` inside
`{#if open()}`. The controller's
`mount()` (`packages/ui-components/src/sui-dialog/sui-dialog.controller.ts:24-31`)
reads `scope.headerEl()` from an effect that runs on mount — before
`open()` is ever true, the scope key is `undefined` and `undefined()`
throws `headerEl is not a function`. Every dialog used by the chrome
(`sui-launcher`, `sui-command-bar`) and every panel mounted with a
conditional bind:this target hit this on first paint.

The `bind_this_target` check (`crates/sui-compiler/src/check/bind_this_target.rs`)
only enforces that a `bind:this={var}` target maps to a `var: ref(null)`
key in the controller's `scope:` factory IF the controller has a scope
factory at all (line 81: `if !table.scope.is_empty()`). Controllers
without a scope factory — like `sui-dialog`'s — pass the check, but
their bind:this slots never get pre-seeded.

**Fix.** The compiler now pre-seeds `$scope.<var> = signal(null)` for
every `bind:this` target discovered by walking the entire template
(including branches inside `{#if}`, `{#for}`, `{#match}`,
`{#while-loading}`, `{#while-error}`). The seed runs after data
declarations and BEFORE the controller's `setup.setup(ctx)` call, so
either the controller's own `scope:` factory overwrites the slot with
an author-declared `ref(null)` (preserving previous behavior), or the
pre-seeded `signal(null)` remains in place. Either way,
`scope.headerEl()` is always callable at mount() time — returning
`null` until the binding's element actually renders, then the real
element. See `crates/sui-compiler/src/emit/js/setup.rs` and
`crates/sui-compiler/src/lower/bindings.rs`.

### 3. `/sui/skills`, `/sui/mesh`, `/sui/events`, `/sui/settings` hang on `networkidle`

**Root cause.** These views own a `live()` source that subscribes to a
server-sent-events stream (the daemon's `/_sui/data?expr=…` endpoint).
EventSource subscriptions keep the connection open for the lifetime of
the mount — that's the point. Playwright's `waitUntil: 'networkidle'`
gate fires only when the network has been idle for 500ms with zero
in-flight requests, so as long as the SSE socket stays open
`networkidle` never resolves. With the probe's 4 s timeout, every live
view registered as TIMEOUT.

This is a probe-contract bug, not a runtime bug — the runtime
correctly opens the live channel and renders SSR content while the
patches stream in. The fix is to switch the verification gate to
`waitUntil: 'load'` (fires after the document + initial sub-resources
parse, which is by definition before the EventSource subscription is
considered "subsequent") and check rendered content via DOM-element
count rather than `body.innerText` (loading skeletons have zero
visible text but real rendered structure).

**Fix.** `apps/shell2/probe.mjs` updated:
- Direct goto uses `waitUntil: 'load'` with a 5 s budget (matches the
  acceptance criteria).
- After load, wait for `#sui-root` to gain a child element (1.5 s cap)
  so the probe captures post-mount state, not the pre-mount empty
  placeholder.
- Trace now records `contentEls` alongside `contentLen`. A view with
  `len=0 els=53` is a healthy loading skeleton, not "silent empty".
- `BASE` derived from `SHELL2_URL` env or the local daemon's port file,
  so the probe runs against the current worktree's shell2 (not a
  stale 3270).

### 4. `/sui/git` → `200 len=0` (empty body)

**Same root cause as (3).** The pre-fix probe used `innerText` for
content presence. Git's SSR-rendered initial state is the `{#while-loading}`
branch — a skeleton tree of `<sui-skeleton>` nodes with zero visible
text — and the runtime's SSE patch for `ship.git.overview()` hadn't
landed in the 200 ms wait window. After the probe's content-presence
metric switched to DOM element count, `/sui/git` reports
`len=0 els=53` — the skeleton renders correctly, and the underlying
view payload contract is intact (verified via
`crates/ship-sui-server/src/view_payload_tests::view_host_without_ssr_live_emits_from_source`).

## Before / after — direct `/sui/<view>` probe

| View | Before | After |
|---|---|---|
| `/sui/home` | 200 len=… | 200 len=501 els=29 |
| `/sui/agents` | 200 (OK) | 200 len=27 els=108 |
| `/sui/overview` | 200 (OK) | 200 len=329 els=94 |
| `/sui/sessions` | 200 (OK) | 200 len=44 els=79 |
| `/sui/workspaces` | 200 (OK) | 200 len=28 els=55 |
| `/sui/skills` | **TIMEOUT (networkidle)** | **200 len=63 els=59** |
| `/sui/chat` | 200 (OK) | 200 len=9 els=53 |
| `/sui/mesh` | **TIMEOUT** | **200 len=0 els=64** |
| `/sui/events` | **TIMEOUT** | **200 len=6 els=35** |
| `/sui/settings` | **TIMEOUT** | **200 len=38 els=54** |
| `/sui/git` | **200 len=0** (silent empty) | **200 len=0 els=53** (loading skeleton — honest state) |
| `/sui/canvas` | 200 (loading-stuck) | 200 len=357 els=20 |
| `/sui/setup` | 200 (loading-stuck) | 200 len=0 els=9 |

## Before / after — rail-click

| Click | Before | After |
|---|---|---|
| `/git` | **`Mount failed: Failed to resolve module specifier "@ship/sui-..."`** | **mounts cleanly (loading state, no console error)** |

After-state screenshot:
`.artifacts/sui-arc/probe-shots-after/02-rail-click-2-_git.png`.

## Before / after — console errors (acceptance §4)

```
$ grep -E "headerEl|bodyEl|Mount failed|Failed to resolve module specifier" \
    .artifacts/sui-arc/probe-shots-after/console.log
# (no matches)
```

The pre-fix probe captured `[sui] effect threw: TypeError: r.headerEl is not a function`
and the matching `r.bodyEl` line on every navigation that touched a
dialog / drawer. Both are gone.

Other console errors that DO remain (`canvasRef` undefined in
overview, `route is not a function` in chat) are view-content bugs in
`views/overview/` and `views/chat/` — out of scope (see file scope §
Forbidden).

## Acceptance criteria

1. ✓ `/git` rail-click → mounts cleanly, no `Mount failed` console
   error. *(see `probe-shots-after/02-rail-click-2-_git.png`)*
2. ✓ `/sui/skills`, `/sui/mesh`, `/sui/events`, `/sui/settings` direct
   goto → `waitUntil: 'load'` resolves within 5 s AND renders content
   (els≥35 each).
3. ✓ `/sui/git` direct goto → renders the `{#while-loading}` skeleton
   (els=53). Honest loading state, not silent empty.
4. ✓ `r.headerEl is not a function` and `r.bodyEl is not a function`
   console errors are gone.
5. ✓ Re-run probe captures attached
   (`.artifacts/sui-arc/probe-shots-after/`); side-by-side above.
6. ✓ `cargo test -p sui-compiler` and `cargo test -p ship-sui-server`
   pass (34 + 34 tests).
   `pnpm -F @ship/sui-runtime test` passes (478 / 483 — 5 pre-existing
   skips). `pnpm -F @ship/ui-components test` fails to load with
   `Failed to resolve import "../../ship-set/src/web-components/core/lucide-icons.generated"`
   in `test/setup.ts:10:29` — reproducible on `main` (verified via
   stash) so the breakage predates this branch.
7. ✓ NO `views/*` or `packages/ui-components/src/sui-*` source
   modified. (Branch diff in `git status` covers compiler + host +
   shell2 + sui-components.js asset rebuild only.)

## Files touched

- `crates/sui-compiler/src/emit/js/setup.rs` — `emit_ref_slot_seeds` +
  `collect_bind_this_targets`; call inserted between data decls and
  setup invocation. (+51 LOC)
- `crates/sui-compiler/src/lower/bindings.rs` — `bind:this` now also
  imports `signal` from `@ship/sui-runtime`. (+1 LOC)
- `crates/sui-compiler/tests/fixtures/bind-this-button.expected.js` —
  snapshot refreshed (one added line: `$scope["btnEl"] = signal(null);`).
- `apps/shell2/index.html` — importmap widened to match view-host
  (added `@ship/sui-runtime/route` and `@ship/sui-runtime/template-hooks`).
- `apps/shell2/probe.mjs` — `waitUntil: 'load'`, content-presence via
  element count, base URL auto-detection from local daemon's port
  file.
- `crates/ship-sui-server/assets/sui-components.js` — regenerated by
  `pnpm -F @ship/ui-components build`; baked into the daemon binary
  via `include_str!`. Now carries the pre-seeded refs for every
  `<sui-*>` component with conditional bind:this targets.

Total framework-code delta (Rust + TS host + HTML host, excluding
asset regeneration): ≈ 65 LOC across 3 files. Each fix < 80 LOC as
required.

## Reproducer

```
just dev-restart                                  # rebuild daemon with new compiler
SHELL2_URL=http://127.0.0.1:3291 \
  node apps/shell2/probe.mjs                      # ports auto-detected; SHELL2_URL overrides
```

`.artifacts/sui-arc/probe-shots-after/console.log` captures the post-fix
console log. The trace JSON includes both rail + nav inventories and
per-view content metrics.
