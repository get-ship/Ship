---
title: "Ship Set — Component Shadow DOM is `<component shadow>`"
description: "ADR: components opt into a shadow root via a `shadow` attribute on the `<component>` directive. Light DOM stays the default."
group: ADRs
order: 58
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, components, shadow-dom, framework]
---

# Ship Set — Component Shadow DOM via `<component shadow>`

## Context

Web components have two rendering modes: **light DOM** (children render in the host element's normal tree, inherit page styles) and **shadow DOM** (children render in an encapsulated `template.content`-style root that scopes styles and prevents style bleed). Each has a use case:

- **Light DOM** — primitives that should pick up page-level theming (button, input, link, chip). Authors expect them to inherit `color`, `font-family`, etc. from ancestors. Style scoping is achieved by tag-prefix selectors (`set-button { ... }`) without isolating the subtree.
- **Shadow DOM** — primitives whose internal styles must NOT bleed and must NOT be overridden by page styles (modal, popover, menu, accordion-item with delicate internal layout). The `<style>` inside the component's shadow root is fully scoped.

The framework needs a way for component authors to say "this component uses shadow DOM" without it being the default (it would surprise authors who expect light-DOM theming to work).

The open question — flagged in `status.md` — was whether the opt-in lives as:

1. An attribute on the `<component>` directive (`<component name="set-modal" shadow>`).
2. A metadata flag in the directive's child (`<component name="set-modal"><meta shadow /></component>`).
3. A separate sibling directive (`<shadow />` inside `<component>`).
4. A boolean prop with a special name (`<prop name="__shadow" default="true" />`).

## Decision

**Components opt into shadow DOM via a boolean `shadow` attribute on the `<component>` directive.**

```html
<component name="set-modal" shadow>
  <prop name="open" type="boolean" default="false" />
  <style>
    .modal-root { /* scoped via shadow root */ }
  </style>
  <div class="modal-root">
    <slot />
  </div>
</component>
```

The compiler emits `attachShadow({ mode: 'open' })` in the generated `customElements.define()` body when the attribute is present. Inline `<style>` children inside the directive are injected into the shadow root automatically. Slots resolve via the platform's standard shadow-DOM slot mechanism.

When `shadow` is absent, the component renders into the host's light DOM. Authors get tag-prefix-scoped CSS (current behavior) without the encapsulation overhead.

## Rationale

1. **Discoverability.** The shadow opt-in lives where the component's identity lives — the `<component>` directive. Anyone reading the component knows immediately if it's shadow or light. A buried metadata child or a sibling directive obscures the choice.
2. **Reuses the existing attribute pattern.** `<component>` already accepts attributes (`name`, future `events`). Adding `shadow` is consistent.
3. **Boolean attribute conventions.** HTML's `<input disabled>` shape is established. `<component shadow>` mirrors it. Authors and agents intuit the semantics without needing a doc.
4. **No grammar surprise.** A child directive (`<meta shadow />`) or sibling (`<shadow />`) introduces grammar shapes the parser doesn't have today. Adding a boolean attribute extends a parser path that's already used.
5. **Mode argument deferred.** The decision picks `attachShadow({ mode: 'open' })` as the default. If a future component needs `mode: 'closed'`, the attribute can carry the value (`shadow="closed"` or a separate `shadow-mode` attribute). Defer until a real need exists.

## Alternatives considered

### Metadata child: `<component><meta shadow /></component>`

Authors declare metadata via a sibling directive inside `<component>`.

**Rejected** because:
- Adds a grammar shape the parser doesn't have.
- Buries the choice — readers have to scan the directive's body to know if it's shadow.
- Doesn't generalize: future per-component flags (mode, slot strategy, etc.) would each need their own meta child, which compounds.

### Sibling directive: `<shadow />` inside `<component>`

Marker directive that signals shadow mode.

**Rejected** for the same reasons as the metadata child plus the additional concern that `<shadow />` is a self-closing element with no semantic content — pure marker. The framework rejects markers-as-directives in favor of attributes.

### Boolean prop: `<prop name="__shadow" default="true" />`

Re-use the prop machinery; `__shadow` becomes a "magic" prop the runtime recognizes.

**Rejected.** Magic props are dishonest — they reuse a mechanism for something it isn't. Props are user-visible attributes consumers set on the element; shadow opt-in is a compile-time declaration that doesn't surface to the consumer at all.

### Default to shadow DOM

Make shadow DOM the default; light DOM requires opt-out.

**Rejected.** Most ported primitives are light-DOM today (set-button, set-chip, set-link, set-divider, etc.). Defaulting to shadow would break existing styling and require every primitive to add an opt-out. Worse, it would surprise authors who haven't internalized that "this is a shadow primitive, themes won't reach in."

## Consequences

### Positive

- Shadow opt-in is discoverable, idiomatic, and minimal.
- Light DOM stays the default — current ports keep working without churn.
- The compiler's emit gets a single conditional branch on the directive's attribute.
- CSS authoring inside the directive (the `<style>` child pattern from defineComponent's CSS-splicing) works for shadow components without further changes — the inline style lands in the shadow root.

### Negative

- The attribute namespace on `<component>` accumulates over time (`name`, `shadow`, future `events`, future `slot-mode`?). No single attribute is heavy, but the directive's surface is now a small grammar to remember.
- Authors mixing light + shadow primitives need to track which is which. The CEM's `kind: 'shadow' | 'light'` field can surface this in IDE tooltips later.
- `attachShadow({ mode: 'open' })` is hard-coded. If `closed` is ever needed, the attribute must extend (`shadow="closed"`). Tracked as an open question below.

### Open questions

- **Mode value (open vs closed).** Today: `mode: 'open'`. If a future component needs `closed`, extend `shadow` to accept a value (`shadow="closed"`). Defer the syntax decision until a real component needs it; document the forcing function then.
- **Slot strategy.** Components with multiple slots (`<slot name="leading">`, `<slot name="trailing">`, default) need slot-mode declarations. Today the platform's standard mechanism works for shadow components. For light-DOM components, slot resolution happens at mount via the light-DOM slot lift in `@ship-set/dom`. Cross-mode slot semantics: documented in the template doc; no ADR needed.

## Implementation phasing

This ADR ships the *decision*. Implementation lands in Arc E:

1. **Parser extension.** `<component shadow>` parses into the IR. ~half day.
2. **Compiler emit.** When the IR carries `shadow: true`, emit `attachShadow({mode:'open'})` in the constructor + inject inline `<style>` children. ~1 day.
3. **CEM generation.** Per-component `kind: 'shadow' | 'light'` field. ~half day.
4. **Tests.** A test component declares shadow; rendered output has a shadow root with scoped styles. ~half day.

Lands as one commit on Arc E-2.

## References

- [Three-role architecture ADR](./ship-set-three-role-architecture.md) — discipline that places API surface in the template.
- [Typed CustomEvents ADR](./ship-set-typed-custom-events.md) — sibling decision; same `<component>` directive grows another attribute.
- [`<component>` directive doc](../../../ship-set-framework/references/docs/template.md) — current shape; will be extended.
- Web components Shadow DOM API: [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Element/attachShadow).
