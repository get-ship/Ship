---
group: Component Packs
title: Component Packs
section: reference
order: 1
description: The escape hatch — authoring reusable web-component packs for Ship. Not the happy path for view authors.
---

Component packs are Ship's **escape hatch**, not its default. Before
anything on this page applies, confirm that you can't solve the
problem with the primitives you already have.

## The core message

Ship's default authoring surface is three things stacked:

1. **View primitives** — `.html.j2` templates + SDK macros
   ([ship-sdk](/reference/sdk/)).
2. **`<ship-*>` elements** from `@ship/set` — buttons, cards,
   list-tiles, inputs, charts. See the compact catalog at
   `GET /api/runtime/assets/ship-elements.md`.
3. **Design tokens** — `var(--surface-1)`, `var(--tone-success-fg)`,
   `var(--radius-md)`. Themes override them ([ship-themes](/reference/themes/)).

That stack covers **90%+ of view work with zero custom components.**
The entire point of the SDK is that happy-path users never see this
skill.

## When a pack is the right answer

Reach for a pack when:

- You are building a **pro-pack product** for external distribution —
  a commercial bundle that slots into a customer's Ship install.
- Your internal app needs components that **genuinely don't fit** the
  primitive + slot model (e.g. a custom graph editor, a domain-specific
  simulator viewer).
- You wrap a **third-party engine** that isn't already in
  `packages/ship-set/src/web-components/islands/` — Monaco, xterm, and
  tiptap are already there, so don't re-wrap them.

## When a pack is NOT the right answer

Do not author a pack when:

- A `<ship-list-tile>` with `leading` / `meta` / `trailing` slots would
  do. Every row/card pattern in Ship today is a list-tile composition.
- A view partial (`.ship/views/<view>/_components/*.j2`) would do.
  Jinja partials are the correct home for view-local compositions.
- A CSS custom-property override on an existing element (e.g.
  `--ship-button-radius`) would do.

If you catch yourself writing `<ship-agent-row>` or `<ship-thing-card>`,
re-read the [distribution test](./authoring.md#the-distribution-test).

## How to use this reference

- [When to author](./when-to-author.md) — the decision tables.
- [Authoring a component](./authoring.md) — the template.
- [Manifest & distribution](./manifest-and-distribution.md) — how
  views opt in, how bundles ship, how pro-packs are published.
- [Testing](./testing.md) — the minimum every pack component ships.
