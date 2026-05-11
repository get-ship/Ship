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

The canonical view directory shape (see ADR `view-directory-convention`):

```
views/<name>/                 # first-party view, baked into shipd
  page.sui                    # route entry — required
  controller.ts               # paired controller — optional
  style.css                   # page-scoped styles — optional, daemon auto-links
  manifest.jsonc              # route metadata — required
  dist/                       # generated artifacts (compiler emits here)
  _components/                # view-private widgets — optional
    <view>-<tag>/
      <view>-<tag>.sui
      <view>-<tag>.controller.ts
      dist/

.ship/views/<name>/           # extension-point view, project-local; same shape
  page.sui
  controller.ts
  manifest.jsonc
  partials/                   # reusable fragments
```

Five rules:

1. **`page.sui` is the route entry.** The filename stem says what the file IS — a page, not a reusable widget. Reusable widgets live under `packages/ui-components/src/sui-*/`.
2. **`controller.ts`, no stem prefix.** The directory IS the view; pairing is unambiguous. Inside `_components/<tag>/`, the `<tag>.controller.ts` rule from `ship-components` applies.
3. **`style.css` for page-scoped styles.** Linked automatically. Use `<style>` blocks inside `.sui` for component-scoped styles.
4. **`manifest.jsonc` is required.** JSONC, not YAML. See "Manifest" below.
5. **`_components/` for view-private widgets.** Tag MUST prefix with the view name (`home-hero` for `views/home/`) so two views can each ship their own `<my-list>` without colliding on `customElements.define`.

Plain `.ts` files (not `controller.ts`, not under `_components/`) are helpers, types, or tests.

Migration note: existing `views/agents/`, `views/counter/`, `views/git/` still use the legacy `view.sui` / `view.controller.ts` / `view.css` shape. They migrate in a separate job once the daemon resolver picks up the new convention. The router (`packages/ship-set/src/router/`) ignores legacy directories — it only walks views that ship a `manifest.jsonc`.

### Worked example: `views/home/`

```
views/home/
  page.sui
  controller.ts
  style.css
  manifest.jsonc
  _components/
    home-hero/
      home-hero.sui
      home-hero.controller.ts
```

Manifest:

```jsonc
// views/home/manifest.jsonc
{
  "name": "home",
  "path": "/",
  "title": "Home",
  "layout": "default",
  "auth": "public",
  "rail": { "label": "Home", "icon": "home" }
}
```

Page composes the view-private widget. Reactive state lives in
`controller.ts`'s `scope:` factory — there is no `<state>` directive:

```html
<!-- views/home/page.sui -->
<div class="home">
  <home-hero prop:title={'Welcome to Ship'}></home-hero>
  <button on:click={() => seen.set(seen() + 1)}>Bump ({seen()})</button>
</div>
```

```ts
// views/home/controller.ts
import { controller, signal } from '@ship/ui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const seen = signal(0)
    return { seen }
  },
})
```

Scaffold a new view:

```sh
node scripts/scaffold-view.mjs settings
node scripts/scaffold-view.mjs settings --component settings-toolbar
```

## Template surface (compact)

```html
<!-- live data subscriptions (server-driven, SSE-backed) -->
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />

<!-- reactive args — re-subscribes when selectedId changes.
     `selectedId` is a signal declared in the controller's `scope:`
     factory (see Controller contract below). -->
<data name="agent" source="ship.agents.get({id: $scope.selectedId()})" live />

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

The file IS the view — no `<component>` wrapper. Top-level directives (`<data>`, `<prop>`, `<use-*>`) declare scope; the rest is the body. Reactive component state is declared in the paired controller's `scope:` factory, not the template.

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

## Cross-component reactive context (shared / useShared)

For complex components that split into a public outer + internal inner pieces, share signals from parent to descendants without prop-drilling.

**Provider** wraps signals with `shared(...)` inside `scope:`:

```ts
// parent.controller.ts
scope: () => {
  const mode  = shared(signal<BarMode>('find'))
  const value = shared(signal(''))
  const local = signal(0)  // not shared
  return { mode, value, local }
}
```

The runtime auto-publishes the marked signals on the host element under the host's tag name.

**Consumer** resolves the parent's scope inside its own `scope:`:

```ts
// child.controller.ts
import type { Scope as ParentScope } from '../parent/parent.sui'

scope: () => {
  const bar = useShared<ParentScope>('parent-tag-name')
  return { bar }
}
```

Template binds `bar.mode()`, `bar.value`, etc. — same signal identity, two-way reactivity flows.

`useShared` walks DOM ancestry crossing shadow boundaries. Returns `null` when no provider is found; descendant guards with `{#if bar}`.

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

```jsonc
// views/<name>/manifest.jsonc — JSONC, the house format
{
  "name": "my-view",        // stable identity; matches directory
  "path": "/my-view",       // URL pattern — `/path` or `/path/[param]`
  "title": "My View",       // <title> when mounted
  "layout": "default",      // layout slot key
  "auth": "public",         // "public" | "user" | "admin"
  "rail": {                 // optional — shell rail entry
    "label": "My View",
    "icon": "sparkles"
  },
  "preload": [              // optional — SDK calls warmed before mount
    "ship.workspaces.list()"
  ],
  "events": [               // optional — SSE channels to subscribe to
    "config.changed"
  ]
}
```

The router (`packages/ship-set/src/router/`) walks `views/` at build/start time and registers each `manifest.jsonc`. URL → view module resolution and `[param]` capture happen there. For daemon route shapes, the SSR + attach pipeline, and the marker contract, see `references/manifests-routes.md`.

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
