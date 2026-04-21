# Ship Jinja SDK — macro reference

Auto-generated from `apps/shipd/templates/_ship.html.j2` by `scripts/build-jinja-docs.mjs`. Do not edit by hand — re-run the generator.

Import macros into a view template with:

```jinja
{% from '_ship.html.j2' import head, signal, show, live %}
```

## Contents

- [Page setup](#page-setup)
  - [head](#head)
  - [styles](#styles)
- [Signals](#signals)
  - [signal](#signal)
  - [show](#show)
  - [toggle_class](#toggle-class)
  - [attr](#attr)
  - [bind](#bind)
- [Events](#events)
  - [live](#live)
  - [on](#on)
  - [live_persistent](#live-persistent)
  - [on_fetch_error](#on-fetch-error)
- [Actions](#actions)
  - [delete](#delete)
  - [navigate](#navigate)
  - [nav](#nav)
  - [view](#view)
  - [get](#get)
  - [post](#post)
  - [put](#put)
  - [put_on_change](#put-on-change)
  - [submit](#submit)
- [Collection actions](#collection-actions)
  - [create](#create)
  - [update](#update)
  - [confirm_delete](#confirm-delete)
- [Job actions](#job-actions)
  - [create_job](#create-job)
  - [emit](#emit)
  - [feedback](#feedback)
- [UI patterns](#ui-patterns)
  - [loading](#loading)
  - [spinner](#spinner)
  - [tab](#tab)
  - [tab_panel](#tab-panel)
  - [chip](#chip)
  - [chip_all](#chip-all)
  - [search](#search)
  - [on_change_debounced](#on-change-debounced)
  - [radio](#radio)
  - [confirm](#confirm)
  - [toast_container](#toast-container)
  - [toast_action](#toast-action)
  - [confirm_dialog](#confirm-dialog)
  - [confirm_trigger](#confirm-trigger)
  - [section](#section)
  - [endsection](#endsection)
- [Auto-generators](#auto-generators)
  - [form](#form)
  - [table](#table)
- [Layout](#layout)
  - [layout_center](#layout-center)
  - [layout_sidebar](#layout-sidebar)
  - [layout_three_col](#layout-three-col)
  - [layout_end](#layout-end)

## Page setup

### head

```
head(title='Ship View')
```

Emit the standard <head> contents for a view: meta tags, title, design
token CSS, Datastar, and the ship-elements bundle. In fragment mode
(loaded inside the shell via Datastar) these assets are already present,
so the macro emits nothing.

**Arguments:**

- `title` (optional, default `'Ship View'`) — Document title. Shown in the browser tab. Defaults to 'Ship View'.

**Example:**

```jinja
<head>{{ head('Tasks') }}</head>
```

_Source: `apps/shipd/templates/_ship.html.j2:23`_

### styles

```
styles()
```

Emit the shared `<style>` block used by `form()` and `table()`:
`.ship-form-field`, `.ship-form-label`, `.ship-table`, `.mono`,
`.muted`, `.null`. Place once in `<head>` when a view renders an
auto-form or auto-table.

**Example:**

```jinja
<head>{{ head('Tasks') }}{{ styles() }}</head>
```

_Source: `apps/shipd/templates/_ship.html.j2:976`_

## Signals

### signal

```
signal(name, default='')
```

Declare a reactive Datastar signal on the current element. String defaults
are emitted wrapped in single quotes, `none` becomes literal `null`, and
other types pass through unquoted so objects/arrays/numbers/booleans
serialize correctly.

**Arguments:**

- `name` (*required*) — Signal name. Use a leading underscore (`_loading`, `_showForm`) for client-only state that should not round-trip to the server.
- `default` (optional, default `''`) — Initial value. String, number, boolean, `none`, or a JSON-like literal such as `{}` or `[]`. Defaults to the empty string.

**Example:**

```jinja
<div {{ signal('collection', 'task') }}
     {{ signal('_showForm', false) }}
     {{ signal('form', {}) }}>
```

_Source: `apps/shipd/templates/_ship.html.j2:55`_

### show

```
show(expr)
```

Hide the element until the Datastar expression becomes truthy. Emits
`display:none` as an inline style so the element cannot flash into view
before Datastar hydrates.

**Arguments:**

- `expr` (*required*) — Datastar expression (references signals with `$name`).

**Example:**

```jinja
<ship-card {{ show('$_showForm') }}>...</ship-card>
```

_Source: `apps/shipd/templates/_ship.html.j2:70`_

### toggle_class

```
toggle_class(cls, expr)
```

Reactively toggle a single CSS class based on a Datastar expression.

**Arguments:**

- `cls` (*required*) — Class name to add/remove.
- `expr` (*required*) — Datastar expression; class is present when truthy.

**Example:**

```jinja
<div {{ toggle_class('active', '$tab == "details"') }}>...</div>
```

_Source: `apps/shipd/templates/_ship.html.j2:84`_

### attr

```
attr(name, expr)
```

Reactively bind an HTML attribute's value to a Datastar expression.

**Arguments:**

- `name` (*required*) — Attribute name (e.g. `selected`, `disabled`, `aria-pressed`).
- `expr` (*required*) — Datastar expression producing the attribute value.

**Example:**

```jinja
<ship-chip {{ attr('selected', '$status == "active"') }}>Active</ship-chip>
```

_Source: `apps/shipd/templates/_ship.html.j2:98`_

### bind

```
bind(signal_name)
```

Two-way-bind an input element's value to a Datastar signal.

**Arguments:**

- `signal_name` (*required*) — Signal to bind. Dotted paths (`form.title`) are supported when the parent signal is an object.

**Example:**

```jinja
<ship-input {{ bind('form.title') }} placeholder="Title"></ship-input>
```

_Source: `apps/shipd/templates/_ship.html.j2:365`_

## Events

### live

```
live(view_name)
```

Subscribe the current element to a view's long-lived SSE stream on
connect. The daemon pushes `datastar-patch-elements` frames that replace
marked regions whenever the view's declared kernel events fire.

**Arguments:**

- `view_name` (*required*) — View slug registered with the daemon. Resolves to `/api/runtime/view/<view_name>/sse`.

**Example:**

```jinja
<div {{ live('tasks') }}>
  <div id="tasks-content">Loading...</div>
</div>
```

_Source: `apps/shipd/templates/_ship.html.j2:118`_

### on

```
on(event, expr)
```

Generic event-listener binding. Use for ship-element custom events
(`ship-change`, `ship-select`, etc.) and any DOM event not covered by
the action-specific macros.

**Arguments:**

- `event` (*required*) — Event name. Prefix is not required (no `on`).
- `expr` (*required*) — Datastar expression to run when the event fires. `evt` is in scope.

**Example:**

```jinja
<ship-select {{ on('ship-change', '$status = evt.detail.value') }}>
  ...
</ship-select>
```

_Source: `apps/shipd/templates/_ship.html.j2:274`_

### live_persistent

```
live_persistent(view_name)
```

Like `live()`, but configures Datastar with `requestCancellation: 'disabled'`
so the SSE connection is not aborted when other `@get`/`@post` actions
fire on the page. Use for views that must stay subscribed across
navigations within the same shell region.

**Arguments:**

- `view_name` (*required*) — View slug whose SSE stream to subscribe to.

**Example:**

```jinja
<div {{ live_persistent('events') }}>
  <div id="events-content"></div>
</div>
```

_Source: `apps/shipd/templates/_ship.html.j2:615`_

### on_fetch_error

```
on_fetch_error(signal_name, message="'HTTP ' + (evt.detail.argsRaw && evt.detail.argsRaw.status || '?')")
```

Bind a signal to receive a failure message when a fetch action on
THIS element returns an HTTP error or non-SSE body.

Datastar fires a `datastar-fetch` event on document with
  detail: { type: 'error', el: <trigger>, argsRaw: { status } }
when `onopen` sees a >= 400 response. This macro attaches a
document-level listener scoped to the current element so a failed
`@post`/`@get`/`@put`/`@delete` patches the named error signal.

**Arguments:**

- `signal_name` (*required*) — Signal that receives the error message when the fetch fails.
- `message` (optional, default `"'HTTP ' + (evt.detail.argsRaw && evt.detail.argsRaw.status || '?')"`) — JS expression producing the error message. Has access to `evt.detail.argsRaw.status`. Defaults to `'HTTP ' + status`.

**Example:**

```jinja
<ship-button
  data-on:click="$step = 3; @post('/api/projects', {payload: ...})"
  {{ on_fetch_error('ws_error') }}>
  Continue
</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:644`_

## Actions

### delete

```
delete(collection, id)
```

Wire a click handler that DELETEs a specific collection item. Does not
prompt — use `confirm_delete()` when a confirmation step is required.

**Arguments:**

- `collection` (*required*) — Collection id.
- `id` (*required*) — Item id to delete.

**Example:**

```jinja
<ship-button {{ delete('task', item.id) }}>Delete</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:206`_

### navigate

```
navigate(url)
```

Wire a click handler that fetches another view via Datastar `@get`,
triggering an in-page morph. Use this for partial updates within the
current view. For full shell-level routing use `nav()` or `view()`.

**Arguments:**

- `url` (*required*) — URL to GET (typically under `/api/runtime/view/...`).

**Example:**

```jinja
<ship-button {{ navigate('/api/runtime/view/pipelines') }}>
  Pipelines
</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:225`_

### nav

```
nav(path)
```

Wire a click handler that navigates via the shell router when present,
falling back to a full page load. Use this for shell-level navigation.

**Arguments:**

- `path` (*required*) — Shell path to navigate to (e.g. `/agents`).

**Example:**

```jinja
<ship-button {{ nav('/agents') }}>Agents</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:239`_

### view

```
view(name)
```

Wire a click handler that navigates to a named view via the shell router
when present, falling back to a full page load at `/<name>`.

**Arguments:**

- `name` (*required*) — View slug. Becomes the path `/<name>`.

**Example:**

```jinja
<ship-button {{ view('tasks') }}>Tasks</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:253`_

### get

```
get(url)
```

Wire a click handler that issues a Datastar `@get` to the given URL.

**Arguments:**

- `url` (*required*) — Fully-qualified path to GET.

**Example:**

```jinja
<ship-button {{ get('/api/runtime/jobs') }}>Refresh jobs</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:289`_

### post

```
post(url, payload='{}')
```

Wire a click handler that issues a Datastar `@post` to the given URL
with the supplied payload expression.

**Arguments:**

- `url` (*required*) — Target URL.
- `payload` (optional, default `'{}'`) — Datastar expression (or JSON literal) passed as the request body. Defaults to an empty object.

**Example:**

```jinja
<ship-button {{ post('/api/runtime/jobs', '$newJob') }}>Run</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:305`_

### put

```
put(url, payload='{}')
```

Wire a click handler that issues a Datastar `@put`.

**Arguments:**

- `url` (*required*) — Target URL.
- `payload` (optional, default `'{}'`) — Request body expression. Defaults to an empty object.

**Example:**

```jinja
<ship-button {{ put('/api/settings', '{theme: $theme}') }}>Save</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:319`_

### put_on_change

```
put_on_change(url, payload='{}')
```

Wire a change-event handler (fires on every committed input change)
that issues a Datastar `@put`. Good for settings inputs that persist
on change without an explicit save button.

**Arguments:**

- `url` (*required*) — Target URL.
- `payload` (optional, default `'{}'`) — Request body expression. Defaults to an empty object.

**Example:**

```jinja
<ship-input {{ bind('theme') }}
            {{ put_on_change('/api/settings', '{theme: $theme}') }}>
</ship-input>
```

_Source: `apps/shipd/templates/_ship.html.j2:337`_

### submit

```
submit(url, payload='{}')
```

Wire a form `submit` handler that POSTs the given payload.

**Arguments:**

- `url` (*required*) — Target URL.
- `payload` (optional, default `'{}'`) — Request body expression. Defaults to an empty object.

**Example:**

```jinja
<form {{ submit('/api/runtime/specs', '$form') }}>...</form>
```

_Source: `apps/shipd/templates/_ship.html.j2:351`_

## Collection actions

### create

```
create(collection, data)
```

Wire a click handler that POSTs a new item into a collection. The
payload's `data` field is a Datastar expression — typically a form
signal such as `$form`.

**Arguments:**

- `collection` (*required*) — Collection id (matches a schema in `.ship/collections/`).
- `data` (*required*) — Datastar expression whose value becomes the item's `data` payload.

**Example:**

```jinja
<ship-button {{ create('task', '$form') }}
             {{ loading('_creating') }}>Create</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:175`_

### update

```
update(collection, id, data)
```

Wire a click handler that PUTs updated data onto a specific collection
item.

**Arguments:**

- `collection` (*required*) — Collection id.
- `id` (*required*) — Target item id.
- `data` (*required*) — Datastar expression for the new `data` payload.

**Example:**

```jinja
<ship-button {{ update('task', item.id, '$form') }}>Save</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:191`_

### confirm_delete

```
confirm_delete(collection, id)
```

Wire a click handler that shows a browser `confirm()` dialog with
"Delete this item?" and, on OK, DELETEs the item. For a typed
Ship-styled confirm dialog use `confirm_dialog()` / `confirm_trigger()`.

**Arguments:**

- `collection` (*required*) — Collection id.
- `id` (*required*) — Item id to delete.

**Example:**

```jinja
<ship-button {{ confirm_delete('task', item.id) }}>Delete</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:578`_

## Job actions

### create_job

```
create_job(agent_expr, pipeline='')
```

Wire a click handler that POSTs a new job to the runtime. The agent is
passed as a Datastar expression so UI state (a select value, signal,
etc.) can drive it. When `pipeline` is supplied it is added to the
payload verbatim.

**Arguments:**

- `agent_expr` (*required*) — Datastar expression producing the agent slug (e.g. `$selectedAgent`).
- `pipeline` (optional, default `''`) — Optional pipeline name. When truthy it is inlined into the request body. Defaults to the empty string (omitted).

**Example:**

```jinja
<ship-button {{ create_job('$selectedAgent', 'review') }}>
  Run
</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:388`_

### emit

```
emit(event_type, data_expr='{}')
```

Wire a click handler that posts a kernel event via `/api/events/emit`.
Use to trigger backend side-effects that other views listen to over SSE.

**Arguments:**

- `event_type` (*required*) — Kernel event type (e.g. `task.requested`).
- `data_expr` (optional, default `'{}'`) — Datastar expression for the event's `data` payload. Defaults to an empty object.

**Example:**

```jinja
<ship-button {{ emit('task.requested', '{title: $title}') }}>
  Request
</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:406`_

### feedback

```
feedback(payload_expr)
```

Wire a click handler that posts structured feedback to the active
agent/session. The payload shape is whatever the agent designed — the
receiving prompt reads it. Use this when an agent-authored view needs
typed human input beyond free-text (ratings, selections, pick-one).

Emits as feedback (kind=action) so it flows through the
studio.feedback.created pipeline. The agent reads `payload` on the
event to see what the user picked.

**Arguments:**

- `payload_expr` (*required*) — Datastar/JS object expression that becomes the feedback `payload`.

**Example:**

```jinja
<ship-button {{ feedback("{selected: 2, comment: 'bolder one'}") }}>
  Pick A
</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:431`_

## UI patterns

### loading

```
loading(signal_name)
```

Mark the element as a Datastar action trigger whose in-flight state
writes a boolean to the given signal. Pair with `spinner()` or
`show()` to render a visual indicator.

**Arguments:**

- `signal_name` (*required*) — Signal that will flip to `true` while the fetch is in flight. Conventionally underscore-prefixed (`_creating`).

**Example:**

```jinja
<ship-button {{ create('task', '$form') }}
             {{ loading('_creating') }}>Create</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:137`_

### spinner

```
spinner(signal_name)
```

Render an inline spinner that is visible while the given signal is
truthy. Designed to sit next to a button that uses `loading()` on the
same signal.

**Arguments:**

- `signal_name` (*required*) — Signal the spinner watches. Same value passed to `loading()`.

**Example:**

```jinja
<ship-button {{ create('task', '$form') }}
             {{ loading('_creating') }}>Create</ship-button>
{{ spinner('_creating') }}
```

_Source: `apps/shipd/templates/_ship.html.j2:155`_

### tab

```
tab(signal_name, value, label='')
```

Wire a tab button. Sets the named signal to `value` on click and
exposes `aria-selected` reactively so CSS can style the active tab.

**Arguments:**

- `signal_name` (*required*) — Signal that tracks the active tab value.
- `value` (*required*) — The string value this tab represents.
- `label` (optional, default `''`) — Accepted for call-site readability; currently unused by the emitted attributes. Defaults to the empty string.

**Example:**

```jinja
<ship-tab {{ tab('activeTab', 'details') }}>Details</ship-tab>
<ship-tab {{ tab('activeTab', 'history') }}>History</ship-tab>
```

_Source: `apps/shipd/templates/_ship.html.j2:451`_

### tab_panel

```
tab_panel(signal_name, value)
```

Show the tab panel only when the tab signal matches `value`. Uses
`display:none` as a base style so panels don't flash before Datastar
hydrates.

**Arguments:**

- `signal_name` (*required*) — The same signal used with `tab()`.
- `value` (*required*) — The value this panel is visible for.

**Example:**

```jinja
<div {{ tab_panel('activeTab', 'details') }}>...</div>
```

_Source: `apps/shipd/templates/_ship.html.j2:467`_

### chip

```
chip(signal_name, value, view_name='')
```

Wire a filter chip: on click, sets the named signal to `value` and
(optionally) refreshes the view's SSE stream so the server reruns the
query with the new filter. Also exposes `selected` reactively for the
active chip.

**Arguments:**

- `signal_name` (*required*) — Filter signal.
- `value` (*required*) — Value this chip selects.
- `view_name` (optional, default `''`) — When set, a `@get('/api/runtime/view/<view_name>/sse')` is appended to refresh results. Defaults to the empty string (client-only filter).

**Example:**

```jinja
<ship-chip {{ chip('status', 'active', 'tasks') }}>Active</ship-chip>
```

_Source: `apps/shipd/templates/_ship.html.j2:489`_

### chip_all

```
chip_all(signal_name, view_name='')
```

A companion to `chip()` that clears the filter (sets the signal to the
empty string) and optionally refreshes SSE. Use for an "All" chip next
to value-specific chips.

**Arguments:**

- `signal_name` (*required*) — Filter signal to clear.
- `view_name` (optional, default `''`) — When set, refreshes `/api/runtime/view/<view_name>/sse` after clearing.

**Example:**

```jinja
<ship-chip {{ chip_all('status', 'tasks') }}>All</ship-chip>
```

_Source: `apps/shipd/templates/_ship.html.j2:506`_

### search

```
search(signal_name, view_name, delay='300ms')
```

Wire a debounced search input: two-way-binds to a signal and refreshes
the view's SSE stream when the user pauses typing.

**Arguments:**

- `signal_name` (*required*) — Query signal (bound with `data-bind`).
- `view_name` (*required*) — View whose SSE stream to refresh on input.
- `delay` (optional, default `'300ms'`) — Datastar debounce modifier (e.g. `300ms`, `500ms`). Defaults to `300ms`.

**Example:**

```jinja
<ship-input {{ search('q', 'tasks') }} placeholder="Search..."></ship-input>
```

_Source: `apps/shipd/templates/_ship.html.j2:525`_

### on_change_debounced

```
on_change_debounced(expr, delay='300ms')
```

Run an expression on `change`, debounced. Useful for settings that
auto-save after the user finishes editing.

**Arguments:**

- `expr` (*required*) — Datastar expression.
- `delay` (optional, default `'300ms'`) — Debounce duration. Defaults to `300ms`.

**Example:**

```jinja
<ship-input {{ bind('name') }}
            {{ on_change_debounced("@put('/api/settings', {payload: {name: $name}})") }}>
</ship-input>
```

_Source: `apps/shipd/templates/_ship.html.j2:542`_

### radio

```
radio(signal_name, value)
```

Wire a radio-style button: sets the named signal to `value` on click
and reflects the selection via `aria-checked`.

**Arguments:**

- `signal_name` (*required*) — Selection signal.
- `value` (*required*) — Value this option represents.

**Example:**

```jinja
<ship-button {{ radio('theme', 'dark') }}>Dark</ship-button>
<ship-button {{ radio('theme', 'light') }}>Light</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:560`_

### confirm

```
confirm(message, action_expr)
```

Wire a click handler that shows a browser `confirm()` dialog with the
given message and, on OK, runs the Datastar expression.

**Arguments:**

- `message` (*required*) — Dialog text.
- `action_expr` (*required*) — Datastar expression to run when the user confirms.

**Example:**

```jinja
<ship-button {{ confirm('Cancel this job?', "@post('/api/runtime/jobs/" ~ job.id ~ "/cancel')") }}>
  Cancel
</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:595`_

### toast_container

```
toast_container()
```

Mount the `<ship-toast-container>` in the bottom-right corner with the
id `ship-toasts`. `toast_action()` looks this up by id to dispatch
toasts. Place once per page.

**Example:**

```jinja
{{ toast_container() }}
```

_Source: `apps/shipd/templates/_ship.html.j2:833`_

### toast_action

```
toast_action(message, tone='default', duration=3000)
```

Wire a click handler that dispatches a toast through the container
mounted by `toast_container()`.

**Arguments:**

- `message` (*required*) — Toast title text.
- `tone` (optional, default `'default'`) — Visual tone — `default`, `info`, `success`, `warning`, `destructive`. Defaults to `default`.
- `duration` (optional, default `3000`) — Time in milliseconds before auto-dismiss. Defaults to `3000`.

**Example:**

```jinja
<ship-button {{ toast_action('Saved!', 'success') }}>Save</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:853`_

### confirm_dialog

```
confirm_dialog(id, title, description, action_attrs)
```

Render a `<ship-alert-dialog>` that gates a destructive action.
Cancel closes the dialog; Confirm executes `action_attrs` and then
closes. Pair with `confirm_trigger(id)` on the opening button.

**Arguments:**

- `id` (*required*) — Unique id suffix. The dialog's DOM id becomes `confirm-<id>`.
- `title` (*required*) — Heading text.
- `description` (*required*) — Body text under the heading.
- `action_attrs` (*required*) — Pre-rendered attributes for the confirm button — typically a `delete()` / `post()` / `put()` macro call.

**Example:**

```jinja
{{ confirm_dialog('delete-item', 'Delete this item?',
   'This cannot be undone.', delete('task', item.id)) }}
<ship-button {{ confirm_trigger('delete-item') }}>Delete</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:876`_

### confirm_trigger

```
confirm_trigger(id)
```

Wire a click handler that opens a `confirm_dialog()` by id (adds the
`open` attribute to `confirm-<id>`).

**Arguments:**

- `id` (*required*) — Matching id passed to `confirm_dialog()`.

**Example:**

```jinja
<ship-button {{ confirm_trigger('delete-item') }}>Delete</ship-button>
```

_Source: `apps/shipd/templates/_ship.html.j2:900`_

### section

```
section(title, icon=none, link_href=none, link_label=none)
```

Open a `<ship-card>` with a standardized slot-header row: optional
leading ship-icon + title on the left, optional trailing link pill
(muted text + right-chevron) on the right. Must be paired with
`endsection()`.

When `link_href` starts with `/api/runtime/view/`, the trailing link
is wired to `@get(link_href)` with `.prevent` so SSE-aware navigation
takes over. Otherwise it is a plain anchor.

The layout is inline-styled on purpose — the macro owns the
standardized design tokens so view authors don't have to re-declare
`.dash-section-head`.

**Arguments:**

- `title` (*required*) — Section heading text. Required.
- `icon` (optional, default `none`) — ship-icon registry name (e.g. `activity`, `cpu`). Optional.
- `link_href` (optional, default `none`) — URL for the trailing pill. Optional.
- `link_label` (optional, default `none`) — Visible text on the trailing pill. Optional.

**Example:**

```jinja
{{ section('Recent jobs', icon='activity',
           link_href='/api/runtime/view/pipelines',
           link_label='View all') }}
  <ship-list-tile>...</ship-list-tile>
{{ endsection() }}
```

_Source: `apps/shipd/templates/_ship.html.j2:933`_

### endsection

```
endsection()
```

Close the `<ship-card>` opened by `section()`.

**Example:**

```jinja
{{ section('Recent jobs') }}
  ...body...
{{ endsection() }}
```

_Source: `apps/shipd/templates/_ship.html.j2:961`_

## Auto-generators

### form

```
form(schema, collection, form_signal='form')
```

Auto-generate a form from a collection schema. Emits one labelled
input per field, with the input type chosen from the field definition:
`enum` -> `<ship-select>`, `boolean` -> `<ship-switch>`, `number` ->
numeric `<ship-input>`, `markdown` -> `<ship-textarea>`, everything
else -> `<ship-input>`. Required fields get a red asterisk. Each input
is two-way-bound under `form_signal.<field>`.

**Arguments:**

- `schema` (*required*) — Collection schema dict (from `ship_schema(collection)`).
- `collection` (*required*) — Collection id. Currently accepted for parity with related macros; not used by the emitted HTML.
- `form_signal` (optional, default `'form'`) — Parent signal object inputs bind into. Defaults to `form`.

**Example:**

```jinja
{{ form(ship_schema('task'), 'task') }}
```

_Source: `apps/shipd/templates/_ship.html.j2:668`_

### table

```
table(schema, items, collection)
```

Auto-generate a data table from a schema plus a list of items. Emits
an ID column, one column per schema field with type-appropriate
formatting (enum -> chip, boolean -> badge, number -> monospace,
array -> up-to-4 chips with overflow badge), an Updated column, and a
per-row delete button wired with `delete(collection, item.id)`.

**Arguments:**

- `schema` (*required*) — Collection schema dict.
- `items` (*required*) — Iterable of item records with `.id`, `.data`, `.updated_at`.
- `collection` (*required*) — Collection id (used by the per-row delete button).

**Example:**

```jinja
{{ table(ship_schema('task'), ship_items('task'), 'task') }}
```

_Source: `apps/shipd/templates/_ship.html.j2:711`_

## Layout

### layout_center

```
layout_center(max_width='768px')
```

Open a centered single-column layout wrapper: `max-width` with
horizontal auto-margins and generous vertical padding (space-6 top,
space-12 bottom). Must be closed with `layout_end()`. Good for
settings pages, forms, and detail views.

**Arguments:**

- `max_width` (optional, default `'768px'`) — CSS length for the column's maximum width. Defaults to `768px`.

**Example:**

```jinja
{{ layout_center('960px') }}
  ...content...
{{ layout_end() }}
```

_Source: `apps/shipd/templates/_ship.html.j2:761`_

### layout_sidebar

```
layout_sidebar(sidebar_width='240px')
```

Open a two-column grid: fixed sidebar + flexible main pane, full
height. Must be closed with `layout_end()`. Good for explorers and
canvas-style views.

**Arguments:**

- `sidebar_width` (optional, default `'240px'`) — CSS length for the sidebar column. Defaults to `240px`.

**Example:**

```jinja
{{ layout_sidebar() }}
  <aside>...</aside>
  <main>...</main>
{{ layout_end() }}
```

_Source: `apps/shipd/templates/_ship.html.j2:782`_

### layout_three_col

```
layout_three_col(left='240px', right='320px')
```

Open a three-column grid: fixed left sidebar + flexible main pane +
fixed right info panel, full height. Must be closed with
`layout_end()`. Good for IDE-style views.

**Arguments:**

- `left` (optional, default `'240px'`) — CSS length for the left column. Defaults to `240px`.
- `right` (optional, default `'320px'`) — CSS length for the right column. Defaults to `320px`.

**Example:**

```jinja
{{ layout_three_col() }}
  <aside>...</aside>
  <main>...</main>
  <aside>...</aside>
{{ layout_end() }}
```

_Source: `apps/shipd/templates/_ship.html.j2:804`_

### layout_end

```
layout_end()
```

Close a layout wrapper opened by `layout_center()`, `layout_sidebar()`,
or `layout_three_col()`.

**Example:**

```jinja
{{ layout_center() }}
  ...content...
{{ layout_end() }}
```

_Source: `apps/shipd/templates/_ship.html.j2:819`_

