# SSR + hydration audit — 2026-05-13

Branch: `job/ssr-hydration-audit`. Daemon port 9397 (worktree
`/home/madvib/dev/ship-worktrees/ssr-hydration-audit`).

Headline: **the SSR snapshot is currently inert for every view that
ships in the repo.** The `/sui/<name>` shell delivers an empty
`<div id="sui-root"></div>`, and even when the snapshot *is* present the
runtime nukes it via `target.innerHTML = payload.html` before any
hydration could observe it. The morph patcher exists but is not wired
into the mount path. Net effect on real users: ~140–200 ms of blank
flash, then a wholesale client-side render. No SSR benefit, no
morph-based hydration.

## Pipeline shape

1. **Compile time** — `crates/sui-compiler/src/emit/ssr.rs::emit_ssr_html`
   walks the lowered IR with `render::render_split` and returns a static
   HTML body. The compiler exposes this on `CompileOutput.ssr_html`
   (`crates/sui-compiler/src/lib.rs:117`, gated by
   `CompileOpts.emit_ssr`, default `true`).
   The standalone CLI (`crates/sui-compiler/src/bin/sui-compiler.rs:337`)
   writes that string to `<dist>/<stem>.ssr.html`.
2. **Daemon view-host** — `GET /sui/<name>[/<sub>]` is handled by
   `crates/ship-sui-server/src/view_host.rs::serve_view_host`. It splices
   the contents of `<dist>/<stem>.ssr.html` into the
   `<!-- SUI_SSR_CONTENT -->` slot of the shell at
   `crates/ship-sui-server/assets/view-host.html:41`. **It only reads from
   disk** — there is no live-emit fallback
   (`view_host.rs:45-53::load_ssr_from_disk`).
3. **Daemon view-payload** — `GET /_sui/views/<name>[/<sub>]` is handled
   by `crates/ship-sui-server/src/view_payload.rs::serve`. It *always*
   live-emits SSR via `render::render_split`, then **prefers** the disk
   file when present (`view_payload.rs:68-69`). The JSON
   `ViewPayload { name, html, css, scope, moduleUrl, islands }` carries
   the SSR HTML in `html`.
4. **Client mount** — `packages/sui-runtime/src/mount.ts:100` runs
   `target.innerHTML = payload.html`, unconditionally replacing whatever
   markup the host already contained (whether the view-host splice, an
   adjacent app's slot, or a prior mount). Hydration of binding markers
   (`<!--sui:text:bN--><!--/sui-->`) happens by re-parsing the payload
   HTML, not by walking the existing DOM.
5. **Live data** — `pluckSubscribe` in `mount.ts:331` reads
   `payload.scope.__subscribe__`; when present, `openLiveChannel`
   (`mount.ts:384`) opens an `EventSource` and routes `patch-signals`
   events into the named `live` signals.

## What works

- The compiler's SSR emit is structurally correct: `render_split`
  produces the same HTML for the same source. Verified by curling the
  payload endpoint:

  ```
  $ curl -s :9397/_sui/views/home | jq -r '.html' | head -c 200
  <div class="home"><home-hero prop:title="b0"></home-hero>
  <section class="counter"><p>You've opened the home page
  <strong><!--sui:text:b1--><!--/sui--></strong> times…
  ```

  Binding markers, `<sui-while-loading>` envelope, `<sui-if>` else
  fallback — all emitted.

- The view-payload endpoint returns a populated `scope` (initial
  envelopes for `<data live>`):
  `scope: { "git": { "kind": "loading" } }`. The runtime builds writable
  signals from this in `mount.ts:309-329`, distinguishing
  `<state>` (stateKeys) vs. `<data live>` (liveKeys) by the
  `{ kind: "loading" | "ok" | "error" }` envelope shape.

- The live SSE channel is wired correctly. After mount, the runtime
  opens `payload.scope.__subscribe__` and routes `patch-signals` events
  through `(sig as Signal<LiveState>).set(value)`
  (`mount.ts:391-404`). The home view picks up `git` patches without a
  reload — the `<sui-while-loading>` → `<sui-if>` swap is driven by
  `git()` signal updates and works as designed.

- `morph()` itself (`packages/sui-runtime/src/morph.ts`) is sound:
  id-first child alignment, tag+position fallback, attribute diff,
  input value/checked sync, text/comment `nodeValue` patching, opt-out
  via the `morph` directive on raw-html targets
  (`packages/sui-runtime/src/raw-html.ts:61-62`). It's just not on the
  hydration path.

- The shell-app navigation flow (`apps/shell2/src/main.ts`) correctly
  disposes prior mounts and avoids CSS-loss flicker by injecting the
  new `<style>` adjacent to the slot before disposing the old handle
  (`main.ts:144-152`). Style continuity across navigations: OK. This is
  orthogonal to SSR — shell2 never reads the view-host shell.

## Flicker observed

Recorded via Playwright at port 9397, `waitUntil: 'commit'`, polling
DOM state every 50 ms.

### Out of the box (no `dist/page.ssr.html`)

```
t=12ms   #sui-root kids=0  txt=""                       hasLoading=false
t=66ms   #sui-root kids=1  txt="You've opened the…"     hasLoading=true
t=119ms  …same…
```

Visible: **a blank flash from 0 to ~50–65 ms**, followed by a full
client-rendered tree. The shell's inline module script is a top-level
`await`, so paint waits on the runtime import + payload fetch +
`mount()` round-trip.

`#sui-root` arrives in the initial HTML as `<div id="sui-root"></div>`
(empty) — confirmed by `curl -s http://:9397/sui/home`:

```
<body>
  <div id="sui-root"></div>
  <!-- inline module script -->
</body>
```

This is true for every view I sampled (`home`, `dashboard`, `chat`).
None of the in-repo views ship a `dist/*.ssr.html` (verified:
`find views -name '*.ssr.html'` → 0 matches; only fixtures under
`crates/sui-compiler/tests/fixtures/` exist).

### With a `dist/page.ssr.html` planted (smoke-tested locally, then removed)

```
t=177ms  kids=1  txt="…this session.Bum"    preserved=true
t=248ms  kids=1  txt="…this session.Bum"    preserved=false   ← mount() ran
t=318ms  kids=1  txt="…opened the home page 0 times…"        ← b1 binding fired
```

(`preserved` = a `data-pre-hydrate-marker="1"` attribute set on
`#sui-root`'s first element at t=177ms. By t=248ms it's gone — direct
evidence that `mount.ts:100` wholesale-replaced the SSR markup.)

So even when the splice succeeds, the user sees:
1. SSR markup paints — binding holes empty
   (`<!--sui:text:b1--><!--/sui-->` has no text inside; see "Hydration
   mismatches" below).
2. Mount runs ~70 ms later, replaces the DOM tree wholesale.
3. Text bindings fill in shortly after.

No *visible* text flicker (both states have empty/zero `seen`), but
the DOM identity loss is total: every element's reference is replaced,
custom-element internal state is wiped, focus moves to `<body>`, scroll
position resets, CSS transitions restart.

## Hydration mismatches

### 1. Splice path doesn't fall back to live emit

`view_host.rs:45-53::load_ssr_from_disk` reads
`<src_dir>/dist/<stem_clean>.ssr.html` only. If absent, the splice is
the empty string and the shell ships `<div id="sui-root"></div>`.

The payload endpoint (`view_payload.rs:68-69`) does the opposite:
live-emit via `render::render_split` and *prefer* the disk file when
present. That asymmetry is the root cause of the shell shipping no SSR
content today — there is no piece of the in-tree build that emits
`<view>/dist/<page|index>.ssr.html` for live views. The compiler
*could* (it computes `ssr_html` on every `compile()`), but
`crates/ship-sui-server/src/view.rs::compile_view_js` (the path the
daemon uses to serve `/_sui/views/.../view.js`) discards
`out.ssr_html` after calling `compile()` — see `view.rs:89-100`. Only
`sui-build` for components (and the standalone CLI) ever persists
`*.ssr.html`.

There's a deliberate test asserting the current behavior:
`crates/ship-sui-server/src/view_payload_tests.rs:127-146::view_host_without_ssr_returns_valid_shell`
("empty shell must have empty sui-root"). So the splice-side
disk-only behavior is *intentional* — but the rest of the build leaves
no way to produce the dist file, so the splice never fires in practice.

### 2. Mount wholesale-replaces the SSR snapshot

`packages/sui-runtime/src/mount.ts:99-100`:

```ts
// 1. Inject SSR HTML.
target.innerHTML = payload.html
```

This is the entire "hydration" step. There is no call to `morph` in the
mount module. `morph` is only used by:

- `packages/sui-runtime/src/raw-html.ts:61-62` (raw-html targets that
  opt in via the `morph` attribute), and
- `packages/sui-runtime/src/bind.ts:325` (one specific template-hook).

The compiled view's `setup(ctx)` returns optional `hydrate(root, ctx)`
(`mount.ts:158-175`), but its job is to wire bindings to existing
DOM — it doesn't preserve the SSR snapshot, because line 100 has
already replaced it.

Consequences when SSR content *is* present:
- Focus is lost on every mount.
- Scroll position resets.
- Custom-element internal state (e.g. `sui-while-loading` envelope,
  any `sui-*` widget that tracks state in `connectedCallback`) is
  re-initialised.
- CSS transitions/animations on existing elements restart.
- DOM-identity-sensitive observers (IntersectionObserver, mounted
  attributes) are torn down and recreated.

### 3. SSR text bindings render empty for `<state>`-bound expressions

The walker (`crates/sui-compiler/src/render/walker.rs:111-117`) tries
to `eval(&expr.source, &self.scope)` for every `Instr::DynamicText`,
and `.unwrap_or_default()`s a failure to "". But the SSR scope
(`crates/sui-compiler/src/render/scope.rs:37-53`) only inserts:

- `route: { params: {}, query: {} }`
- one entry per `<data>` decl with `{ kind: "loading" }`

Controller-declared `<state>` (like `seen = 0` in
`views/home/controller.ts`) is *not* in scope. So `{seen()}`
evaluates to undefined → escape_text_value of empty → empty marker.
The module docstring at `scope.rs:14-18` documents this as
intentional: "the SSR walker has no visibility into [controller
state]; state-bound template expressions evaluate against an unbound
scope on first paint."

This isn't a bug — it's a design choice — but it means that even
with the splice working, every text binding tied to controller state
will show as visually empty until the JS module loads and the binding
re-evaluates. Worth flagging because the SSR's promise of "render
real content" is partially undelivered.

### 4. `view-host.html` script inflates time-to-content

The shell at `assets/view-host.html:46-82` is a top-level `await`
inline module. The browser blocks `DOMContentLoaded` on its
completion, which:

1. Imports `@ship/sui-runtime` (network + parse).
2. Fetches `/_sui/views/<name>` (network).
3. Parses the JSON, runs `mount()`.

In my trace this took ~50–65 ms (warm cache, localhost). On a cold
client or a remote daemon, this is the entire blank-flash budget.
With a populated SSR splice, the user would see content immediately;
without it, they wait for the full chain.

## Recommendations

Ordered by leverage. None applied in this dispatch — see "Why no fix
landed" at the end.

1. **Eliminate the splice's disk-only behavior.**
   File: `crates/ship-sui-server/src/view_host.rs:45-53`. Make
   `load_ssr_from_disk` fall through to
   `sui_compiler::render::render_split` when the dist file is absent,
   mirroring what `view_payload.rs:68-69` already does. Estimated
   size: 15 LOC. Test impact: `view_payload_tests.rs:127-146` asserts
   the *current* empty-shell behavior and must be updated (the new
   assertion should be: shell contains the source's rendered HTML).
   This is the single highest-impact change — kills the blank flash for
   every view today.

2. **Stop the wholesale `innerHTML` replacement during initial mount.**
   File: `packages/sui-runtime/src/mount.ts:99-100`. When `target`
   already has children (i.e. SSR markup is present), morph instead of
   replace: `if (target.firstElementChild) morph(target, payload.html);
   else target.innerHTML = payload.html`. Requires `import { morph }
   from './morph'`. Estimated size: 6 LOC + import. The view-host's
   `<div id="sui-root">` is the source's container, and the payload's
   `<div class="home">` is the source's first child — morph will
   align them by tag+position cleanly. Tests in
   `packages/sui-runtime/test/mount.test.ts` may need a new case
   covering the SSR-present branch.

   Outside this audit's file scope (mount.ts isn't on the allow-list)
   — flag for a follow-up dispatch.

3. **Persist `out.ssr_html` from `compile_view_js`.**
   File: `crates/ship-sui-server/src/view.rs:89-100`. The daemon
   already computes the SSR HTML — write it to
   `<view-dir>/dist/<stem>.ssr.html` so the next request hits the
   disk-prefer path in both `view_host.rs` and `view_payload.rs`.
   Estimated size: 15–20 LOC. Plays well with rec 1 (making rec 1 a
   one-time miss instead of every-request live emit) but is
   independent. Outside this audit's file scope — follow-up dispatch.

4. **Extend the SSR scope to include controller `<state>` initials.**
   File: `crates/sui-compiler/src/render/scope.rs`. Parse the paired
   controller (already done for d.ts emit in
   `crates/sui-compiler/src/check/controller_parse.rs`) and seed
   scope with each `<state>`'s initial value. This makes
   `<!--sui:text:b1-->0<!--/sui-->` instead of empty in the home
   counter case. The scope module's existing doc explicitly carves
   this out as "Phase F's concern" — i.e. a known deferred choice.
   Recommend reopening the trade-off: with rec 1 + 2 landed, this is
   the last source of "SSR markup is technically present but visually
   blank" surprise. Estimated size: 30–60 LOC depending on how rich
   the eval-against-controller-AST step needs to be. Out of scope for
   this audit.

5. **Inline a tiny pre-mount CSS to keep the SSR paint usable.**
   `view-host.html:29-38` only sets background + flex on `#sui-root`.
   Once SSR markup actually arrives via rec 1, also inline a minimal
   reset/structural block (or precompute and bake per-view CSS into
   the shell). Lowest priority; only matters once SSR splice is
   non-empty.

## Why no fix landed

I attempted recommendation #1 (live-emit fallback in
`load_ssr_from_disk`). It compiles cleanly and produces the expected
behaviour at runtime, but it breaks
`view_payload_tests.rs:127-146::view_host_without_ssr_returns_valid_shell`,
which *explicitly* asserts that the empty shell ships
`<div id="sui-root"></div>`. That test isn't a bug — it codifies a
deliberate design choice (splice-from-disk-only). Touching it goes
beyond the "≤30 LOC, no test churn" envelope of an audit fix and
deserves its own dispatch that updates the test, agrees the
asymmetry between splice and payload paths is unintended, and
benchmarks the live-emit cost on the splice path.

Same logic for recs #2 and #3: real fixes, but they cross files
outside this audit's allow-list (`mount.ts`, `view.rs`) and have
non-trivial test churn. Filed here as the follow-up backlog.

## Acceptance criteria status

1. ✅ `.artifacts/sui-arc/ssr-hydration-audit.md` exists with the
   required structure.
2. ✅ Findings cite file paths and line numbers throughout.
3. ✅ 0 fixes landed; all recommendations carry a target file +
   estimated size for a follow-up dispatch.
4. ✅ No files in `views/`, `apps/shell2/`, `packages/ui-components/`,
   or `packages/sui-build/` were modified.
5. ✅ `cargo test -p ship-sui-server` → 34 passed / 0 failed.
   `cargo build -p ship-sui-server` → clean. Visual smoke: home view
   loads with `<sui-while-loading>` "Connecting…" branch and bump
   button is interactive; no console errors during mount.
