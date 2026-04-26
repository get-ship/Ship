---
group: Components
title: Lifecycle and reactivity
description: Props, signals, effects, lifecycle hooks, and events — how components react and communicate.
order: 3
---

A `.ship` component is a custom web element. The framework wraps your script + template + style into a class extending `HTMLElement`. This page covers what runs when, and how state flows in and out.

## Mount sequence

When `<my-component>` appears in the DOM:

1. **`constructor()`** — instance counter increments, signal handles initialized empty.
2. **`connectedCallback()`** runs. In order:
   1. Style injected into `<head>` (once per page, regardless of instance count).
   2. `data-component="<tag>"` attribute set on the host (used by scoped CSS).
   3. Script block executes: prop destructuring resolves, `signal()` calls register handles, `effect()` and `onConnect()` register their callbacks.
   4. Signals get initialized in Datastar's signal store under the per-instance namespace.
   5. Template HTML rendered into `innerHTML`. Datastar attribute values reference the scoped signal names.
   6. Registered `onConnect` callbacks run, in declaration order, each receiving the host element.

When the element is removed:

1. **`disconnectedCallback()`** runs.
2. Registered `onDisconnect` callbacks run, in declaration order.
3. The Datastar signal-patch listener is removed.
4. Datastar cleans up its observers on the (now-removed) DOM.

## Attribute changes

If the host element's observed attribute changes after mount (e.g. via `el.setAttribute('count', '7')`), `attributeChangedCallback` runs and the new value flows through the prop decoder. **The script does NOT re-execute** — props are read at `connectedCallback` time only. To react to prop changes, the consuming view should re-mount the component (typically by changing a `data-key` in a `data-for` loop).

This is a deliberate constraint. Components are meant to be largely stateless or signal-backed; if you need props that update reactively after mount, model them as signals instead.

## Signals

Signals are the only reactivity primitive in components. They live in Datastar's signal store under per-instance keys (`_<safe_tag>_<instance_id>_<name>`).

```js
const $count = signal(0)

$count.get()           // read current value
$count.set(42)         // update
$count.set($count.get() + 1)  // increment via .set
```

In Datastar attribute expressions, signals are referenced with `$<name>`:

```html
<button data-on:click="$count++" data-text="$count"></button>
```

Signals are JSON-serializable values: numbers, strings, booleans, arrays, objects. Nested objects work but the framework does not deep-watch — replacing the whole signal value is what triggers updates. Mutating a nested property (e.g. `$items.get().push(x)`) does not.

For arrays, set the new array:

```js
$items.set([...$items.get(), newItem])
```

For objects, the same:

```js
$user.set({ ...$user.get(), name: newName })
```

## Effects

Run a function whenever a signal changes:

```js
effect($query, () => {
  console.log('query changed to:', $query.get())
})
```

Multiple dependencies — pass an array:

```js
effect([$query, $filter], () => {
  // runs when either $query or $filter changes
})
```

Effects fire on every change, including the first set after mount. The current value is whatever `.get()` returns at the time of the call. There is no equivalent of React's "skip first run" — if you need that, gate inside the function.

Async effects work — but if the signal changes again before the previous async work finishes, you get out-of-order resolutions. Use a request id or a debounce timer:

```js
let timer
effect($query, () => {
  clearTimeout(timer)
  timer = setTimeout(async () => {
    const r = await fetch('/api/v1/agents/search?q=' + encodeURIComponent($query.get()))
    $items.set(await r.json())
  }, 200)
})
```

## Lifecycle hooks

`onConnect(el => {...})` runs in `connectedCallback`. The element parameter is the host (`<my-component>`), not the template root.

```js
onConnect(el => {
  // Querying inside the component:
  const input = el.querySelector('input')
  input.focus()
  input.addEventListener('keydown', handleKey)
})
```

`onDisconnect(el => {...})` runs in `disconnectedCallback`. Use for:

- Cleaning up listeners attached to `document` or `window` (not children — those die with the DOM).
- Cancelling pending `setTimeout` / `setInterval` / `AbortController`.
- Unsubscribing from external stores or WebSockets.

Both hooks can be registered multiple times. They run in declaration order.

## Emitting events

Components communicate up to their parent via `CustomEvent`s. Use `emit`:

```js
function selectItem(item) {
  emit('select', item)
}
```

Equivalent to:

```js
host.dispatchEvent(new CustomEvent('select', { detail: item, bubbles: true }))
```

Consume in the parent view:

```html
<app-typeahead data-on:select="$selected = evt.detail" />
```

The Datastar `data-on:<event>` attribute makes `evt` available in the expression.

## Props vs signals — when to use which

| Concern | Use prop | Use signal |
|---|---|---|
| Configuration set at mount, doesn't change | ✅ | |
| State that changes during the component's lifetime | | ✅ |
| Value that the consuming view computes once | ✅ | |
| Value the component itself computes from input | | ✅ |
| Static interpolation in template `{{ }}` | ✅ | |
| Reactive interpolation in `data-*` attributes | | ✅ |

Rule of thumb: props flow IN at mount, signals carry STATE during life, events emit OUT.
