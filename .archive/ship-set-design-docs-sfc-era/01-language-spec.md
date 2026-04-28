# Sail language spec

The complete syntax reference for `.set` files. This is the authoritative source — compiler, LSP, docs, and migration tooling implement against this document.

## File format

Three optional blocks. Block order is convention (script → template → style); the compiler does not enforce.

```
my-component.set
├── <script lang="ts">         optional — declarations + functions
│   ...
│ </script>
│
│ <!-- optional template — HTML with Sail interpolations -->
│ <div>...</div>
│
└── <style>                     optional — CSS, scoped, typed tokens
    ...
  </style>
```

A file with no `<script>` block is pure-template (renders static HTML, possibly with `data` declarations passed through props). A file with no `<style>` is unstyled. A file with no `<template>` is invalid.

### Filename rules

- Lowercase kebab-case.
- Must contain a hyphen (custom-element spec requirement).
- `.set` extension.
- Filename → custom-element tag (e.g., `agent-modal.set` → `<agent-modal>`).

```
✅ agent-modal.set        → <agent-modal>
✅ pipeline-step.set      → <pipeline-step>
✅ x-y.set                → <x-y>
❌ modal.set              ← no hyphen, won't register
❌ AgentModal.set         ← uppercase rejected
❌ agent_modal.set        ← underscores rejected (must be kebab)
```

## The `<script>` block

TypeScript by default. The daemon strips types via swc/esbuild/oxc; the runtime is JS. No author-visible build step.

### Top-level declarations

Seven keyword forms, each with one shape. Order independent (compiler resolves dependencies).

```ts
// 1. prop — typed component prop
prop label: string = 'Default'
prop count: number = 0
prop required: boolean!                              // ! = no default, required
prop variant: 'primary' | 'ghost' = 'primary'
prop onSelect: (id: string) => void                  // function props allowed

// 2. state — local reactive state
state turns: Turn[] = []
state draft = ''                                     // type inferred
state cursor: number | null = null

// 3. derived — computed, dependencies tracked
derived canSend = !!draft.trim() && !sending
derived turnCount = turns.length
derived display = props.variant === 'primary' ? 'A' : 'B'

// 4. data — server-owned, typed against schema source
data agent = ship.agents.detail(id=props.agentId)
data live mesh = ship.mesh.snapshot()                // `live` enables SSE subscription
data jobs: JobRecord[] = ship.jobs.list({ limit: 50 })

// 5. effect — re-runs when dependencies change
effect on props.sessionId, draft {
  console.log('changed')
  cleanup => unsub()
}

effect once {                                        // runs once at mount
  // setup
  cleanup => teardown()
}

// 6. function — regular fns, callable from template
function send() {
  if (!canSend) return
  fetch(`/api/v1/sessions/${props.sessionId}/message`, ...)
  draft = ''                                         // direct mutation re-triggers reactivity
}

async function refresh() {
  const r = await fetch(...)
  jobs = await r.json()                              // assign to a `state` to update reactively
}

// 7. lifecycle — minimal; effects cover most cases
mount {
  // runs once after first render
}

unmount {
  // runs on disconnectedCallback
}
```

#### Reserved word: `props`

Inside a script block, `props` is a read-only proxy of the component's prop values. Each declared `prop X` becomes accessible as both `props.X` and the bare identifier `X` (compiler-resolved):

```ts
prop label: string = 'Default'

// Both forms work; compiler emits the same code:
console.log(label)
console.log(props.label)
```

Bare identifiers are preferred for terseness. `props.X` is preferred when the script imports modules that might shadow names.

#### Reserved word: `el`

`el` references the host custom element from inside lifecycle hooks and effects:

```ts
mount {
  el.querySelector('input')?.focus()
}
```

#### Reactivity model

- `state X = init` declares a fine-grained signal. Reads track dependencies; writes notify subscribers.
- `derived X = expr` is a memoized computed. Re-evaluates only when its dependencies change.
- `data X = expr` calls the schema source's resolver at compile time AND emits initial-paint HTML server-side. At runtime, the value is hydrated; `live` modifier enables SSE subscription for ongoing updates.
- `effect on a, b { ... }` runs initially + on any of `a, b` changing. Optional `cleanup => fn` runs before next re-run and on unmount.
- `effect once { ... }` runs once at mount.
- Plain JS variables (declared with `const`/`let` outside the keyword forms) are non-reactive. The compiler validates that template expressions only reference reactive declarations or compile-time-knowable values.

## The `<template>` block

HTML with three additions: `{expr}` interpolation, control flow blocks, and reactive bindings.

### Interpolation

```html
<h1>{title}</h1>
<p>Hello, {user.name}</p>
<span>{count + 1}</span>
<a href="/profile/{user.id}">Profile</a>     <!-- works in attribute values -->
```

The compiler classifies each `{expr}`:

- Expression depends only on compile-time-knowable values → baked into SSR'd HTML, no client wiring
- Expression depends on a reactive declaration (`state`, `derived`, `data`, `props`) → emits a reactive binding that updates that text node only when the dependency changes

Authors don't pick. The compiler decides. **One mental model.**

### Control flow

```html
{#if condition}
  <p>Truthy branch</p>
{:else if other}
  <p>Other branch</p>
{:else}
  <p>Fallback</p>
{/if}

{#for item of list}
  <li>{item.name}</li>
{/for}

<!-- with key for keyed reconciliation (recommended for any non-trivial list) -->
{#for item of list key={item.id}}
  <li>{item.name}</li>
{/for}

<!-- with index -->
{#for item, i of list key={item.id}}
  <li>{i + 1}. {item.name}</li>
{/for}

<!-- empty fallback -->
{#for item of list}
  <li>{item.name}</li>
{:empty}
  <p>No items</p>
{/for}

<!-- await for promises (state of type Promise<T>) -->
{#await fetchPromise}
  <p>Loading…</p>
{:then result}
  <p>{result.name}</p>
{:catch err}
  <p>Failed: {err.message}</p>
{/await}
```

### Component composition

Sail components are custom elements. Compose by tag:

```html
<ship-button variant="primary" disabled={!canSend} on:click={send}>
  Send
</ship-button>

<agent-thread sessionId={props.sessionId} />

<my-typeahead
  placeholder="Search agents…"
  fetchUrl="/api/v1/agents/search"
  on:select={onAgentPicked} />
```

The compiler validates:
- Component tag is registered (in ship-set, in the project, or via plugin)
- Each prop value's type matches the component's declared prop type
- Each `on:event` handler is a function

### Bindings

```html
<!-- two-way binding for form inputs -->
<input bind:value={draft} placeholder="Type…" />
<select bind:value={selectedId}>
  ...
</select>
<input type="checkbox" bind:checked={agree} />

<!-- one-way attribute binding (compile to data-attr equivalent) -->
<button class="btn {variant}" disabled={!canSend}>Send</button>

<!-- typed component prop binding -->
<ship-input bind:value={draft} />
```

### Event handlers

```html
<!-- DOM events -->
<button on:click={send}>Send</button>
<input on:input={(e) => draft = e.target.value} />
<form on:submit|preventDefault={save}>...</form>

<!-- modifiers -->
<a href="/somewhere" on:click|preventDefault|stopPropagation={navigate}>Link</a>
<input on:keydown|key:Enter={submit} />
<input on:input|debounce:300={search} />

<!-- custom events from child components -->
<my-typeahead on:select={onAgentPicked} />
<!-- in onAgentPicked, e.detail is typed against the component's declared event payload -->

<!-- window/document events -->
<ship-set:window on:keydown={onKey} on:resize={onResize} />
<ship-set:document on:visibilitychange={onVisChange} />
```

Modifiers compile to addEventListener wiring with appropriate flags or call wrappers.

### Slots

```html
<!-- in a component's template, declare slots -->
<div class="card">
  <header>
    <slot name="header">
      <span>Default header</span>
    </slot>
  </header>
  <div class="body">
    <slot />                                     <!-- default slot -->
  </div>
</div>

<!-- consumer fills slots -->
<ship-card>
  <span slot="header">My Card</span>
  <p>Body content goes in the default slot</p>
</ship-card>

<!-- slots can pass props down (scoped slots) -->
<!-- in a list-component template: -->
<ul>
  {#for item of items}
    <li>
      <slot name="item" item={item} index={i}>
        <span>{item.label}</span>
      </slot>
    </li>
  {/for}
</ul>

<!-- consumer accesses via let:NAME -->
<my-list items={users}>
  <span slot="item" let:item let:index>
    {index + 1}. <strong>{item.name}</strong>
  </span>
</my-list>
```

Slots in Sail use **light DOM** semantics — the compiler resolves them at render time, slot content lives in the host's actual DOM tree (good for accessibility, devtools, theme cascade).

### Raw HTML

```html
{@html sanitizedMarkdown}
```

The expression's value is set as `innerHTML` of a wrapper. **Author is responsible for sanitization.** The compiler emits a deprecation warning if used without an obvious sanitizer in the expression chain.

### Self-closing tags

Both forms allowed for void elements; required for empty Sail/HTML elements:

```html
<input />                                          <!-- void HTML element -->
<my-component prop="x" />                          <!-- empty Sail component, self-close shorthand -->
<my-component prop="x"></my-component>             <!-- equivalent -->
```

## The `<style>` block

CSS with scoping, typed tokens, and selector rewrites.

### Scoping

By default, every selector is prefixed with `[data-component="<tag>"]`:

```css
/* Author writes */
button { padding: 0.5rem; }

/* Compiler emits */
[data-component="my-button"] button { padding: 0.5rem; }
```

Bypass with `<style global>` for the rare cross-component rule:

```html
<style global>
  /* Affects every page; use sparingly. */
  :root { --my-app-token: red; }
</style>
```

### `:host` rewrite

Light-DOM components don't have a real `:host`, but the syntax is too useful to lose. The compiler rewrites:

```css
:host { display: flex; }
:host([disabled]) { opacity: 0.5; }
:host(.expanded) > .body { display: block; }
```

becomes

```css
[data-component="my-tag"] { display: flex; }
[data-component="my-tag"][disabled] { opacity: 0.5; }
[data-component="my-tag"].expanded > .body { display: block; }
```

### External style files

```html
<style src="./styles.css">
  /* optional inline CSS appended after the external file */
  .local { color: red; }
</style>
```

Path is resolved relative to the `.set` file. External + inline can coexist (external loads first, inline appends). Both go through scoping unless `global`.

### Typed token interpolation

```css
.thread {
  display: flex;
  gap: ${space.3};                                   /* typed token reference */
  background: ${color.surface};
  border: 1px solid ${color.border};
  font-family: ${font.body};
}

.turn[data-role="user"] {
  background: ${color.accent.soft};
}

@media ${breakpoint.md} {
  .thread { gap: ${space.4}; }
}
```

`${name.path}` references the project's design-token tree. The compiler:
- Validates the token name exists
- Inlines as `var(--ship-name-path)` (CSS variable convention)
- Errors at compile time on unknown tokens
- LSP autocompletes the token name

Token sources are configured per-project (Ship's `tokens.css` is the default; OSS users wire their own).

## Type safety

### Prop types

```ts
prop count: number = 0                  // primitive
prop variant: 'primary' | 'ghost'       // string literal union
prop items: Item[] = []                 // generic
prop onSelect?: (id: string) => void    // optional function
```

- Default values are JS expressions (evaluated at compile time)
- `T!` syntax means "required, no default"
- Types flow into the emitted `<tag>.d.ts` declaration file
- Consumer call sites are validated by the LSP

### Reactive declaration types

```ts
state x = 0                             // inferred: number
state y: string[] = []                  // explicit
derived z = x * 2                       // inferred: number
data agents: Agent[] = ship.agents.list()  // type from schema source
```

Inference falls back to the initial value's type when not annotated.

### Template type checking

The LSP and compiler verify:
- `{expr}` references only reachable declarations
- `<my-component foo={value}>` — value type matches `foo`'s declared type
- `<my-component on:select={fn}>` — `fn`'s parameters match the event's payload type
- `bind:value={x}` — `x` is a state declaration (writable), not a derived (readonly)

Errors are file:line:col with the framework's vocabulary.

### Event payload types

```ts
// Declare an event the component emits
event select: { id: string, item: Item }

function onClick(item: Item) {
  emit('select', { id: item.id, item })
}
```

Compiler emits the event-payload type to the `.d.ts`. Consumers using `on:select={fn}` get `fn`'s parameter typed automatically.

## Compile output

For `my-component.set`, the compiler emits:

| File | Purpose |
|---|---|
| `my-component.js` | ES module with the custom-element class. Includes server-rendered HTML, hydration code, registers `customElements.define`. |
| `my-component.html.tmpl` | Server-side rendering function — daemon calls with initial data, returns HTML string. |
| `my-component.d.ts` | TypeScript declaration for the custom element + its prop types + event payloads. |
| `my-component.schema.json` | Runtime-introspectable schema (props, events, slots, dependencies). LSP + devtools consume this. |

These are emitted on the daemon on first request (cached by file mtime + content hash).

## Reserved syntax (not in v1)

The following are reserved for future use; v1 implementations should error on encounter:

- `{#each}` — alias to `{#for}` not supported (pick one syntax)
- `<ship-set:component>` — dynamic component swap (later)
- `<ship-set:fragment>` — fragment wrapper (later if needed)
- `{@const}` — local computed in template (later if patterns demand it)
- Top-level `import` in script (v1: only relative imports for type-only or pure-utility modules; bundling logic deferred)

## Examples

### Minimal component

```html
<!-- counter.set -->
<script lang="ts">
  state count = 0
  function inc() { count += 1 }
</script>

<button on:click={inc}>Clicked {count} times</button>

<style>
  :host { display: inline-block; }
  button { padding: ${space.2} ${space.4}; }
</style>
```

### Server-data example

```html
<!-- agent-list.set -->
<script lang="ts">
  prop workspaceId: string!
  data agents = ship.agents.list({ workspace: workspaceId })
  data live mesh = ship.mesh.snapshot()

  derived liveAgents = agents.filter(a =>
    mesh.actors.some(m => m.id === a.id && m.live)
  )

  function pickAgent(agent: Agent) {
    emit('select', { agent })
  }
</script>

<ul class="list">
  {#for agent of liveAgents key={agent.id}}
    <li class="item" on:click={() => pickAgent(agent)}>
      <ship-avatar src={agent.icon} />
      <span class="name">{agent.id}</span>
      <span class="provider">{agent.provider}</span>
    </li>
  {:empty}
    <p>No live agents in workspace.</p>
  {/for}
</ul>

<style src="./agent-list.css"></style>
```

### Composition example

```html
<!-- agent-modal.set (route file) -->
<script lang="ts">
  state open = false
  state sessionId: string | null = null

  function onPick(e: CustomEvent<{ agent: Agent, provider: string }>) {
    spawnSession(e.detail).then(sid => sessionId = sid)
  }

  function close() { open = false }
</script>

<ship-set:window on:keydown|key:meta+j={() => open = !open} on:keydown|key:Escape={close} />

<div class="modal" hidden={!open}>
  <ship-card>
    <span slot="header">
      {#if sessionId}
        Chat session
      {:else}
        Pick an agent
      {/if}
    </span>

    {#if !sessionId}
      <agent-picker on:select={onPick} />
    {:else}
      <agent-thread sessionId={sessionId} />
      <agent-composer sessionId={sessionId} />
    {/if}
  </ship-card>
</div>

<style>
  .modal { position: fixed; inset: 0; display: flex; align-items: center; justify-content: center; }
  .modal[hidden] { display: none; }
</style>
```

## Compatibility / non-goals

- Not Svelte. The syntax is influenced by it; the compiler and runtime are different.
- Not a JSX framework. Templates are HTML. Authors who want JSX have other choices.
- Not a transpiler ecosystem. We use TypeScript via type-stripping, but advanced TS features (decorators, namespaces, complex generics in template positions) are out of scope for v1.
- Not a build tool. The daemon is the build. No webpack, no vite, no esbuild config from authors.

## Versioning

The language spec is versioned. v1 is documented here. Breaking changes go through deprecation cycles (compiler emits warnings, LSP shows fix-its, migration tool handles automated rewrites).

The wire patch protocol is versioned independently (see [04-patch-protocol.md](04-patch-protocol.md)).
