---
title: "Manifest & SDK Reference"
description: "manifest.json fields, the _ship.html.j2 macro surface, signal/bind/event conventions, view context variables."
sidebar:
  label: "Manifest & SDK Reference"
  order: 2
---
A view is `manifest.json` + `index.html.j2` plus optional assets. This page lists every field, macro, and convention.

## manifest.json

Required fields in every view manifest:

| Field | Type | Purpose |
|---|---|---|
| `name` | string | Must match the directory name. |
| `type` | string | `"html"` for templates. Other types reserved for future (`"json"`, `"stream"`). |
| `description` | string | Human-readable purpose. Shown in Studio's view listings. |

Optional fields:

| Field | Type | Purpose |
|---|---|---|
| `data` | object | Data sources the view needs. Today: `{ "collections": [id, ...] }` declares which collections the template can read. |
| `events` | string[] | Event types the view subscribes to. Matching events arrive over SSE and trigger template re-render. Examples: `"collection.item_created"`, `"workspace.started"`. |
| `title` | string | Page title. Defaults to `name`. |
| `icon` | string | Optional icon hint for Studio. |

Example — the overview canvas view:

```json
{
  "name": "overview",
  "type": "html",
  "description": "Canvas-based planning surface for the Ship systems collection.",
  "data": {
    "collections": ["systems"]
  },
  "events": [
    "collection.item_created",
    "collection.item_updated",
    "collection.item_deleted"
  ]
}
```

The official JSON Schema for `manifest.json` is at [`schemas/view-manifest.schema.json`](https://raw.githubusercontent.com/madvib/ship/main/schemas/view-manifest.schema.json). Add a `$schema` key to get editor autocompletion and inline validation:

```json
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/view-manifest.schema.json",
  "name": "my-view",
  ...
}
```

## The SDK — `_ship.html.j2`

Every view starts with:

```jinja
{% from '_ship.html.j2' import head, signal, show, live, loading, spinner,
   create, update, delete, form, table, styles,
   get, post, put, navigate, on, bind, attr,
   tab, tab_panel, chip, chip_all, search, radio, confirm_delete %}
```

Import what you use. The macros are stable; the SDK module evolves additively — old macros never break.

### Page scaffolding

| Macro | Purpose |
|---|---|
| `head(title)` | Emits `<meta>` tags, loads fonts, imports ship-elements bundle, loads Datastar. Call once inside `<head>`. |
| `styles(scoped)` | Optional scoped styles. Rare — prefer ship-elements for consistency. |

### Signals and binding

Signals are the reactive state. They live in Datastar and drive DOM updates.

| Macro | Purpose |
|---|---|
| `signal(name, initial)` | Declare a signal. Emits a Datastar `data-signals` attribute. Put on a root element. |
| `bind(name)` | Two-way bind an input element to a signal. Emits `data-bind`. |
| `show(expr)` | Conditionally render. Truthy expr = show, falsy = hide. |
| `live(topic, template)` | Re-render a fragment when events on `topic` arrive over SSE. |

Signal access in markup uses `$name`:

```html
<div {{ signal('count', 0) }}>
  Count: <span data-text="$count"></span>
  <ship-button data-on-click="count++">+</ship-button>
</div>
```

### Events and actions

| Macro | Purpose |
|---|---|
| `on(event, action)` | Shorthand for `data-on-{event}` Datastar attribute. |
| `attr(name, expr)` | Compute an attribute value from signals. |
| `get(url, signal)` | Fire a GET on event; optionally store response in a signal. |
| `post(url, body_signal, response_signal)` | Same for POST. |
| `put(url, body_signal, response_signal)` | Same for PUT. |
| `delete(url)` | Same for DELETE. |
| `navigate(url)` | Client-side navigation helper. |

### Feedback

| Macro | Purpose |
|---|---|
| `loading(signal)` | Show a spinner when `signal` is truthy. |
| `spinner()` | Just the spinner, no wrapper. |

### Higher-level composites

| Macro | Purpose |
|---|---|
| `form(action, fields, submit_label)` | Render a full form — handles submission, loading state, error display. |
| `table(rows, columns, actions)` | Render a data table with optional row actions. |
| `tab(label, target)` / `tab_panel(id, content)` | Tabbed interfaces. |
| `chip(label, tone)` / `chip_all(items)` | Tag-chip rendering. |
| `search(signal, placeholder)` | Search input bound to a filter signal. |
| `radio(options, signal)` | Radio group bound to a signal. |
| `confirm_delete(item_id, label)` | Dialog-guarded delete button. |

### CRUD helpers

| Macro | Purpose |
|---|---|
| `create(collection, fields)` | Form that POSTs to a collection's create endpoint. |
| `update(collection, id, fields)` | Form that PATCHes an existing item. |
| `delete(collection, id)` | Button that DELETEs after confirmation. |

These wrap the lower-level `get`/`post`/`put`/`delete` with conventions for collection endpoints.

## View context

Inside the template, these variables are available:

| Variable | Type | Contents |
|---|---|---|
| `view` | object | Self-description: `view.name`, `view.description`. |
| `collections` | object | Each declared collection available as `collections.<id>`. Items: `collections.specs.items`, `collections.specs.count`. |
| `workspace` | object | Active workspace: `workspace.id`, `workspace.branch`, `workspace.active_agent`. |
| `session` | object | Active session if any: `session.id`, `session.started_at`. |
| `recent_events` | array | Recent event log entries matching any of the view's declared subscriptions. |

Collection data is fetched at render time — the template has direct access. Don't issue your own HTTP calls from the browser; let the template and SSE do the work.

## View-scoped routes

If your view needs custom endpoints (not just collection CRUD), add `.html.j2` files for HTML handlers or emit them from the view itself via Datastar attributes that POST to `/views/<name>/actions/<slug>`. These are resolved by the view's handler module when present.

Custom routes are an advanced pattern. The common path is: use ship-elements + collections + signals + SSE, and you rarely need a custom endpoint.

## Assets

Put CSS, images, or auxiliary files inside `.ship/views/<name>/assets/`. shipd serves them at `/views/<name>/assets/<file>`. No preprocessing — raw bytes.

ship-elements and fonts are loaded by `head()`. You generally don't need per-view assets unless you're embedding static images.

## Datastar conventions

Views are rendered with the `@datastar` SSE client loaded automatically by `head()`. The important attributes:

- `data-signals='{"name": "value", ...}'` — initialize signals (use `signal()` macro).
- `data-bind="name"` — two-way bind an input (use `bind()`).
- `data-text="$name"` — substitute signal value into text content.
- `data-attr-<attr>="$name"` — bind an attribute.
- `data-class-<class>="$expr"` — toggle a class on an expression.
- `data-on-<event>="action"` — handle DOM events.
- `data-show="$expr"` — conditional visibility.

The macros above wrap these for consistency. Use raw attributes when you need something the macros don't cover.

## Ship-specific component attributes

ship-elements like `<ship-input>`, `<ship-button>`, `<ship-card>` support the same `data-*` attributes plus their own:

- `ship-button[tone]` — `primary | secondary | destructive | muted`.
- `ship-card[padding]` — `none | sm | md | lg`.
- `ship-input[placeholder]`, `[type]`, `[disabled]`.

See ship-elements catalog at `packages/ui/catalog.md` for the full per-component API.

## SSE topic format

A view's `events` array maps to SSE topics shipd uses for delivery. Topic format is `events.<event_type>` — subscribing to `collection.item_created` under the hood listens on `events.collection.item_created`. The view template doesn't need to care; shipd handles the mapping.

If your view needs a custom topic (e.g., a per-entity subscription that filters by entity_id), that's currently not expressible in the manifest — you'd subscribe in the template via Datastar's SSE helpers directly. Tracked as a future manifest enhancement.

## Limits

- Views are rendered in shipd's process. Heavy computation in Jinja blocks other view requests.
- No server-side state per-view outside collections and the event log. If you need per-view state, store it in a collection.
- No direct filesystem access from templates — only what the view context exposes.
- Per-session token bucket limits canvas-view event emission to prevent runaway agents.

## Related

- [Views Overview](/reference/views/) — what a view is, when to use one.
- [Runtime](/reference/views/runtime/) — how shipd renders and streams them.
- [Collections](/reference/collections/) — the most common data source.
