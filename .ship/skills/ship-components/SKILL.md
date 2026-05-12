---
name: ship-components
stable-id: ship-components
description: Author Ship components in SUI — `<sui-*>` primitives compiled AOT from `.sui` source with optional paired `.controller.ts`.
tags: [ui, components, web-components, sui, design-system]
authors: [ship]
audience: agent
---

# ship-components

A `<sui-*>` component is a `.sui` template plus an optional `.controller.ts`. The compiler emits one ESM per component; importing it triggers `customElements.define()`. Real custom elements, real shadow DOM, no runtime parser.

## The rule

**Templates declare. Controllers do.** Dynamic markup goes in the `.sui` template via `{#if}/{#for}/{#match}` and the binding primitives (`bindAttr`, `bindText`, `bindClass`, `bindValue`, …). Controllers own state, refs, lifecycle, async I/O, and the public API. A controller that produces markup is a bug — `sui-controller-purity` rejects it at compile and at edit.

Forbidden in `*.controller.ts`:
- `<el>.innerHTML = …` / `<el>.outerHTML = …` / `<el>.insertAdjacentHTML(…)`
- A sibling `render.ts` of HTML-string template-literal helpers
- String-selector queries against `shadowRoot` after `mount` (use `bind:this` for refs)
- A `.sui` body that is just `<div bind:this={ref}></div>` — the template is the renderer

For HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/ui-runtime`.

## File layout

```
packages/ui-components/src/<tag>/
  <tag>.sui                  # template + <prop> + <style>     (required)
  <tag>.controller.ts        # paired controller               (optional, auto-detected)
  <tag>.css                  # extracted styles                (optional)
```

Convention is the contract: directory IS the component, filename stem IS the tag, shadow is always on, paired `.controller.ts` is auto-detected. Plain `.ts` files are not controllers — they're helpers, types, or tests.

## Authoring shape

File IS the component — no `<component>` wrapper. Top-level `<prop>` directives declare the API; the rest is the body.

```html
<!-- packages/ui-components/src/sui-button/sui-button.sui -->
<prop name="tone" values="default|primary|destructive" default="default" reflect />
<prop name="disabled" type="bool" default="false" reflect />
<prop name="size" values="sm|md|lg" default="md" reflect />

<style>
  :host { display: inline-flex; }
  button { /* … */ }
</style>

<button disabled={disabled()} class="btn btn-{tone()} btn-{size()}">
  <slot name="leading"></slot>
  <slot reflect-empty="data-icon-only"></slot>
</button>
```

Bindings, control flow, hooks — same surface as views. See `ship-views`'s `references/template-surface.md` for the full directive + bindings + control-flow reference (the template surface is shared between views and components).

## Paired controller

```ts
// sui-terminal.controller.ts
import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-terminal.sui'  // compiler-emitted

export default controller<Scope>({
  mount: (state, host) => {
    // host = the <sui-terminal> element; shadow root + template are
    // already rendered. Wire listeners, return disposer.
    return () => { /* cleanup */ }
  },
})
```

Use `controller(def)` only — raw `function setup(ctx)` is not accepted. Get refs via `bind:this={el}` in the template; declarative slot reactivity (`<slot reflect-empty="<attr>">`) replaces hand-rolled slotchange listeners.

## Form-aware components

When a component participates in HTML form submission (input, textarea, checkbox, switch, select, file upload, …), mark its value prop with `form-value` and its disabled prop with `form-disabled`:

```html
<prop name="value" type="string" default="" form-value />
<prop name="disabled" type="bool" default="false" reflect form-disabled />
```

The compiler emits `static formAssociated = true` on the generated host class; the runtime calls `host.attachInternals()` on connect and auto-wires the full platform lifecycle:

- `setFormValue(<value>)` runs on every change to the marked prop.
- `formResetCallback()` resets the prop to its declared `default`.
- `formStateRestoreCallback(state)` rehydrates the prop on bfcache restore.
- `formDisabledCallback(disabled)` syncs the marked disabled prop when the surrounding form's disabled state cascades.

Drop any hand-rolled `ElementInternals` plumbing — the runtime owns it. The setup ctx surfaces `internals?: ElementInternals` if a controller needs `setValidity()` / `checkValidity()` for custom validation.

Worked examples: `packages/ui-components/src/sui-input/sui-input.sui`, `packages/ui-components/src/sui-select/sui-select.sui`, `packages/ui-components/src/sui-checkbox/sui-checkbox.sui`.

Constraints (compiler-enforced):

- Only one prop per component may carry `form-value`.
- `form-value` is rejected on `type="object"` / `type="array"` props (only `string` / `number` / `bool` / `union`).
- `form-disabled` is only valid on `type="bool"` props.

## Compile shape

```js
// AUTO-GENERATED by sui-compiler — do not edit
import { component, prop, bindAttr, bindText, … } from "@ship/ui-runtime"
import setup from "./sui-terminal.controller.ts"  // when paired

export default component("sui-terminal", {
  props: { /* … */ },
  shadow: true,
  styles: "…",
  setup(ctx) { /* scope construction; calls paired setup if present */ },
})
```

Authors don't call `component()` directly — the compiler emits it.

## Bundling + registration

1. `node scripts/build-bundles.mjs --force` compiles every `.sui` source and bundles `apps/shipd/assets/sui-components.js`. Paired controllers get their `.controller.ts` esbuild-inlined here.
2. The shell loads `sui-components.js` once at boot via the importmap → all `<sui-*>` elements register globally.
3. `<sui-button>` and friends are usable in any view.

## Hard rules

1. **`tone` for semantic color, `variant` for visual style.** `variant="destructive"` is wrong — destructive is a tone.
2. **Composition over reimplementation.** Any list row / tile / card uses `<sui-list-tile>` (slots: `leading`, default, `meta`, `trailing`). Don't ship a thin wrapper whose only value is filling slots with view-specific content.
3. **Distribution test.** An element earns a spot in the library only if it's a semantic primitive, a platform concept, or has its own brain. View-local compositions live in `partials/`. See `references/component-contract.md` § "The distribution test."
4. **Icon-only buttons need `aria-label`.**
5. **`<sui-select>` uses native `<option>`** (no `<sui-option>`). Emits `sui-change`.
6. **No `el.innerHTML/.outerHTML/.insertAdjacentHTML`** in `*.controller.ts`. Use `bindHtml()` for HTML-blob cases.
7. **No `customElements.define` in component code** — the compiler emits the registration.
8. **`{#for}` blocks declare `key={EXPR}`.** Without a stable key the reconciler falls back to positional matching and DOM identity is lost on reorder. `sui-for-key` lint warns when missing.
9. **`on:<event>` event names must match the platform whitelist:** DOM events, `sui-*` / `ship-*` custom-event families, `tag:event`-prefixed cross-component events. `sui-event-name` lint warns on typos with Levenshtein suggestions.
10. **Form-aware components use `form-value` / `form-disabled` on `<prop>`** — never hand-roll `attachInternals()` / `setFormValue` / `formResetCallback`. See "Form-aware components" above.

## What's available in a view

Components fall into three buckets:

- **Primitives** — every `<sui-*>` in `packages/ui-components/src/`. Always registered, available in any view.
- **Domain packs** — bundles of related components for a specific surface (editor, graph, chat). Opt-in per view via the manifest's `packs:` field. Loaded on demand to keep boot bundles small.
- **Islands** — heavy components that lazy-load their engine only when mounted (terminal, code-editor, markdown-editor). Declared as a `packs:` entry in the view manifest; loaded at first instantiation.

Discover what's registered by reading the MCP resource `ship://components` — the canonical agent-side component catalog. It returns the aggregated Custom Elements Manifest (every `<sui-*>` primitive's tagName, attributes, slots, parts, events, CSS custom properties) as JSON. For a single tag, read `ship://components/<tag>` (e.g. `ship://components/sui-button`). Don't `curl` the daemon's `/api/assets/sui-custom-elements.json` — that endpoint exists for browser tooling; agent auth blocks raw HTTP.

## References

| When you need | Read |
|---|---|
| Full component-authoring contract: scope resolution, slot patterns, `bind:this`, the distribution test | `references/component-contract.md` |
| Component-pack manifests, distribution, on-demand loading | `references/packs.md` |
