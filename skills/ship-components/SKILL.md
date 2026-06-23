---
name: ship-components
stable-id: ship-components
description: Author Ship components in SUI — `<sui-*>` primitives compiled AOT from `.sui` source with optional paired `.controller.ts`.
tags: [ui, components, web-components, sui, design-system]
authors: [ship]
audience: public
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

For HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/sui-runtime`.

## File layout

```
packages/components/sui/primitives/src/<tag>/
  <tag>.sui                  # template + <prop> + <style>     (required)
  <tag>.controller.ts        # paired controller               (optional, auto-detected)
  <tag>.css                  # extracted styles                (optional)
```

Convention is the contract: directory IS the component, filename stem IS the tag, shadow is always on, paired `.controller.ts` is auto-detected. Plain `.ts` files are not controllers — they're helpers, types, or tests.

## Authoring shape

File IS the component — no `<component>` wrapper. Top-level `<prop>` directives declare the API; the rest is the body.

```html
<!-- packages/components/sui/primitives/src/sui-button/sui-button.sui -->
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

Templates declare; controllers contribute state via the `scope:()` factory.
The compiler reads that factory and emits each returned key into the
`Scope` interface of `<file>.sui.d.ts`, so `mount(state, host)` sees direct,
typed members — no casts, no defensive throws.

```ts
// sui-error-boundary.controller.ts — the canonical exemplar.
import { controller, signal, attachBoundary, detachBoundary } from '@ship/sui-runtime'
import type { Scope } from './sui-error-boundary.sui'

export default controller<Scope>({
  scope: (_baseScope, host) => {
    const caughtError = signal<Error | null>(null)
    const reset = () => { caughtError.set(null) }
    return { caughtError, reset }
  },
  mount(state, host) {
    attachBoundary(host, {
      catch(err) { state.caughtError.set(err instanceof Error ? err : new Error(String(err))) },
      reset: state.reset,
    })
    return () => detachBoundary(host)
  },
})
```

Three things to notice:

- `scope:` returns plain identifiers (`caughtError`, `reset`). Those keys
  land in template scope AND on the typed `state` arg to `mount`.
- `state.caughtError.set(err)` — direct, typed `Signal<Error | null>`. No
  `state['caughtError'] as unknown as Signal<...>` cast. No
  `throw new Error('scope signals missing')` defensive guard.
- If you rename `caughtError` in the factory but forget to update a
  template reference, the TypeScript compiler catches it.

Use `controller(def)` only — raw `function setup(ctx)` is not accepted. Get refs via `bind:this={el}` in the template; declarative slot reactivity (`<slot reflect-empty="<attr>">`) replaces hand-rolled slotchange listeners.

**Don't create an empty controller.** Controllers are optional — if your
component has no `scope`, no `derived`, no `on`, and no `mount`, skip the
`.controller.ts` file entirely. The compiler synthesizes a no-op controller
from the `.sui` template alone. The LSP rule `controller.empty` flags
`controller({})`-shape files (and `export default null`) as a warning;
delete them rather than satisfy the convention with ceremony.

Working exemplars: `packages/components/sui/primitives/src/sui-error-boundary/`,
`packages/components/sui/primitives/src/sui-portal/`,
`packages/components/sui/primitives/src/sui-toast-container/`.

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

Worked examples: `packages/components/sui/primitives/src/sui-input/sui-input.sui`, `packages/components/sui/primitives/src/sui-select/sui-select.sui`, `packages/components/sui/primitives/src/sui-checkbox/sui-checkbox.sui`.

Constraints (compiler-enforced):

- Only one prop per component may carry `form-value`.
- `form-value` is rejected on `type="object"` / `type="array"` props (only `string` / `number` / `bool` / `union`).
- `form-disabled` is only valid on `type="bool"` props.

## Compile shape

```js
// AUTO-GENERATED by sui-compiler — do not edit
import { component, prop, bindAttr, bindText, … } from "@ship/sui-runtime"
import setup from "./ship-terminal.controller.ts"  // when paired

export default component("ship-terminal", {
  props: { /* … */ },
  shadow: true,
  styles: "…",
  setup(ctx) { /* scope construction; calls paired setup if present */ },
})
```

Authors don't call `component()` directly — the compiler emits it.

## Bundling + registration

1. The daemon's in-process Rolldown bundler (#163) compiles every `.sui` source and bundles `/home/madvib/dev/sui/crates/deck/deck-core/assets/sui-components.js` (paired `.controller.ts` resolved + inlined by the bundler's load hook). `@ship/sui-build` produces the same tree at CI time.
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

## Verify before "done"

**The gate is the compiler, and it is always available.** Run
`sui-compiler build <tag.sui | component-dir>` after every edit — it ships on
PATH (`~/.cargo/bin/sui-compiler`, `just install-dev`), needs no MCP and no
daemon, and runs grammar + semantic checks **and the full lint pass** (the
oxc-based controller lint in `/home/madvib/dev/sui/crates/sui/lint`, e.g. `sui-controller-purity`,
`sui-css-content-text`). A printed `Error:` is a hard fail. Never substitute
`tsc` — Ship does not bundle TypeScript; the compiler's lint is the source of truth.

```sh
sui-compiler build packages/components/sui/primitives/src/sui-X/sui-X.sui
```

## Agent surfaces (CLI)

> The MCP tools below have been **removed** in favour of generated
> skills + CLI. They wrapped the **same** analyser as `sui-compiler build`, so they
> added nothing the gate above doesn't already cover — use the CLI equivalents listed here.

The view-authoring tools all apply to components since components
are just `<sui-*>` views with shadow DOM. Full catalog:
`docs/ship-sui/references/docs/agent-surfaces.md`.
The component-specific essentials:

| Tool | When |
|---|---|
| `ship sui diagnostics <file>` | Structured parse + analyse. Catches parse errors, missing `<prop>` / `<state>` attrs, unbound identifiers, invalid template expressions. |
| `ship sui hover <file> <line> <col>` | Declaration source + setup return type at a cursor. |
| `ship sui dependencies <file>` | What sibling `<sui-*>` tags + signals this component reaches into. |
| `sui-compiler build <file>` | Full lowering pipeline. Run after `ship sui diagnostics` is clean to catch lowering-only issues. |

### Visual + behavioural verification

| Surface | Use it for |
|---|---|
| `ship-ui-studio` at `/sui/ship-ui-studio/<tag>` | Single-page visual sweep — prop panel + variant matrix + source view. Fastest after-edit eyeball. |
| `@ship/sui-eval` `runEval({ description, spec, catalog })` | Programmatic behavioural scoring. Sub-scores: `compile / scope / behaviour`. See `docs/ship-sui/references/docs/agent-surfaces.md` § 3. |

```ts
// Pseudocode of the verify loop:
// 1. write template + (optional) controller
// 2. ship sui diagnostics packages/components/sui/primitives/src/sui-X/sui-X.sui
//    → fix until []
// 3. sui-compiler build packages/components/sui/primitives/src/sui-X/sui-X.sui
//    → fix lowering-only issues
// 4. Open ship-ui-studio for visual sweep across the variant matrix
// 5. (optional) runEval against a behavioural spec for regression gating
```

## LSP integration

The same analyser that backs `sui_diagnostics` is exposed as a
standalone language server.

| Surface | How |
|---|---|
| **CLI** (agents) | `ship sui diagnostics` / `hover` / `dependencies`. See above. |
| **VS Code** | `apps/vscode/` auto-launches the `sui-lsp` binary on `.sui` open. |
| **Any editor / Claude Code / opencode** | Point the LSP client at the `sui-lsp` binary (`/home/madvib/dev/sui/crates/sui/lsp/`, on PATH via `just install-dev`) over stdio. Claude Code uses the repo `.lsp.json`. |
| **Browser** | `@ship/sui-lsp-browser` runs the WASM `analyze` (same Rust grammar) for diagnostics in the docs playground; hover/completion are static-dictionary for now. |

## What's available in a view

Components fall into these buckets:

- **Primitives** — generic `<sui-*>` in **`@ship/sui-primitives`** (`packages/components/sui/primitives/src/`). Charts in **`@ship/sui-charts`** (`packages/components/sui/charts/src/`). Both are decomposed workspace packs with their own bundle + `custom-elements.json`. Always registered, available in any view.
- **Editor packs** — `<ship-code-editor>`, `<ship-terminal>`, and the `<ship-git-diff>` tree live in **`@ship/code`** (`packages/components/ship/code/src/`); `<ship-markdown-editor>` lives in **`@ship/markdown`** (`packages/components/ship/markdown/src/`).
- **Chrome + domain `<ship-*>`** — picker, nav, devtools, view-authoring, and the rest live in their own `ship/*` packs under `packages/components/ship/<pack>/src/` (e.g. chrome in **`@ship/chrome`**, agents in **`@ship/agents`**). Each pack ships its own bundle + `custom-elements.json`.
- **Islands** — heavy components that lazy-load their engine only when mounted (ship-terminal, ship-code-editor, ship-markdown-editor). The bundle code-splits these; lazy chunks live under `.deck/sui-components/<tag>.js` (deck's build output).

The `@ship/sui-catalog` walker emits two projections under `.deck/catalogs/`:
- `workspace.json` — every pack, public + private. Use for authoring.
- `published.json` — `!private` packs only. Use when generating consumer-facing docs or npm publish flows.

Each entry gains a `pack` field — filter by pack to scope a query. Read these projections from `.deck/catalogs/` directly. Don't `curl` the daemon's `/api/assets/sui-custom-elements.json` — that endpoint exists for browser tooling; agent auth blocks raw HTTP.

## Visual sweep: ship-ui-studio

Once a component is registered, `ship-ui-studio` is the fastest way to eyeball it. Open `/sui/ship-ui-studio/<tag>` (e.g. `/sui/ship-ui-studio/sui-button`) and the detail page renders:

- A **single live preview** plus a prop panel for ad-hoc tweaking.
- A **state variant matrix** — when the component declares enum props, every cell in the cartesian product of `variant × tone` (or the single largest enum, if there's only one) is rendered side-by-side, independent of the prop panel. This is the Storybook-style sweep that catches "ghost tone-warning has wrong contrast" without clicking through 40 combinations one at a time.
- A **View source** link in the header that opens `/_deck/source/<tag>` — a static HTML page showing the on-disk `.sui` template and `.controller.ts` for the component. Useful when reviewing how an existing primitive wires its scope before authoring a similar one.

## Platform features (shared with views)

Components live inside the same runtime as views; the same primitives are
available. Skim the matrix below; for deep treatment of each, see the
`ship-views` skill section of the same name.

### Morph

Add the `morph` attribute to any element whose contents are produced by
`bindRawHtml` (or another HTML-string path) to preserve focus, scroll,
selection, and event-listener identity across re-renders. `bindHtml`
morphs unconditionally — the directive only matters for `bindRawHtml`.

```html
<div morph>{/* server-rendered HTML — bindRawHtml in the controller */}</div>
```

Limitations: no `<select>`-value preservation, no shadow-DOM matching,
matches by `id` and tag-plus-position only.

### Error boundaries

Wrap fault-prone subtrees in `<sui-error-boundary>` — throws inside
effects, controller `mount`, or render expressions are caught and the
fallback renders. `fallback-text` for a string fallback or
`<template slot="fallback">` for custom markup. Call `.reset()` on the
host to retry. Fires `sui-error-boundary:caught` (bubbling, composed)
with `{ error, source: 'effect' | 'mount' | 'render' | 'unknown' }`.

```html
<sui-error-boundary fallback-text="Couldn't load">
  <sui-some-component prop={maybeThrows()}></sui-some-component>
</sui-error-boundary>
```

### Portal

`<sui-portal target="body">` teleports slotted content out of the host
shadow tree for stacking-context escape (modals, popovers, toasts).
`target` is a CSS selector resolved against `document`; default `body`.
The portal element stays the logical parent, so reactive bindings and
event bubbling still flow through the original DOM ancestry. Missing
target warns and leaves children in place. See `sui-dialog`,
`sui-popover`, `sui-toast-container` for composition patterns.

### Transitions

`transition:<name>` animates enter/exit of `{#if}` / `{#for}` mounts.
Built-ins: `fade`, `slide`, `scale`. Use `in:` / `out:` for asymmetric
directions. Custom transitions live on the controller's `scope:` factory
and match the `Transition` type. `prefers-reduced-motion: reduce`
collapses duration + delay to `0`. Transitions don't surface in SSR —
they register on construction (CSR + hydrate paths).

```html
{#if open()}
  <div transition:scale={{ duration: 120 }}>...</div>
{/if}
```

### Routes

URL state is reactive. Components rarely consume routes directly — that's
a view-controller concern — but a component controller may read
`route()` to surface URL-dependent behavior (e.g. a tab strip whose
active tab is `route().query.tab`).

```ts
import { route, defineRoutes, navigate } from '@ship/sui-runtime/route'
// route().params.<name> / route().query.<name> are Signal<string | undefined>
```

Full treatment: `ship-views` § Routes.

### HMR

In dev mode, `.sui` and `.controller.ts` edits trigger an in-place re-mount.
shell's boot calls `initHmr()` (`apps/shell/src/hmr.ts`), which uses
`subscribeHmr` from `@ship/sui-runtime/dev` to listen on shipd's
`/_deck/dev/hmr` SSE channel. Component-tag changes still need a full
reload (custom-element re-registration isn't yet supported); compile
errors surface in the browser console. Production builds inline
`import.meta.env.DEV` to `false`, so HMR is a no-op.

### Performance gates

- `just bench-check` — js-framework-benchmark vs latest baseline under
  `tests/perf/baselines/`; fails on >15% regression. Run before merging
  runtime-affecting component changes.
- `just visual` — Playwright screenshots vs committed PNGs (daemon must
  be running). Use `--update-snapshots` after a deliberate visual change.
- `just smoke-deploy` — full build + daemon restart + every baselined
  view curled + Playwright. Proxy for CI; aim for green before PR.

## References

| When you need | Read |
|---|---|
| Full component-authoring contract: scope resolution, slot patterns, `bind:this`, the distribution test | `references/component-contract.md` |
| Component-pack manifests, distribution, on-demand loading | `references/packs.md` |
