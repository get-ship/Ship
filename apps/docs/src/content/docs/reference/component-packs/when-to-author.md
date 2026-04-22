---
title: "When to author"
description: "The decision rule. Three tables that tell you whether you need a pack or you're about to ship slop."
sidebar:
  label: "When to author"
  order: 2
---
Most proposed components should not exist. The tables below are the
filter.

## You do NOT need a pack if…

| Situation | Do this instead |
|---|---|
| You want a "row" with icon, primary text, sub-text, and a trailing control. | `<ship-list-tile>` with `leading` / `meta` / `trailing` slots. |
| You want a "card" that groups content under a header. | `<ship-card>` with the `header` slot. |
| The same row/tile composition repeats across two views. | Extract it as a Jinja partial in `.ship/views/<view>/_components/*.j2`. |
| You want to change a component's corner radius, padding, or color. | Override the CSS custom property exposed by the element (`--ship-button-radius`) or re-theme via tokens. |
| You want to render status-colored text or a pill. | `<ship-badge tone="…">` or `<ship-chip tone="…">`. The fixed tone vocabulary already covers `success / warning / destructive / info / accent / muted`. |
| You want a custom chart. | Check `packages/ship-set/src/web-components/viz/` first — sparkline, line, area, bar, donut, heatmap, gauge, meter, progress-ring are already there. |
| You want a code editor, terminal, or markdown editor. | Use the existing islands: `ship-code-editor`, `ship-terminal`, `ship-markdown-editor`. |
| You want a view-local JS hook. | You don't. Either a macro, a `data-effect="…"`, or an existing element covers it. See the [SDK rules](/reference/sdk/rules/). |

## You DO need a pack if…

| Situation | Why a pack is warranted |
|---|---|
| You are wrapping a heavy third-party engine not already in `islands/` (e.g. a graph library, a 3D viewer). | Islands exist to bundle non-trivial engines separately. One engine = one island component = one pack bundle. |
| You are distributing a commercial pro-pack that customers install into their Ship. | Packs are the distribution unit. The `packs: [...]` manifest key is how views opt in. |
| You have a platform concept with its own structural shape that domain-`<ship-*>` elements don't cover (e.g. `ship-pipeline-step` covers pipeline-step; a new concept needs the same treatment). | Domain components encode concept shape. If the concept is Ship-wide, it belongs in `@ship/set` under `domain/`. If it's vendor-specific, it's a pack. |
| You need a component that composes existing primitives but MUST be reusable across **many** views you don't own (third-party views consuming it). | Cross-tenant reuse is the line between "Jinja partial" and "distributed component." |
| You own an internal app whose surface genuinely outgrew primitives — e.g. a bespoke editor, a domain-specific canvas. | Internal packs are legitimate; they live alongside first-party packs in `packages/ship-set/src/packs/`. |

## You are in pro-pack territory if…

| Signal | Implication |
|---|---|
| You are selling the pack, or licensing it per-seat/per-install. | Publish as a standalone npm package with a peer-dep on `@ship/set`. Don't land it in the Ship monorepo. |
| Customers install it alongside their own views. | Ship a CEM (`custom-elements.json`) so their agents can discover the tags. |
| The pack has its own release cadence and versioning. | Separate repo, separate changelog, separate pipeline. |
| The pack needs a runtime license check. | That's a product-level concern. Keep it off the component surface; gate at install or at daemon registration. |

If your situation isn't in the "DO need" or "pro-pack" tables, it's in
the "do not need" table. That's the default answer.
