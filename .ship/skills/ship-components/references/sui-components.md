# SUI components

`<set-*>` primitives compiled AOT from `.sui` source (with paired `.ts` controllers when needed). Real custom elements; same registration model as `customElements.define()`. Bundled into `apps/shipd/assets/sui-components.js` and loaded once by the shell at boot.

## File layout

The canonical shape — used by `set-button` and the going-forward standard for new components and re-ports:

```
packages/ship-set-components/src/<tag>/
  <tag>.sui                # template + <prop> + <style>
  <tag>.ts                 # paired controller (optional; opt-in via setup:true in .sui.json)
  <tag>.sui.json           # { "component": "<tag>", "shadow": true, "setup": <bool> }
```

The compiler reads `<tag>.sui.json` for tag name + shadow flag. When `"setup": true` AND `<tag>.ts` exists, the compiler emits `import setup from "./<tag>.ts"` and esbuild inlines the controller during the components-bundle pass. When omitted or false, the component is template-only.

The package's `index.ts` imports each `<tag>.compiled.js` directly (gitignored, generated). Importing it triggers `customElements.define()` — no runtime parser involved.

## Transition state — `.html.set` is legacy

17 of the original 18 ports still live as `<tag>.html.set` with a 1-line side-effect `<tag>.ts` (`import './<tag>.compiled.js'`). These are template-only shells without paired controllers. They predate the canonical shape and are missing interactive behavior the Lit originals carry (slotchange detection, keyboard nav, focus management).

`set-button` was rewritten to the canonical shape as the first real port. Each remaining component gets a real port as its turn comes — one component at a time, full capability + a11y parity vs the Lit original, on SUI's API.

## Authoring shape

File IS the component — no `<component>` wrapper. Top-level `<prop>` directives declare the API; the rest is the body:

```html
<!-- packages/ship-set-components/src/set-button/set-button.sui -->
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

## Paired controller

When the component has interactive behavior (slotchange listeners, keyboard handlers, focus management, third-party engine wiring), pair the template with a `.ts` controller:

```ts
// set-button.ts
import { controller } from '@ship/ui-runtime'
import type { Scope } from './set-button.sui'  // compiler-emitted

export default controller<Scope>({
  mount: (state, host) => {
    // host = the <set-button> element; shadow root + template are
    // already rendered. Wire listeners, return disposer.
  },
})
```

Set `"setup": true` in the `.sui.json` and the build pipeline picks it up.

Controllers must not query the shadow DOM with string selectors. Use `bind:this={el}` in the template to get a typed `Signal<HTMLElementType | null>` in scope; declarative slot reactivity (`<slot reflect-empty="<attr>">`) replaces hand-rolled slotchange listeners.

## Compile shape

The compiler emits one ESM per `.sui`:

```js
import { component, prop, bindAttr, bindText, … } from "@ship/ui-runtime"
import setup from "./set-button.ts"  // when "setup": true

export default component("set-button", {
  props: {
    tone:     prop.union(["default","primary","destructive"], { default: "default", reflect: true }),
    disabled: prop.bool({ default: false, reflect: true }),
    size:     prop.union(["sm","md","lg"], { default: "md", reflect: true }),
  },
  shadow: true,
  styles: "…",
  setup(ctx) { /* scope construction; calls paired setup if present */ },
})
```

Authors don't call `component()` directly — the compiler emits it. The same `controller(def)` shape applies to views and components.

## Bundling + registration

1. `node scripts/build-bundles.mjs --force` compiles every component source (`.sui` or legacy `.html.set`) and bundles `apps/shipd/assets/sui-components.js`. Paired-controller components get their `.ts` esbuild-inlined here.
2. The shell loads `sui-components.js` once at boot via the importmap → all `<set-*>` elements register globally.
3. `<set-button>` and friends are usable in any view — SUI or j2 — anywhere on the page.

## Mixing rule

Pragmatic: SUI views compose Lit `<ship-*>` until the SUI library has parity. Don't block on porting. When a primitive is missing in SUI, use the Lit version; flag for strategic port.

## Ported set (18 primitives, 1 fully on SUI shape)

`set-button` ✓ (full SUI port — `.sui` + paired controller)

`set-status-dot`, `set-spinner`, `set-badge`, `set-chip`, `set-card`, `set-link`, `set-divider`, `set-skeleton`, `set-stat`, `set-mark`, `set-kbd`, `set-avatar`, `set-progress`, `set-tabs`, `set-tab`, `set-empty-state`, `set-input` — template-only `.html.set` shells, missing the interactive layer the Lit originals carry. Real ports follow set-button's pattern as turns come.

Read `set-button/` for the canonical reference — covers every prop kind, slot, scoped style, the shadow flag, and the paired-controller seam.

## Distribution test (same as Lit)

The distribution test from the Lit charter applies identically to SUI components — an element earns its spot in the library only if it's a semantic primitive, a platform concept, or has its own brain. View-local compositions stay in `partials/`. Read `lit-charter.md` § "The distribution test" for the full rule.
