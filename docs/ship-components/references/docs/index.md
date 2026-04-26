---
group: Components
title: Authoring components
description: When to write a custom .ship component, the happy path, and the canonical example.
order: 1
---

Ship components are custom web elements you write to compose into your views. They're declared as `.ship` single-file components (SFCs) with three optional blocks — JavaScript for declarations + lifecycle, Jinja template body for markup, CSS for scoped styles.

## When to author a component

The agent happy path is **composing existing primitives**. Reach for an `<app-foo>` of your own only when:

1. **A pattern repeats across views** — you find yourself writing the same five primitives in the same arrangement in two or three places.
2. **The arrangement carries non-trivial behavior** — debounced input, keyboard nav, async fetching, focus management — that doesn't belong inline in every consuming view.
3. **You want a typed contract** — props with defaults, signal-backed state, an `emit('selected', payload)` event.

Don't author a component for one-off layouts. A `.html.j2` view template composing primitives is shorter and clearer.

## File location and tag mapping

Components live under `.ship/components/`. The tag name is the kebab-cased filename:

| File | Tag |
|---|---|
| `.ship/components/app-typeahead.ship` | `<app-typeahead>` |
| `.ship/components/agent-card.ship` | `<agent-card>` |
| `.ship/components/inline-editor.ship` | `<inline-editor>` |

The HTML spec requires custom-element tags to contain a hyphen. Single-word filenames (e.g. `card.ship`) won't register.

## Canonical example

The reference component is `.ship/components/app-typeahead.ship`. Read it once before writing your own — it exercises every part of the format:

- Multiple props with defaults
- Five signals
- An `effect()` watching a signal
- Imperative DOM event setup in `onConnect`
- Async fetching
- Slot for custom item rendering
- Datastar attributes for binding, events, conditionals, loops
- A scoped `<style>` block

## The shape of a component

```html
<script>
  // 1. Destructure props (read-only proxy from the wrapper).
  const { placeholder = 'Search...', fetchUrl, minChars = 1 } = props

  // 2. Declare per-instance signals. Names are scoped automatically.
  const $query = signal('')
  const $items = signal([])

  // 3. Optional reactive logic.
  effect($query, () => {
    if ($query.get().length >= minChars) {
      fetch(fetchUrl + '?q=' + encodeURIComponent($query.get()), { credentials: 'same-origin' })
        .then(r => r.json())
        .then(items => $items.set(items))
    }
  })

  // 4. Optional imperative setup. Runs in the host element's connectedCallback.
  onConnect(el => {
    el.querySelector('input').focus()
  })
</script>

<input data-bind:value="$query" placeholder="{{ props.placeholder }}" />
<ul role="listbox">
  <template data-for="item in $items" data-key="item.id">
    <li data-text="item.label"></li>
  </template>
</ul>

<style>
  input { padding: var(--ship-space-2); }
</style>
```

## What happens at request time

1. Browser loads `<script src="/_ship/components/app-typeahead.js" type="module">`.
2. Daemon reads `.ship/components/app-typeahead.ship`, splits the three blocks, parses the JS for prop and signal declarations.
3. Daemon emits a JS module that defines a custom element class wrapping your script + template + style.
4. Browser executes the module, calls `customElements.define('app-typeahead', ...)`.
5. Every `<app-typeahead>` tag in the DOM mounts an instance.
6. Each instance gets its own scoped signal namespace — typing in instance A doesn't affect instance B.

No build step. Edit the file, refresh the browser, the daemon recompiles on the next request.
