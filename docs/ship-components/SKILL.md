---
name: ship-components
stable-id: ship-components
description: Historical reference for the `.ship` single-file component format (template / script / style three-block). DEAD — new component authoring uses the ship-ui skill (`.sui` files, top-level `<prop>` directives, paired `.ts` controllers). Kept for one-time historical lookup; do not author new components against this surface.
tags: [ship, components, sfc, web-components, historical]
authors: [ship]
---

> **⚠ Historical only.** The `.ship` SFC format documented here is **dead**.
> New component authoring uses the **`ship-ui`** skill — components are
> `.sui` files with top-level `<prop>` directives, an optional sibling `.ts`
> controller, and an optional sibling `.css`. Component packs are
> `references/docs/packs/` under this skill. See
> `docs/ship-ui/references/docs/status.md` for the current SUI map.

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
