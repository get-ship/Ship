---
group: Ship UI
order: 6
title: SSR and the attach runtime
description: How `shipd` renders `.sui` to HTML server-side, and how the client runtime attaches reactivity to the rendered DOM. Progressive enhancement first.
---

# SSR and the attach runtime

SUI is **server-rendered first**. `shipd` renders the template to HTML with initial values inlined; the page is meaningful before any JavaScript runs. A small client runtime then attaches event handlers, signals, and SSE subscriptions to the existing DOM.

This is progressive enhancement, not hydration of an empty shell. The framework's design assumes the daemon owns the data and the page should reflect it immediately on first paint.

## The pipeline

```
.sui source
    │
    ▼
SUI-compiler  (Rust, AOT)
    │
    ▼
IR (lowered, span-tracked)         ─── consumed by both sides ───
    │                                                            │
    ├──────────────────────────┐                                 │
    ▼                          ▼                                 ▼
Rust render walker   ESM attach instructions   (eval rig, LSP, tooling)
in shipd                    in browser
    │                          │
    ▼                          ▼
Server HTML  ────►  Browser parses  ────►  Attach runtime wires reactivity
                                            + opens SSE for `data live`
```

The IR is the single source of truth. Both the server render walker (Rust, in `shipd`) and the client attach runtime read the same IR. There is no second grammar, no client parser on the production path.

## Server render

`shipd` evaluates the template against initial signal values:

| Template construct | Server behavior |
|---|---|
| Static HTML | Emitted verbatim. |
| `{expr}` (renderable subset) | Evaluated against initial state; result inlined. |
| `{expr}` (non-renderable) | Marker emitted; client evaluates on attach. |
| `{#if cond}` | Server evaluates `cond`; only the matching branch is emitted, with markers identifying which branch is live. Client swaps on signal change. |
| `{#for x of xs}` | Server expands `xs` against initial value; emits all items with `key` markers. Client adds/removes by key. |
| `{#match expr}` | Server evaluates `expr`; only the matching case is emitted. |
| `{#await promise}` | If the promise is sync-resolvable from a `<data>` source, server emits the resolved branch. Otherwise emits the loading branch and the client takes over. |
| `<data live />` | Server emits the initial value inline (with envelope `kind: 'ok'`); client opens the SSE channel for patches. |
| `<while-loading bind="X">` | Server already has the value — emits children if `X` is `ok`, suppresses otherwise. |
| `on:click={fn}` | Marker emitted; no listener attached server-side. |
| `bind:value={sig}` | Marker emitted; client wires both directions. |
| `<use-* />` | Marker emitted; client invokes the hook factory at mount. |
| Custom elements (`<ship-button>`) | Emitted as their tag. Browser self-upgrades on parse via standard custom-element registration. |

## The renderable expression subset

Expressions inside `{ ... }` are SSR'd if they stay inside this subset:

- Signal calls — `count()`, `agents().value.items`
- Property access — `user().name`, `a.id`
- Arithmetic and comparison — `count() + 1`, `age() >= 18`, `n() % 2 === 0`
- Boolean operators — `loading() || error()`, `enabled() && active()`
- Ternaries — `cond ? a : b`
- Template literals with no embedded function calls — `` `${name()} (${total()})` ``

Expressions **outside** the subset (function invocations beyond signal reads, closures, `new`, `await`, `this`, arbitrary JS) become client-only. The compiler emits a placeholder marker; the client runtime evaluates on attach. The compiler also emits a warning so the author can decide whether to refactor toward the subset.

## Markers (implementation in active dev)

The serialization shape between server-emitted HTML and the client attach runtime is in active development. The contract is:

- Each reactive binding (text interp, attribute interp, event handler, two-way bind, control flow boundary, custom element instance) is emitted with a marker the client can find unambiguously.
- The marker carries enough information to resolve the binding from the IR — typically a stable ID into a sidecar manifest, or an inline annotation that names the binding directly.
- Markers do not pollute author-visible HTML semantics (no role-changing attributes, no `id` collisions).

We are not committing to Datastar's `data-*` attribute names. The shape is ours; the philosophy of "small client runtime + server-rendered HTML + SSE patches" is shared. See [status.md](status.md) for the open question.

## Client attach runtime

The browser-side runtime is **attach-only**. It does not parse templates, does not construct DOM from scratch, does not run the IR's render path. Its job:

1. Walk the rendered DOM looking for markers.
2. For each marker, read the binding from the manifest (or inline annotation).
3. Initialize signals from inlined initial values (`<state>` markers carry the initial state JSON; `<data>` markers carry `kind: 'ok'` envelopes with values).
4. Attach event listeners (`on:click`, `bind:`).
5. Wire reactive bindings (text interp, attribute interp, control-flow boundaries) to track signals and update the DOM on change.
6. Open SSE channels for each `<data live>` and apply patches as they arrive.
7. Invoke template hook factories (`<use-*>`) and register their cleanups.
8. Run controller `mount` after all of the above.

State changes from local signals (button clicks, bind:value) flow through handlers, mutate state, and trigger surgical DOM updates against the existing nodes. There is no morph step — the runtime knows precisely which nodes correspond to which bindings.

`data live` SSE patches arrive as JSON; the runtime applies them to the live signal and downstream bindings update.

## Progressive enhancement

A view rendered without JavaScript is still meaningful:

- Static structure renders.
- Initial values from `<state>` and `<data>` are inlined.
- `{#if}` / `{#for}` / `{#match}` show their initial branches.
- Custom elements show their server-rendered children (the framework does not require client-side construction for non-island components).

What's lost without JS:

- Event handlers (`on:click` does nothing; submit-style handling depends on form fallback)
- `bind:value` updates (form fallback works; reactive updates do not)
- `data live` patches (initial values are visible; updates are not)
- Template hooks (`<use-kbd>`, `<use-title>` etc. are no-ops)

For the local-daemon case Ship targets, this matters less than for the public-web case — JS is essentially always available. But the architecture *enables* PE, and that's the discipline: the server renders meaningful HTML, the client enhances it.

## The deleted middle

Two pieces of the previous (Phase 3) architecture are being deleted:

- **`@SUI/dom`** — the client-side template parser/interpreter. Once SSR ships, the client never parses `.sui` source. The package survives only to support the eval rig (jsdom probes) and tests.
- **The empty HTML shell** — `shipd` no longer serves an empty `<div id="root">` shell at `/sui/<view>`. It serves rendered HTML directly.

Both deletions follow from the IR-as-source-of-truth move. Two parsers (Rust + JS) and two emit shapes (empty shell + client-rendered DOM) become one parser (Rust) and one emit shape (rendered HTML).

## Import surface: default vs. legacy

`@ship/ui-dom` exposes two import paths with distinct responsibilities:

| Path | Exports | Who uses it |
|---|---|---|
| `from '@ship/ui-dom'` | `attach()`, `AttachOptions`, `loadSetup` | Production views, daemon asset bundles |
| `from '@ship/ui-dom/legacy'` | `hydrate()`, `parseSetSource()`, `HydrateOptions` | Tests, eval harness |

The parser (`hydrate`, `parseSetSource`, `directives/`) lives exclusively in the `legacy` subpath. The default `@ship/ui-dom` bundle (`sui-dom.js`) contains zero parser code; its gzip budget is ≤ 9 KB and is enforced by the build-bundles.mjs CI gate.

**Never import `@ship/ui-dom/legacy` from a production view.** Doing so would pull the full parser (~5 KB gzip) into the view bundle and violate the size budget. The build-bundles.mjs audit asserts zero references to `parseSetSource` in `sui-dom.js`.
