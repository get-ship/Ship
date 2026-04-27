# Component library — Sail + ship-set

How `<ship-*>` Lit components from `packages/ship-set/` and `<app-*>` Sail components from `.ship/components/` coexist and compose.

## The two-library reality

Ship apps will draw from two component pools indefinitely:

1. **ship-set** — ~200 Lit-based components in `packages/ship-set/`. Buttons, cards, popovers, terminal, code-editor, graph-viz, etc. Mature, themed, tested, published as `@ship/set`.
2. **app-** Sail components — application-specific composition under `.ship/components/`. Author-written, project-scoped. New ones are `.set`.

Both are custom elements. Both register via `customElements.define`. Both compose into Sail templates the same way.

```html
<!-- a route file: pipelines.set -->
<script lang="ts">
  data live jobs = ship.jobs.list()
</script>

<ship-card>                                   <!-- Lit component from ship-set -->
  <span slot="header">Pipelines</span>
  <app-pipeline-board jobs={jobs} />          <!-- Sail component, project-local -->
</ship-card>
```

The browser doesn't know or care which library each tag comes from. Both are just custom elements.

## Schema discovery

For the Sail compiler to type-check call sites, both libraries publish schemas in the same format.

### ship-set's schema emission

`packages/ship-set/` already builds bundles. The build is extended to emit one schema per component:

```
packages/ship-set/
├── dist/
│   ├── web-components/...
│   └── schemas/
│       ├── ship-button.schema.json
│       ├── ship-card.schema.json
│       ├── ship-popover.schema.json
│       └── ...
└── package.json
```

Each schema file (per [02-compiler-architecture.md § Phase 10](02-compiler-architecture.md#phase-10--schema-emission)):

```json
{
  "tag": "ship-button",
  "version": "1.0",
  "props": {
    "variant": { "type": "enum", "values": ["primary", "ghost", "danger"], "default": "primary" },
    "size": { "type": "enum", "values": ["sm", "md", "lg"], "default": "md" },
    "disabled": { "type": "boolean", "default": false }
  },
  "events": {
    "click": { "payload": "MouseEvent" }
  },
  "slots": [
    { "name": "default" },
    { "name": "icon-leading" },
    { "name": "icon-trailing" }
  ]
}
```

The schemas are derived from the Lit component's static properties + JSDoc + an explicit `@event` / `@slot` annotation set. A small build script (`scripts/emit-component-schemas.mjs`) walks the Lit source and emits the JSON. ~1 day of work to write the script + annotate the existing components.

### Sail components' schema emission

The Sail compiler emits `<tag>.schema.json` automatically for every `.set` file (Phase 10).

```
.ship/components/
├── app-agent-modal.set
├── app-agent-modal.schema.json    ← emitted by compiler
├── app-pipeline-board.set
├── app-pipeline-board.schema.json
└── ...
```

Same shape as ship-set's schemas. Compiler reuses its own emission.

## Component registry

The Sail compiler maintains a unified registry across both pools:

```rust
pub struct ComponentRegistry {
    /// Indexed by tag, populated from ship-set's published schemas + project's .set files.
    tags: HashMap<String, ComponentSchema>,
}

impl ComponentRegistry {
    pub fn from_sources(sources: &[ComponentSource]) -> Self {
        let mut tags = HashMap::new();
        for source in sources {
            for (tag, schema) in source.discover() {
                if let Some(existing) = tags.insert(tag.clone(), schema.clone()) {
                    // Conflict — log warning, last-source-wins
                }
            }
        }
        Self { tags }
    }
}

pub trait ComponentSource {
    fn discover(&self) -> HashMap<String, ComponentSchema>;
}

pub struct NpmComponentSource { /* reads node_modules/<pkg>/dist/schemas/*.json */ }
pub struct LocalComponentSource { /* reads .ship/components/*.schema.json */ }
```

Configured in `sail.config.ts`:

```ts
export default {
  componentSources: [
    new NpmComponentSource({ package: '@ship/set' }),
    new LocalComponentSource({ dir: '.ship/components' }),
  ],
}
```

For OSS users without ship-set, just the local source is needed. Other component libraries (`@my-org/components`) plug in the same way.

## How compilation uses the registry

When the compiler sees `<ship-button variant="primary">`:

1. Look up `ship-button` in the registry → ComponentSchema
2. Validate `variant="primary"` against the prop schema:
   - Prop exists ✓
   - Value `"primary"` is in the enum ✓
3. If `<ship-button variant="bogus">` — compile error with the allowed values listed
4. If `<unknown-tag>` — compile error: "unknown component" with did-you-mean suggestions

Same for events:

```html
<ship-button on:click={handle}>
```

Compiler validates:
- `click` is a published event ✓
- `handle`'s parameter type matches `MouseEvent` (from the schema) ✓

And props on Sail components:

```html
<app-pipeline-board refresh-ms={2000} jobs={data.jobs} />
```

Compiler validates `refreshMs: number` and `jobs: JobRecord[]` against `app-pipeline-board.schema.json`.

## Lit interop edge cases

A few quirks of mixing Lit + Sail to know about:

### 1. Lit components default to shadow DOM

Most ship-set components use shadow DOM (Lit's default). That means:
- Slot content goes into the shadow tree
- Theme tokens cascade through CSS custom properties (works)
- `querySelector` from outside doesn't penetrate (intentional encapsulation)

Sail components are light DOM. Mixed-mode in a single tree:

```html
<ship-card>                       <!-- shadow DOM -->
  <span slot="header">Hi</span>   <!-- distributed into ship-card's shadow -->
  <app-thread />                  <!-- light DOM child of ship-card -->
</ship-card>
```

Works. Browser handles the boundaries.

### 2. Property vs attribute setting

Lit components reflect properties to attributes only if `reflect: true`. Sail compiles to property setters by default for non-string types (numbers, booleans, objects):

```html
<ship-button variant="primary" />   <!-- string → attribute -->
<my-component items={data} />        <!-- array → property (compiled to el.items = data) -->
```

The compiler decides:
- String literals: `setAttribute`
- Reactive expressions or non-string types: `el.propName = value` (and a reactive update on signal change)

### 3. Custom event payloads

Lit components dispatch CustomEvents with payload in `event.detail`. Sail handlers expect typed `event.detail`:

```html
<my-typeahead on:select={handle} />

// handle receives event with detail typed against the schema's event payload
function handle(event: CustomEvent<{ id: string, item: Item }>) {
  console.log(event.detail.id)  // typed
}
```

The compiler emits the addEventListener wiring; the runtime invokes the handler. No conversion layer.

## ship-set migration path (eventual, not gating)

ship-set components are Lit today. They could migrate to Sail over time. Some pros and cons.

### Pros of migration

- Drop Lit dependency from the runtime bundle (~5KB savings)
- Unified codebase: one framework, not two
- Sail's compile-time validation reaches the primitives too
- Patch-protocol-aware primitives (e.g., a `<ship-data-table>` whose rows are signal-patchable)

### Cons of migration

- Real engineering work: ~200 components × maybe 2-4 hours each = 400-800 hours
- Risk of regression on already-working primitives
- No urgent functional gain — Lit's primitives work fine
- Ship-set has years of design polish; rewriting is a chance to break it

### Recommended approach

**Migrate when migration earns its keep.**

A `<ship-button>` working fine in Lit doesn't need to become Sail. A `<ship-data-table>` with complex internal state and a patch-protocol use case earns the migration. The criteria:

- Component has signal-shaped state that Sail's reactivity would simplify
- Component is a hot-spot for performance (Sail's fine-grained updates would help)
- Component is being significantly redesigned anyway (combine with the migration)
- Component would benefit from the patch protocol (canvas, table, list, editor)

Migrate piece by piece. Both worlds coexist forever via the customElements registry.

### Migration mechanics

Per component:
1. Author the Sail equivalent at `packages/ship-set/src/sail/<tag>.set`
2. Export both: the Lit class continues to be the default; the Sail version is registered under the same tag with `customElements.define`. **Last registration wins** — but `customElements.define` errors if the tag is already registered, so we do feature-detect:
   ```ts
   if (!customElements.get('ship-button')) {
     // Sail version registers
   }
   ```
3. Add a prefer-sail flag to ship-set's bundle that controls which is loaded
4. Consumers opt in via `import '@ship/set/sail-bundle'` (vs the default Lit bundle)

This way ship-set ships both forever; consumers pick.

## ship-set's own evolution

Independent of Sail migration, ship-set's design needs continuing investment:

- More components: `ship-data-table`, `ship-virtual-list`, `ship-canvas`, `ship-form`, `ship-tree-view`, `ship-tabs` (Sail port already started), …
- Token system breadth: more nuanced spacing, color semantics, motion
- Theming polish: dark mode + high-contrast + brand variants
- A11y rigor: ARIA testing, keyboard-only flows, focus management
- Documentation: every component with a real example + Playwright spec

These are ship-set concerns, not Sail concerns. The framework provides the platform; the library provides the building blocks.

## Token system

ship-set publishes design tokens via `tokens.css` (Layer 3, Ship-specific). The Sail compiler resolves `${color.surface}` token references against this file at compile time.

For OSS users, the convention is:
- Project root has a `tokens.css` (or similar) with CSS custom properties
- The Sail compiler reads it as a token source via the SchemaSource trait
- Token references in `.set` files are validated and resolved at compile time

ship-set provides a reference token set; OSS users replace with their own.

## OSS path: standalone Sail apps

A Sail app without ship-set:

```
my-app/
├── sail.config.ts                    # configures sources
├── tokens.css                        # design tokens
├── components/
│   ├── my-button.set
│   ├── my-card.set
│   └── my-list.set
└── pages/
    ├── index.set
    └── about.set
```

The author writes their own primitives. The framework + compiler + runtime handle everything else.

For Ship users, ship-set is the headstart — drop-in 200 components on day one.

## Summary

- Both Lit (`<ship-*>`) and Sail (`<app-*>` or new `<ship-*>`) compose seamlessly via custom elements
- A unified component registry indexes both schemas for compile-time validation
- The registry is configurable; OSS users plug their own component sources
- ship-set migrates piece-by-piece when migration earns its keep, not on a flag day
- Token system is the design-system glue, plug-in for OSS

This keeps ship-set's investment safe, gives Sail authors immediate access to a rich library, and preserves the OSS path for non-Ship users.
