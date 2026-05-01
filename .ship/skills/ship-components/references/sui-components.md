# SUI components

`<set-*>` primitives compiled AOT from `.html.set` source. Real custom elements; same registration model as `customElements.define()`. Bundled into `apps/shipd/assets/sui-components.js` and loaded once by the shell at boot.

## File layout

```
packages/ship-set-components/src/<tag>/
  <tag>.html.set           # template + <prop> + <style>
  <tag>.sui.json           # { "component": "<tag>", "shadow": true }
  <tag>.ts                 # import './<tag>.compiled.js'  — side-effect only
```

The compiler reads `<tag>.sui.json` for tag name + shadow flag, emits `<tag>.compiled.js` (gitignored). Importing the package's `index.ts` loads every `<tag>.ts` and triggers `customElements.define()` per component — no runtime parser involved.

## Authoring shape

File IS the component — no `<component>` wrapper. Top-level `<prop>` directives declare the API; the rest is the body:

```html
<!-- packages/ship-set-components/src/set-button/set-button.html.set -->
<prop name="tone" values="default|primary|destructive" default="default" reflect />
<prop name="disabled" type="bool" default="false" reflect />
<prop name="size" values="sm|md|lg" default="md" reflect />

<style>
  :host { display: inline-flex; }
  button { /* … */ }
</style>

<button disabled={disabled()} class="btn btn-{tone()} btn-{size()}">
  <slot name="leading"></slot>
  <slot></slot>
</button>
```

Bindings, control flow, hooks — same as views. See `../../ship-views/references/sui-authoring.md` for the full template surface (template syntax is shared between views and components).

## Compile shape

The compiler emits one ESM per `.html.set`:

```js
import { component, prop, bindAttr, bindText, … } from "@ship/ui-runtime"

export default component("set-button", {
  props: {
    tone:     prop.union(["default","primary","destructive"], { default: "default", reflect: true }),
    disabled: prop.bool({ default: false, reflect: true }),
    size:     prop.union(["sm","md","lg"], { default: "md", reflect: true }),
  },
  shadow: true,
  styles: "…",
  setup(ctx) { /* scope + bindings */ },
})
```

Authors do not call `component()` directly — the compiler does. Same shape as `view()` for views, but with `props` / `shadow` / `styles` and no `bindings` manifest (components don't go through SSR + attach; they upgrade via `customElements.define()` at registration time).

## Bundling + registration

1. `node scripts/build-bundles.mjs --force` compiles every `.html.set` and bundles `apps/shipd/assets/sui-components.js`.
2. The shell loads `sui-components.js` once at boot via the importmap → all `<set-*>` elements register globally.
3. `<set-button>` and friends are now usable in any view — SUI or j2 — anywhere on the page.

## Mixing rule

Pragmatic: SUI views compose Lit `<ship-*>` until the SUI library has parity. Don't block on porting. When a primitive is missing in SUI, use the Lit version; flag for strategic port.

## Ported set (18 primitives)

`set-status-dot`, `set-spinner`, `set-badge`, `set-chip`, `set-card`, `set-link`, `set-divider`, `set-skeleton`, `set-stat`, `set-mark`, `set-kbd`, `set-avatar`, `set-progress`, `set-button`, `set-tabs`, `set-tab`, `set-empty-state`, `set-input`.

Read any of these in `packages/ship-set-components/src/<tag>/` for shape. `set-button/` is the canonical reference — covers every prop kind, slot, scoped style, and the shadow flag.

## Distribution test (same as Lit)

The distribution test from the Lit charter applies identically to SUI components — an element earns its spot in the library only if it's a semantic primitive, a platform concept, or has its own brain. View-local compositions stay in `partials/`. Read `lit-charter.md` § "The distribution test" for the full rule.
