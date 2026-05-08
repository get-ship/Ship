---
group: Views
order: 1
title: Views
description: Custom UI in your Ship app — server-rendered Jinja templates with live reactivity. No build step.
---

A view is a folder you drop into `.ship/views/`. The runtime serves it at `http://localhost:9315/views/<name>/`. One template plus a small manifest, and your app has a new live-updating screen.

## What you get

<ship-card padding="md">
<strong slot="header">Server-rendered HTML</strong>
The browser receives ready-to-read markup. No client-side framework boot, no hydration step, no build pipeline.
</ship-card>

<ship-card padding="md">
<strong slot="header">Live reactivity</strong>
Bindings keep signals and DOM in sync; the runtime pushes updates over SSE. Type in an input — the view reflects it instantly.
</ship-card>

<ship-card padding="md">
<strong slot="header">Direct access to your data</strong>
Templates read collections, events, workspaces, and sessions through the view context. No API calls from the browser.
</ship-card>

<ship-card padding="md">
<strong slot="header">Ship's component library</strong>
`<ship-card>`, `<ship-input>`, `<ship-button>`, and the rest render with the platform's design system automatically.
</ship-card>

## When to author one

Reach for a view when you want:

- A UI tied to a collection — browse, edit, create items.
- A dashboard summarizing runtime state.
- A canvas surface where an agent renders HTML and you annotate, approve, or reply.

A view is the right tool when you want to extend the app with something new. It is **not** the place for a standalone marketing site, an arbitrary third-party app, or anything that needs its own build pipeline.

## A minimal view

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

Open `/views/hello/` in your browser. Type. The heading updates as you type. That's the whole loop — no server round-trip, no rebuild.

## Live updates from elsewhere

When your view declares `events` in its manifest, the runtime pushes matching events over SSE and re-renders the affected fragment automatically. Author once, watch updates flow in from agents, pipelines, and your own actions.

## Next

- [Editor integration](./editor-integration/) — set up the Ship LSP for inline lint diagnostics.
- [Setup view](./setup-view/) — the first-launch wizard, used as a worked example.
