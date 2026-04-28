# OSS architecture and portability

How Sail is designed as an open-source framework that runs anywhere, with Ship as the killer reference platform.

## The layered architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 3 — Platform Integration                              │
│ ─────────────────────────────────────                       │
│ • Ship daemon adapter (or Rails / Django / Express / ...)   │
│ • Component-library bundle (ship-set / @my-org/components)  │
│ • Design tokens (tokens.css)                                │
│ • SSE patch broadcaster                                     │
│ • Schema source                                             │
│ ─ proprietary or per-platform ─                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ stable, public API
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2 — Compiler + LSP                                    │
│ ─────────────────────────────────────                       │
│ • crates/sail-compiler — Rust, pluggable                    │
│ • SchemaSource trait (plug your backend)                    │
│ • ComponentSource trait (plug your component library)       │
│ • LSP server                                                │
│ • Migration tools                                           │
│ ─ MIT, npm/cargo published ─                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ stable, public API
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 1 — Runtime                                           │
│ ─────────────────────────────────────                       │
│ • @sail/runtime — TypeScript, ~12KB                         │
│ • signal/derived/effect/batch                               │
│ • bind/on/for/slot                                          │
│ • DOM morph (vendored idiomorph)                            │
│ • Patch protocol applicator                                 │
│ • SSE subscriber                                            │
│ ─ MIT, npm published ─                                      │
└─────────────────────────────────────────────────────────────┘
```

Each layer has a stable, documented contract with the layer above. Layer boundaries are enforced by code structure (separate packages) and CI (Layer 1 cannot import Layer 3 types; if it tries, the build fails).

## What's open source

| Component | License | Repo | Distribution |
|---|---|---|---|
| `@sail/runtime` | MIT | `github.com/sail-framework/runtime` | npm |
| `sail-compiler` (Rust crate) | MIT | `github.com/sail-framework/compiler` | crates.io |
| `@sail/lsp` | MIT | `github.com/sail-framework/lsp` | npm |
| `@sail/cli` | MIT | `github.com/sail-framework/cli` | npm |
| `@sail/openapi-source` | MIT | `github.com/sail-framework/openapi-source` | npm |
| `@sail/static-source` | MIT | `github.com/sail-framework/static-source` | npm |
| `@sail/migrate` | MIT | `github.com/sail-framework/migrate` | npm |
| `agent-bench` corpus + harness | MIT | `github.com/sail-framework/agent-bench` | repo |
| `docs.set-framework.dev` | MIT (content) | site repo | published site |

## What's Ship-proprietary

| Component | License | Repo | Distribution |
|---|---|---|---|
| Ship daemon (`shipd`) | proprietary or source-available | this repo | Ship platform |
| `crates/sail-ship-source` | proprietary | this repo | bundled with shipd |
| ship-set (`<ship-*>` components) | source-available or MIT | this repo | npm `@ship/set` |
| Design tokens (`tokens.css`) | open (CSS is public) | this repo | npm or static asset |
| Ship's LSP integration | proprietary | this repo | bundled with shipd |
| Ship CLI (`ship` command) | proprietary | this repo | platform binary |

The pattern: framework is open, platform is proprietary. Same as Phoenix LiveView (open) + Fly.io (commercial), Astro (open) + Vercel/Cloudflare (commercial), SvelteKit (open) + various deployment hosts.

## Boundary contracts

What Layer 1 exposes (consumed by Layer 2 and applications):

- Public types: `Signal`, `Derived`, `ComponentFactory`, `Patch`, ...
- Public functions: `signal`, `derived`, `effect`, `defineComponent`, `bind*`, `on`, `data`, `applyPatch`, ...
- Custom-element registration API: standard `customElements.define`

Layer 1 must not depend on Layer 2 or Layer 3.

What Layer 2 exposes (consumed by applications and platform integrations):

- `SchemaSource` trait
- `ComponentSource` trait
- `compile_file(path, opts) -> CompileResult` API
- LSP protocol (LSP standard)
- CLI commands

Layer 2 depends on Layer 1's public types only.

What Layer 3 implements (per platform):

- A `SchemaSource` for the platform's backend
- A `ComponentSource` for the platform's component library
- An SSE patch broadcaster wired to the backend's event log
- Token sources, view-serving routes, dev experience

## Plugin examples

### Ship's adapter

```rust
// crates/ship-set-ship-source/src/lib.rs
use sail_compiler::{SchemaSource, ComponentSchema, DataResolution};
use runtime::services::sdk_manifest::SdkManifest;

pub struct ShipSchemaSource {
    manifest: SdkManifest,                 // existing typed SDK
    component_registry: ComponentRegistry, // ship-set + project .set
    tokens: TokenSet,
}

impl SchemaSource for ShipSchemaSource {
    fn resolve_data(&self, expr: &DataExpr) -> Result<DataResolution> {
        // ship.<ns>.<method>(args) — resolve against the daemon's manifest
    }
    fn ssr_fetch(&self, res: &DataResolution, ctx: &SsrContext) -> Result<Value> {
        // Invoke the daemon's actual service for SSR
    }
    fn live_channel(&self, res: &DataResolution) -> Option<String> {
        // Build the SSE URL: /api/v1/signals/<channel>/subscribe?...
    }
    // ...
}
```

### OpenAPI adapter (OSS)

```ts
// packages/@sail/openapi-source/src/index.ts
import type { SchemaSource, DataExpr, DataResolution } from '@sail/compiler-types'
import { OpenAPIDoc } from './openapi'

export class OpenAPISchemaSource implements SchemaSource {
  constructor(private opts: { spec: string, baseUrl: string, namespace?: string }) {}

  resolveData(expr: DataExpr): DataResolution { /* ... */ }
  ssrFetch(res: DataResolution, ctx: SsrContext): Promise<unknown> {
    // HTTP fetch against the OpenAPI endpoint
  }
  liveChannel(res: DataResolution): string | null {
    // OpenAPI doesn't natively define SSE; return null unless extended
  }
}
```

Authors install + configure:

```ts
// sail.config.ts
import { OpenAPISchemaSource } from '@sail/openapi-source'

export default {
  schemaSources: [
    new OpenAPISchemaSource({
      spec: './api/openapi.yaml',
      baseUrl: '/api',
      namespace: 'api',
    }),
  ],
}
```

Now `data users = api.users.list()` resolves against the OpenAPI spec.

## Hosting Sail apps

### Ship daemon (canonical, integrated)

`ship daemon start` serves Sail apps natively. Components compile on-demand, routes resolve from `.ship/views/`, SSE patches flow through the kernel router. The OOTB experience.

### Static export (zero backend)

```bash
npx @sail/cli build --target=static
```

Compiles all `.set` files to static HTML + JS. SSR'd at build time. No live updates (no SSE backend). Suitable for content sites, docs, marketing pages. Output is regular HTML files deployable on any static host (Vercel, Cloudflare, Netlify, S3+CloudFront).

### Node server

```bash
npx @sail/cli build --target=node
node dist/server.js
```

Compiles to a Node-side SSR server. Routes match `.set` files. SSE patches require an event source the developer supplies. Used for non-Ship deployments.

### Cloudflare Workers / Edge

```bash
npx @sail/cli build --target=cloudflare
wrangler publish
```

Compiles to a Cloudflare Worker. SSR runs at the edge. Components are static modules. SSE works via Worker SSE (with constraints).

### Self-hosted with any backend

`@sail/cli` provides primitives. Hook into any HTTP server.

## Why this layered design matters

### For Sail's adoption

Open framework + open compiler + open runtime → low barrier. Anyone can `npm create sail-app`, get a working "hello world" in 60 seconds, no Ship daemon required. The framework's pitch is judged on its merits.

If Sail were Ship-coupled, every potential user would have to evaluate "do I want to install Ship?" before "do I want to try this framework?" That's two decisions when one would do.

### For Ship's commercial position

The framework being OSS doesn't dilute Ship; it sells Ship. Authors who try Sail standalone, hit the limits of static-only or basic OpenAPI, and want the killer features (typed kernel queries, agent-aware components, patch protocol over local SSE) upgrade to Ship.

The pattern that keeps working: free framework, paid platform.

### For the spec's quality

Public APIs are forced to be coherent. Internal APIs always rot. Layer boundaries that are validated by external implementers stay clean.

If we build daemon-coupled, in 6 months we'd have hundreds of lines of "this thing accidentally depends on Ship's runtime types" that we'd discover when we tried to extract. Designing the boundary now costs days; ignoring it costs quarters.

### For Ship as a brand

Sail being an OSS project that Ship anchors makes Ship more credible, not less. Ship demonstrates that we can build an OSS framework that other teams want to use. That competence is brand equity.

## Repository layout

If we go layered:

```
ship/                                 ← Ship monorepo (this repo)
├── apps/shipd/                         ← daemon, depends on sail-compiler crate
├── crates/ship-set-compiler/               ← Layer 2 — Rust, OSS
├── crates/ship-set-ship-source/            ← Layer 3 — Ship's SchemaSource
├── packages/@ship-set/runtime/             ← Layer 1 — TypeScript, OSS
├── packages/@sail/lsp/                 ← LSP, OSS
├── packages/@sail/cli/                 ← CLI, OSS
├── packages/ship-set/                  ← Layer 3 — components, source-available
├── tests/agent-bench/                  ← benchmark, OSS
└── tools/migrate/                      ← migration tools, OSS
```

The monorepo hosts all of it. Public packages get tagged-release pushes to npm/crates.io; internal packages stay versioned in-repo. Single-version-of-truth, easy cross-package refactoring, OSS releases automated.

When Sail matures, we extract Layers 1 and 2 into their own repo (`sail-framework/` org on GitHub). Until then, monorepo.

## Versioning

Each public package has its own semver. Breaking changes in `@sail/runtime` v2 don't force-upgrade everyone; the compiler can target multiple runtime versions during transition periods.

Stable signals (v1.0):
- Spec syntax ([01-language-spec.md](01-language-spec.md))
- SchemaSource trait
- ComponentSource trait
- Public runtime API
- Patch protocol

These get deprecation cycles, not flag days, after v1.

## Documentation

`docs.set-framework.dev` (or `sail.dev`):
- Tutorial: build a counter, then a typeahead, then a chat app
- Guide: each spec section explained with examples
- API reference: every public function + trait
- Patch protocol reference: full op set + examples
- Migration guide (Solid, Svelte, React → Sail)
- Adapter authoring guide: how to write a SchemaSource for your backend

Ship's docs link to Sail's docs for framework concerns; cover Ship-specific integration separately.

## Community + contribution

Standard OSS playbook:
- GitHub issue templates
- Contributing guide
- Code of conduct
- Release notes per version
- Discord or similar for real-time
- Bi-monthly office hours from Ship's team

Contributors who add features, adapters, or examples get credited and (where their work fits Ship's roadmap) are invited to maintain those areas.

## What if we don't go layered

If we build daemon-coupled instead:

- Faster initial implementation by ~1 week
- No public API discipline
- Ship-only audience for the framework
- "Why is this in Ship's repo and not extracted?" question every time someone evaluates
- Future extraction is a quarter-of-cleanup tax
- No OSS contribution path, no public benchmark, no community to draw from

That's a worse outcome for the same engineering work. The layered architecture costs 2-3 days of upfront design and saves quarters of "we should extract this someday" debt.

## Decision

**Build layered from day 1.**

- Layer 1 + Layer 2 are OSS, in their own packages within the monorepo (extractable later).
- Layer 3 is Ship's value add — the daemon adapter, ship-set, the design system, the agent integration.
- The benchmark, the docs, the migration tools — all OSS alongside the framework.

This is the design that gives Sail a long life as a framework AND keeps Ship's competitive position. It costs the smallest amount of design discipline now to preserve all future options.

The framework's job is to make Ship apps the easiest reactive apps in the world. The OSS layering's job is to make sure Sail outlives any one company's roadmap. Both happen if we draw the boundary now.
