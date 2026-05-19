# SUI component contract — depth

Depth reference for `ship-components`. The contract lives in `../SKILL.md`.

## The distribution test

An element earns a spot in `packages/ui-components/src/` only if at least one applies:

1. **Semantic primitive** — `<sui-button>`, `<sui-input>`, `<sui-select>`, `<sui-dialog>`. Maps to a familiar HTML/ARIA concept; reusable across products.
2. **Platform concept** — `<sui-rail>`, `<sui-nav-top>`, `<sui-panel-dock>`, `<sui-status-bar>`. Models a Ship-specific shell concept that multiple views share.
3. **Has its own brain** — `<sui-graph-canvas>`, `<sui-terminal>`, `<sui-markdown>`. Wraps a substantial third-party engine or non-trivial behaviour the consumer would otherwise have to glue.

Everything else is view-local. View-local compositions live in the consuming view's `partials/` directory, not in the component library. A thin wrapper whose only value is filling slots with view-specific content is a partial, not a component.

## Slot patterns

```html
<!-- Named + default slot -->
<slot name="leading"></slot>
<slot></slot>
<slot name="trailing"></slot>

<!-- Reflect empty state to a host attribute (declarative slot reactivity) -->
<slot reflect-empty="data-icon-only"></slot>
```

`reflect-empty` sets `data-icon-only` on the host when the slot has no assigned nodes; clears it when populated. The runtime listens to slotchange and toggles the attribute. Replaces hand-rolled `slotchange` listeners in controllers.

## Refs via `bind:this`

```html
<!-- in <tag>.sui -->
<input class="input" bind:this={inputEl} />
```

```ts
// in <tag>.controller.ts
import { controller } from '@ship/ui-runtime'
import type { Scope } from './<tag>.sui'

export default controller<Scope>({
  mount: ({ inputEl }, host) => {
    inputEl()?.focus()
    return () => {}
  },
})
```

`bind:this={inputEl}` produces a `Signal<HTMLInputElement | null>` on the controller's scope. Read by call. The compiler emits the type into `<tag>.sui.d.ts` so `ctx.inputEl` is typed without manual annotation.

## Tone vs variant

- **`tone`** — semantic colour: `default | primary | info | success | warning | destructive | accent | muted`.
- **`variant`** — visual style: `solid | soft | outline | ghost | …`.

A `tone` answers "what does this mean?"; a `variant` answers "how is it drawn?". `variant="destructive"` is wrong because destructive is a meaning, not a visual style.

## What the compiler infers

- The custom-element tag is the directory name + filename stem (`sui-button/sui-button.sui` → `<sui-button>`).
- Shadow DOM is always on for components.
- `<prop>` directives at the top of `.sui` become `props: { … }` in the emit; types are inferred from `values=` / `type=` attributes.
- A sibling `<tag>.controller.ts` is auto-detected; if absent, the component is template-only.
- A sibling `<tag>.css` is extracted into the component's `styles:` (alternative to inline `<style>` block).

No metadata file, no manifest, no per-component config.
