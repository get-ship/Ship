---
group: SDK
title: SDK
section: reference
order: 1
description: Ship View SDK — the zero-JS, zero-CSS view-authoring surface. Jinja + Datastar + sui-components, wrapped in ~15 primitives.
---

Ship views are server-rendered reactive HTML. You drop
`.ship/views/<name>/index.html.j2` on disk, the daemon compiles it with
Minijinja, the browser hydrates it with Datastar, and the
`<sui-*>` component library styles and behaves it.

The thesis: **zero raw JavaScript, zero raw CSS**. The framework gives
you primitives that cover 95% of view authoring. The remaining 5% is a
well-known escape hatch — authoring a web component — and belongs in
the `ship-components` skill, not in your view.

## Three layers

Every view is a composition across three layers. Keep them separate.

| Layer | What it does | How you touch it |
|---|---|---|
| Jinja (Minijinja) | Structure, loops, partials, server-side data access. Runs on the daemon, once per render. | `{% for %}`, `{% if %}`, `ship_items()`, `ship_query()`, `ship_schema()`. |
| Datastar | Client reactivity: signals, expressions, attribute bindings, SSE patches. Runs in the browser on every interaction. | SDK macros only: `signal()`, `show()`, `bind()`, `on()`, `live()`, `get()`/`post()`/… |
| sui-components | Visual + behavioral components. Style, focus, a11y are owned by the element. | `<sui-button>`, `<sui-card>`, `<sui-list-tile>`, `<sui-input>`, `<sui-select>`, … |

When you want an action, you almost never write `data-on:click=...`
directly. You call a macro — `create()`, `navigate()`, `emit()`,
`feedback()` — that emits the correct Datastar attributes plus the
right loading/indicator plumbing.

## The SDK surface

The live SDK is published two ways:

- `GET /api/sdk` — JSON manifest of every macro, every URL,
  every registered view context and collection.
- `GET /api/assets/jinja-macros.md` — human-readable macro
  reference, generated from the template source so it never drifts.

The curated pages under this skill are the same material, organized
for reading and retrieval rather than raw enumeration.

## Where to go next

- Start with **[Primitives](./primitives.md)** if you're building a
  view right now.
- Read **[Reactivity](./reactivity.md)** once to internalize the
  Datastar model Ship exposes.
- Keep **[Authoring loop](./authoring-loop.md)** open in a tab — it
  has the curl + MCP incantations for the tight feedback cycle.
- Skim **[Rules](./rules.md)** before opening a PR.
- Jump to **[Macro catalog](./catalog.md)** when you need a reference
  card for every macro at once.
