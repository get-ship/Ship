---
title: "Reactivity"
description: "The Datastar subset Ship endorses, in Ship's words. Signals, expressions, attribute bindings, events, actions, SSE patches, and error surfacing."
sidebar:
  label: "Reactivity"
  order: 3
---
Ship's reactivity model is Datastar, but you almost never touch
Datastar directly — the SDK macros wrap the 90% pattern. This page
explains the underlying model so you can read the emitted attributes
and debug them, then points at the macro you should have reached for.

## Signals

Signals are reactive state. Three kinds:

- **Local** — name starts with `_`. Never leaves the browser. For UI
  state like `_showForm`, `_creating`, `_selectedRow`.
- **Server-visible** — any other name. Submitted with form actions and
  readable by the daemon when actions fire.
- **SSE-synced** — a signal the server patches via
  `datastar-patch-signals`. Any view-visible region that the daemon
  writes into.

Declare signals with `{{ signal(name, default) }}` or, for three or
more, `{{ signals({...}) }}`. The object form emits
`data-ship-signal-types` so AST-lint can flag later type mismatches
(`$count = 'hello'` against an `int`-typed signal).

A signal with default `{}` is an object; dotted access (`$form.title`)
works in expressions and in `bind('form.title')`.

**Macro:** `signal()`, `signals()`.

## Expressions

An expression is a small JavaScript snippet Datastar evaluates. `$foo`
is sugar for "the current value of signal `foo`." Assignments write
back. String signals render with single quotes around their value —
`$collection` where `collection='task'` evaluates to `'task'`.

```html
data-text="$count"
data-on:click="$count = $count + 1"
data-show="$tab == 'details'"
```

Multiple statements separate with `;`. Pitfall: forward slashes in
Jinja string interpolations are HTML-escaped to `&#x2F;` inside attribute
values; a `data-show="$branch == '{{ name | e }}'"` comparing against a
branch like `feat/x` will never match. Slugify or index-key around it.

**Macros that emit expressions:** `show()`, `toggle_class()`, `attr()`.

## Attribute bindings

Datastar's read-side bindings:

| Attribute | Purpose | Macro |
|---|---|---|
| `data-text` | Write expression value into element text | (direct) |
| `data-class:X` | Toggle class `X` when expr truthy | `toggle_class()` |
| `data-attr:X` | Reactively set attribute `X` | `attr()` |
| `data-show` | Show/hide (without flash) | `show()` |
| `data-bind` | Two-way-bind form input to signal | `bind()` |
| `data-indicator` | Write `true` while fetch in flight | `loading()` |

Direct `data-text="$foo"` is fine — there's no macro for "write text
into this element," and writing the macro would obscure the intent.
Everything else has a macro and you should use it.

## Events

`data-on:<event>="expr"` runs `expr` when the DOM or custom event
fires. `evt` is in scope. Datastar supports modifiers:

- `__debounce.300ms` — fires at most once per trailing window.
- `__throttle.500ms` — leading-edge rate limit.
- `__prevent` — `evt.preventDefault()`.
- `__stop` — `evt.stopPropagation()`.

Always prefer the macro. `on('ship-change', …)` for a ship-element
custom event; `search(signal, view)` for the debounced input pattern;
`on_change_debounced(expr)` for "auto-save on pause."

## Actions (`@verb`)

Datastar exposes four fetch verbs inside expressions:

| Verb | Shape | Ship macro |
|---|---|---|
| `@get(url)` | read / SSE | `get()`, `navigate()`, `live()` |
| `@post(url, {payload: …})` | write | `post()`, `create()`, `create_job()`, `emit()`, `feedback()` |
| `@put(url, {payload: …})` | update | `put()`, `update()`, `put_on_change()` |
| `@delete(url)` | delete | `delete()`, `confirm_delete()` |

The `{payload: {...}}` wrapper is Datastar's contract — the payload is
the request body. The Ship macros always emit this shape; you should
not have to write it yourself.

**Macros to default to:** `create()` / `update()` / `delete()` for
collections, `create_job()` / `emit()` for runtime actions, `post()` /
`put()` / `delete()` for arbitrary endpoints.

## SSE patterns

Ship runs on Datastar's SSE event framing. The two frame types you'll
see in the network tab:

- `datastar-patch-elements` — replaces a region of DOM by selector.
  The daemon emits these from `partials/content.html.j2` renders
  keyed on the view's manifest `events:` list.
- `datastar-patch-signals` — patches specific signals without touching
  DOM. Less common in first-party views; used for cross-region
  coordination.

Subscribe once per view with `{{ live('<name>') }}`. The CQRS pattern
is strict: reads flow over the SSE stream, writes are one-shot
`@post`/`@put`/`@delete`. The write doesn't render anything — the read
stream pushes the visual update after the kernel event lands.

`data-init="expr"` runs once on element connect. Useful for
one-time signal setup that can't be expressed as a default; prefer
`signal()` defaults when possible.

## Errors

When a fetch action on an element fails (HTTP >= 400 or a non-SSE
body), Datastar emits a `datastar-fetch` document event with
`{type: 'error', el, argsRaw: {status}}`. Two ways to surface it:

- **Per-element:** `{{ on_fetch_error('signal') }}` writes a
  signal-scoped error message. Pair with a `<ship-alert>`.
- **Global debugging:** the shell's error reporter posts every
  uncaught error, view render failure, and fetch failure to
  `POST /api/runtime/view-errors`. Agents retrieve these via the
  `ship_view_errors` MCP tool (see the Authoring loop page).

The daemon also validates each render. A missing signal, a malformed
macro call, or a template syntax error renders a structured error
payload on the response and posts an entry to `view-errors` so the
next `ship_view_errors` call surfaces it.
