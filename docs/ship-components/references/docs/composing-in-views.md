---
group: Components
title: Composing components in views
description: Using your custom .ship components inside view templates.
order: 5
---

A `.ship` component you author becomes a custom HTML element. Use it in any view by writing the tag.

## Basic use

Drop the tag into a view template:

```html
{# .ship/views/agents/index.html.j2 #}
<div class="agent-list">
  {% for agent in ship.agents.summaries().items %}
    <agent-card agent-id="{{ agent.id }}"></agent-card>
  {% endfor %}
</div>
```

Pass props as attributes. Attribute names are the kebab-cased version of the prop names declared in the component's script:

| Component declared | View attribute |
|---|---|
| `const { agentId } = props` | `agent-id="..."` |
| `const { fetchUrl } = props` | `fetch-url="..."` |
| `const { minChars = 1 } = props` | `min-chars="..."` (or omit for default) |

## Listening to events

Components emit events via `emit('name', payload)`. Consume with `data-on:<name>`:

```html
<app-typeahead
  fetch-url="/api/v1/agents/search"
  placeholder="Search agents..."
  data-on:select="$selected = evt.detail"
/>
```

`evt` is the `CustomEvent`. `evt.detail` is the payload.

## Wiring the bundle

The component's compiled JS is served at `/_ship/components/<tag>.js`. Either:

### Option A: per-view script tag (simple, explicit)

```html
{# .ship/views/agents/index.html.j2 — head section #}
<script type="module" src="/_ship/components/agent-card.js"></script>
```

The browser fetches the module on view load and the custom element is registered before any tags are encountered. Good for views with one or two components.

### Option B: pack declaration in manifest (forthcoming)

Long-term the view's `manifest.jsonc` will declare `packs:` that include component bundles. Today, packs cover the core element library only — author-defined components use the per-view script tag pattern.

## Slots

Components that expose slots accept HTML children:

```html
<app-typeahead fetch-url="/api/v1/agents/search">
  <li slot="item" data-text="item.name + ' (' + item.role + ')'"></li>
</app-typeahead>
```

The slot name (`item`) and the variables exposed inside the slot (e.g. `item`, `query`) are documented per-component.

## Two instances on a page

Each component instance gets its own scoped signal namespace — typing in one doesn't affect the other:

```html
<app-typeahead fetch-url="/api/v1/agents/search" placeholder="Search agents..." />
<app-typeahead fetch-url="/api/v1/skills/search" placeholder="Search skills..." />
```

The framework auto-generates instance IDs and rewrites signal names internally. Authors don't manage this.

## Inspecting in dev

Daemon serves the compiled JS at:

```
GET /_ship/components/<tag>.js
```

Curl it to see what got generated:

```bash
curl http://127.0.0.1:9315/_ship/components/app-typeahead.js | head -40
```

Useful for debugging "the component isn't behaving" — confirm the served HTML/JS matches what you authored.

## Linking back to view authoring

For view-side concerns (mounting, routing, layouts, `_partials`), see `ship-views`. This page covers only the consumer side of components; for authoring see [Overview](./index.md) and [SFC format](./format.md).
