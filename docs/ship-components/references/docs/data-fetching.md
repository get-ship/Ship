---
group: Components
title: Data fetching
description: Calling ship.* ops and other endpoints from a component.
order: 4
---

Components don't have direct access to `ship.*` ops the way views do. Views are server-rendered — `{{ ship.agents.detail(id=$id).name }}` happens server-side at the time the daemon serves the view. Components are browser-side — they need to issue an HTTP request.

`data(fn)` is the canonical mount-time fetch primitive. For dependency-aware refetch, drop down to `effect()` + `signal()`.

## Pattern: `data()` for one-shot mount-time fetch

```js
const { agentId } = props
const $agent = data(({ signal }) =>
  fetch('/api/v1/agents/' + agentId, { signal, credentials: 'same-origin' })
    .then(r => r.json())
)
```

The signal initializes to `null`, then resolves to the fetch result.

- `signal` is an `AbortSignal` wired to `disconnectedCallback` — pass it to `fetch` so in-flight requests cancel cleanly when the component unmounts.
- `credentials: 'same-origin'` reuses the cookie auth from the parent view. No bearer-token threading needed.
- Errors are logged AND dispatched as a `data-error` CustomEvent from the host (composed: true, bubbles: true) — consume in the parent view with `data-on:data-error`.

In the template:

```html
<div data-show="$agent">
  <h3 data-text="$agent.name"></h3>
  <p data-text="$agent.description"></p>
</div>
<div data-show="!$agent" class="loading">Loading…</div>
```

In the template:

```html
<div data-show="$agent">
  <h3 data-text="$agent.name"></h3>
  <p data-text="$agent.description"></p>
</div>
```

`data()` is the simplest case (one fetch at mount). For everything else, use `effect()` directly — see below.

## Pattern: refetch on a signal change

When an internal signal triggers a refetch (typeahead query, filter change, paginated list cursor):

```js
const $query = signal('')
const $items = signal([])

let timer
effect($query, () => {
  clearTimeout(timer)
  timer = setTimeout(async () => {
    const r = await fetch('/api/v1/agents/search?q=' + encodeURIComponent($query.get()), {
      credentials: 'same-origin',
    })
    $items.set(await r.json())
  }, 200)
})
```

Debounce inside the effect — Datastar fires on every signal change.

## Pattern: optimistic update

Set the signal first, then issue the request. Roll back on failure:

```js
async function toggleArchive(item) {
  const previous = $items.get()
  $items.set(previous.map(i => i.id === item.id ? { ...i, archived: !i.archived } : i))

  const r = await fetch('/api/v1/items/' + item.id + '/archive', {
    method: 'POST',
    credentials: 'same-origin',
  })

  if (!r.ok) {
    $items.set(previous)  // roll back
    emit('error', { message: 'archive failed' })
  }
}

onConnect(el => {
  el.addEventListener('archive-clicked', e => toggleArchive(e.detail))
})
```

## Auth caveats

- **Cookie auth**: works automatically with `credentials: 'same-origin'`, IF the component is mounted in a view that the user already authed into (hit `/views/...` and got a session cookie).
- **No cookie**: 401. Components mounted from a non-authed page (a public landing page, an embedded iframe on another origin) need explicit token handling. Not the common case for Ship apps; defer until you hit it.
- **Cross-origin**: don't. Ship's daemon is local-only by default; components fetching from a different origin run into CORS and break the cookie story. Use the local daemon as the proxy.

## Testing fetch logic

Component tests at the Rust level cover compilation, not runtime fetches. To test the actual fetch behavior:

1. Spin the daemon (`ship daemon start`).
2. Use any browser or `curl` to mount the component-using view.
3. Inspect Network tab / dev overlay.

End-to-end browser tests live under `tests/` if you need automated regression coverage.
