# SUI components

`<sui-*>` primitives compiled AOT from `.sui` source (with paired `.ts` controllers when needed). Real custom elements; same registration model as `customElements.define()`. Bundled into `apps/shipd/assets/sui-components.js` and loaded once by the shell at boot.

## File layout

Convention is the contract — directory IS the component, filename stem IS the tag, shadow is always on, paired `.ts` is auto-detected:

```
packages/ui-components/src/<tag>/
  <tag>.sui                # template + <prop> + <style>     (required)
  <tag>.ts                 # paired controller(def)          (optional; auto-detected)
  <tag>.css                # extracted styles                (optional)
```

No `<tag>.sui.json` config file. No metadata layer. The compiler infers everything from the directory and filename.

The package's `index.ts` imports each `<tag>.compiled.js` directly (gitignored, generated). Importing it triggers `customElements.define()` — no runtime parser involved.

## Authoring shape

File IS the component — no `<component>` wrapper. Top-level `<prop>` directives declare the API; the rest is the body:

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

Bindings, control flow, hooks — same as views. See `../../ship-views/references/sui-authoring.md` for the full template surface (template syntax is shared between views and components).

## Paired controller

When the component has interactive behavior (keyboard handlers, focus management, third-party engine wiring), pair the template with a `.ts` controller. The compiler auto-detects the sibling `.ts`; no flag, no manifest.

```ts
// sui-terminal.ts
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

Use `controller(def)` only — raw `function setup(ctx)` is not accepted.

Controllers must not query the shadow DOM with string selectors. Use `bind:this={el}` in the template to get a typed `Signal<HTMLElementType | null>` in scope; declarative slot reactivity (`<slot reflect-empty="<attr>">`) replaces hand-rolled slotchange listeners.

## Compile shape

The compiler emits one ESM per `.sui`:

```js
import { component, prop, bindAttr, bindText, … } from "@ship/ui-runtime"
import setup from "./sui-terminal.ts"  // emitted only when sibling .ts exists

export default component("sui-terminal", {
  props: { /* … */ },
  shadow: true,
  styles: "…",
  setup(ctx) { /* scope construction; calls paired setup if present */ },
})
```

Authors don't call `component()` directly — the compiler emits it.

## Bundling + registration

1. `node scripts/build-bundles.mjs --force` compiles every `.sui` source and bundles `apps/shipd/assets/sui-components.js`. Paired-controller components get their `.ts` esbuild-inlined here.
2. The shell loads `sui-components.js` once at boot via the importmap → all `<sui-*>` elements register globally.
3. `<sui-button>` and friends are usable in any view — SUI or j2 — anywhere on the page.

## Mixing rule

Pragmatic: SUI views compose Lit `<ship-*>` until the SUI library has parity. Don't block on porting. When a primitive is missing in SUI, use the Lit version; flag for strategic port.

## Distribution test (same as Lit)

The distribution test from the Lit charter applies identically to SUI components — an element earns its spot in the library only if it's a semantic primitive, a platform concept, or has its own brain. View-local compositions stay in `partials/`. Read `lit-charter.md` § "The distribution test" for the full rule.
