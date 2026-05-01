---
group: Ship UI
order: 5
title: Data subscriptions
description: `<data>` is the canonical bridge to the daemon. Live envelope shape, signal conventions, why hooks don't subscribe.
---

# Data subscriptions

The Ship daemon (`shipd`) is the model. Views bridge to it via `<data>` elements declared in the template. This is the **only** sanctioned way to bring server data into a view. It is not interchangeable with hooks.

## The element

```html
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />
<data name="agent" source="ship.agents.get({id: route.params.id()})" live />
```

Attributes:

| Attr | Required | Purpose |
|---|---|---|
| `name="..."` | yes | Identifier exposed in view scope. |
| `source="..."` | yes | A `ship.*` SDK call expression. Evaluated with `ship` and `route` in scope. |
| `live` | no | If present, the runtime opens an SSE channel and patches the signal as the daemon emits updates. |

The runtime exposes `<name>` as a signal in [view scope](index.md#the-shared-view-scope). Read it via `name()` in expressions and controller sections.

## Live envelope

`live` signals always wrap the daemon's payload in a uniform envelope. The shape is contractual; hooks rely on it.

```ts
// from @SUI/runtime — `live.ts`
type LiveState<T> =
  | { kind: 'loading' }
  | { kind: 'error'; error: Error }
  | { kind: 'ok'; value: T }

type LiveSignal<T> = Signal<LiveState<T>>
```

Three variants. `loading` carries no payload; `error` carries an `Error`; `ok` carries the resolved value. The `value` field lives **only** on the `ok` variant — discriminate by `kind` before reading it.

Read it directly in templates:

```html
{#if agents().kind === 'ok'}
  <p>{agents().value.items.length} agents</p>
{:else if agents().kind === 'error'}
  <p>Failed: {agents().error.message}</p>
{:else}
  <p>loading…</p>      {/* kind === 'loading' */}
{/if}
```

Or use the convenience boundary directives:

```html
<while-loading bind="agents">
  <p>loading…</p>
</while-loading>

<while-error bind="agents" let:err>
  <p>Failed: {err.message}</p>
</while-error>

{#if agents().kind === 'ok'}
  <!-- happy path -->
{/if}
```

`<while-loading>` and `<while-error>` are syntactic sugar over the kind check. They render their children only while the bound signal is in the matching state and clean up automatically.

## Static `<data>` (no `live`)

Without `live`, the SDK call runs once at mount; the result is exposed as a plain signal containing the resolved value (no envelope). Used for one-shot reads where streaming updates aren't relevant — config, schemas, lookups.

```html
<data name="schema" source="ship.schema.read('agents')" />

<p>Defined fields: {schema().fields.map(f => f.name).join(', ')}</p>
```

If the call is async, the runtime exposes the signal once it resolves; pre-resolution reads return `undefined`. Use `{#await}` if you need to render a loading state.

## Why subscriptions don't go in hooks

Subscriptions are declarations, not pattern slices. The author should always see, in the template, every server channel a view depends on. Hiding a subscription inside `useAgents()` (or any hook) loses that visibility — readers can't tell what data the view needs without opening the hook.

Concretely:

```html
<!-- right: every subscription visible at the top of the file -->
<data name="agents"   source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />

<!-- wrong: subscriptions hidden behind a hook -->
<!-- (don't author this) -->
```

```ts
// wrong: subscriptions in a hook
export const useAgents = () => ({
  live: { agents: ship.agents.summaries() },
})
```

Repetition across views is the right trade. If three views all subscribe to `ship.agents.summaries()`, all three have the `<data>` element. The pattern *of subscribing to agents* is not abstracted; the visibility is preserved.

The hook layer is for **bundling state + derived + handlers**. A hook that *uses* an `agents` signal (filtering, selecting, paginating, sorting) is doing real work and earns its existence. A hook that just wraps a subscription is double-dipping.

## Passing data signals to hooks

When a controller hook needs a data signal, **pass the signal itself**, not a closure.

```ts
// in a controller (function form)
export default ({ ship }) => ({
  use: {
    list: useFilteredList(agents, (a, q) => a.name.includes(q)),
    sel:  useSelection(agents, a => a.id),
  },
})
```

The hook destructures the envelope internally:

```ts
// inside useFilteredList
const items = () => {
  const s = source()                              // source: LiveSignal<{ items: T[] }>
  return s.kind === 'ok' ? s.value.items : []
}
```

This is why the envelope is contractual — hooks rely on the shape to unwrap consistently.

## Reactive `route` in source thunks

`route.params.<name>` and `route.query.<name>` are themselves signals. Reading one inside a `<data live source="…">` thunk subscribes the live channel to the param: when navigation changes the URL, the prior SSE subscription disposes and a new one opens with the updated arg. No author-side wiring; this is the same `live()` tracking that re-fires on `<state>` changes.

Concrete pattern — a `/agents/<id>` view:

```ts
// view.ts
import { defineRoutes, controller } from '@SUI/runtime'

defineRoutes([{ pattern: '/agents/:id' }])

export default controller({
  derived: {
    name: ({ agent }) => agent().kind === 'ok' ? agent().value.name : '',
  },
})
```

```html
<!-- view.sui -->
<data name="agent" source="ship.agents.get({id: route.params.id()})" live />

<h1>{name()}</h1>
{#if agent().kind === 'loading'}<p>Loading…</p>{/if}
```

Navigation flow:

1. The user lands on `/agents/42`. `route()` (called by the boot script) seeds `route.params.id` from the pathname-to-pattern match: `'42'`.
2. `<data live>` evaluates `ship.agents.get({id: route.params.id()})`. `route.params.id()` is a signal read; `live()` records it as a dep. The SSE channel for agent 42 opens.
3. The user clicks a link to `/agents/99`. The view's click handler (or the framework's link interceptor) calls `navigate('/agents/99')`. That pushes history and triggers the global `__shipSetRouteUpdate`, which sets `route.params.id` to `'99'`.
4. The `live()` dep fires. The agent-42 SSE channel disposes; a new one opens for agent 99. The signal flips through `loading → ok` exactly as on first mount.

`defineRoutes` is per-view. Without a registered shape that captures `id`, `route.params.id()` returns `undefined`; the source thunk still tracks the signal, so the channel re-binds the moment a future navigation introduces a matching segment. (A compiler-emitted `<route pattern="…" />` directive will make the imperative `defineRoutes` call optional in a follow-up.)

## SDK access

The `ship.*` SDK is the typed bridge to `shipd`. It is *not* a global; it's injected by the runtime into:

- The `source=` attribute of `<data>` elements (where `ship` and `route` are in scope)
- The function form of a controller (`({ ship }) => ...`)
- Each controller section's destructure (`{ ship }` is part of the scope object)
- Each controller hook's factory (when it accepts scope)

Authors do not import `ship`. The runtime owns the binding.
