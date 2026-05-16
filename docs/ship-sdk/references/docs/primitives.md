---
group: SDK
title: Primitives
order: 2
description: The core SDK macros that cover almost every view. Each one wraps a Datastar pattern and an accessibility / loading concern you would otherwise hand-roll.
---

Every macro is imported from `_ship.html.j2`:

```jinja
{% from '_ship.html.j2' import head, signal, signals, show, live,
   bind, on, loading, spinner,
   create, update, delete, form,
   navigate, tab, tab_panel, chip, chip_all %}
```

Reach for these before raw Datastar. If a macro doesn't exist for your
pattern, file an `{#- SDK gap: ... -#}` comment (see the impl skill)
rather than inlining raw `data-on:*`.

## `head(title)`

Emits the standard `<head>` contents — meta, title, design tokens,
Datastar, the `sui-components` bundle. In fragment mode (loaded inside
the shell) the macro emits nothing because the shell already owns
those assets. Use once at the top of every full-page view.

```jinja
<head>{{ head('Tasks') }}</head>
```

## `signal(name, default)` / `signals({...})`

Declares reactive Datastar signals on the current element. Use
`signal()` for one-off values; use `signals({...})` when you have more
than two, because the object form **emits type metadata** that feeds
AST-lint's type-mismatch rule.

Signals starting with `_` are client-only UI state and never
round-trip to the server (`_showForm`, `_creating`, `_loading`).

```jinja
<div class="ship-view" {{ signals({'collection': 'task',
                                    '_showForm': false,
                                    'form': {}}) }}>
```

**Anti-pattern:** writing `data-signals:foo="'bar'"` by hand. The
quoting rules are subtle; the macro gets them right.

## `show(expr)`

Hides an element until a Datastar expression is truthy. Emits
`display:none` as an inline style so nothing flashes before hydration.

```jinja
<sui-card {{ show('$_showForm') }}>…</sui-card>
```

Prefer this to `data-show` directly — the flash-of-unstyled-content
fix is easy to forget.

## `live(view_name)` / `live_persistent(view_name)`

Subscribes the current element to a view's SSE stream. The daemon
pushes `datastar-patch-elements` frames keyed on the events listed in
the view manifest. `live_persistent` is the variant that survives
in-page `@get`/`@post` cancellations — use it only when the stream
must stay hot across navigation.

```jinja
<div {{ live('tasks') }}>
  <div id="tasks-content">Loading…</div>
</div>
```

**Anti-pattern:** opening multiple SSE streams per view. One long-lived
`@get` per view is the rule; let the kernel event list in
`manifest.yaml` drive which regions re-render.

## `bind(signal_name)`

Two-way binds an input's value to a signal. Dotted paths work when the
parent is an object (`form.title` into the `form` signal).

```jinja
<sui-input {{ bind('form.title') }} placeholder="Title"></sui-input>
```

## `on(event, expr)`

Generic event listener. Reach for this for ship-element custom events
(`ship-change`) and DOM events not covered by action macros. `evt` is
in scope in the expression.

```jinja
<sui-select {{ on('ship-change', '$status = evt.detail.value') }}>
  <option value="todo">To do</option>
</sui-select>
```

**Anti-pattern:** using `on('click', '@post(…)')` when the
action-specific macros (`post`, `create`, `emit`) already exist.

## `loading(signal)` / `spinner(signal)`

Pair for indicating an in-flight action. `loading()` flips a boolean
signal while the fetch is in flight; `spinner()` renders the visible
indicator watching the same signal.

```jinja
<sui-button {{ create('task', '$form') }} {{ loading('_creating') }}>
  Create
</sui-button>
{{ spinner('_creating') }}
```

## `create(collection, data)` / `update(collection, id, data)` / `delete(collection, id)`

Collection CRUD. Each emits a single-element click handler that POSTs,
PUTs, or DELETEs to the runtime. `data` is a Datastar expression —
typically `$form`.

```jinja
<sui-button {{ create('task', '$form') }}>Create</sui-button>
<sui-button {{ update('task', item.id, '$form') }}>Save</sui-button>
<sui-button {{ delete('task', item.id) }}>Delete</sui-button>
```

The write never needs to render — the SSE read stream pushes the new
row once the kernel event fires. CQRS is the default.

**Anti-pattern:** handling success in the writer ("if we created the
item, re-fetch"). The read stream does it for you. Let it.

## `form(schema, collection)`

Auto-generates a labelled input per schema field. Enum fields become
`<sui-select>`, boolean becomes `<sui-switch>`, markdown becomes
`<sui-textarea>`, numbers become numeric `<sui-input>`. Each field
two-way-binds under the `form` signal (or the `form_signal` you
supply).

```jinja
{{ form(ship_schema('task'), 'task') }}
```

Use when the collection is simple and the default layout is fine. For
anything custom, compose your own inputs with `bind()`.

## `navigate(url)`

Wires a click handler that issues a Datastar `@get` to another view —
used for in-page morphs. For shell-level navigation (changing the
route), reach for `nav(path)` or `view(name)` instead.

```jinja
<sui-button {{ navigate('/api/view/pipelines') }}>Pipelines</sui-button>
```

## `tab(signal, value)` / `tab_panel(signal, value)`

Tab UI without a plugin. `tab()` flips the signal and exposes
`aria-selected`; `tab_panel()` shows only when the signal matches.

```jinja
<sui-tab {{ tab('activeTab', 'details') }}>Details</sui-tab>
<sui-tab {{ tab('activeTab', 'history') }}>History</sui-tab>

<div {{ tab_panel('activeTab', 'details') }}>…</div>
```

## `chip(signal, value, view?)` / `chip_all(signal, view?)`

Filter chips. `chip()` flips a filter signal and (optionally)
triggers an SSE refresh so the server reruns the query. `chip_all()`
is the companion that clears the filter.

```jinja
<sui-chip {{ chip_all('status', 'tasks') }}>All</sui-chip>
<sui-chip {{ chip('status', 'active', 'tasks') }}>Active</sui-chip>
<sui-chip {{ chip('status', 'done',   'tasks') }}>Done</sui-chip>
```

**Anti-pattern:** filtering client-side across a full dataset. Chips
are meant to trigger server-side `ship_query` reruns; the debounce and
selected-state plumbing is already in the macro.
