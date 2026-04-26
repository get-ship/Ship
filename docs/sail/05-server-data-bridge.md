# Server data bridge

How Sail components reach typed data from a backend. Layer 3 of the stack — the integration that makes Sail apps feel co-designed with their server.

For Ship, this is where the daemon's schemas become typed compile-time references. For OSS users, this is the plug-in point where any backend becomes a Sail data source.

## The `data` declaration revisited

```ts
// In a .sail script block:
data agent = ship.agents.detail(id=props.agentId)
data live mesh = ship.mesh.snapshot()
data jobs: JobRecord[] = ship.jobs.list({ limit: 50 })
```

What the compiler does:

1. **Resolve the expression** (`ship.agents.detail(...)`) against the configured *schema source*.
2. **Validate args + return type** at compile time.
3. **Emit the SSR initial value** — at server-render, the daemon calls the resolved query and bakes the result into the page HTML.
4. **Emit the client-side hydration** — runtime reads the SSR'd value into the signal.
5. **For `live`** — additionally emit the SSE subscription URL so the runtime opens an EventSource.

## SchemaSource trait

```rust
pub trait SchemaSource: Send + Sync {
    /// Resolve a `data <name> = <expr>` declaration.
    fn resolve_data(&self, expr: &DataExpr) -> Result<DataResolution>;

    /// Resolve a component tag for prop/event validation.
    fn resolve_component(&self, tag: &str) -> Option<ComponentSchema>;

    /// Look up a named token in the design system.
    fn resolve_token(&self, name: &str) -> Option<TokenValue>;

    /// SSR call: fetch initial value for a resolved data expression.
    fn ssr_fetch(&self, resolution: &DataResolution, ctx: &SsrContext) -> Result<Value>;

    /// Build the SSE subscription URL for a `data live` declaration.
    /// Returns None if the source doesn't support live updates.
    fn live_channel(&self, resolution: &DataResolution) -> Option<String>;
}
```

The trait is the boundary. Anything that implements it can be a Sail data source.

## Ship's adapter

```rust
// In Ship's daemon — Layer 3
pub struct ShipSchemaSource {
    runtime: Arc<ShipRuntime>,
    component_registry: Arc<ComponentRegistry>,
    token_set: Arc<TokenSet>,
}

impl SchemaSource for ShipSchemaSource {
    fn resolve_data(&self, expr: &DataExpr) -> Result<DataResolution> {
        // Parse `ship.<namespace>.<method>(args...)` against the daemon's
        // SDK manifest. Validate arg names + types. Return the typed
        // result schema.
        //
        // ship.agents.list({ workspace: string })
        //   -> returns Agent[]
        //   -> SSR call: invoke runtime::services::agents::list(...)
        //   -> live channel: "agents.list?workspace=..."
    }

    fn resolve_component(&self, tag: &str) -> Option<ComponentSchema> {
        // 1. Look up <ship-*> Lit components from packages/ship-set
        //    (the build emits a manifest of prop/event/slot schemas)
        // 2. Look up <app-*> or <ship-*> Sail components from
        //    .ship/components/*.sail (the compiler emits .schema.json
        //    files; the registry indexes them)
        self.component_registry.lookup(tag)
    }

    fn resolve_token(&self, name: &str) -> Option<TokenValue> {
        // Read tokens.css; resolve `color.surface` to `var(--ship-color-surface)`.
        self.token_set.lookup(name)
    }

    fn ssr_fetch(&self, resolution: &DataResolution, ctx: &SsrContext) -> Result<Value> {
        // Invoke the daemon's actual service for the resolved data path.
        // E.g., ship.agents.list -> runtime::services::agents::list(ctx.workspace, ...)
    }

    fn live_channel(&self, resolution: &DataResolution) -> Option<String> {
        if resolution.live {
            Some(format!("/api/v1/signals/{}/subscribe?{}",
                resolution.path,
                resolution.query_string()
            ))
        } else { None }
    }
}
```

## SDK manifest as the schema source

Ship's daemon already emits a typed SDK manifest (under `packages/ship-set/sdk/`). The Sail compiler reads this manifest at compile time to validate `data` expressions.

```json
// packages/ship-set/sdk/agents.json (existing)
{
  "namespace": "ship.agents",
  "methods": {
    "list": {
      "args": { "workspace": { "type": "string", "required": false } },
      "returns": { "type": "array", "items": "Agent" },
      "live": { "channel": "agents.list" }
    },
    "detail": {
      "args": { "id": { "type": "string", "required": true } },
      "returns": "Agent",
      "live": { "channel": "agents.detail" }
    }
  }
}
```

The Sail compiler indexes these manifests at startup; resolves `ship.agents.list({ workspace: 'x' })` against the manifest:
- Method `list` exists ✓
- Arg `workspace` is optional string, value `'x'` matches ✓
- Return type is `Agent[]` — flows into `data agents: Agent[]`

If the agent's name is mistyped (`ship.agnts.list`), compile error with suggestion. If the arg type is wrong (`workspace: 5`), compile error with the actual type.

## Component-library schemas

`<ship-*>` Lit components publish their schemas the same way:

```json
// packages/ship-set/dist/schemas/ship-button.schema.json
{
  "tag": "ship-button",
  "props": {
    "variant": { "type": "enum", "values": ["primary", "ghost", "danger"], "default": "primary" },
    "size": { "type": "enum", "values": ["sm", "md", "lg"], "default": "md" },
    "disabled": { "type": "boolean", "default": false }
  },
  "events": {
    "click": { "payload": "MouseEvent" }
  },
  "slots": [{ "name": "default" }]
}
```

The Sail compiler reads these to validate `<ship-button variant="..." size="..." disabled={...}>` usages.

ship-set's existing build process emits these schemas as part of its release bundle. Adding the Sail-aware fields is a one-time investment.

## OpenAPI adapter (OSS path)

For non-Ship backends, an OpenAPI source plugs in:

```ts
// packages/@sail/openapi-source
import { OpenAPISchemaSource } from '@sail/openapi-source'

const source = new OpenAPISchemaSource({
  spec: './api/openapi.yaml',
  baseUrl: '/api',
})

// In sail.config.ts:
export default {
  schemaSources: [source],
}
```

A `.sail` component then writes:
```ts
data users: User[] = api.users.list({ limit: 20 })
```

The compiler resolves `api.users.list` against the OpenAPI spec, validates args, types the return as `User[]` from the spec's components.schemas. SSR fetches via HTTP; live updates require a WebSocket or SSE convention the user opts into.

## GraphQL adapter

Mirror shape:

```ts
data agent: Agent = graph`
  query GetAgent($id: ID!) {
    agent(id: $id) { id name model }
  }
`({ id: props.agentId })
```

Tagged template literal, validated against the GraphQL schema, typed return.

## Static / no-backend adapter

For prototyping or fully static apps:

```ts
data demoData = staticJSON('./fixtures/agents.json')
data fromUrl = fetchJSON('/api/whatever')
```

No live updates. SSR'd value is the file content or fetch result. The simplest possible source.

## How adapters compose

A project can mix sources:

```ts
// sail.config.ts
import { ShipSchemaSource } from '@sail/ship-source'
import { OpenAPISchemaSource } from '@sail/openapi-source'

export default {
  schemaSources: [
    new ShipSchemaSource({ workspace: '.' }),       // ship.* namespace
    new OpenAPISchemaSource({                       // api.* namespace
      spec: './api/openapi.yaml',
      baseUrl: '/api',
      namespace: 'api',
    }),
  ],
}
```

Now `data x = ship.agents.list()` and `data y = api.users.list()` both work, against different sources, with full type safety.

## SSR context

The daemon (or whatever is doing SSR) calls `schema_source.ssr_fetch(resolution, ctx)`. The context provides:

```rust
pub struct SsrContext {
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub auth: AuthContext,
    pub locale: Option<String>,
    pub query: HashMap<String, String>,
    // ... whatever the schema source needs
}
```

Sources use this to scope queries. `ship.agents.list()` without explicit workspace defaults to the current one from context.

## Live channel binding

For `data live`:

```ts
data live mesh = ship.mesh.snapshot()
```

Compiler emits:
```js
const $mesh = data({
  initial: __SSR_VALUE_FOR_MESH__,    // baked at SSR time
  subscribe: '/api/v1/signals/mesh.snapshot/subscribe',
  onPatch: applyPatchTo($mesh),
})
```

The runtime opens the EventSource on mount. Server pushes patches. Patches apply via the protocol in [04-patch-protocol.md](04-patch-protocol.md).

For Ship: the daemon's kernel router already broadcasts events; we add a kernel-router actor per subscription that filters events by signal channel + emits patches downstream.

## Type emission

`data` declarations contribute to the component's `.d.ts`:

```ts
// my-component.sail.d.ts (generated)
export interface MyComponentData {
  readonly agent: Agent
  readonly mesh: MeshSnapshot              // updates live
  readonly jobs: JobRecord[]
}
```

LSP completion in the script block knows the types. References like `agent.name`, `mesh.actors[0].pid` are validated.

## Cache + invalidation

Schema source results are cached per-call (cache key = `(method, args)`). The daemon invalidates relevant cache entries when the underlying data changes (e.g., a new agent is registered → invalidate `ship.agents.list` cache). Subscribers receive patches.

For OSS adapters, caching is the source's responsibility; the framework provides cache primitives.

## What this gives us

A typed, validated, SSR-friendly data layer that:

- **Works with any backend** via the SchemaSource trait
- **Generates initial-render HTML** so first paint requires no JS roundtrip
- **Subscribes via SSE** for live updates (where the source supports it)
- **Type-checks call sites at compile time** — no stringly-typed query bugs
- **Composes multiple sources** (Ship + OpenAPI + GraphQL all in one project)
- **Open source for Layers 1+2**, Ship-specific only at Layer 3

The Ship adapter is the killer reference. Other adapters demonstrate the framework's portability and are launched alongside as part of the OSS release.

## Implementation phasing

- **Week 1-2:** SchemaSource trait + DataResolution types in `sail-compiler`. Test with a static-fixture source.
- **Week 3:** Ship adapter — read existing SDK manifests, validate against them. Use this for the first migration.
- **Week 4:** SSE channel wiring — Ship's daemon emits patches via the kernel router; runtime subscribes and applies.
- **Week 5+:** OpenAPI adapter, GraphQL adapter, static adapter. These ship as separate npm packages alongside @sail/runtime + @sail/compiler.

## Non-goals for v1

- **Mutation queries.** `data` declarations are read-only. Mutations go via plain HTTP (`fetch(...)`) or via the patch-emit endpoint (`POST /api/v1/signals/<channel>/patch`). A typed mutation API is a future enhancement.
- **Pagination as a first-class concept.** `data jobs = ship.jobs.list({ cursor })` works; the runtime doesn't auto-paginate. Authors compose pagination in user code or via dedicated components.
- **Caching policies declared in `.sail`.** Cache lifetime is the source's concern. v1 keeps it implicit.

## Why the boundary matters

Sail's value proposition depends on `data` working seamlessly. If it's clunky, half the differentiation evaporates.

The trait-based design lets us:
- Open source the framework with multiple useful adapters out of the box
- Sell Ship's adapter as the integrated reference (not as a lock-in)
- Allow third parties to write adapters for their stack (Drupal? Wordpress? Salesforce?) without forking
- Keep Ship's competitive edge in Layer 3 while making Layers 1+2 a real OSS project

This is the architectural choice that lets Sail be **a framework**, not a Ship implementation detail.
