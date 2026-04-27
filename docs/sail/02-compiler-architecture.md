# Sail compiler architecture

How `.sail` files become shippable JS. Design and implementation plan for `sail-compiler` — Layer 2 of the Sail stack (see [09-oss-portability.md](09-oss-portability.md)).

## High-level pipeline

```
.sail file
  │
  ▼
[1] Lexer / parser           ─→ AST (script, template, style)
  │
  ▼
[2] Schema resolution         ─→ types pulled from schema source
  │   (Ship daemon, OpenAPI, etc.)
  ▼
[3] Type analysis             ─→ types for props, state, derived, data
  │   (TS server or via tsc API)
  ▼
[4] Template lowering         ─→ AST with reactive bindings explicit
  │
  ▼
[5] Reactivity classification ─→ static vs reactive interpolation
  │
  ▼
[6] Style scoping             ─→ scoped CSS, :host rewrite, token resolution
  │
  ▼
[7] SSR emission              ─→ HTML render function (.html.tmpl)
  │
  ▼
[8] Runtime emission          ─→ ES module (.js)
  │
  ▼
[9] Type emission             ─→ .d.ts for the custom element
  │
  ▼
[10] Schema emission          ─→ .schema.json for LSP / devtools
```

Each phase has a clean input → output contract. Phases 1-6 are pure (deterministic, given inputs). Phases 2 and 3 read external state (schemas, types). Phases 7-10 are emission only.

## Where it lives

```
crates/
└── sail-compiler/                    # Rust crate, no Ship dependencies
    ├── src/
    │   ├── lib.rs                    # public API: compile_file(path, opts)
    │   ├── parse/                    # phase 1: lexer + parser
    │   │   ├── lexer.rs
    │   │   ├── parser.rs
    │   │   └── ast.rs                # AST types
    │   ├── analyze/                  # phases 2-5: schema resolution + type + classification
    │   │   ├── schema.rs             # SchemaSource trait + resolution
    │   │   ├── types.rs              # type analysis
    │   │   ├── reactivity.rs         # static/reactive classification
    │   │   └── deps.rs               # dependency-graph for derived/effect
    │   ├── lower/                    # phase 4: AST → lowered IR
    │   │   ├── template.rs           # template directive lowering
    │   │   ├── style.rs              # CSS scoping, :host, tokens
    │   │   └── ir.rs                 # lowered IR types
    │   └── emit/                     # phases 7-10: emission
    │       ├── ssr.rs                # .html.tmpl
    │       ├── runtime.rs            # .js
    │       ├── types.rs              # .d.ts
    │       └── schema.rs             # .schema.json
    └── tests/
        ├── corpus/                   # snapshot tests against fixed .sail files
        └── ui/                       # browser-tested Playwright integration

apps/shipd/src/views/
└── sail.rs                           # Ship's wrapper around sail-compiler;
                                      # serves /_sail/components/<tag>.js etc.
```

The `sail-compiler` crate has zero Ship-specific code. Ship's daemon depends on the crate and adds Ship-specific schema sources + serving routes.

## Phase 1 — parser

Hand-rolled parser for the three-block format. Already proven by the existing SFC compiler (`apps/shipd/src/views/sfc/parser.rs`). Sail's parser extends it for the new template syntax.

### Parser features

- Three-block extraction: `<script>`, `<template>`, `<style>` (or no `<template>` — bare `.sail` is technically valid).
- Block attribute parsing: `<script lang="ts">`, `<style src="./x.css">`, `<style global>`.
- Template parser handles:
  - HTML elements with attributes
  - Interpolations `{expr}` (where `expr` is a JS expression)
  - Control-flow blocks `{#if}/{:else}/{/if}`, `{#for}/{:empty}/{/for}`, `{#await}/{:then}/{:catch}/{/await}`
  - Bindings `bind:value={x}`, `bind:checked={x}`
  - Event handlers `on:click={fn}` with modifier syntax `on:event|modifier:arg={fn}`
  - Slots `<slot name="x" />` and slot fill `<el slot="x" let:item>`
  - Special elements `<sail:window>`, `<sail:document>`
- Script parser: tokenize JS/TS to find top-level keyword declarations (`prop`, `state`, `derived`, `data`, `effect`, `mount`, `unmount`, `event`, `function`).
- Style parser: minimal CSS tokenizer (existing one in `apps/shipd/src/views/sfc/style.rs` is fine).

### Output: AST

```rust
pub struct SailFile {
    pub script: Option<ScriptBlock>,
    pub template: Option<Template>,
    pub style: Option<StyleBlock>,
    pub source: SourceMap,             // line/col → byte offset for error reporting
}

pub struct ScriptBlock {
    pub lang: ScriptLang,              // Js | Ts
    pub props: Vec<PropDecl>,
    pub state: Vec<StateDecl>,
    pub derived: Vec<DerivedDecl>,
    pub data: Vec<DataDecl>,
    pub effects: Vec<EffectDecl>,
    pub events: Vec<EventDecl>,
    pub mount: Option<String>,         // body source
    pub unmount: Option<String>,
    pub functions: Vec<FunctionDecl>,
    pub raw: String,                   // verbatim for type-stripping pass
}

pub struct Template {
    pub root: Vec<Node>,
}

pub enum Node {
    Element { tag, attrs, children, key: Option<Expr> },
    Component { tag, props, events, slots, children },
    Text(String),
    Interpolation(Expr),
    If { cond: Expr, then_: Vec<Node>, elifs: Vec<(Expr, Vec<Node>)>, else_: Option<Vec<Node>> },
    For { item: Pattern, iter: Expr, key: Option<Expr>, body: Vec<Node>, empty: Option<Vec<Node>> },
    Await { promise: Expr, then_: Option<(Pattern, Vec<Node>)>, loading: Vec<Node>, catch: Option<(Pattern, Vec<Node>)> },
    Slot { name: String, props: Vec<(String, Expr)>, fallback: Vec<Node> },
    SlotFill { name: String, lets: Vec<(String, String)>, body: Vec<Node> },
    Window { events: Vec<(String, Expr)> },
    Document { events: Vec<(String, Expr)> },
    RawHtml(Expr),
}
```

## Phase 2 — schema resolution

Resolve `data` declarations against schema sources.

```rust
pub trait SchemaSource: Send + Sync {
    /// Resolve `data x = expr` to a typed query.
    fn resolve_data(&self, expr: &str) -> Result<DataResolved>;

    /// Look up a component tag's prop schema (for call-site validation).
    fn resolve_component(&self, tag: &str) -> Option<ComponentSchema>;
}

pub struct DataResolved {
    pub return_type: TypeRef,          // schema-source-specific type ID
    pub initial_query: String,         // SSR call (e.g., daemon-side)
    pub patch_subscription: Option<PatchSub>,  // for `data live`
}

pub struct ComponentSchema {
    pub props: HashMap<String, PropSchema>,
    pub events: HashMap<String, EventSchema>,
    pub slots: Vec<SlotSchema>,
}
```

Schema sources are pluggable. For Ship:

```rust
pub struct ShipSchemaSource {
    daemon: Arc<ShipRuntime>,
}

impl SchemaSource for ShipSchemaSource {
    fn resolve_data(&self, expr: &str) -> Result<DataResolved> {
        // Parse `ship.agents.list(workspace=...)` against the daemon's
        // schema. Validate args. Return type. Generate SSR fetch + SSE
        // subscription channel for `live` queries.
    }
    fn resolve_component(&self, tag: &str) -> Option<ComponentSchema> {
        // Look up the tag in the project's `.sail` files and ship-set's
        // published schemas. Returns merged prop/event/slot info.
    }
}
```

OpenAPI / GraphQL adapters are similar shape, different resolution. See [05-server-data-bridge.md](05-server-data-bridge.md).

## Phase 3 — type analysis

Two paths depending on configuration.

### Path A: type-stripping only (default for runtime)

Run swc/oxc/esbuild on the script block's raw source to strip type annotations. Output: pure JS. **No type checking.** This is the runtime path; the LSP handles type checking separately.

Cost: ~100μs per file. Embed swc as a Rust dep (`swc_ecma_*` crates) or shell to a node-side stripper.

### Path B: full type-check (LSP / CI gate)

Spawn `tsc --noEmit` with a synthesized `tsconfig` that includes:
- The script block as a module
- The synthesized prop/state/derived/data declarations as ambient types
- The schema-source-resolved types for `data` declarations
- Imports from `@ship/sail/types` for runtime types
- Reference types from `<tag>.d.ts` files for cross-component validation

Reports any errors via TS's diagnostic format. LSP surfaces them in the editor; CI uses them as a gate.

This path is slow (~50ms per file with type checking). Run on save / commit, not per request.

### Output of analysis

```rust
pub struct AnalyzedFile {
    pub ast: SailFile,
    pub schema: AnalyzedSchema {       // typed declarations
        pub props: HashMap<String, TypedProp>,
        pub state: HashMap<String, TypedState>,
        pub derived: HashMap<String, TypedDerived { deps: Vec<DepRef> }>,
        pub data: HashMap<String, TypedData { resolution: DataResolved }>,
        pub effects: Vec<TypedEffect { deps: Vec<DepRef> }>,
        pub events: HashMap<String, EventSchema>,
    },
    pub diagnostics: Vec<Diagnostic>,
}

pub enum DepRef {
    State(String),
    Derived(String),
    Data(String),
    Prop(String),
}
```

`deps` is the dependency graph: which signals each derived/effect tracks. Computed by walking the expression's AST and finding identifier references that match reactive declarations.

## Phase 4 — template lowering

Transform the parser's AST into a flatter, emit-ready IR.

### Lowering rules

```
{x}              →   InterpolationNode { expr: x, deps: analyze(x) }
{#if cond}A{/if} →   IfBlock { cond: cond, deps: analyze(cond), then: [A] }
{#for x of y}…   →   ForBlock { item: x, iter: y, key: optional, body: […] }
on:click={fn}    →   EventBinding { event: "click", handler: fn, modifiers: [] }
bind:value={x}   →   TwoWayBinding { input: "value", target: x, target_kind: state }
prop={value}     →   PropBinding { name: "prop", value: value, deps: analyze(value) }
```

### Reactivity classification (Phase 5, fused with Phase 4)

For each `{expr}` and each binding's value expression, walk the AST and classify:

- **Static** — no reactive dependencies (props, state, derived, data, builtin reactive APIs)
  - Bake into SSR'd HTML at server-render
  - No client wiring emitted
- **Reactive** — at least one reactive dependency
  - Emit a `bind(node, depsArray, () => evaluator())` call in the runtime emission
  - Server-renders with the initial value baked; client wires up the subscription

Compiler's classifier:

```rust
fn classify(expr: &Expr, scope: &AnalyzedSchema) -> Classification {
    let deps = collect_idents(expr).into_iter()
        .filter_map(|id| scope.find_reactive(&id))
        .collect::<Vec<_>>();
    if deps.is_empty() { Classification::Static }
    else { Classification::Reactive(deps) }
}
```

### Output: lowered IR

```rust
pub struct LoweredFile {
    pub schema: AnalyzedSchema,
    pub template: LoweredTemplate,
    pub style: ScopedStyle,
}

pub struct LoweredTemplate {
    pub static_html: String,           // SSR-renderable HTML with placeholders for reactive binds
    pub bindings: Vec<RuntimeBinding>, // wiring instructions for the client
}

pub struct RuntimeBinding {
    pub anchor: BindingAnchor,         // "the 5th text node of the 3rd element"
    pub kind: BindingKind,             // Text, Show, Attr, Class, Bind, Event, For, If
    pub deps: Vec<DepRef>,
    pub evaluator: String,             // JS expression source
}
```

Anchors are stable paths into the SSR'd HTML. The runtime walks the actual DOM once at hydrate time, resolves anchors to nodes, attaches subscriptions.

## Phase 6 — style scoping

Existing `apps/shipd/src/views/sfc/style.rs` is the reference. Sail extends it:

- `:host` rewrite to `[data-component="<tag>"]` (already implemented)
- `${token.path}` interpolation: replace with `var(--ship-token-path)`, validate token exists
- `<style global>` flag: skip scoping
- `<style src="./x.css">` external import: read file, prepend before inline content
- Optional CSS lint pass: error on bare hex literals (recommends `${color.X}` token)

Output: `ScopedStyle { css: String, dependencies: Vec<TokenRef> }`.

## Phase 7 — SSR emission

Generate a function the daemon calls to render initial HTML:

```rust
pub fn emit_ssr(lowered: &LoweredFile) -> String {
    // Output is a Jinja-like or Tera template the daemon evaluates
    // OR a Rust function (preferred — typed, faster)
    //
    // Function signature:
    //   fn render(ctx: &SsrContext) -> String
    //
    // ctx provides initial values for state, derived, data
    // (state is initial-default; data is resolved server-side via schema source)
}
```

The Rust-function option means the daemon dynamically loads compiled SSR functions per component. They're cheap to invoke (few μs).

For mass-deployment scenarios where the daemon doesn't run Rust (OSS users), emit a `.html.tmpl` text template the deployer's chosen engine handles. **Default: Rust function path. Optional: text-template fallback.**

## Phase 8 — runtime emission

The biggest output. The compiler emits an ES module that defines the custom element class.

### Module shape

```js
// Auto-generated by Sail compiler.
import { signal, derived, effect, mount as $mount, defineComponent, bind, on, slot, ... } from '@sail/runtime'

export class MyComponent extends HTMLElement {
  static observedAttributes = ['prop1', 'prop2']

  #signals = {}
  #cleanups = []

  constructor() { super() }

  connectedCallback() {
    this.dataset.component = 'my-component'

    // 1. read attributes → typed prop signals
    const $prop1 = signal(decodeProp('prop1', this.getAttribute('prop1'), { type: 'string', default: '' }))
    this.#signals.prop1 = $prop1

    // 2. user-script declarations
    const $count = signal(0)
    const $double = derived(() => $count() * 2)
    this.#cleanups.push(effect(() => { /* effect body */ }, [$count]))

    // 3. user-script functions (closure-resolved; available to event bindings below)
    const inc = () => $count.set($count() + 1)

    // 4. mount: render initial HTML if not SSR-hydrated
    if (!this.hasAttribute('ssr-hydrated')) {
      this.innerHTML = STATIC_HTML
    }

    // 5. wire reactive bindings
    bind(this.querySelector('[data-bid-1]'), 'text', $count, () => $count())
    on(this.querySelector('[data-bid-2]'), 'click', inc)
    // ... etc

    // 6. lifecycle: mount {} block
    /* user mount code */
  }

  disconnectedCallback() {
    for (const c of this.#cleanups) c()
    /* user unmount code */
  }

  attributeChangedCallback(name, _, val) {
    const sig = this.#signals[camelOf(name)]
    if (sig) sig.set(decodeProp(name, val, this.#schema[name]))
  }
}

defineComponent('my-component', MyComponent)
```

### Anchor strategy

The compiler annotates the SSR HTML with stable IDs:

```html
<button data-bid="b1">Clicks: <span data-bid="b2"></span></button>
```

The runtime resolves anchors via `querySelector`. IDs are unique per component instance (the wrapper rewrites `data-bid` values per instance to avoid collisions across multiple instances of the same component).

Cheap, deterministic, no fragile DOM-walk.

### Bundle size target

Per-component module: <2KB minified+gzipped for typical components. Total runtime+component for a non-trivial page: <30KB.

## Phase 9 — type emission

Emit a `.d.ts` declaration that lets TypeScript consumers see the custom element:

```ts
declare module '@sail/components' {
  export interface MyComponent extends HTMLElement {
    prop1: string
    prop2: number
    addEventListener(type: 'select', listener: (e: CustomEvent<{ id: string }>) => void): void
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'my-component': MyComponent
  }
}
```

This is what makes JSX usage (in non-Sail consumers) and direct DOM API usage type-checked.

## Phase 10 — schema emission

Public schema for LSP, devtools, docs:

```json
{
  "tag": "my-component",
  "version": "1.0",
  "props": {
    "prop1": { "type": "string", "default": "", "required": false, "doc": "..." },
    "prop2": { "type": "number", "default": 0, "required": false }
  },
  "events": {
    "select": { "payload": { "id": "string" }, "doc": "Fired when..." }
  },
  "slots": [
    { "name": "header", "props": {} },
    { "name": "default", "props": {} }
  ],
  "dependencies": {
    "components": ["ship-button", "ship-card"],
    "data": ["ship.agents.list"]
  }
}
```

LSP reads this when the user types `<my-component ` to offer completion + validation.

## Caching

The compiler caches by file (path, mtime, content hash). On request, the daemon checks the cache; if valid, serves from cache. If not, recompiles.

Per-file compile time should be <50ms cold, <5ms warm.

## Errors and diagnostics

All errors include:
- File path + line/col span
- Severity (error / warning / info)
- Code (e.g., `SAIL/E0042`)
- Message in the framework's vocabulary
- Optional fix suggestion

Errors are emitted as JSON for the LSP and as formatted text for the CLI:

```
sail-compiler: error[SAIL/E0042]: prop 'count' expects number, got string
   ╭─ agent-modal.sail:14:35
   │
14 │   <my-counter count="not a number">
   │                     ^^^^^^^^^^^^^^ — type mismatch
   │
   = help: pass a number expression: `count={5}`
   = note: defined at counter.sail:3:1
```

## Testing

The compiler has three test layers:

1. **Snapshot tests** — `tests/corpus/` holds fixed `.sail` files; tests assert the emitted JS / SSR / .d.ts / schema match committed snapshots. Drift is caught instantly.
2. **Integration tests** — Playwright specs that mount compiled components in a real browser and verify behavior. See [08-testing-benchmarks.md](08-testing-benchmarks.md).
3. **Token-efficiency benchmark** — fixed corpus of UI problems, agent-authoring runs against multiple frameworks. Sail's emission must hit the targets.

## Implementation phasing

- **Week 1:** Phase 1 parser + Phase 6 style scoping (extend existing). Snapshot tests.
- **Week 2:** Phases 2-5 (analysis + lowering). The reactivity classifier is the heart.
- **Week 3:** Phase 7 SSR + Phase 8 runtime emission. First Sail file end-to-end browser-rendered.
- **Week 4:** Phases 9-10 (.d.ts + schema). LSP integration. Migration tool from `.ship` to `.sail`.

After week 4, the compiler is MVP. Beyond MVP: caching polish, error message quality, more schema sources, type-check-on-save in LSP.

## Non-goals

- **Multi-file inheritance / mixins.** v1 is one-component-one-file.
- **Cross-cutting hooks (Vue composables, React hooks).** State and derivations are file-local. Shared logic via plain TS modules imported into the script block.
- **Dynamic component loading** (lazy `<sail:component this={...}>`). Reserved for future.
- **HMR for the script block.** Hot-reload templates and styles only in v1; script changes require refresh. Same as Svelte / Solid.

## What this gives us

A compiler under our control that produces:
- Type-safe components
- SSR'd HTML for first paint
- Fine-grained reactive hydration
- Patchable signals (see [04-patch-protocol.md](04-patch-protocol.md))
- Schema-aware error messages
- Open-source-able, backend-agnostic output

The compiler is the strategic asset. The runtime is small ([03-runtime-architecture.md](03-runtime-architecture.md)). The platform integration ([05-server-data-bridge.md](05-server-data-bridge.md)) is where Ship-specific value lands.
