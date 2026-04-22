---
name: ship-views
stable-id: ship-views
description: Author Ship views — server-rendered reactive HTML with zero build step. Drop a file in .ship/views/<name>/, reload the page, see changes. Uses the Ship View SDK (Jinja macros + ship-elements + Datastar under the hood). This is the primary skill for building Ship extensions.
tags: [views, sdk, extension-authoring, jinja, datastar, ship-elements]
authors: [ship]
---

# Ship Views — The Extension Authoring Surface

A Ship view is a server-rendered reactive HTML page that updates live via SSE, responds to user actions via Datastar attributes, and is composed from the Ship component library (ship-elements). You author it as a single `.html.j2` file in `.ship/views/<name>/`. No build step. No framework. Edit the file, refresh the browser, see your changes.

**This is how you build extensions on Ship.**

## Quick start — a minimal view in 30 seconds

Create `.ship/views/hello/index.html.j2`:

```jinja
{% from '_ship.html.j2' import head, signal, show %}
<!DOCTYPE html>
<html>
<head>{{ head('Hello') }}</head>
<body>
<div class="ship-view" {{ signal('message', 'world') }}>
  <ship-card style="max-width: 40rem; margin: 40px auto; padding: 24px;">
    <h1 style="font-family: var(--font-display);">Hello, <span data-text="$message"></span>!</h1>
    <ship-input data-bind="message" placeholder="Type your name"></ship-input>
  </ship-card>
</div>
</body>
</html>
```

Load `http://localhost:3000/hello`. Type. The heading updates as you type. No JS. No build.

## Shell chrome — views contribute, they don't reimplement

The shell (`apps/shell/`) owns the top bar (brand + breadcrumb + ⌘K + rail toggle + theme + chat) and the status bar. **Your view must not render its own breadcrumb, branch pill, search box, or status bar.** Those ship with the shell. Views contribute metadata and optional actions via a single element:

```jinja
<ship-view-chrome
  title="Git"
  branch="{{ branch }}"
  ahead="{{ ahead }}"
  behind="{{ behind }}"
  tone="{% if clean %}success{% else %}warning{% endif %}">
  <ship-button slot="actions" variant="ghost" size="icon" aria-label="Refresh"
    data-on:click="@get('{{ view_url('git') }}')">
    <ship-icon name="refresh-cw" size="sm"></ship-icon>
  </ship-button>
  <ship-chip slot="status-end" size="sm" variant="soft">{{ total_changed }} changed</ship-chip>
</ship-view-chrome>
```

**Attributes** (shell-owned fields):
- `title` — overrides the breadcrumb leaf (defaults to route name).
- `branch`, `ahead`, `behind` — rendered in the status bar (`main ↑2 ↓1`).
- `tone` / `busy` — tints and pulses the status dot.

**Slots** (free-form contribution regions):
- `actions` → right side of the topbar (after branch/title-relevant chrome).
- `status-start` → left side of the status bar (after branch).
- `status-end` → right side of the status bar (before theme indicator).

**Mechanism.** Same-DOM: the element moves slotted children into shell-owned containers (`#view-topbar-actions`, `#view-status-start`, `#view-status-end`) and writes attribute values into `#breadcrumbs-title`, `#status-branch`, `#status-ab`. Datastar bindings on slotted elements survive the move. On route change the view fragment unmounts and the element's `disconnectedCallback` auto-clears the contributed chrome.

**Absence is safe.** If `ship-view-chrome` is missing, the shell breadcrumb falls back to the route name; the status bar's view regions stay empty. Omit it for mini/embedded views; include it for full-page views.

## Hard rule — views are composition only

**A view template contains no inline `<script>` and no inline `<style>`.**
Not "should avoid"; does not contain. A view has two jobs:

1. **Compose `<ship-*>` elements and Datastar attributes.** The
   elements bring styling and behavior with them; the view picks
   which ones and wires signals between them.
2. **Declare view-local partials** in
   `.ship/views/<view>/_components/*.j2` when a composition repeats
   within the view.

If you reach for something the rule forbids, the fix is upstream:

| Temptation | Right home |
|---|---|
| Inline `<script>` for interactivity | A `<ship-*>` element, a `data-effect="..."` reactive expression (see Reactivity below), or a custom Datastar plugin in the `datastar-views` skill. |
| Inline `<style>` for layout or theming | Shadow DOM inside a ship-element, or CSS custom properties the element already exposes. |
| Raw CSS layout in a first-party view | A pack component that owns its layout |
| A custom topbar / breadcrumb / status bar inside the view | `<ship-view-chrome>` (see Shell chrome above). The shell owns that chrome. |
| Raw `data-on:click="@get('/api/...')"` | A URL macro from `_urls.html.j2` |
| Inline `data-signals:x="'val'"` | `{{ signal('x', 'val') }}` macro |
| `data-attr:data-active="$tab == 'x'"` | `{{ attr('data-active', "$tab == 'x'") }}` macro or `data-class:active="..."` |
| Copy-pasting the same markup block five times in one template | A Jinja partial in `.ship/views/<view>/_components/`. |
| A new `<ship-X-row>` element that only exists to hold view-specific text | Compose `<ship-list-tile>` with slots. See the distribution test in the `ship-components` skill. |

When the elements genuinely don't support a pattern, don't paper over
it with raw HTML + CSS. File the gap with the
`{#- SDK gap: ... -#}` convention (see the iteration loop below) and
escalate to extending the element or adding an SDK macro. That is
how the library grows; shadow workarounds in views is how it rots.

View-local Jinja partials are the right abstraction when a composition
**repeats within one view** but isn't yet reusable enough to earn a
library element. Once the same partial appears in 3+ views, promote
it — preferably by adding slots to an existing `<ship-*>` element.

## The SDK — always import from `_ship.html.j2`

Every view starts with:

```jinja
{% from '_ship.html.j2' import head, signal, show, live, loading, spinner,
   create, update, delete, form, table, styles,
   get, post, put, navigate, on, bind, attr,
   tab, tab_panel, chip, chip_all, search, radio, confirm_delete %}
```

These are all the macros you need. They hide Datastar syntax, wrap common patterns, and enforce consistency. **Don't write raw `data-on:click`, `data-signals:`, etc. — use the macros.** Raw Datastar is an escape hatch for rare cases.

## The component library — ship-elements

Every UI element is a `<ship-*>` web component. They style themselves. They handle keyboard, focus, and accessibility. You compose them like HTML.

### Buttons — get this right first

**The button model has two orthogonal attributes:**

| Attribute | Purpose | Values |
|-----------|---------|--------|
| `tone` | Semantic color meaning | `default`, `info`, `success`, `warning`, `destructive`, `accent`, `muted` |
| `variant` | Visual style | `solid`, `soft`, `outline`, `ghost` |

**These are not interchangeable.** `variant="destructive"` is wrong — destructive is a tone, not a variant.

**CTA cookbook:**

```jinja
{# Primary action — colored, prominent #}
<ship-button tone="accent" variant="solid">Save</ship-button>

{# Destructive action — soft red, not shouty #}
<ship-button tone="destructive" variant="soft">Delete</ship-button>

{# Secondary/cancel — no color, subtle #}
<ship-button variant="ghost">Cancel</ship-button>

{# Irreversible high-risk — solid red #}
<ship-button tone="destructive" variant="solid">Permanently delete</ship-button>

{# Icon-only — needs aria-label #}
<ship-button variant="ghost" aria-label="Settings">
  <ship-icon slot="leading" name="settings"></ship-icon>
</ship-button>

{# With loading indicator #}
<ship-button tone="accent" variant="solid"
  {{ create('task', '$form') }}
  {{ loading('_creating') }}>
  Create
</ship-button>
```

### Inputs and selects

```jinja
<ship-input data-bind="form.title" placeholder="Title"></ship-input>
<ship-textarea data-bind="form.notes" rows="3"></ship-textarea>
<ship-switch data-bind="form.enabled"></ship-switch>

{# IMPORTANT: ship-select uses native <option>, NOT <ship-option> #}
<ship-select data-bind="status">
  <option value="todo">To do</option>
  <option value="doing" selected>Doing</option>
  <option value="done">Done</option>
</ship-select>

{# IMPORTANT: listen for 'ship-change', NOT 'ship-select-change' #}
<ship-select {{ on('ship-change', '$status = evt.detail.value') }}>
  ...
</ship-select>
```

Before writing any `<ship-*>` element, query the CEM (Custom Elements Manifest):
```
GET /api/runtime/assets/custom-elements.json
```
This is the authoritative catalog. Every element in the library is listed with its properties, attributes, events, slots, and CSS parts. Use it to discover what exists before reaching for raw HTML. The CEM is machine-readable — an agent should fetch it at the start of any view authoring session.

### Common components (reach for these first)

| Need | Use |
|------|-----|
| Button | `<ship-button tone="..." variant="...">` |
| Text input | `<ship-input>` |
| Multi-line input | `<ship-textarea>` |
| Dropdown | `<ship-select>` (native `<option>` children) |
| Toggle | `<ship-switch>` |
| Checkbox | `<ship-checkbox>` |
| Card/panel | `<ship-card>` |
| Toolbar/button group | `<ship-toolbar>` |
| Tag/filter chip | `<ship-chip>` |
| Status label | `<ship-badge>` |
| Divider | `<ship-separator>` |
| Empty state | `<ship-empty-state>` |
| Status indicator | `<ship-status-dot>` |
| Icon | `<ship-icon name="...">` |
| Timestamp | `<ship-timestamp>` |
| Tabs | `<ship-tabs>` + `<ship-tab>` + `<ship-tab-panel>` |
| Segmented control (tab strip, density toggle, view-mode switcher) | `<ship-segmented>` + `<ship-segmented-option>` |
| Dialog | `<ship-dialog>` |
| Toast | `<ship-toast>` |
| **Any list row, tile, or card** — agent row, event row, log line, history entry, settings row, dashboard card | **`<ship-list-tile>`** (slots: `leading`, default for primary, `meta` for sub-label, `trailing` for badges/controls; `variant=row\|card`, `density=compact\|normal\|relaxed`, `interactive`, `selected`, `tone`) |

Full list: `GET /api/runtime/assets/ship-elements.md` (served by the daemon).

## The manifest — declarative data + events

Every view has `.ship/views/<name>/manifest.yaml`:

```yaml
name: my-view
type: html

# What data the daemon loads into Jinja context on initial render
data:
  collections: all           # load all collection schemas
  items: $collection         # load items for the active collection signal

# What kernel events trigger SSE re-renders
events:
  - collection.item_created
  - collection.item_updated
  - collection.item_deleted
```

Available `data:` loaders: `collections`, `agents`, `skills`, `settings`, `events_data`, `specs`, `chat`, `pipelines`, `git`.

Events: any kernel namespace prefix (`collection.`, `job.`, `workspace.`, `config.`, etc.) or empty string for all events.

### `packs` — per-view bundle loading

The shell eager-loads only two JS assets on boot: `datastar.js` (reactivity) and `ship-elements.js` (the `<ship-*>` registry). Heavy island bundles — TipTap, CodeMirror, xterm — are ~5 MB combined and stay off the wire unless a view declares it needs them.

Declare them via `packs:` in the manifest. The daemon reads the list at render time and injects `<script type="module" src=...>` tags into the response; `mount.ts` rehydrates injected scripts after fragment navigation so they actually execute.

```yaml
name: skills
type: html
data:
  skills: true
events:
  - config.changed
packs:
  - editor    # provides <ship-code-editor> and <ship-markdown-editor>
```

Available packs:

| Pack | Bundles | Elements enabled |
|---|---|---|
| `editor` | `ship-code-editor.js` + `ship-markdown-editor.js` | `<ship-code-editor>`, `<ship-markdown-editor>` |
| `terminal` | `ship-terminal.js` | `<ship-terminal>` |

## Component packs

The `@ship/set` library is organized in layers:
- **Core** (`ship-button`, `ship-icon`, `ship-input`, etc.) — primitives, always loaded
- **Domain** (`ship-git-diff`, `ship-history-row`, `ship-pipeline-dag`, etc.) — domain components, loaded on demand via `packs:`
- **Packs** — cohesive bundles of domain components for a specific context (git, data, pipelines)

Domain packs live in `packages/ship-set/src/packs/<name>/` and are bundled as separate JS files. Declare them in the view manifest under `packs:`. Each pack ships its own CEM fragment so the catalog stays complete.

Current packs:
| Pack | Manifest key | Components |
|---|---|---|
| Git | `git` | `<ship-git-view>`, `<ship-file-row>`, `<ship-branch-row>`, `<ship-diff-pane>` |

When a view needs domain-specific UI that doesn't exist yet, the right move is to create or extend a pack component — not inline raw CSS/HTML in the template. File an `{#- SDK gap: pack/<name>: ... -#}` comment and escalate.

Only declare packs the view actually uses. The `ship lint views` linter enforces this in both directions: `missing-pack-declaration` (error) fires when a template uses a heavy-island tag without the matching pack, and `stale-pack-declaration` (warn) fires when a declared pack has no corresponding tag in the view. Unknown pack names are ignored by the daemon but not enforced by the linter (reserved for future additions like `viz`).

## Working with collections — the primary data story

Collections are structured data tables defined by JSON schema. They're the extension author's main data surface.

### Schema definition

`.ship/collections/task.jsonc`:
```jsonc
{
  "id": "task",
  "name": "Task",
  "fields": {
    "title":    { "type": "string", "required": true },
    "status":   { "type": "enum", "values": ["todo", "doing", "done"] },
    "priority": { "type": "number" },
    "done":     { "type": "boolean" }
  }
}
```

### Querying from templates

```jinja
{# Load ALL items #}
{% for task in ship_items('task') %}
  <div>{{ task.data.title }}</div>
{% endfor %}

{# Query with SQL pushdown (Django-style operators) #}
{% for task in ship_query('task', {
    status: 'active',
    priority__gte: 3,
    _sort: '-priority',
    _limit: 20
}) %}
  <div>{{ task.data.title }}</div>
{% endfor %}

{# Load schema for auto-forms #}
{% set schema = ship_schema('task') %}

{# List all available collections #}
{% for c in ship_collections() %}
  <option value="{{ c.id }}">{{ c.name }} ({{ c.field_count }})</option>
{% endfor %}
```

Filter operators: `field__eq` (shorthand: just `field`), `field__ne`, `field__gt`, `field__gte`, `field__lt`, `field__lte`, `field__in`, `field__nin`, `field__contains`, `field__exists`. `_or` for OR groups. `_sort` (prefix `-` for desc), `_limit`, `_offset`.

### CRUD actions

Use SDK macros — they output Datastar attributes composed onto elements:

```jinja
{# Create #}
<ship-button {{ create('task', '$form') }} {{ loading('_creating') }}>
  Create
</ship-button>

{# Update #}
<ship-button {{ update('task', item.id, '$form') }}>Save</ship-button>

{# Delete #}
<ship-button {{ delete('task', item.id) }}>Delete</ship-button>

{# Delete with confirmation #}
<ship-button {{ confirm_delete('task', item.id) }}>Delete</ship-button>
```

### Auto-form and auto-table

For full CRUD, use the generated macros:

```jinja
{# Generates typed inputs for every schema field, bound to $form #}
{{ form(schema, 'task') }}

{# Generates a data table with typed cells and per-row delete buttons #}
{{ table(schema, items, 'task') }}
```

## Reactivity — signals and SSE

### Signals (client state)

```jinja
{# Declare a signal with default value #}
<div class="ship-view"
     {{ signal('collection', 'task') }}
     {{ signal('_showForm', false) }}
     {{ signal('form', {}) }}>
  ...
</div>
```

Signals starting with `_` are **local** (not sent to server — use for UI state like `_showForm`, `_creating`).

### Reactive patterns

```jinja
{# Show/hide #}
<ship-card {{ show('$_showForm') }}>...</ship-card>

{# Toggle class #}
<div {{ toggle_class('active', '$tab == "details"') }}>...</div>

{# Reactive attribute #}
<ship-chip {{ attr('selected', '$status == "active"') }}>Active</ship-chip>

{# Two-way input binding #}
<ship-input {{ bind('form.title') }}></ship-input>

{# Display a signal value #}
<span data-text="$count"></span>
```

### Reactive side-effects — `data-effect`

When a view needs to run arbitrary code whenever signals change (canvas
redraws, DOM measurements, syncing to a non-Datastar-aware widget),
Datastar's built-in `data-effect` directive is the escape hatch.
**Reach for this instead of an inline `<script>` that polls
`window.ds.signals`.**

```jinja
{# Redraw a canvas whenever zoom or node state changes.
   The expression auto-subscribes to every $signal it reads,
   runs once on element connect, and re-runs on every change. #}
<canvas id="graph"
        data-effect="drawGraph(document.getElementById('graph'), $zoom, $nodes)"></canvas>

{# Sync a Datastar signal into a third-party widget's API. #}
<div id="editor"
     data-effect="window.editor?.setValue($draftText)"></div>

{# Log-only side effect for debugging. #}
<div data-effect="console.log('tab changed:', $tab)"></div>
```

Rules:

- The expression runs **once on connect** and again on **every
  dependency change**. Dependencies are the `$signal` references
  Datastar observes in the expression body.
- **No cleanup semantic** in the current Datastar build — if you
  return a function from the expression, it's discarded. That means
  `setInterval`/`addEventListener` patterns need explicit teardown
  wiring (e.g., keyed by a signal you can flip). Flag as an SDK gap
  when you hit it; most canvas/DOM-sync cases are idempotent and
  don't need cleanup.
- Keep expressions short. Long imperative blocks are a smell — either
  the work belongs in a custom element (encapsulates state) or the
  signals are structured wrong.

### SSE — long-lived reads

```jinja
{# Subscribe to the view's own SSE stream #}
<div {{ live('my-view') }}>
  <div id="my-view-content">Loading...</div>
</div>
```

The daemon's generic SSE handler reads your manifest `events:` list, subscribes to those kernel events, re-renders `partials/content.html.j2` on each, and pushes via Datastar's `datastar-patch-elements`.

**CQRS pattern:** reads flow over SSE (long-lived `@get`), writes are short-lived `@post`/`@delete` (the read stream pushes the visual update).

### Datastar usage depth

Today's views use Datastar shallowly — mostly `data-on`, `data-show`,
`data-bind`, `data-text`, `data-effect` (see above), and the wrapped
`@get`/`@post` actions. That's deliberate; it's what the current
patterns need. Other advanced Datastar primitives (store composition,
custom plugins beyond the built-ins) are in scope as the SDK matures,
but don't reach for them pre-emptively. When a pattern genuinely
needs it, file a `{#- SDK gap: ... -#}`, prototype once, and lift it
into a macro before copying it across views.

## Design tokens

Everything uses CSS custom properties from `view-tokens.css`:

| Category | Tokens |
|----------|--------|
| Spacing | `--space-1` through `--space-8` (4/8/12/16/20/24/32/40px) |
| Typography | `--text-xs` through `--text-4xl` |
| Font weight | `--font-weight-normal/medium/semibold/bold` |
| Radii | `--radius-sm/md/lg/xl/full` |
| Colors (semantic) | `--foreground`, `--background`, `--muted`, `--muted-foreground`, `--border`, `--card`, `--primary`, `--accent`, `--destructive`, `--success`, `--info`, `--warning` |
| Fonts | `--font-body`, `--font-display`, `--font-mono` |
| Motion | `--duration-fast/medium/slow`, `--ease-out/in/in-out` |

**Don't hardcode px or hex values.** Use tokens.

## Gotchas (hard-won)

### Ship-elements quirks

- `ship-select` uses native `<option>` children. `<ship-option>` does not exist.
- `ship-select` emits `ship-change`, not `ship-select-change`.
- `ship-button` uses `tone` for color meaning, `variant` for style. `variant="destructive"` is wrong.
- Icon-only buttons need `aria-label`.

### Jinja in the view env

- Auto-escape is **off** in view templates (so Datastar attribute values with quotes/slashes work).
- `tojson` filter doesn't exist in minijinja. Use `is string`/`is none` checks.
- `{% if %}` tags don't work inside `{{ macro() }}` string arguments. Use ternary: `(value if cond else "")`.
- URL macros from `_urls.html.j2` emit relative paths (`/api/...`), not absolute URLs.

### Datastar expressions

- String signal values are wrapped in single quotes: `$collection` evaluates to `'task'`, not `task`.
- Multiple statements: separate with semicolons. `data-on:click="$tab = 'new'; @get('/sse')"`
- Loading indicators pair with a signal that can be ANY name, conventionally `_creating`, `_loading`, `_sending`.
- Forward slashes in Jinja string values are HTML-escaped to `&#x2F;` when rendered inside HTML attributes. Branch names like `feat/x` in a `data-show="$branch == '{{ b.name | e }}'"` will never match because the rendered attribute contains `&#x2F;` but the signal holds the literal `/`. **Workaround:** avoid slashes in signal comparison keys — use the index in the branches array instead, or a slugified key. This is a known framework gap.

### CQRS patterns

- Writes (`@post`, `@delete`) don't need to render anything — the SSE read stream pushes the visual update via kernel events.
- One long-lived `@get(/sse)` per view. Multiple SSE connections is an anti-pattern.

## Example: full CRUD view in 50 lines

`.ship/views/tasks/manifest.yaml`:
```yaml
name: tasks
type: html
data:
  collections: [task]
  items: $collection
events:
  - collection.
```

`.ship/views/tasks/index.html.j2`:
```jinja
{% from '_ship.html.j2' import head, signal, show, live, create, loading, spinner, form, styles %}
<!DOCTYPE html>
<html>
<head>{{ head('Tasks') }}{{ styles() }}</head>
<body>
<div class="ship-view"
     {{ signal('_showForm', false) }}
     {{ signal('_creating', false) }}
     {{ signal('form', {}) }}>
  <div style="max-width: 72rem; margin: 0 auto; padding: var(--space-6) var(--space-4);">

    <h1 style="font-family: var(--font-display); font-size: var(--text-2xl);">Tasks</h1>

    <ship-toolbar style="margin: var(--space-4) 0;">
      <ship-button tone="accent" variant="solid"
        data-on:click="$_showForm = !$_showForm">
        <ship-icon slot="leading" name="plus"></ship-icon>New task
      </ship-button>
    </ship-toolbar>

    <ship-card {{ show('$_showForm') }} style="margin-bottom: var(--space-4);">
      {{ form(ship_schema('task'), 'task') }}
      <ship-separator></ship-separator>
      <div style="display:flex; gap:var(--space-2); margin-top:var(--space-3);">
        <ship-button tone="accent" variant="solid"
          {{ create('task', '$form') }}
          {{ loading('_creating') }}>Create</ship-button>
        <ship-button variant="ghost"
          data-on:click="$_showForm = false">Cancel</ship-button>
        {{ spinner('_creating') }}
      </div>
    </ship-card>

    <div {{ live('tasks') }}>
      <div id="tasks-content">Loading...</div>
    </div>
  </div>
</div>
</body>
</html>
```

`.ship/views/tasks/partials/content.html.j2`:
```jinja
{% from '_ship.html.j2' import table %}
<div id="tasks-content">
  {% if items | length > 0 %}
    {{ table(schema, items, 'task') }}
  {% else %}
    <ship-empty-state>
      <span slot="title">No tasks yet</span>
      <span slot="description">Click + New task to create one</span>
    </ship-empty-state>
  {% endif %}
</div>
```

That's it. Full CRUD with live updates, auto-generated form and table, proper button semantics, design tokens throughout. Edit the file, refresh, see changes.

## The iteration loop

Ship's view authoring is a co-evolving system. If you hit a pattern the SDK doesn't cover well, or a component that doesn't look right:

1. **Don't work around it in templates** — that's how the SDK rots.
2. **File the gap.** Add a `{#- SDK gap: ... -#}` comment in the template.

Gap comment formats:
```jinja
{#- SDK gap: component — ship-X doesn't support Y prop yet -#}
{#- SDK gap: macro — need a macro for Z pattern -#}
{#- SDK gap: pack/git — need ship-git-view component -#}
{#- SDK gap: filter — need tojson filter for array attributes -#}
{#- SDK gap: datastar — signal comparison with slash in value breaks -#}
```

The linter will surface these as warnings in `ship lint views` output so they're tracked.

3. **Escalate:** either (a) improve the macro in `_ship.html.j2`, (b) fix the ship-element component, or (c) add a Jinja filter in `view_filters.rs`.
4. **Dogfood first, document after.** Views must prove patterns before they become recommendations.

Today the SDK covers collections CRUD, forms, tables, navigation, actions, reactivity, live updates, filter chips, tabs, search, and confirmation. It doesn't yet cover inline edit, complex validation, optimistic UI, drag-and-drop. Those come next.

## Reference — all SDK macros

Import from `_ship.html.j2`.

### Page setup
- `head(title)` — standard `<head>` contents
- `styles()` — standard CSS utility classes

### Signals
- `signal(name, default)` — declare a reactive signal
- `show(expr)` — hide when expr is false (with display:none to prevent flash)
- `toggle_class(cls, expr)` — reactive class toggle
- `attr(name, expr)` — reactive attribute
- `bind(signal_name)` — two-way input binding

### Events
- `on(event, expr)` — generic event listener (use for custom events like `ship-change`)
- `live(view_name)` — long-lived SSE subscription
- `live_persistent(view_name)` — SSE with `requestCancellation: 'disabled'`

### Actions
- `get(url)` / `post(url, payload)` / `put(url, payload)` / `delete(url)` — HTTP actions
- `put_on_change(url, payload)` — PUT on input change
- `submit(url, payload)` — POST on form submit
- `navigate(url)` / `view(name)` — navigation

### Collection actions
- `create(collection, data)` — POST new item
- `update(collection, id, data)` — PUT item
- `delete(collection, id)` — DELETE item
- `confirm_delete(collection, id)` — DELETE with confirm dialog

### Job actions
- `create_job(agent, pipeline?)` — POST new job
- `emit(event_type, data)` — emit kernel event

### UI patterns
- `loading(signal_name)` — `data-indicator`
- `spinner(signal_name)` — inline spinner element
- `form(schema, collection)` — auto-generate form fields from schema
- `table(schema, items, collection)` — auto-generate data table
- `tab(signal, value)` / `tab_panel(signal, value)` — tab UI
- `chip(signal, value, view?)` / `chip_all(signal, view?)` — filter chips
- `search(signal, view, delay?)` — debounced search input
- `on_change_debounced(expr, delay?)` — debounced change handler
- `radio(signal, value)` — radio-style selection
- `confirm(message, action)` — confirm before action
- `section(title, icon?, link_href?, link_label?)` /
  `endsection()` — paired macros that open/close a `<ship-card>` with
  a standardized header: optional leading ship-icon + title on the left,
  optional right-side link pill (chevron + muted text). When
  `link_href` starts with `/api/runtime/view/<name>`, the link uses
  `@get(link_href)` with `.prevent` for SSE-aware navigation;
  otherwise it falls back to plain anchor semantics. Replaces
  hand-rolled `<ship-card padding="none">` + `<div slot="header" class="dash-section-head">` patterns.

## Reference — Jinja functions

- `ship_items(collection)` — all items from a collection
- `ship_query(collection, opts)` — SQL-pushdown query (see filter operators above)
- `ship_schema(collection)` — collection schema
- `ship_collections()` — all collection definitions
- `ship_agents()` / `ship_agent(id)` — agent configs
- `ship_skills()` — skill configs

## Reference — Jinja list filters

- `items | where('field', value)` — equality
- `items | where('field', '>=', value)` — comparison
- `items | sort_by('field')` / `items | sort_by('field', true)` — sort
- `items | take(n)` — limit

These work on any list — they're general utilities, not collection-specific. For collection queries, prefer `ship_query` (SQL pushdown vs memory filter).

## Full macro reference

The SDK macros defined in `apps/shipd/templates/_ship.html.j2` are
published as both a machine-readable manifest and a grouped Markdown
reference:

- `GET /api/runtime/assets/jinja-macros.json` — structured manifest
  (name, signature, args, summary, description, example, source
  location). Editor tooling should consume this for autocomplete and
  signature help.
- `GET /api/runtime/assets/jinja-macros.md` — human-readable reference
  grouped by category, with a table of contents.

Both are generated from the macro source by
`scripts/build-jinja-docs.mjs` (`pnpm build:jinja-docs`) and embedded
into the daemon at compile time — they never drift from the template.

## See also

- `ship-components` skill — the `@ship/set` library charter, including
  the **distribution test** (when does an element belong in the library
  vs stay a view-local composition?) and the `ship-list-tile` API.
- `datastar-views` skill — wire-level details for SSE framing,
  selectors, signal patches, and writing Datastar plugins when a view
  genuinely needs one.
- `.artifacts/specs/view-sdk-design.md` — SDK architecture decisions
- `apps/shipd/templates/_ship.html.j2` — the canonical macro source
- `.ship/views/data/` — reference implementation of the patterns above
