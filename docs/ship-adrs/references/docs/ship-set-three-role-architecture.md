---
title: "Ship Set — Three-Role Architecture"
description: "ADR: every authoring decision in the ship-set framework lands in exactly one of three roles — template, controller, or data subscription. No fourth role; edge cases combine the three."
group: ADRs
order: 50
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, architecture, framework, discipline]
---

# Ship Set — Three-Role Architecture

## Context

The ship-set framework replaces the j2 + Datastar prod view stack and the dead `.ship` SFC three-block format. Two prior framework attempts (`sail`, the SFC) fragmented authoring across script blocks, template blocks, style blocks, and ad-hoc data subscriptions — authors and agents reached for whichever shape felt closest, the framework drifted, and the discipline that was supposed to make agent-authored UI legible vanished.

The framework is designed for AI authoring as much as human authoring. Agents will produce tens of views in parallel; if there are *multiple* valid places to put a piece of state, an agent with imperfect taste will scatter the same logic across all of them. The framework has to make the right choice obvious by leaving only one valid choice per concern.

## Decision

Every authoring decision lands in exactly one of three roles:

| Role | File | Job |
|---|---|---|
| **Template** | `view.html.set` (or `<tag>.html.set`) | Structure + declarative wiring (`<data>`, `<state>`, `<use-*>`, `on:`, `bind:`, `{#if}` / `{#for}` / `{#match}`) |
| **Controller** | `view.ts` (or `<tag>.ts`) — optional | Behavior — `state` / `derived` / `on` / `use` / `mount` / `unmount` |
| **Data subscription** | `<data>` element in template | The bridge to `shipd` (`source="ship.X.Y(...)" [live]`) |

The roles do not overlap. The mapping to MVVM is direct:

- **Model** = the daemon (`shipd`). Out of band — not a framework concept; the SDK is the seam.
- **View** = the template. Markup + declarative wiring.
- **ViewModel** = the controller. State + derived + handlers + lifecycle + composable hook bundles.

### Hard rules

1. **Mutation lives only in `on.*` handlers.** Templates and `derived.*` sections receive a read-only state proxy. Statically enforced via the type split between `MutableState<S>` and `ReadonlyState<S>`. Lint-enforced via a runtime warning when a handler writes a key that isn't in the controller's `state:` section.
2. **Subscriptions live in `<data>` elements**, never in controller code. Don't call `ship.*` outside `<data>`. Hooks may *consume* a data signal but never *create* one.
3. **Templates declare; controllers do.** Anything declarative goes in the template. Anything imperative goes in the controller.
4. **Six controller sections, no others** — `state`, `derived`, `on`, `use`, `mount`, `unmount`. No `effect`, `watch`, `computed`, `data`, `live`, `task`. If a case doesn't fit one of the six, the answer is almost always a combination of existing sections.
5. **Three roles, no fourth.** When an edge case arises, the question is *can it be expressed with the three we have?* If yes, do that. If not, it's a deliberate-widening discussion — never an accommodation.

## Alternatives considered

### Single-file component (SFC)

Everything in one `.html.set` file with `<template>`, `<script>`, `<style>` blocks. Vue / Svelte shape.

**Rejected** because:

- The `.ship` SFC attempt (now phased out — see `ship-components` skill) showed that one file with three slots makes parsing harder, tooling harder, and authors more likely to mix concerns within a section. SFCs only work when there's a single compiler the framework owns end-to-end; ship-set's authoring model is plain HTML + plain TS + plain CSS, where each tool (LSP, formatters, esbuild) sees a familiar file.
- Splitting into three sibling files (`view.html.set` + `view.ts` + `view.css`) lets every existing tool work on the right file without a custom plugin: LSP on .ts, htmlhint-style tools on .html.set, plain CSS tooling on .css. The "co-location" benefit of SFC is preserved by filename pairing.

### Free-form controller (the precursor `setup() → bindings`)

`view<Bindings>(setup)` returning whatever object the author wanted in template scope. The shape we shipped first, before this ADR.

**Rejected** because:

- The shape is too open. Authors put state, deriveds, handlers, subscriptions, and lifecycle code wherever in the setup body; agents do worse. The flat-bindings return collapses concerns that should be visibly separated.
- No type-level enforcement of the mutation discipline. A handler that writes state and a derived that *reads* state look identical — both are arrow functions returned from setup.
- The single shipped view (`views/agents/`) under that shape was readable but didn't scale; a more complex view (the git port) needs the named-section structure to stay legible.

The `view()` factory has been removed. Every shipped view uses `controller(def)` with the six-section shape.

### Subscriptions in controller (`live: { ... }`)

A `live:` section in the controller that declares server subscriptions, mirroring the existing `state` / `derived` / `on` shape.

**Rejected** because:

- The author can no longer see, at the top of the template, what server channels the view depends on. Subscriptions hidden inside controller code are a real reading-code regression for views that are mostly about live data.
- Once `<data>` is canonical, an `as-well-as` controller path multiplies the rules without simplifying any case. Two doors for one job.

The `<data>` element is the single sanctioned bridge to `ship.*`. Hooks that operate on data (filter, select, sort) consume an existing `<data>` signal; they don't create new ones.

### A fourth role for "platform side-effects" (kbd, click-outside, document title, …)

Considered: a `platform:` section in the controller, or a separate config file, that declares document-scope listeners and host integrations.

**Rejected** because the existing template-hook surface (`<use-*>` elements with attributes and child config) covers the use case more legibly. A `<use-kbd>` element in the template makes the keyboard shortcuts visible alongside the structure they operate on; a `platform:` section in TS hides them in another file.

## Consequences

### Positive

- **One canonical shape per concern.** Agents and humans authoring under this framework converge on the same patterns. Code review becomes "is this code correct?" not "is this code in the right place?"
- **Statically enforceable discipline.** TypeScript's variance over `ReadonlyState<S>` vs `MutableState<S>` rejects the most common discipline violation (writing state from a derived) at compile time.
- **Visible subscriptions.** The top of every `.html.set` file lists every `ship.*` call the view makes via `<data>` elements. No hidden-in-hooks data dependencies.
- **Composable hooks (controller + template) without two-doors-for-one-job.** Each category has its own home.

### Negative

- **The fourth-role temptation is real.** Edge cases will keep arising — async lifecycles, cross-view shared state, observer integrations. Holding the line costs taste and willpower.
- **Authors who want SFC-style co-location are pushed toward filename pairing.** Three sibling files is more cognitive overhead than one file with three blocks; the framework absorbs the trade by keeping each file plain HTML / plain TS / plain CSS.
- **The `<data>`-only-in-template rule means subscriptions are duplicated across views that share a data dependency.** The trade is intentional; visibility wins over DRY.

### Open questions

Tracked in `docs/ship-set-framework/references/docs/status.md`. Notable items that touch the architecture:

- Reactive `<data>` arg re-subscribe when args reference signals.
- LSP `use:`-namespace awareness (deep references like `list.filtered`).
- Compiler-emitted `.d.ts` per `.html.set` so the controller's `Scope` generic can be inferred.

None of these revisit the three-role split. They tighten the existing surface.

## References

- Skill: `docs/ship-set-framework/SKILL.md` and `docs/ship-set-framework/references/docs/index.md`.
- Set authoring cheat sheet: `.ship/skills/set-authoring/SKILL.md`.
- Phased-out skills (do not author against): `docs/ship-views/`, `docs/ship-components/`, `docs/ship-component-packs/`.
