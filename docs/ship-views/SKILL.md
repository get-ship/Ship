---
name: ship-views
stable-id: ship-views-docs
description: Reference docs for Ship views — the extension mechanism for user-facing, daemon-rendered HTML surfaces. Covers manifest format, SDK surface, and runtime internals.
tags: [ship, views, extension, reference]
authors: [ship]
---

# Ship Views

Views are the extension mechanism for user-facing surfaces in Ship. You author a Jinja template + manifest in `.ship/views/<name>/`; shipd serves it as a live-updating HTML page. The user sees your UI; ship-elements render the components; Datastar keeps state reactive; SSE pushes updates.

Three pages for views:

- **[Overview](/ship-views/)** — what a view is, when to author one, how a request flows through shipd.
- **[Manifest & SDK reference](/ship-views/)** — `manifest.json` fields, the `_ship.html.j2` macro surface, signal/bind/event conventions.
- **[Runtime](/views/runtime/)** — shipd's view serving stack: resolve, render, session registry, SSE topics, ship-elements bundling.

For **authoring custom components** (`<my-tag>` elements you write to compose into views), see the **`ship-components`** skill — `.ship` single-file components with props, signals, lifecycle, and scoped styles.

The operational counterpart is `.ship/skills/ship-views/SKILL.md` — a bundled skill agents load when authoring views. Different audience, overlapping content. The bundled skill teaches you to write a view; this reference describes what Ship provides.
