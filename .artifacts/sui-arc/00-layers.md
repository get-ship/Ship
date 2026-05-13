# SUI Arc — 00 — Layers

Status: draft for review.
Scope: this file establishes the layer boundaries that every other spec in `.artifacts/sui-arc/` references. Specs only — not user-facing docs.

## What SUI is

SUI ("Ship UI") is the framework Ship uses to render its UI. It is one product, internally organized into four layers so each can be designed, tested, and evolved independently. The layer split is for our work focus — externally it is still one thing.

| # | Layer | Analogy | What it owns |
|---|---|---|---|
| 1 | **SUI-render** | React | The full `.sui` language (including `<data>`), `sui-compiler` (single unified compiler), reactivity primitives, runtime mount/hydrate/lifecycle, DOM bindings, `<prop>` components. Pure browser. No Ship knowledge. |
| 2 | **SUI-stack** | Next.js (the framework half) | The `Resource` runtime primitive that compiled `<data>` lookups bind to, mutation/form bindings, the `TransportClient` *interface* a host must satisfy. Ships as a layered entry point of `sui-runtime`; tree-shaken out if no `<data>` is used. Imports no Ship code; declares the interface, does not implement it. |
| 3 | **Ship-host** | Next.js (the server half) + the app | The axum daemon that serves SUI views, the SDK that implements SUI-stack's `TransportClient`, HMR/SSE/mutation transports, project resolution, view-host HTML, the Ship design system built on SUI, the shell app. |
| 4 | **Ship-toolchain** | Vercel | `ship build`, Bun integration, dist artifact layout, distribution / store. Out of scope for this arc beyond "we will not foreclose a sensible toolchain." |

Layers 1–3 are the scope of this arc. Layer 4 is deferred.

### Where the SUI-render / SUI-stack line actually runs

The split is in **runtime imports**, not in the compiler:

- `sui-compiler` is unified. It parses the full `.sui` grammar including `<data>` and emits one JS module per view. No feature flag, no fragment.
- `@ship/sui-runtime` (default entry) is SUI-render. Provides signals, mount, hydrate, DOM bindings, control flow, `<prop>` components. Knows nothing about `Resource` or transports. A view that doesn't use `<data>` mounts with zero transport configured.
- `@ship/sui-runtime/data` (sub-entry) is SUI-stack. Exports the `Resource` primitive that compiled `<data>` references resolve to, and declares the `TransportClient` interface. Tree-shaken out of any bundle whose views don't use `<data>`.
- A view that uses `<data>` requires the host to install a `TransportClient` before mount. Missing transport produces a clear runtime error, not a compile error. The compiler does not gatekeep this.

Consequence: SUI-render is testable in isolation against the unified compiler's output, without any transport. SUI-stack is testable against an in-memory fake `TransportClient`. Ship-SDK is the production implementation; nothing in SUI-render or SUI-stack imports it.

## Cross-layer contracts

Each arrow is a contract that lives in its own spec under `.artifacts/sui-arc/`. Both sides validate against the same shape.

```
+----------------------------+      compile artifacts       +-------------------+
|  sui-compiler (Rust)       |  ─────────────────────────►  | SUI-render        |
|  .sui → JS/.d.ts/manifest  |                              | (sui-runtime,     |
+----------------------------+                              |  sui-dom)         |
                                                            +---------┬---------+
                                                                      │
                                                                      │ reactive
                                                                      │ resource shape
                                                                      ▼
                                                            +-------------------+
                                                            | SUI-stack         |
                                                            | (<data>,          |
                                                            |  <mutation>)      |
                                                            +---------┬---------+
                                                                      │
                                                                      │ TransportClient
                                                                      │ interface
                                                                      ▼
+----------------------------+      ViewPayload + assets    +-------------------+
|  Ship-host                 |  ─────────────────────────►  |  Ship-SDK         |
|  (axum, HMR, mutation,     |  ◄────  signal wire   ────►  |  (impls           |
|   project resolver)        |                              |   TransportClient)|
+----------------------------+                              +-------------------+
```

Contracts to be specified in subsequent files:

- `02-sui-lang.md` — `.sui` grammar + semantics (render core + stack additions clearly demarcated).
- `03-compiler-output.md` — what `sui-compiler` emits per `.sui` source. The boundary between compiler and runtime.
- `04-reactivity.md` — algorithm spec shared by `crates/sui-signals` (Rust) and `packages/sui-runtime/src/signal.ts` (TS). Property-test corpus.
- `05-runtime-lifecycle.md` — mount, hydrate, scope tree, cleanup ordering, controller resolution.
- `06-dom-bindings.md` — how `sui-dom` binds reactive values to DOM nodes.
- `07-view-payload.md` — the wire shape served by Ship-host and consumed by SUI-render.
- `08-data-resource.md` — the `<data>` reactive resource shape and the `TransportClient` interface SUI-stack declares.

## What goes where (by directory)

The naming pass that follows from this layer cut is specified in `01-naming.md`. In summary, prefix convention:

- `sui-*` — layer 1 or layer 2 (framework internals).
- `ship-sui-*` — layer 3 components that *integrate* SUI (host wiring).
- `ship-*` — other layer 3 product modules (SDK, design system, shell).
- `ship-set*` — legacy `.html.j2` + Lit, frozen, deleted at SUI parity. Do not rename.

## Rules the layers must obey

1. **No upward imports.** Layer 1 must not import from layers 2, 3, 4. Layer 2 must not import from layer 3. Layer 3 satisfies layer 2's interfaces; it does not modify them.
2. **Contracts are versioned.** Each cross-layer contract has a schema in `.artifacts/sui-arc/` and a version. Breaking changes require migration notes in the same file.
3. **One way to do things per layer.** If a capability exists, it lives in exactly one layer. Parallel implementations across layers are a bug.
4. **The compiler is the single source of derived facts.** Binding maps, islands lists, `<prop>` shapes, `<data>` source declarations, signal-provenance metadata — all of these are emitted by the compiler at build time. Host and runtime consume; they do not recompute.
5. **The host satisfies SUI's interfaces; it does not define them.** `TransportClient` is declared in SUI-stack. `ship-sdk-client` implements it. Tests for SUI-stack run against an in-memory fake implementation, not Ship-SDK.

## What this changes about how we work

- Robustness pass for "the client/js/rendering bits" = layers 1 and 2, module by module, against the contracts in this arc.
- Anything tagged as a layer-3 or layer-4 concern in conversation gets parked until layers 1 and 2 are reliable.
- Renaming (`01-naming.md`) is the first mechanical change. Everything else waits on the directory and import paths being stable.
