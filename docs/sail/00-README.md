# Sail — the framework spec

A reactive web framework designed for agent-authored apps with a local-daemon runtime. Sails are single-file components (`.sail`) that compile to web components, server-render by default, and patch over SSE in real time.

## Documents in this spec

- **[00-README.md](00-README.md)** — vision, motivations, "aha" moments. Read this first.
- **[01-language-spec.md](01-language-spec.md)** — `.sail` file format, every syntax detail.
- **[02-compiler-architecture.md](02-compiler-architecture.md)** — how the daemon turns `.sail` into shipped JS.
- **[03-runtime-architecture.md](03-runtime-architecture.md)** — the in-browser signal engine, morph, custom-element wrapper.
- **[04-patch-protocol.md](04-patch-protocol.md)** — the SSE wire format. The killer feature.
- **[05-server-data-bridge.md](05-server-data-bridge.md)** — schema sources, Ship adapter, OpenAPI/GraphQL plug-in shapes.
- **[06-component-library.md](06-component-library.md)** — how Sail and ship-set's Lit components coexist.
- **[07-migration-plan.md](07-migration-plan.md)** — phased path from current state to Sail-only.
- **[08-testing-benchmarks.md](08-testing-benchmarks.md)** — Playwright suite, the token-efficiency benchmark.
- **[09-oss-portability.md](09-oss-portability.md)** — layered architecture, what's open source vs Ship-platform.

## Why Sail exists

### The accidental complexity ladder

Each layer in modern web frameworks was solving a problem the previous one created:

1. **Server renders HTML, forms post, light JS.** Worked for 80% of the web. Wikipedia, GitHub-classic, Stripe docs. Still works.
2. **SPAs (~2010-2015).** Real solutions for genuinely app-shaped products. Then declared the default for everything. A blog became a Webpack project.
3. **SSR for SPAs (~2016-2019).** Performance + SEO regressions made SPAs untenable for content. The "fix" was running the same code twice in two completely different environments. Hydration mismatches became a cottage industry.
4. **RSC and the App Router (~2020-2023).** Hydration was expensive, so split components: server-only, client-only, both. Each with rules about what JS APIs are available, what hooks work, what serializes across the boundary.
5. **BFFE (~2023-now).** The server-client boundary became so muddied that you need a third server (in JavaScript, naturally) sitting between your real backend and your frontend.

Each step solved a real problem. Each step's "fix" cost more complexity than reversing course would have. The whole stack is solving the wrong problem: how do we keep React in the picture even though HTML existed before us.

### The Ship insight

When the server is on the same machine as the browser (Ship's local-daemon model), every justification for the SPA-SSR-edge-CDN stack evaporates:

- No cold starts — daemon is always running
- No edge optimization — there is no edge
- No CDN strategy — same-origin, instant
- No "where does this data live" — `~/.ship/platform.db`, two hops away
- No streaming-SSR partial-paint dance — the daemon renders synchronously, the browser receives a complete page
- No "what runs at build vs request time" — every render is a request, every request is a fresh render

The performance budget that justifies "ship 200KB of React" doesn't exist when your origin is 0.1ms away.

### The agent insight

Every framework alive today was designed for human authors writing for human users. Ship's apps are increasingly authored by LLM agents. That changes the cost function: the right framework minimizes **tokens to correctness**, not lines of code.

Concrete implications:

- **One template language, not two.** Today's `{{ x }}` for static + `data-text="$x"` for reactive is the cognitive tax that kills agent throughput. One syntax, compiler classifies.
- **One reactivity model.** Not signals + effects + computed + Datastar attributes. The smallest set that covers cases.
- **Component library knows the schema.** `<ship-button>` props are typed in the LSP, validated by the compiler. The agent writes the tag and gets the rest.
- **Errors speak the framework's vocabulary.** `ship-button: prop 'variant' expects 'primary' | 'ghost', got 'secondary' (modal.sail:14)` — not a Datastar URL fragment.
- **No build step to fight.** Every build error not generated is tokens not spent recovering from it.
- **Patches as the wire model.** Agents emit small patches over SSE; the framework applies them with no full re-render. Token cost is proportional to changes, not state.

## "Aha" moments captured from the design thread

### The static/reactive dichotomy is the source of pain

Two interpolation syntaxes for the same concept (a value's display) split by *who owns the binding* (server vs client). Authors flipping between Jinja `{{ }}` and Datastar `data-text` is the cognitive tax that doesn't pay off. **Sail collapses this:** one syntax `{x}`, compiler classifies static vs reactive at compile time.

### `:host` doesn't work in light DOM

Years of frameworks documenting `<style>:host { ... }</style>` patterns that silently fail in light-DOM components. Sail rewrites `:host` → `[data-component="<tag>"]` at compile time. Authors get what the docs claimed.

### `data-on:click="userFn"` doesn't work in Datastar

Datastar evaluates expressions in its own scope; can't reach IIFE closures. Authors must wire via `addEventListener` in `onConnect`. Sail's compiler emits the wiring directly — `on:click={fn}` works because the compiler has the closure.

### `data-for` doesn't exist in Datastar v1.0.0-RC.7

The SFC docs promised a directive its underlying library didn't have. Both my picker AND ship-tabs (the conversion stress test) rendered zero items in browser. **Sail owns iteration:** `{#for x of list key={x.id}}` is a compile-time directive that emits keyed reconciliation. Same authoring shape, working semantics.

### Web components are the right substrate

Custom elements give universal interop, native browser API, devtools, registration, lifecycle. Sail emits them. Doesn't use Lit (different reactivity model, different rendering choices), but Lit components compose freely because they're also custom elements.

### Patches are the wire model, not full re-renders

The killer use case (agent-authored canvas) demands the wire format be patches, not state snapshots. Token cost is proportional to *changes*, not *state*. The framework supports this from day 1: every reactive value is path-addressable, the patch protocol is a public spec, server-owned signals are first-class.

### Layered architecture enables OSS without losing Ship's edge

Layer 1 (runtime) and Layer 2 (compiler) are framework — open source, work against any backend. Layer 3 (Ship daemon adapter, `<ship-*>` library, design tokens) is the platform. Mirrors Phoenix LiveView/Fly.io, Astro/Vercel, SvelteKit/hosts.

### Performance: 1.10-1.15x of vanilla

Solid/Svelte territory. Better than Lit, much better than React. Won't beat vanilla, but the difference is invisible in real apps where network and rendering dominate. Patch latency end-to-end ~1ms — well under one frame at 60Hz.

## Why Sail wins against alternatives

| Concern | Solid / SolidStart | Svelte / SvelteKit | Next.js | Phoenix LiveView | Dash | Sail |
|---|---|---|---|---|---|---|
| No build step (author-facing) | ❌ | ❌ | ❌ | ✅ | ❌ (Python env) | ✅ |
| Server-truth integration | partial | partial | ❌ (BFFE pattern) | ✅ | ✅ | ✅ |
| Hypermedia by default | ❌ | partial | ❌ | ✅ | ✅ | ✅ |
| Web components emission | ❌ | partial (custom elements compile) | ❌ | ❌ | ❌ | ✅ |
| Fine-grained reactivity | ✅ | ✅ | ❌ (vDOM) | partial | ❌ | ✅ |
| Patch protocol as wire model | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| Typed component library | ✅ (TS) | ✅ (TS) | ✅ (TS) | partial | ❌ | ✅ |
| Token economy for agent authoring | average | average | poor | average | poor | optimized |
| Local-daemon native | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |

The combination is unique. Phoenix LiveView is the closest peer architecturally (server-driven, patch protocol, hypermedia) but isn't optimized for JS-shaped authoring or agent-authored apps. Solid is closest on reactivity but has no server-truth story. Sail combines what's right about both.

## Defensible answer to "why your own framework"

> Sail is a framework designed for agent-authored reactive apps with a local-daemon runtime. The framework is co-designed with the runtime — daemon, compiler, LSP, component library, patch protocol — and that co-design is what makes Sail apps the easiest reactive apps to write. Every other framework lives outside its runtime. Sail lives inside it.

Co-design unlocks features no bolt-on framework can match: typed server queries against actual schemas, schema-driven component-library awareness, permission-aware rendering, atomic optimistic updates, agent-tuned token economy. "No build step" and "vertical control" are tactics; **co-design is the strategy**.

## Glossary

- **Sail** — the framework. Open source, layered (runtime + compiler + adapters).
- **Ship** — the platform. Sail's killer reference implementation; daemon, LSP, component library, agent surfaces.
- **`.sail`** — single-file component format. Three blocks: `<script lang="ts">`, template HTML, `<style>`.
- **ship-set** — Ship's Lit-based component library (`<ship-*>` tags). Composes inside Sail components.
- **Patch protocol** — the SSE wire format that lets the daemon apply targeted updates to client signals without re-render.
- **Schema source** — a plugin to the Sail compiler that resolves `data x = api.x.y(args)` declarations against a typed backend (Ship daemon, OpenAPI, GraphQL, etc.).
- **Layer 1 / 2 / 3** — runtime / compiler / platform-integration boundaries. See [09-oss-portability.md](09-oss-portability.md).

## Status of legacy code

The current codebase has:
- `.html.j2` views in `.ship/views/` — Jinja-rendered server pages.
- `.ship` SFC components in `.ship/components/` — first-cut SFC framework, recently fixed (camelCase signals, `:host` rewrite, `data-for` runtime).
- `<ship-*>` Lit components in `packages/ship-set/` — these stay; Sail composes them.

Working but unfinished. Zero views in this codebase have been signed off as production-complete. **No legacy code blocks this rewrite.** The migration plan ([07-migration-plan.md](07-migration-plan.md)) treats existing components as starter fixtures for the migration tooling, not as constraints.
