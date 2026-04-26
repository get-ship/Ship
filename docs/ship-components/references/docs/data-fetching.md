---
group: Components
title: Data fetching
description: Calling ship.* ops and other endpoints from a component (current manual pattern).
order: 4
---

Components don't have direct access to `ship.*` ops the way views do. Views are server-rendered — `{{ ship.agents.detail(id=$id).name }}` happens server-side at the time the daemon serves the view. Components are browser-side — they need to issue an HTTP request.

The pattern is `fetch()` inside a lifecycle hook or effect. Auth comes for free via cookies if the component is mounted inside a paired view.

## Pattern: fetch on mount

```js
const { agentId } = props
const $agent = signal(null)

onConnect(async () => {
  const r = await fetch('/api/v1/agents/' + agentId, { credentials: 'same-origin' })
  if (r.ok) $agent.set(await r.json())
})
```

`credentials: 'same-origin'` reuses the cookie auth from the parent view. No bearer-token threading needed.

In the template:

```html
<div data-show="$agent">
  <h3 data-text="$agent.name"></h3>
  <p data-text="$agent.description"></p>
</div>
```

## Pattern: fetch on a signal change

When a prop changes effectively (via consumer remounting with a different key), or when an internal signal triggers a refetch:

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

## Why no `data:` declaration shorthand?

The eventual shorthand will let authors declare data dependencies in the script block:

```js
// Future API — not yet implemented
const $agent = data(() => ship.agents.detail({ id: props.agentId }))
```

The framework would handle the fetch + auth + signal-set + retry + cancellation on unmount. Until that lands, the manual pattern above is the contract.

## Testing fetch logic

Component tests at the Rust level cover compilation, not runtime fetches. To test the actual fetch behavior:

1. Spin the daemon (`ship daemon start`).
2. Use any browser or `curl` to mount the component-using view.
3. Inspect Network tab / dev overlay.

End-to-end browser tests live under `tests/` if you need automated regression coverage.
