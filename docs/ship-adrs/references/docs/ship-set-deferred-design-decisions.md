---
title: "Ship Set — Deferred Design Decisions"
description: "ADR: a registry of design alternatives the ship-set framework has explicitly considered and chosen NOT to ship in v1, with the reasoning. Future contributors revisit these only with a forcing function — not as polish for its own sake."
group: ADRs
order: 52
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, framework, deferred, decision-log]
---

# Ship Set — Deferred Design Decisions

## Context

The framework's design discussions have surfaced many shapes that *seem* like wins until you trace them. Once removed from the design conversation, those rejected alternatives are gone — the next agent or contributor doesn't see them and is liable to re-propose the same shape with the same trade-offs.

This ADR is a registry of "we considered this and said no, here's why." Each row is short. The point isn't a polished essay per item; it's so the next person doesn't redo the analysis.

Revisit any of these only when a real forcing function arises (a view we can't write cleanly, a measured ergonomic gap, a security or perf issue). Polish for polish's sake leaves the discipline weaker, not stronger.

## Decisions deferred

### `<use-title>` — reactive document title

**Considered.** A template hook that sets `document.title` reactively from a template-fragment child:

```html
<use-title>{count()} agents — Ship</use-title>
```

**Rejected for v1** because in practice `document.title = '…'` from a controller `mount` is one line and rarely changes per-view. Local-daemon UIs often live in a single window where the title rarely matters. Theoretical-good-to-have, not load-bearing.

**Revisit when** a real view needs reactive title that's awkward to express in `mount`.

---

### `<use-clipboard>`, `<use-focus-trap>`, `<use-hotkeys>`, `<use-idle>`, `<use-online>`

**Considered** — pre-emptively shipping these in the v1 template-hook library to mirror the Stimulus / Alpine "batteries included" surface.

**Rejected for v1** because every primitive in the library is a future maintenance burden, and we don't have a real view that needs them. Three template hooks ship today (`<use-kbd>`, `<use-click-outside>`, `<use-shell>`) — each tied to a real authoring need surfaced by the agents and git ports.

**Revisit when** a real view needs one. Each one is ~50–100 LOC; adding on demand keeps the library focused.

---

### `useTable`, `usePagination`, `useSort`

**Considered** — pre-emptively shipping these in the v1 controller-hook library.

**Rejected for v1**, same reasoning. Three controller hooks ship today (`useFilteredList`, `useSelection`, `useForm`) — each load-bearing for a view that exists.

**Revisit when** a real view needs one. The compose-by-namespace pattern is proven; adding hooks is a mechanical extension once a real use case shows up.

---

### Action-by-name shorthand for DOM event handlers

**Considered.** Phoenix LiveView-style:

```html
<button on:click="select" on:click-arg={a.id}>
```

…to remove the closure-over-iteration-args pattern (`<button on:click={() => sel.select(a.id)}>`).

**Rejected** because the two patterns serve different jobs:

- DOM event handlers (`on:click={fn}`) integrate with the web platform — event objects, modifiers, computed args. The closure form earns its existence.
- Template-hook actions (`<on action="select">` inside `<use-kbd>`) are declarative bindings that don't have closure scope. Name-by-string is correct *for that job*.

Forcing a single shape costs more than the inconsistency. The two doors do different work.

**Revisit when** a real ergonomic-tax measurement says otherwise — currently the closure form is idiomatic and authors handle it cleanly.

---

### Shadow DOM as the default for components

**Considered** — making `<component shadow>` implicit so every ported primitive defaults to shadow-DOM isolation.

**Rejected** because shadow DOM is appropriate for primitives that own a tone-matrix CSS surface (status-dot, button, badge), but unnecessary for components that style via class names within a parent's stylesheet. Light DOM is the default; `shadow` is opt-in. The runtime supports both.

The first ported primitive (`<set-status-dot>`) opts in; the next dozen will opt in or out per case.

---

### `<state>` writable from controller `state:` (auto-bridge)

**Considered.** Make the controller's writable-state proxy bridge to template-declared `<state>` signals so authors can mix `<state>` and controller `state:` freely.

**Rejected** because the bridge would mask a real authorial confusion. `<state>` and controller `state:` are *different* — one is template-trivial, the other is coordinated. When the same key appears in both, the framework can't tell which authoritatively owns it.

The current rule: a handler's writable proxy only mutates keys declared in the controller's `state:` section. Writing to a key that lives in template `<state>` triggers a runtime warning pointing at the right fix (move the declaration to the controller).

**Revisit when** authoring data shows a meaningful frequency of "I needed a template-side `<state>` *and* controller mutation" cases. Today the warning is sufficient.

---

### Cross-view shared state as a framework primitive

**Considered.** A `store:` section, a global `useStore()` hook, a "pinia-style" reactive container the framework provides natively.

**Rejected** because cross-view shared state is just a controller hook instantiated once and imported wherever it's used. ES module instance reuse + `useFilteredList`-style composition cover it. No new primitive needed.

**Revisit when** a real cross-view dependency shows that the import-and-instantiate pattern is meaningfully worse than a native primitive.

---

### Effect lifecycle (`effect:` section)

**Considered.** A seventh controller section for "run a side effect when this signal changes," sister to `derived` but for side effects rather than memoized values.

**Rejected** because:

- `mount` + closures over signals can express most cases. The pattern is `mount: (s, el) => { effect(() => { /* read signals; do side effect */ }) }`.
- A dedicated section invites authors to put DOM manipulation, fetches, and event registrations inline — exactly the things that should live in template hooks (`<use-*>`) or `<data>` elements.

**Revisit when** a real authoring pattern shows that `mount` + `effect()` is meaningfully worse than a `effect:` section.

---

### Reactive `<data source>` re-subscribe — *resolved for state args*

**Considered, then implemented for state-arg reactivity.** Reading any signal in `<data source>` via `$scope.<key>()` is tracked by `live()`. When the signal changes, the prior SSE subscription disposes and a new one opens with the updated args. Authors write `<data source="ship.agents.get({id: $scope.selectedId()})" live />` and the channel re-binds automatically.

**Still open: route-param signals.** `route` today is a plain string record. To make `route.params.id` reactive, the daemon shell would need to populate `route` as a reactive proxy and update its signals on navigation. Tracked separately as a route-system task.

---

### LSP `use:` namespace awareness

**Considered.** Today's LSP recognizes top-level controller section keys (state / derived / on / use), so `total`, `incr`, `list` resolve in template scope. It does *not* track `use:`-namespace keys deeply, so `list.filtered` and `commitFilter.query` resolve only the leftmost identifier.

**Open** as a real follow-up rather than a rejected alternative. Documented in `status.md` under "in active development" — the LSP needs to walk the hook return shape and surface the inner keys. Not deferred indefinitely; just not in v1.

---

### Compiler-emitted `.d.ts` per `.html.set`

**Considered.** Auto-derive the controller's `Scope` generic from the template's `<data>` / `<state>` / `<use-* as=>` declarations by emitting a sibling `.d.ts` from the AOT compiler.

**Open** as a real follow-up. The manual `Scope` declaration pattern (documented in `controller.md`) is the bridge for v1. Status: the compiler infrastructure exists (`crates/ship-set-compiler/`); the type-emission step is not yet wired.

## Consequences

The framework is deliberately small. The above list is what the discipline costs — every "not now" is a real ergonomic give-up that we're choosing because the framework's coherence depends on a tight surface.

The list is *long*. That's expected. A framework's quality is more visible in what it refused than in what it shipped.

When proposing additions to the framework's surface, check this ADR first. If the proposed shape is already here, the answer is either "the prior reasoning still applies, no" or "here's the new forcing function that changes the calculus." Don't re-litigate without one.

## References

- Three-role split: `ship-set-three-role-architecture.md` (this directory).
- Set-prefix migration: `ship-set-prefix-during-migration.md` (this directory).
- Active-dev tracking: `docs/ship-set-framework/references/docs/status.md`.
