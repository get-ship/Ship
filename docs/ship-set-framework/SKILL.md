---
name: ship-set-framework
stable-id: ship-set-framework
description: Use when authoring or modifying `.html.set` views and components for Ship — the daemon-rendered, signals-based, agent-authorable UI framework that succeeds the j2/Datastar prod stack. Covers template surface, controller shape, template + controller hooks, server-side rendering, and the three-role architecture (template / controller / data).
tags: [ship, set, framework, ui, views, components, html-set, ssr]
authors: [ship]
audience: internal
---

# Ship Set Framework

The framework Ship uses to author UI surfaces. `.html.set` files compile AOT to ESM and render server-side from `shipd`; a small attach-only client runtime wires events, signals, and SSE patches over the rendered DOM.

This skill is **the** authoring reference. If a doc elsewhere conflicts with what's here, this wins; the other doc is being phased out.

## When to use this skill

- Writing or editing a `.html.set` view or component
- Writing or editing a paired `.ts` controller
- Building a template or controller hook
- Wiring a `<data>` subscription to a `ship.*` SDK call
- Designing an SSR-able expression or wondering why something isn't SSR'd
- Reviewing a port of a j2/Datastar view to ship-set

## Architecture in three sentences

Ship views have three roles: **template** (the `.html.set` file — structure + declarative wiring), **controller** (an optional paired `.ts` file — state, derived, handlers, lifecycle), and **data** (`<data>` elements in the template that subscribe to the daemon). Together they map cleanly to MVVM: the daemon is the model, the template is the view, the controller is the view-model. The three roles do not overlap — every authoring decision goes in exactly one of them.

## Documents in this skill

- **[index.md](references/docs/index.md)** — architecture, the three-role split, MVVM mapping, the shared view scope, the discipline. Read first.
- **[template.md](references/docs/template.md)** — `.html.set` directive surface: `<state>`, `<data>`, `<component>`, `<prop>`, control flow, `on:`, `bind:`, `<use-*>`, slots.
- **[controller.md](references/docs/controller.md)** — paired `.ts` shape: three tiers (no controller / object / function), six sections (state / derived / on / use / mount / unmount), scope rules.
- **[hooks.md](references/docs/hooks.md)** — template hooks (`<use-*>`) vs controller hooks (`use:` namespace). When to reach for which. Authoring contracts.
- **[data-subscriptions.md](references/docs/data-subscriptions.md)** — `<data>` is canonical. Live envelope shape. Pass signals, not closures.
- **[ssr-hydration.md](references/docs/ssr-hydration.md)** — Rust IR render walker, attach-only client runtime, the renderable expression subset, marker contract.
- **[examples.md](references/docs/examples.md)** — counter (no controller), agents (function controller + two hooks), form (controller hook + two-way binding).
- **[status.md](references/docs/status.md)** — what's stable, what's in active dev, what's not yet implemented. Honest accounting.

## Phased-out adjacent skills

These remain current for shipping but are being replaced. New work goes in this skill; production porting tracks against [status.md](references/docs/status.md).

- **ship-views** — j2 templates + Datastar signals. The current prod surface; will be ported view-by-view.
- **ship-components** — `.ship` SFC three-block format. Dead. New components are `.html.set`.
- **ship-component-packs** — packs of `.ship` components. Pack distribution survives; the `.ship` shape inside does not.
