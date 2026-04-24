---
name: ship-sdk
stable-id: ship-sdk
description: "Author Ship views with zero-JS, zero-CSS intent. The SDK wraps Datastar + Minijinja behind ~15 primitives. This is the load-this-first skill for view authoring."
tags: [ship, sdk, views, jinja, datastar, authoring]
authors: [ship]
---

# Ship SDK

The Ship View SDK is how you build UI on Ship. You write a single
`.html.j2` file per view, import macros from `_ship.html.j2`, and
compose `<ship-*>` elements. The SDK hides Datastar and design tokens
behind named primitives so a view never contains raw `<script>`, raw
`<style>`, or hand-rolled `data-signals:`/`data-on:*` strings.

Humans and deep-research agents land here. View-authoring agents load
this skill first, then reach for the `ship-views` impl skill for the
live template rules.

## Reference pages

- [Overview](./references/docs/index.md) — thesis and the three-layer
  mental model (Jinja structure, Datastar reactivity, ship-elements
  components).
- [Primitives](./references/docs/primitives.md) — the ~15 macros that
  cover 95% of view authoring. What each does, when to reach for it,
  anti-patterns, examples.
- [Reactivity](./references/docs/reactivity.md) — the Datastar subset
  Ship endorses: signals, expressions, attribute bindings, events,
  actions, SSE patches, error surfacing.
- [Authoring loop](./references/docs/authoring-loop.md) — SDK
  discovery, scaffolding, hot-reload, lint, dry-run, runtime errors.
  The feedback loop that makes authoring fast.
- [Rules](./references/docs/rules.md) — hard rules for view authors.
  No inline script/style, tokens always, pack declarations explicit.
- [Macro catalog](./references/docs/catalog.md) — every macro in the
  SDK, grouped. Rendered from the canonical spec at
  `packages/ship-set/sdk/sdk-macros.json`.

## See also

- `.ship/skills/ship-views/SKILL.md` — impl skill. The exact rules a
  view-authoring agent runs against.
- `.ship/skills/ship-components/SKILL.md` — authoring web components
  (the escape hatch when the SDK genuinely doesn't cover a pattern).
- `docs/ship-themes/` — tokens and theme catalog. Referenced from the
  Rules page.
- `GET /api/sdk` — live machine-readable SDK manifest.
- `GET /api/assets/jinja-macros.md` — generated macro reference
  served by the daemon.
