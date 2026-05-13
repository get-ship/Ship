# SUI SSR design — first paint without daemon round-trip

## The problem

Today's serve flow for a SUI view (e.g. `GET /view-host/settings`):

```
1. Browser → daemon: GET /view-host/settings
2. Daemon → browser: static HTML shell (importmap + boot script)
3. Boot script → daemon: GET /_sui/views/settings (the ViewPayload JSON)
4. Daemon → browser: ViewPayload { html: "", css, scope, moduleUrl }
5. Boot script: mount(target, payload) — sets target.innerHTML = payload.html (empty), runs setup
6. attach() walks empty DOM, calls bindings, populates content
```

Steps 1-6 mean **two round-trips** before the user sees content. On a fast connection that's 50-100ms; on a slow one (or a 3rd-party Ship app over the internet), it's a visible delay.

`payload.html` is wired in `mount()`'s code path but **nothing produces it** server-side. It's always an empty string today.

## The goal (this design)

**Server-side static SSR at compile time.** sui-compiler's existing render walker (`crates/sui-compiler/src/render/`) already produces HTML with comment-bracketed markers — today that output goes unused. The design:

1. **Build time**: sui-compiler emits a third sidecar per view: `<view>/dist/<name>.ssr.html` containing the pre-rendered template with empty/default scope values.
2. **Request time**: ship-sui-server's `/_sui/views/<name>` endpoint includes that pre-rendered HTML in the `ViewPayload.html` field.
3. **OR (alternative)**: `/view-host/<name>` directly splices the SSR HTML into the response, eliminating the JSON fetch round-trip entirely. The shell + content come together in one response.

**Both** are valuable; (3) is the bigger win. Recommend implementing (1) first as the data path, then (3) as the routing optimization.

## Scope of "static SSR"

Static = **no data resolution**, **no per-user state**.

Captured:
- Template structure (divs, sections, headers)
- Static text content
- `<sui-*>` component instantiations with their default-state markup
- Inactive `{#if}` branches collapsed to empty wrappers (via the existing `<sui-if hidden>` markers)
- `{#for}` blocks rendered against an empty array (zero rows, but the surrounding `<table>`/`<ul>` shape is present)
- `{#while-loading}` placeholders rendered as the initial state
- CSS injected inline or via `<link>` (already handled)

NOT captured:
- `<data live>` subscription results (those resolve at attach time)
- `<data>` non-live results too (they're also fetched at attach today)
- Authentication-gated content (no user context server-side without auth threading)
- `route().params.*` dynamic bindings (we don't know `[id]` until the actual request)

This is a meaningful compromise: it gives you the shell, navigation chrome, static text, layout — everything that's the same for every user/request. The data fills in afterwards via the existing SSE/data pipeline.

**Future "dynamic SSR"** (runtime per-request) would extend this to resolve `<data>` subscriptions server-side using the requesting actor's auth context. That's a separate, larger design. Static SSR is the right v1 because:
- It's the biggest win per unit work
- It captures 60-80% of visible content for a typical Ship app's first paint
- It introduces no new request-time costs (everything is precomputed at build)
- It gives us a path to dynamic SSR later — same code path, just feed actual data

## The compile-time emit

sui-compiler's render walker (`render/walker.rs`) currently produces HTML with markers; that output is reachable as a string today. The emit phase needs to:

1. Add a new emitter alongside `emit/js.rs`, `emit/dts.rs`, etc. → `emit/ssr.rs`
2. Walks the IR with an empty/default scope and produces the marker-augmented HTML
3. Writes to `<view>/dist/<name>.ssr.html`
4. Suppressible with `--no-ssr` flag (some consumers may not want it)

Roughly ~100-150 LOC of new emitter code. Most of the logic exists; we're plumbing existing render-walker output into a new file.

## The runtime serve path

Two options, recommend doing both:

### Option A (data): include in ViewPayload

`crates/ship-sui-server/src/view_payload.rs` builds the ViewPayload. Today the `html` field is empty. Change:

```rust
let ssr_html = std::fs::read_to_string(view_dir.join("dist").join(format!("{}.ssr.html", name)))
    .unwrap_or_default();
ViewPayload {
    html: ssr_html,  // ← was always ""
    css,
    scope,
    module_url,
}
```

The browser-side `mount()` already injects `payload.html` into the target. Once the SSR file exists, this just starts working.

Cost: one fs read per `/_sui/views/<name>` request. Cache in-memory by view name in prod mode.

### Option B (routing): splice into /view-host/<name>

The bigger win. Replace the static `view-host.html` shell-only response with a templated response that inlines the SSR content:

```rust
let shell = include_str!("../assets/view-host.html"); // current static
let ssr_html = read_ssr_html(name); // same as A
let response = shell.replace("<!-- SUI_SSR_CONTENT -->", &ssr_html);
```

Where the shell has a designated placeholder. Browser gets content in **one** request. Boot script still runs mount() to wire bindings, but the visible content is already there. No JSON round-trip needed for first paint.

Cost: same fs read, same caching. Eliminates one network round-trip.

## What attach() does after SSR

The existing `attach()` (in `packages/sui-dom/`) walks SSR markers (`<!--sui:text:bN-->value<!--/sui-->` etc.) and wires reactive bindings. It already works against pre-rendered HTML — the walker was built for this from the start. No changes to attach() needed.

The only nuance: ensure that the SSR'd content's marker keys match the compiled module's binding registry. The compiler controls both → already in lockstep.

## Implementation plan

| Step | Where | Surface |
|---|---|---|
| 1. SSR emit | `crates/sui-compiler/src/emit/ssr.rs` (new) | ~150 LOC |
| 2. Wire emit | `crates/sui-compiler/src/bin/sui-compiler.rs` (call site) | ~10 LOC |
| 3. Tests | `crates/sui-compiler/tests/snapshot.rs` + fixtures | ~100 LOC |
| 4. ViewPayload.html | `crates/ship-sui-server/src/view_payload.rs` | ~30 LOC |
| 5. /view-host splice | `crates/ship-sui-server/src/view_host.rs` + assets/view-host.html | ~50 LOC |
| 6. Integration tests | `crates/ship-sui-server/src/view_payload_tests.rs` (new) | ~100 LOC |
| 7. Bench | Re-run js-framework-benchmark `43_first-paint` — should drop noticeably | ~5 min |

Total: ~440 LOC + tests. One rust-lane dispatch worth of work.

## Trade-offs to discuss

1. **What about `<data>` placeholders?** The SSR walker can emit the `{#while-loading}` content as the default state. That gives users a visible "Loading..." spinner during initial paint instead of an empty container. Tightens UX without needing real data. Recommend doing this — already part of the render walker's repertoire.

2. **CSP and inline content.** Inlining SSR HTML into `/view-host/<name>` response means the content lives in the response body, not a CSS-isolated payload. Should still work but worth confirming with the existing CSP middleware.

3. **Cache invalidation.** SSR HTML is regenerated when the .sui source changes. Build-time emit means cache is just "current .compiled.js mtime." Don't over-engineer; this is fine.

4. **Skip-SSR escape hatch.** Some interactive playgrounds may not want SSR (e.g. signin pages with sensitive UI). Add `manifest.jsonc { "ssr": false }` to opt out per-view. Default: on.

5. **Production-only.** Static SSR is the same in dev and prod — but dev's HMR might want to skip it to stay fast. Tie this to the dev/prod switch (separate task). Default: SSR on in both modes; opt-out via env flag for dev.

## What this design does NOT cover

- **Per-request data SSR.** Future arc; static SSR is v1.
- **Auth-aware SSR.** Same arc; static skeleton works for unauth or low-privacy content.
- **Streaming SSR.** Defer.
- **SSR for islands (lazy bundles).** Islands load on demand client-side; not part of first paint. Static SSR handles only the eager `<sui-*>` registry.

## Open questions for sign-off

1. **Option B (splice into /view-host) — yes or defer?** Bigger win, slightly more invasive. My recommendation: do it as part of the same task; ~50 LOC extra.
2. **`manifest.jsonc { "ssr": false }` opt-out — include or defer?** Include if cheap; the implementation is "check the field, skip the read." ~5 LOC.
3. **Cache layer for SSR HTML reads in prod?** Memoize at request handler level. Simple `HashMap<String, String>` keyed by view name; invalidate via dev/prod-switch-aware flag.
