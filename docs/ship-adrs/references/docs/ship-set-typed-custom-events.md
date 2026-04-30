---
title: "Ship Set — Typed CustomEvent Declarations on `<component>`"
description: "ADR: components declare emitted events on the `<component>` directive; views don't get a typed event surface (they don't emit)."
group: ADRs
order: 56
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, components, events, types, framework]
---

# Ship Set — Typed CustomEvent Declarations on `<component>`

## Context

Components dispatch `CustomEvent`s to communicate with their hosts. A `<set-button>` might emit a `set-press` event with `{ via: 'click' | 'keyboard' }`. A `<set-input>` emits `set-input` with `{ value: string }`. Today these are untyped string-keyed events — authors discover the API by reading the source.

The framework's docs have flagged "typed CustomEvent declarations" as an open question for several phases. The question wasn't whether to add types — it was *where* the declaration lives:

- **Template (the `.html.set` itself)**: declared next to the `<component>` directive, exposed as part of the template's compiled type surface.
- **Controller (the `.ts` file)**: declared as a section alongside `state` / `derived` / `on` — the controller's typed output channel.

The two homes shape how authors think about events: as part of the component's **API surface** (template) or as part of its **behavior** (controller).

## Decision

**Components declare emitted events on the `<component>` directive in the template.** Views do not get a typed event surface — they consume events but don't emit them.

The directive shape extends with an `events` attribute carrying a list of declarations. Each declaration names the event and its payload type:

```html
<component name="set-button" events="set-press: { via: 'click' | 'keyboard' }">
  <prop name="tone" type="'default' | 'primary'" default="default" />
  <prop name="disabled" type="boolean" default="false" />
  <button on:click={() => emit('set-press', { via: 'click' })}>
    <slot />
  </button>
</component>
```

The compiler emits a typed `dispatchEvent` shim available as `emit` in event handlers — `emit('set-press', payload)` is type-checked against the declared events. The compiled CEM (Custom Elements Manifest) carries the event types so IDEs and consumers see the API surface.

Multiple events follow comma-separated declaration syntax. Whitespace tolerant:

```html
<component name="set-tabs" events="
  set-tab-change: { id: string },
  set-tab-close:  { id: string }
">
```

## Rationale

1. **Events are API surface, not behavior.** A component's events are part of what consumers depend on — same kind as `<prop>` declarations. Putting them in the template (alongside props) reflects this.
2. **Controllers are optional; events aren't.** Tier-1 components (rare for primitives but possible) have no controller `.ts`. Putting events in the controller means controller-less components can't declare events — which is wrong for a library primitive.
3. **CEM consumption.** The Custom Elements Manifest is the cross-tool API surface. CEM's `events` field expects per-component event metadata. Generating it from the template's directive is more direct than generating from a separate controller section.
4. **Reuses the existing `<prop>` pattern.** Props and events are both surface declarations. Having them in the same directive (template) keeps the surface coherent.
5. **Views aren't event emitters.** Views render UI and dispatch handlers. They don't expose a public event API to a consumer (there's no consumer of a view — it IS the consumer). Adding an event section to view controllers buys nothing.

## Alternatives considered

### Controller `events: { ... }` section

Add a 6th controller section (after the 5-section reduction from declarative-state):

```ts
controller<Scope>({
  events: { 'set-press': null as { via: 'click' | 'keyboard' } | null },
  on: { handleClick: ({ emit }) => emit('set-press', { via: 'click' }) },
})
```

**Rejected** because:
- Tier-1 components (no controller) lose typed events.
- The 5-section discipline was just established; adding a 6th breaks the spec window.
- CEM generation has to pull from two places (template `<prop>`, controller `events`) which is more brittle than one.

### Untyped events with a runtime registry

Skip type declarations; provide a `defineEvents([...])` runtime helper for documentation only.

**Rejected.** Loses the type safety the question is asking for. The whole point is letting consumers know `set-press` carries `{ via: 'click' | 'keyboard' }` at compile time.

### TypeScript declaration files alone

Authors declare events in a sibling `.d.ts` that the consumer's TypeScript picks up via module augmentation.

**Rejected.** Authors maintaining a parallel `.d.ts` in lockstep with the component is fragile. The CEM should be source-of-truth, generated.

## Consequences

### Positive

- Single declaration site for component API (props + events together in the template).
- CEM generation is direct: walk the directive's `events` list, emit per-event metadata.
- Tier-1 components can declare events.
- The `emit(name, payload)` helper inside event handlers gets a precise type from the declaration — invalid event names or wrong payloads are compile errors.

### Negative

- The `<component>` directive's grammar gains a new attribute. Compiler parser needs an extension for the comma-separated declaration list.
- Controllers have no record of what events the template emits. Authors who think behaviorally (events = side effects of handlers) may be surprised. Documentation needs to address this.
- The `events="…"` attribute syntax is a string with embedded type syntax. Parsing is non-trivial. Implementation should reuse the existing `<prop type="…">` parser machinery.

### Open questions

- **Bubbling and cancelability.** Should the declaration carry these flags (`{ bubbles: true, cancelable: false }`)? Defaults are reasonable (bubbling on, cancelable off); adding flags expands the surface. Defer until a real component needs non-default behavior.
- **Listener types in views.** When a view writes `<set-button on:set-press={handle}>`, should the handler argument be typed automatically from the component's CEM? Yes, eventually — but that requires the view's compiler to read the consumed component's CEM at compile time. Implementation deferred to D-1 (compiler-emitted `.d.ts`).

## Implementation phasing

This ADR ships the *decision*. Implementation lands in the framework's component arc:

1. **Compiler grammar extension.** `<component events="...">` parses into the IR. ~1 day.
2. **Compiler emits the `emit` shim.** Setup gets `emit: <Name extends keyof Events>(name: Name, payload: Events[Name])`. Typed end-to-end. ~1 day.
3. **CEM generation.** The compiler's CEM emit walks the `events` field. ~half day.
4. **Listener side typing.** View consumers see the typed handler shape. Joins D-1's compiler-emitted `.d.ts` arc.

Don't ship piecemeal — phases 1–3 land together; phase 4 is a separate concern.

## References

- [Three-role architecture ADR](./ship-set-three-role-architecture.md) — the spine that places API surface in the template.
- [`<component>` directive doc](../../../ship-set-framework/references/docs/template.md) — current shape; will be extended.
- [Custom Elements Manifest spec](https://github.com/webcomponents/custom-elements-manifest) — the consumption target for the typed events.
