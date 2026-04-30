---
name: ship-components
stable-id: ship-components
description: Author custom web components for Ship as `.ship` single-file components. Use when building a new `<my-tag>` element to compose into views — props, signals, lifecycle, scoped styles, all in one file. NOT the publishing/distribution layer; that's ship-component-packs. PHASING OUT — the `.ship` SFC three-block format is dead; new components are authored as `.html.set` files via the ship-set-framework skill.
tags: [ship, components, sfc, web-components, datastar, jinja, phasing-out]
authors: [ship]
---

> **⚠ Phasing out.** This skill documents the **`.ship` single-file component
> format** (three optional blocks: template / `<script>` / `<style>`). The
> SFC shape is **dead**. **New component authoring** uses the
> **`ship-set-framework`** skill — components are `.html.set` files with
> `<component name="..."><prop name="..."/></component>` directives at the
> top, an optional sibling `.ts` controller, and an optional sibling `.css`.
> Components ported off `.ship` one at a time; see
> `docs/ship-set-framework/references/docs/status.md`.

# Ship Components

Custom web components for Ship are authored as `.ship` single-file components (SFCs). One file declares props, lifecycle, template, and scoped styles. The daemon parses the file at request time and emits a `customElements.define()` registration — no build step, no framework runtime, hot-reloads on edit.

A `.ship` component file:

```html
<script>
  // Props destructured from the implicit `props` proxy.
  const { label = 'Click', count = 0 } = props

  // Per-instance signals — names auto-scoped by the framework.
  const $count = signal(count)

  // Optional lifecycle hooks.
  onConnect(el => el.querySelector('button').focus())
</script>

<button data-on:click="$count++">
  {{ props.label }}: <span data-text="$count"></span>
</button>

<style>
  button { padding: var(--ship-space-3); }
</style>
```

Saved at `.ship/components/my-counter.ship`, this becomes `<my-counter>` automatically. Tag is the kebab-cased filename.

For full details see `references/docs/`.

## Reference pages

- [Overview](references/docs/index.md) — when to author a component, the happy path
- [SFC format](references/docs/format.md) — block layout, declarations, scoping rules
- [Lifecycle](references/docs/lifecycle.md) — props, signals, effects, hooks, events
- [Data fetching](references/docs/data-fetching.md) — calling ship.* ops from a component (current manual pattern)
- [Composing in views](references/docs/composing-in-views.md) — using your component from a view
- [Caveats](references/docs/caveats.md) — known gaps, workarounds, what's coming

## When NOT to use

- **You're a view author composing existing primitives.** Stick with `<ship-*>` from the SDK in your `.html.j2` views. See `ship-views`.
- **You're shipping a pack for distribution.** That's `ship-component-packs` — manifest, bundle, publishing. Authoring the components themselves still uses this skill.
- **You need a complex stateful widget wrapping a third-party library** (CodeMirror, xterm, TipTap). The `.ship` SFC format is for stateless-or-signal-backed components. Heavier wrappers stay as TypeScript LitElement classes in a pack.
