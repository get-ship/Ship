---
group: Set Framework
order: 2
title: The template surface
description: Everything you can write in a `.html.set` file — directives, control flow, bindings, hooks, slots, expressions.
---

# The template surface

A `.html.set` file is HTML with a small set of directive elements at the top, mustache control flow inside, and a few binding shorthands on attributes. There is no script block and no style block. Logic lives in a sibling `.ts` (paired by filename); styles live in a sibling `.css` linked by `<link rel="stylesheet">`.

## Directive elements

These appear at the top of the file (before the rendered DOM). They declare reactivity but do not render to HTML.

### `<state>` — local signal

```html
<state name="count" value="0" />
<state name="open" value="{false}" />
```

Declares a mutable signal in template scope. **`<state>` is the only state primitive** — controllers do not have a `state:` section (see the [declarative-state ADR](../../../ship-adrs/references/docs/ship-set-declarative-state.md)). Templates always declare; controllers always read and mutate via `on.*` handlers and lifecycle hooks.

Computed-initial values (`crypto.randomUUID()`, an SDK call, a derivation over `route`) declare a placeholder here and assign in `mount`:

```html
<state name="id" value="''" />
```

```ts
mount: ({ state }) => {
  state.id = crypto.randomUUID()
}
```

### `<data>` — daemon subscription

```html
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />

<!-- Reactive args via $scope: re-subscribes when selectedId changes. -->
<state name="selectedId" value="'abc'" />
<data name="agent" source="ship.agents.get({id: $scope.selectedId()})" live />
```

Subscribes to a `ship.*` SDK call. With `live`, the runtime opens an SSE channel and patches the signal as the daemon emits updates. Without `live`, the value is read once at mount.

The `source` expression is plain JS that runs inside `live()`'s tracked reactive scope. It receives three arguments: `ship` (the SDK), `route` (a [`RouteSignals`](data-subscriptions.md#reactive-route-in-source-thunks) — `route.params.<name>` and `route.query.<name>` are themselves signals), and `$scope` (every identifier already in view scope — state signals, controller bindings, earlier `<data>` declarations). Reading a signal via `$scope.<key>()` or `route.params.<key>()` is tracked; when the signal changes, the prior subscription disposes and a new one opens with the updated args.

`<data>` is the **canonical** way to bring server data into a view. See [data-subscriptions.md](data-subscriptions.md).

### `<component>` and `<prop>` — component declaration

```html
<component name="ship-button">
  <prop name="tone" values="default|primary" default="default" reflect />
  <prop name="disabled" type="bool" default="false" reflect />
  <prop name="label" type="string" default="" />
</component>

<button disabled={disabled() || loading()}>{label()}</button>
```

Used in component-shaped `.html.set` files (the file *defines* a custom element). The `<component>` block names the tag and lists props; everything after is the template that will render inside the element. Views (top-level page surfaces) do not have `<component>` blocks.

Prop kinds: `string` (default), `bool`, `number`, `values="a|b|c"` (union). `reflect` mirrors the prop to the host element's attributes.

## Control flow

Mustache-style, directly in the template body:

### `{#if}` / `{:else if}` / `{:else}` / `{/if}`

```html
{#if loading()}
  <ship-spinner />
{:else if error()}
  <p>Failed: {error().message}</p>
{:else}
  <p>{count()} items</p>
{/if}
```

### `{#for}` / `{/for}` (with optional `key`)

```html
{#for a of agents().value.items key={a.id}}
  <ship-card>{a.name}</ship-card>
{/for}
```

`key={...}` provides stable identity across re-orders and updates. Without `key`, items are re-created on any list change.

### `{#match}` / `{:case ...}` / `{:default}` / `{/match}`

```html
{#match status()}
  {:case 'idle'}    <p>Ready</p>
  {:case 'loading'} <ship-spinner />
  {:case 'error'}   <p>Failed</p>
  {:default}        <p>Unknown</p>
{/match}
```

### `{#await}` / `{:then x}` / `{:catch err}` / `{/await}`

```html
{#await fetchAgents()}
  <p>loading…</p>
{:then list}
  {#for a of list}<p>{a.name}</p>{/for}
{:catch err}
  <p>Error: {err.message}</p>
{/await}
```

## Expressions

Any `{ ... }` opens an expression. Resolved against the [shared view scope](index.md#the-shared-view-scope).

```html
<p>Hello, {user().name}.</p>
<p>{count() === 1 ? '1 item' : count() + ' items'}</p>
<button data-disabled={disabled() || loading()}>...</button>
```

Signals are read by call: `count()`. The framework does not auto-call. Read explicitly, write via `count.set(v)`.

### The renderable subset (for SSR)

Server-side rendering evaluates expressions against initial signal values. Stay in the renderable subset to get free SSR; step outside it and the expression becomes client-only with a placeholder on the server.

**Renderable**: signal calls (`count()`), property access (`user().name`), arithmetic (`count() + 1`), comparison (`x() > 5`), ternaries (`a ? b : c`), template literals with no embedded function calls.

**Not renderable** (client-only): function invocations beyond `signal()` reads, closures, `new ...`, async, `this`, arbitrary JS.

The compiler flags non-renderable expressions with a warning. See [ssr-hydration.md](ssr-hydration.md).

## Bindings

### `on:` — event handlers

```html
<button on:click={() => count.set(count() + 1)}>+</button>
<input on:input={(e) => setQuery(e.target.value)} />
<form on:submit={submit}>...</form>
```

The handler value is an arrow function whose body should be a single call expression. The runtime invokes it with the DOM event. Modifiers will be added later (`on:click:once`, `on:keydown:escape`); current source supports the bare form.

### `bind:` — two-way binding

```html
<input bind:value={query} />
<input type="checkbox" bind:checked={enabled} />
```

The runtime wires both directions: signal → DOM and DOM → signal. Do not also write `on:input={...}` — `bind:` owns the listener. If you need a side effect on change, write a `derived` over the bound signal or an `effect` (when effects ship).

### Attribute interpolation

```html
<div class="card card-{tone()}" data-state={state()}>...</div>
<button disabled={isDisabled()} aria-pressed={pressed()}>...</button>
```

Empty / `null` / `undefined` / `false` auto-omit the attribute. `true` is presence-only.

## Template hooks (`<use-*>`)

Element-shaped registrations for host-side side effects and platform context. Live in the template alongside the structure they augment. See [hooks.md](hooks.md) for the full contract.

```html
<use-shell as="shell" />
<use-kbd>
  <on key="/" action="search" />
  <on key="cmd+n" action="newAgent" />
</use-kbd>
<use-title>{agents().value?.items.length ?? 0} agents</use-title>
```

Authoring rule: hook **attributes** are bindings (`as="X"`, `bind:foo={ref}`), never expressions. Hook **children** are template fragments and may contain expressions, control flow, slots.

## Slots

Standard web-component slot semantics resolved in light DOM at mount time:

```html
<!-- in ship-button.html.set -->
<button>
  <span class="content">
    <slot name="leading"></slot>
    <slot></slot>
    <slot name="trailing"></slot>
  </span>
</button>

<!-- in a consumer -->
<ship-button>
  <ship-icon slot="leading" name="plus" />
  New agent
</ship-button>
```

Named slots, default slot, fallback content, scoped slots (`<slot name="x" item={x}>` + consumer `let:item`) all supported.

## What's NOT in the template

For clarity — these go in the controller, never the template:

- Imperative logic (`async`, `await`, `try/catch` outside `{#await}`)
- Coordinated state (multiple signals that interact)
- Derived values used in more than one place
- Effects that do something other than render
- Anything that calls `ship.*` outside a `<data>` declaration

If you find yourself wanting a multi-line script block in the template, you want a controller. Add a paired `.ts`.
