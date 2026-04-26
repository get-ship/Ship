---
group: Components
title: SFC format
description: The .ship single-file component format — block layout, declarations, scoping rules.
order: 2
---

A `.ship` file has three optional blocks. The Jinja template body is the ambient content; `<script>` and `<style>` are HTML-style block delimiters.

```html
<script>
  /* JavaScript — declarations, lifecycle, optional setup */
</script>

{# Jinja template body — ambient HTML + Datastar attributes + Jinja interpolation #}
<div>...</div>

<style>
  /* CSS — auto-scoped to this component */
</style>
```

Block order is convention (script → template → style), but the daemon doesn't enforce. Only one of each block per file.

## The script block

JavaScript with implicit injectables. The framework provides:

| Injectable | Purpose |
|---|---|
| `props` | Read-only proxy with the component's typed prop values |
| `signal(initial)` | Declares a per-instance signal; returns a handle with `.get()` / `.set(value)` |
| `effect(deps, fn)` | Runs `fn` whenever any of `deps` (signal handles or array of them) changes |
| `emit(name, payload?)` | Dispatches a `CustomEvent('name', { detail: payload, bubbles: true })` from the host |
| `onConnect(el => {})` | Hook in the host's `connectedCallback`; receives the host element |
| `onDisconnect(el => {})` | Hook in `disconnectedCallback` |

### Props

Authors destructure from `props`:

```js
const { count = 0, label = 'Counter' } = props
```

Defaults are JS expressions. Names become observed attributes on the custom element with kebab-case conversion:

| Prop | Attribute | Example use |
|---|---|---|
| `count` | `count` | `<my-counter count="5">` |
| `fetchUrl` | `fetch-url` | `<my-counter fetch-url="/api/...">` |
| `minChars` | `min-chars` | `<my-counter min-chars="2">` |

### Signals

```js
const $count = signal(0)
const $items = signal([])
```

The framework auto-scopes names per-instance. Author writes `$count` everywhere; under the hood the storage key is `_<safe_tag>_<instance_id>_count` so multiple instances of the same component don't collide. Tag hyphens are converted to underscores in the namespace (Datastar parses attribute values as JS expressions, where `-` would mean subtraction).

Signal handles expose `.get()` and `.set(value)`:

```js
const $count = signal(0)
$count.get()        // current value
$count.set(42)      // update
```

In Datastar attribute values, reference signals with `$<name>`:

```html
<button data-on:click="$count++" data-text="$count"></button>
```

### Effects

```js
effect($query, async () => {
  const r = await fetch('/api/v1/agents/search?q=' + encodeURIComponent($query.get()))
  $items.set(await r.json())
})
```

Pass a single signal or an array — `effect([$a, $b], () => ...)` runs when either changes.

### Lifecycle

```js
onConnect(el => {
  el.querySelector('input').addEventListener('keydown', handleKey)
})

onDisconnect(el => {
  // listeners attached to children inside the component go away with the DOM;
  // listeners attached to document/window MUST be cleaned up here
})
```

## The template body

Jinja, exactly as in views:

- **Static interpolation**: `{{ props.label }}` — rendered once at mount, server-side.
- **Control flow**: `{% if %}`, `{% for %}`.
- **SDK macros**: `card_grid(...)`, `indicator(...)` — globally available, no import needed.
- **Datastar attributes** for everything reactive:
  - `data-text="$signal"` — bind text content
  - `data-show="$signal"` — toggle visibility
  - `data-bind:value="$signal"` — two-way input binding
  - `data-on:click="$signal++"` — event handler
  - `data-on:input="$signal = el.value"` — event handler with element access
  - `data-attr:disabled="!$signal"` — bind any attribute
  - `data-class:active="$signal"` — toggle a class
  - `data-for="(item, i) in $list"` `data-key="item.id"` — list rendering on a `<template>` element

Reactive interpolation differs from JSX/Svelte:

- Use `{{ ... }}` for **static** values rendered once at mount (props, computed-once values).
- Use Datastar attributes for **reactive** values that change at runtime.

`{{ $count }}` does NOT update when `$count` changes — that's a static Jinja expression. Use `<span data-text="$count"></span>` instead.

### Slots

Native `<slot>` element, no Ship layer:

```html
<li>
  <slot name="item" item="item">
    <span data-text="item.label"></span>
  </slot>
</li>
```

The fallback content (`<span data-text="item.label">`) renders if the consumer doesn't provide slot content.

## The style block

CSS auto-scoped at load time. Every selector gets prepended with `[data-component="<tag>"]`:

```css
/* Author writes */
button { padding: var(--ship-space-3); }

/* Daemon emits */
[data-component="my-counter"] button { padding: var(--ship-space-3); }
```

Bypass scoping with the `global` attribute for the rare cross-component case:

```html
<style global>
  /* Affects every page; use sparingly. */
  :root { --my-special-token: red; }
</style>
```

Use `var(--ship-*)` tokens for theme adaptability — bare hex colors won't follow the user's theme choice.

## File-naming rules

- Lowercase kebab-case filename.
- Must contain a hyphen (HTML spec requirement for custom elements).
- The `.ship` extension distinguishes it from view templates (`.html.j2`).

Examples:
- ✅ `app-typeahead.ship` → `<app-typeahead>`
- ✅ `agent-card.ship` → `<agent-card>`
- ❌ `card.ship` → no hyphen, won't register
- ❌ `AgentCard.ship` → uppercase rejected

## What it compiles to

The daemon emits a self-contained ES module with:

- Your script block, embedded verbatim inside an IIFE.
- The Jinja-rendered template as a string constant, with signal references rewritten to scoped placeholders.
- The CSS, scoped, injected once into `<head>`.
- A `class ShipComponent extends HTMLElement` wrapper that:
  - Decodes attributes per the prop schema.
  - Initializes signals via Datastar's signal store at the per-instance namespace.
  - Sets `innerHTML` on `connectedCallback`.
  - Calls registered `onConnect` / `onDisconnect` hooks.
- A `customElements.define(TAG, ShipComponent)` registration.

Inspect the output directly in dev:

```bash
curl http://127.0.0.1:9315/_ship/components/app-typeahead.js
```
