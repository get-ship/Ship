---
group: Views
order: 1
title: Views Overview
description: The extension mechanism for user-facing HTML surfaces in Ship. Daemon-rendered Jinja templates with ship-elements components and Datastar reactivity.
---

A view is a folder in `.ship/views/<name>/` that shipd serves at `http://localhost:{port}/views/<name>/`. One `index.html.j2` template plus a `manifest.json` is enough to ship a live-updating UI. No build step, no framework to install — drop the files in place, reload, see your changes.

## What a view gives you

- **Server-rendered HTML** — the browser gets ready-to-read markup; no client-side routing or framework boot.
- **Live reactivity** — Datastar attributes on elements + SSE from shipd keep signals and DOM in sync. Type in an input, data updates immediately.
- **Ship component library** — `<ship-card>`, `<ship-input>`, `<ship-button>`, and the rest render with the platform's design system automatically.
- **Access to runtime data** — Jinja templates read collections, events, workspaces, sessions directly via the view context. No API calls from the browser.
- **Subscription to events** — the manifest declares which event types the view listens to; shipd delivers them over SSE so the template re-renders automatically.

## When to author a view

Views are the right choice for:

- A UI surface tied to a collection (browse/edit/create items).
- A dashboard summarizing runtime state (workspace status, job queue, event log).
- A canvas-style interaction where an agent renders HTML and the user annotates or approves.
- Anything you'd otherwise embed in Studio but that doesn't need Studio's SPA infrastructure.

They're **not** for: arbitrary standalone web apps (use `apps/` for those), static marketing pages (`apps/site`), or anything that needs its own build pipeline.

## Minimal example

`.ship/views/hello/manifest.json`:

```json
{
  "name": "hello",
  "type": "html",
  "description": "Hello world view"
}
```

`.ship/views/hello/index.html.j2`:

```jinja
{% from '_ship.html.j2' import head, signal %}
<!DOCTYPE html>
<html>
<head>{{ head('Hello') }}</head>
<body>
<div class="ship-view" {{ signal('name', 'world') }}>
  <ship-card>
    <h1>Hello, <span data-text="$name"></span>!</h1>
    <ship-input data-bind="name" placeholder="Your name"></ship-input>
  </ship-card>
</div>
</body>
</html>
```

Load `/views/hello/`. Type. The heading updates as you type. That's the full loop — no server round-trip, no build step.

## Request flow

When the browser requests `/views/hello/`:

1. shipd's view handler looks up `.ship/views/hello/manifest.json`.
2. Reads `index.html.j2`.
3. Renders it through Jinja with the view context — collections, events, workspace state.
4. Serves the rendered HTML. Includes the ship-elements bundle and Datastar runtime via script tags.

Subsequent live updates flow over SSE — shipd keeps a connection open, pushes new events as they arrive in the kernel, and the page's Datastar directives re-render the affected regions.

For an event-driven update: the manifest declares `events: ["collection.item_created"]`; when a matching event fires in the kernel, shipd pushes a fragment over SSE; the page swaps it in place.

## Views vs Studio vs static sites

| Surface | Purpose | How authored |
|---|---|---|
| **View** (`.ship/views/*`) | Extend Ship's daemon-served UI for a specific domain | Jinja template + manifest |
| **Studio** (`apps/web`) | The full-featured SPA — agents, settings, skills IDE | TanStack Start app |
| **Marketing site** (`apps/site`) | Public-facing content | Astro site |
| **ship-set site** (`apps/ship-set`) | Component gallery | Astro site |

Views are the extension mechanism. Studio is a first-party consumer of the same kernel+runtime, not an extension — you don't author "a new Studio" the way you author "a new view."

## Canvas views

A special case of views. Canvas views are ephemeral, agent-owned surfaces for bi-directional interaction — an agent emits HTML, the user annotates/approves/clicks, the agent sees the feedback. Canvas artifacts live in the `files` collection tagged `collection_id: "canvas"`; the `view.open` MCP tool opens one.

See the `ship-views` bundled skill (`.ship/skills/ship-views/SKILL.md`) for authoring patterns including canvas. This doc skill covers what Ship provides; the bundled skill covers how to use it well.

## Related

- [Manifest & SDK reference](/reference/views/reference/) — every field, every macro.
- [Views runtime](/reference/views/runtime/) — how shipd serves views.
- [Collections](/reference/collections/) — the most common data source for views.
- [ship-elements](https://getship.dev/set) — the component library views render with.
