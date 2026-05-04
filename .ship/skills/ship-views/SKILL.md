---
name: ship-views
stable-id: ship-views
description: Author Ship views in SUI — `.sui` templates with optional paired `.controller.ts`, AOT-compiled, server-rendered first, hydrated by attach().
tags: [views, sui, sdk, extension-authoring, ship-elements]
authors: [ship]
audience: agent
---

# ship-views

A Ship view is a `.sui` template with an optional paired `.controller.ts`. The compiler emits an ESM per template; the daemon SSRs it; `attach()` wires bindings on the client. Compose `<sui-*>` components inside.

## The rule

**Templates declare. Controllers do.** Dynamic markup goes in the `.sui` template via `{#if}/{#for}/{#match}` and the binding primitives. Controllers own state, refs, lifecycle, async I/O, and the public API the template binds against. A controller that produces markup is a bug — `sui-controller-purity` rejects it at compile and at edit.

Forbidden in `*.controller.ts`:
- `<el>.innerHTML = …` / `<el>.outerHTML = …` / `<el>.insertAdjacentHTML(…)`
- A sibling `render.ts` of HTML-string template-literal helpers
- Re-querying the shadow DOM after `mount` (use `bind:this` for refs)
- A `.sui` body that is just `<div bind:this={ref}></div>` — the template is the renderer

For HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/ui-runtime`.

## File layout

```
views/<name>/                 # first-party view, baked into shipd
  view.sui                    # template (required)
  view.controller.ts          # paired controller (optional, auto-detected)
  view.css                    # styles (optional, daemon auto-links)
  manifest.yaml               # name, type: sui, rail, events, packs
  package.json
  [param]/view.sui            # parameterised sub-route

.ship/views/<name>/           # extension-point view, project-local
  index.sui
  index.controller.ts
  manifest.yaml
  partials/                   # reusable fragments
  [param]/index.sui
```

Convention is the contract. Filename stem identifies the view; `.controller.ts` paired siblings are auto-detected. Plain `.ts` files are not controllers — they're helpers, types, or tests.

## Template surface (compact)

```html
<!-- live data subscriptions (server-driven, SSE-backed) -->
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />

<!-- reactive args — re-subscribes when selectedId changes -->
<state name="selectedId" value="'abc'" />
<data name="agent" source="ship.agents.get({id: $scope.selectedId()})" live />

<!-- local state -->
<state name="count" value="0" />

<!-- props (component .sui files only) -->
<prop name="tone" values="default|primary" default="default" reflect />
<prop name="disabled" type="bool" default="false" reflect />

<!-- mutations (write-side complement to <data>) -->
<form mutation="ship.agents.create">
  <input name="name" />
  <button type="submit" disabled={$mutation().kind === 'loading'}>Create</button>
</form>

<!-- control flow — block syntax -->
{#if loading()}…{:else if error()}…{:else}…{/if}
{#for a of items() key={a.id}}…{/for}
{#match status()}{:case 'idle'}…{:default}…{/match}
{#while-loading of agents}<p>Loading…</p>{/while-loading}
{#while-error err of agents}<p>{err.message}</p>{/while-error}

<!-- bindings -->
<button on:click={() => count.set(count() + 1)}>+</button>
<input bind:value={query} />
<input type="checkbox" bind:checked={enabled} />
<div class:active={isActive()} style:color={color()} attr:href={url()}>
  {count() === 1 ? '1 item' : count() + ' items'}
</div>

<!-- pass complex objects (signals, arrays, structured data) to a child component -->
<sui-grid prop:rows={rows} prop:columns={columns()} prop:total={count()}></sui-grid>

<!-- template hooks -->
<use-shell as="shell" />
<use-kbd><on key="/" action="search" /></use-kbd>
<use-click-outside bind:target={dialogEl} bind:action={close} />
<use-focus-trap bind:target={dialogEl} />
<use-clipboard />
<use-title source={title} />
<use-scroll-anchor />
<use-intersection callback={onIntersect} />

<!-- slots (components only) -->
<slot name="leading"></slot>
<slot></slot>
```

The file IS the view — no `<component>` wrapper. Top-level directives (`<state>`, `<data>`, `<prop>`, `<use-*>`) declare scope; the rest is the body.

For full examples see `references/template-surface.md`.

## Controller contract

```ts
// view.controller.ts
import { controller, signal, derived, shared, useShared, useFilteredList, unwrap } from '@ship/ui-runtime'
import type { Scope } from './view.sui'   // compiler-emitted .d.ts

export default controller<Scope>({
  scope: ({ agents }) => {
    const filter = signal('')
    const total  = derived(() => unwrap(agents)?.items.length ?? 0)
    const list   = useFilteredList(agents, (a, q) => a.name.includes(q))
    return { filter, total, list }
  },
  on: {
    newAgent: ({ navigate }) => navigate('/agents/create'),
  },
  mount:   ({ state }, el) => { /* runs after first DOM attach */ },
  unmount: ({ state })     => { /* runs on dispose */ },
})
```

Three blocks, each one job:

- **`scope:`** — factory that contributes typed signals, derived, hooks, shared providers and consumers. Runs first; entries land in template scope. Replaces the historical state/derived/use split.
- **`on:`** — multi-statement event handlers. The first arg is a writable proxy: `s.count = s.count + 1` mutates state.
- **`mount:` / `unmount:`** — lifecycle. Runs after first attach / on disconnect.

Tiers, smallest that fits:

1. **No controller** — pure `.sui` with `<prop>` / `<data>` and single-expression inline handlers. No state, no derived, no hooks. Pure presentational.
2. **Paired controller** — anything with state, derived, hooks, lifecycle, or multi-statement handlers.

A multi-statement `on:event` in a tier-1 file (no `.controller.ts`) is a compile error. Enforced by `crates/sui-compiler/src/check/handler_complexity.rs`.

For tiers, hooks (`useFilteredList`, `useSelection`, `useForm`), and the live-data envelope, see `references/controller-tiers.md`.

## Live data envelope

```ts
type LiveState<T> = { kind: 'loading' } | { kind: 'error'; error: Error } | { kind: 'ok'; value: T }

unwrap(sig)               // T | undefined
unwrapOr(sig, fallback)   // T
```

```html
{#while-loading of agents}<p>Loading…</p>{/while-loading}
{#while-error err of agents}<p>{err.message}</p>{/while-error}
{#if agents().kind === 'ok'}
  {#for a of agents().value.items key={a.id}}…{/for}
{/if}
```

## Manifest

```yaml
# .ship/views/<name>/manifest.yaml
name: my-view
type: sui                        # discriminates from j2 stack
rail:                            # optional — exposes the view in the shell rail
  label: My View
  icon: sparkles
events:
  - config.changed
  - workspace.
packs:
  - editor                       # heavy-island bundles loaded on demand
```

For daemon route shapes, the SSR + attach pipeline, and the marker contract, see `references/manifests-routes.md`.

## Hard rules

1. **Mutations in `on.*` handlers only.** `s.x = v` in a derived or template expression is a type error.
2. **Subscriptions in `<data>` elements.** Never call `ship.*` outside `<data>`.
3. **Templates declare; controllers do.** Declarative → template; imperative → controller.
4. **No `<script>` blocks in templates.** Logic in paired `.controller.ts`.
5. **No `customElements.define` in view code.**
6. **Read signals by call:** `count()` reads, `count.set(n)` writes.
7. **Block syntax for control flow.** `{#if}/{#for}/{#match}/{#while-loading}/{#while-error}`.
8. **No `el.innerHTML/.outerHTML/.insertAdjacentHTML`** in `*.controller.ts`. Use `bindHtml()` for HTML-blob cases.

## Components

Views compose `<sui-*>` components from `packages/ui-components/`. See the `ship-components` skill for the component contract.

## References

| When you need | Read |
|---|---|
| Worked template examples, controller tiers, hooks, manifest schema, daemon routes, SSR + attach pipeline | `references/template-surface.md` |
| Working on the SUI codebase itself (compiler, runtime, daemon SUI handlers) | `references/sui-internals.md` |
