# Sail spec — index + thread compression

Authoritative entry point. Read in order.

1. **[00-README.md](00-README.md)** — vision, motivations, "aha" moments, defensible answers.
2. **[01-language-spec.md](01-language-spec.md)** — `.set` file format, full syntax.
3. **[02-compiler-architecture.md](02-compiler-architecture.md)** — Rust compiler design.
4. **[03-runtime-architecture.md](03-runtime-architecture.md)** — JS runtime design.
5. **[04-patch-protocol.md](04-patch-protocol.md)** — SSE wire format. The killer feature.
6. **[05-server-data-bridge.md](05-server-data-bridge.md)** — schema sources, Ship adapter, OpenAPI/GraphQL.
7. **[06-component-library.md](06-component-library.md)** — Sail + ship-set integration.
8. **[07-migration-plan.md](07-migration-plan.md)** — phased path from current state to Sail-only.
9. **[08-testing-benchmarks.md](08-testing-benchmarks.md)** — Playwright + token-efficiency benchmark.
10. **[09-oss-portability.md](09-oss-portability.md)** — layered architecture, OSS strategy.
11. **[10-dev-tooling.md](10-dev-tooling.md)** — LSP, CLI, hot reload, devtools, agent-specific tools.

## Thread compression — how the design got here

This spec emerged from a multi-stage conversation. The compressed history, for the agent picking this up:

### Where the thread started

Implementing chat modal + pipeline view as `.ship` SFC components against the existing first-cut framework (compiler in `apps/shipd/src/views/sfc/`, Datastar v1.0.0-RC.7 as the runtime). Tier 1 + 2 fixes were planned to shore up the framework — type story polish, auto-loader, custom directives.

### The discovery

Browser-testing the first SFC components via Playwright surfaced **three structural framework bugs**:

1. **camelCase signal names lost their values.** `__sigKey` returned `_<tag>_<inst>_<name>` preserving case, but data-* attribute names are case-folded by HTML. Lookups silent-missed.
2. **`:host { display }` did nothing.** SFC components are light DOM; `:host` is a shadow-DOM-only pseudo-class. Every previously shipped SFC silently rendered display:inline + zero height.
3. **`<template data-for>` never iterated.** Datastar v1.0.0-RC.7 doesn't have a for/each plugin. The docs promised it. Both the picker and ship-tabs (the conversion stress test) silently rendered zero items.

Plus two documentation lies:
- `data-bind:value="$x"` syntax is rejected by Datastar (use `data-bind`)
- `data-on:click="userFn"` doesn't work — Datastar evaluates expressions in its own scope, can't reach IIFE closures

All three structural bugs were fixed in-place ([commit `34d8e11d`](../../../docs/plans/sdk-sfc-roadmap.md)). But fixing them required taking ownership of substantial parts of Datastar's contract — at which point the question became: **why are we building on Datastar at all?**

### The user's pushback (and the right answer)

Multiple key points sharpened the design:

- **"Datastar is 10kb of opensource code. it is not going to take months to replace."** Correct — the morph engine (idiomorph, ~500 LOC) and signal engine (Solid-style, ~150 LOC) are well-understood. Three weeks total for a first-cut runtime.
- **"I don't want to fork Datastar if we are keeping the data-* api."** Right — keeping Datastar's authoring API while owning the runtime is silly. The interesting move is to *not* keep that API; own the authoring surface end-to-end.
- **"Token usage is the north star."** The framework should optimize for agent-authored apps. Current: two interpolation syntaxes (Jinja `{{ }}` + Datastar `data-text`), no type checking at component call sites, errors that speak Datastar's vocabulary not Ship's. Each cost is invisible to humans, expensive for agents.
- **"SSR with React is the most ridiculously overengineered solution going."** True — SPAs were a wrong turn for 80% of the web; SSR-on-SPAs is a wrong-turn correction worse than reversing course. RSC, Next.js App Router, BFFE — each layer solves problems the previous one created.
- **"Local-daemon as a constraint."** Eliminates 80% of the SPA-SSR-edge stack's justifications. Same-origin instant, no cold start, no edge optimization. Ship's runtime can be hypermedia-first because that's what HTML always wanted, plus modern primitives (web components, signals, TypeScript, SSE) for the parts that benefit.
- **"Killer feature: agents manipulating DOM in real-time via SSE updates for canvas."** The patch protocol becomes the wire model from day 1. Agents author canvases at token costs proportional to *changes*, not *state*. This is the design pivot that makes Sail uniquely valuable.
- **"Fine-grained reactivity is paramount."** Datastar IS fine-grained for its primitives; my "data-for re-iterates whole array" was an implementation choice in the runtime I added, not a Datastar limit. Sail's design is SolidJS-grade fine-grained throughout, including keyed list reconciliation.
- **"Do we use web components / Lit?"** Yes for web components (browser's native composition primitive, universal interop). No for Lit (different reactivity model, default shadow DOM conflicts with our light-DOM choice, would couple us to Lit's release cycle). ship-set's Lit components keep working — they're custom elements, Sail composes them via the customElements registry.
- **"Easiest way to build reactive apps period — not just Ship apps."** Framework is the strategic asset; OSS is the path to that. Layered architecture (runtime + compiler open, platform integration proprietary) preserves both Ship's commercial position and Sail's potential as a real framework.
- **"Sunset .html.j2."** Yes — components are composition roots, routes are sail files, one mental model. Jinja stays for non-component templating only.
- **"Comparison to Dash by Plotly."** Dash demonstrates the model (server-truth + components + hypermedia) at scale, but Python-bound and analyst-audienced. Ship's structural choices (HTML+JS authoring, web components, daemon-as-runtime, agent-tuned) are better positioned for the agent-authored future.
- **"Why your own reactivity?"** Strongest defensible answer: **co-design with the runtime**. The framework knows the daemon, the daemon knows the framework. Other frameworks live outside their runtimes; Sail lives inside it. That co-design unlocks features (typed server queries, schema-driven components, permission-aware rendering, atomic optimistic updates, agent-tuned token economy) impossible to bolt on. "No build step" and "vertical control" are tactics; **co-design is the strategy**.
- **"`.set` is a better file extension than `.ship`."** Right — `.ship` overloads the platform name. `.set` pairs naturally (vessel + rigging metaphor), is OSS-brandable as a framework name, no SEO/trademark collision.

### Where the design landed

A reactive web framework for agent-authored apps with a local-daemon runtime, designed from first principles given those constraints:

- **Single file format** (`.set`): script + template + style, three blocks
- **Single template language**: `{x}` interpolation, compiler classifies static vs reactive
- **Seven keyword forms**: `prop`, `state`, `derived`, `data`, `effect`, `event`, `function`
- **TypeScript via type stripping**: no author-visible build step, full type-check in LSP
- **Web components emission**: universal interop, native browser API
- **Solid-grade fine-grained reactivity**: signal/derived/effect/batch/untrack
- **Patch protocol as wire model**: signal-targeted updates over SSE, token cost proportional to changes
- **Server-data declarations**: `data x = ship.x.y(args)` typed, SSR'd, optionally `live`
- **Schema sources pluggable**: Ship daemon, OpenAPI, GraphQL, static
- **Component library aware**: ship-set's Lit components compose; their schemas validate Sail call sites
- **Layered for OSS**: runtime + compiler MIT-licensed; platform integration is Ship's value add

The whole stack collapses dichotomies: static/reactive interpolation, Jinja/Datastar templating, server/client state, prop types/runtime decoders, route/component file formats. **One mental model.** That's the agent-authoring win.

### What we won't do

- Roll our own reactivity primitive that's not Solid-shaped (no need; the model is well-understood)
- Build a virtual DOM (fine-grained signals don't need one; would be cost without benefit)
- Mandate types (opt-in via TypeScript; bare JS still works)
- Ship a build step authors run (the daemon is the build)
- Migrate ship-set to Sail wholesale (piece by piece, when migration earns its keep)
- Couple Layer 1 / Layer 2 to Ship internals (CI-enforced boundary)

### The falsification gate

Before announcing Sail, run the token-efficiency benchmark ([08-testing-benchmarks.md](08-testing-benchmarks.md)). Sail wins decisively if:
- ≥30% fewer tokens to first-passing component vs the next-best framework
- ≥80% first-try success rate
- ≥40% fewer tokens for read+edit tasks
- Lint fix correctness ≥95%

If the numbers aren't there, **the rewrite is unjustified**. We tighten the spec or revert to fixing what we have. The benchmark is the marketing AND the falsifier.

### Implementation order

Per [07-migration-plan.md](07-migration-plan.md):

- **Week 1-2:** Sail compiler + runtime MVP. First end-to-end Sail component browser-verified.
- **Week 3:** Migrate the 8 existing `app-*` components to real Sail (auto + hand finishing).
- **Week 4-5:** Migrate `.html.j2` views to `.set` route files.
- **Week 6:** Retire the legacy SFC compiler. Datastar dropped from production.
- **Week 7+:** Run the benchmark. Iterate. Announce.

~6-7 weeks total. Worst-case 3-4 weeks of parallel impl. Bounded.

### Key decisions documented in this spec

| Decision | Spec doc |
|---|---|
| Why we're building (vision, motivations) | [00-README.md](00-README.md) |
| `.set` vs `.ship` extension | [00-README.md § Naming](00-README.md) + [01-language-spec.md § Filename rules](01-language-spec.md) |
| Single template language | [01-language-spec.md § The template block](01-language-spec.md) |
| Seven keyword forms | [01-language-spec.md § Top-level declarations](01-language-spec.md) |
| TypeScript via type stripping | [01-language-spec.md](01-language-spec.md) + [02-compiler-architecture.md § Phase 3](02-compiler-architecture.md) |
| Web components emission, no Lit | [06-component-library.md](06-component-library.md) |
| Solid-grade fine-grained reactivity | [03-runtime-architecture.md § Signal engine](03-runtime-architecture.md) |
| Patch protocol as primary wire model | [04-patch-protocol.md](04-patch-protocol.md) |
| `data` declarations + SchemaSource trait | [05-server-data-bridge.md](05-server-data-bridge.md) |
| Layered OSS architecture | [09-oss-portability.md](09-oss-portability.md) |
| Migration plan for current code | [07-migration-plan.md](07-migration-plan.md) |
| Token-efficiency benchmark as falsifier | [08-testing-benchmarks.md § Layer 4](08-testing-benchmarks.md) |

### What's not yet decided

- **Specific runtime API names.** I sketched `bindText`, `bindShow`, etc.; these may rename during implementation.
- **CSS-in-JS vs CSS-in-style-block.** Spec says style block; alternative (CSS-in-script via tagged-template) is reserved for v2.
- **Mutation API.** `data` declarations are read-only in v1; mutation goes via `fetch()` or patch-emit endpoint. Typed mutation API is v2.
- **Multi-writer signal conflicts.** Sail assumes one writer (server) per signal in v1. Multi-writer / OT is v2.
- **HMR for script blocks.** Templates and styles hot-reload; scripts require browser refresh in v1. Script HMR via SSE patch swap is a stretch goal.
- **Specific OSS repo structure.** Monorepo until Sail matures; extraction later.

### What the next agent needs to do

If picking up this spec to implement:

1. Read [00-README.md](00-README.md) and [01-language-spec.md](01-language-spec.md) for the what + why
2. Read [02-compiler-architecture.md](02-compiler-architecture.md) and [03-runtime-architecture.md](03-runtime-architecture.md) for the how
3. Read [07-migration-plan.md](07-migration-plan.md) for the order of operations
4. Start with the runtime — `packages/@sail/runtime/` per [03's implementation phasing](03-runtime-architecture.md#implementation-phasing). The signal engine is the heart; everything else is bookkeeping around it.
5. In parallel, start the compiler skeleton — `crates/sail-compiler/` per [02's implementation phasing](02-compiler-architecture.md#implementation-phasing). Phase 1 (parser) + Phase 6 (style scoping) are the easiest entry points; the parser can be evolved from the existing `apps/shipd/src/views/sfc/parser.rs`.
6. Wire compiler + runtime end-to-end with a single `app-counter.set` test case. Browser-verify.
7. Build the migration tool against the existing `app-*` components. Run + verify.
8. Migrate one view as a smoke test. Iterate.
9. Run the token benchmark ([08-testing-benchmarks.md](08-testing-benchmarks.md)) before announcing.

The ten spec docs cover everything. Each is self-contained and links to the others where relevant.

## Status

These specs are the design. **No implementation has started against them.** The legacy SFC framework (`apps/shipd/src/views/sfc/`) is the existing component path; its three structural bugs are fixed but architecturally it's a dead end. Sail replaces it.

When implementation begins, this directory becomes the canonical reference. Each spec doc is updated as decisions firm up; significant changes get a CHANGELOG-style note at the top of the relevant doc.
