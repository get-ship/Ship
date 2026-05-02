# SUI view authoring

`.sui` templates, optional paired `.ts` controllers, AOT-compiled, server-rendered first, attach() runtime hydrates leaf bindings. New views land here; j2 is fix-only.

## File layout

```
.ship/views/<name>/         # extension-point views (project-local, daemon resolves on disk)
  index.sui                 # template
  index.ts                  # paired controller (optional)
  manifest.yaml             # name, type: sui, rail: { label, icon }, events
  [param]/index.sui         # parameterized sub-routes
  partials/                 # reusable fragments

views/<name>/               # first-party views baked into shipd
  view.sui                  # template
  view.ts                   # controller (optional)
  view.css                  # styles (daemon auto-links)
  package.json
```

Reference views: `views/counter/` (tier-1, no controller), `views/agents/` (tier-2 with paired controller), `views/git/` (live data + control flow + hooks).

## Template surface

```html
<!-- live data subscriptions (server-driven, SSE-backed) -->
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />

<!-- reactive args — re-subscribes when selectedId changes -->
<state name="selectedId" value="'abc'" />
<data name="agent" source="ship.agents.get({id: $scope.selectedId()})" live />

<!-- local state -->
<state name="count" value="0" />

<!-- props (component .sui files only; tag = directory/filename stem) -->
<prop name="tone" values="default|primary" default="default" reflect />
<prop name="disabled" type="bool" default="false" reflect />

<!-- mutations (write-side complement to <data>) -->
<form mutation="ship.agents.create">
  <input name="name" />
  <button type="submit" disabled={$mutation().kind === 'loading'}>Create</button>
</form>

<!-- control flow — block syntax only -->
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

File IS the component / view — no `<component>` wrapper. Top-level directives (`<state>`, `<data>`, `<prop>`, `<use-*>`) declare scope; the rest is the body.

## Controller (paired `view.ts`, five sections)

```ts
import { controller, unwrap, useFilteredList } from '@ship/ui-runtime'
import type { Scope } from './view.sui'   // compiler-emitted .sui.d.ts

export default controller<Scope>({
  derived: { total: ({ agents }) => unwrap(agents)?.items.length ?? 0 },
  on:      { newAgent: ({ navigate }) => navigate('/agents/create') },
  use:     { list: useFilteredList(agents, (a, q) => a.name.includes(q)) },
  mount:   ({ state }, el) => { /* runs after first DOM attach */ },
  unmount: ({ state })     => { /* runs on dispose */ },
})
```

Three tiers, smallest that fits:

1. **Tier 1 — no controller.** Pure `.sui` with `<state>` and single-expression inline handlers.
2. **Tier 2 — object-literal controller.** Most coordinated views. Multi-statement handlers require a paired `.ts`.
3. **Tier 3 — function-form controller.** When setup needs closure-scoped variables.

A multi-statement `on:event` handler in a tier-1 file (no `.ts`) is a compile error: *multi-statement handler — paired controller required for tier-2 work*. Enforced by `crates/sui-compiler/src/check/handler_complexity.rs`.

## Controller hooks (compose via `use:`)

```ts
import { useFilteredList, useSelection, useForm } from '@ship/ui-runtime'
```

Templates reference them namespaced: `list.filtered()`, `sel.selected()`, `form.values()`, `form.submit`.

## Live data envelope

```ts
type LiveState<T> = { kind: 'loading' } | { kind: 'error'; error: Error } | { kind: 'ok'; value: T }

unwrap(sig)               // T | undefined  — undefined when loading/error
unwrapOr(sig, fallback)   // T              — returns fallback when loading/error
```

```html
{#while-loading of agents}<p>Loading…</p>{/while-loading}
{#while-error err of agents}<p>{err.message}</p>{/while-error}
{#if agents().kind === 'ok'}
  {#for a of agents().value.items key={a.id}}…{/for}
{/if}
```

## Manifest (`.ship/views/<name>/manifest.yaml`)

```yaml
name: my-view
type: sui                        # discriminates from j2
rail:                            # optional — exposes the view in the shell rail
  label: My View
  icon: sparkles
events:
  - config.changed
  - workspace.
packs:
  - editor                       # heavy-island bundles loaded on demand
```

## Daemon route shapes

| Route | Purpose |
|---|---|
| `GET /sui/<name>` | Full-page SSR — complete HTML document with attach() boot script |
| `GET /_sui/fragment/<name>[/<sub>]` | Shell-embeddable SSR fragment |
| `GET /_sui/views/<name>/view.js` | Compiled ESM module for the view |
| `GET /_sui/render/<name>` | Debug: SSR HTML + importmap |

Embedding into the shell from JS: `mountSuiView(slot, name, subPath?)` — fetches the fragment, runs attach, returns disposer. No wrapper custom element.

## SSR + attach pipeline

The `sui-compiler` Rust crate emits one ESM per `.sui`:

```js
import { signal, on, bindText, view, … } from "@ship/ui-runtime"
import setup from "./view.ts"               // when paired
export default view((ctx) => { /* scope + bindings */ })
export const bindings = [ … ]               // manifest read by attach()
```

The daemon SSR walker emits HTML with author-syntax markers:

| Template construct | HTML marker |
|---|---|
| `{expr}` text interpolation | `<!--sui:text:bN-->value<!--/sui-->` |
| `on:click={fn}` | `on:click="bN"` attribute preserved |
| `bind:value={sig}` | `bind:value="bN"` attribute preserved |
| `class:active={expr}` | `class:active="bN"` attribute preserved |
| `style:color={expr}` | `style:color="bN"` attribute preserved |
| `attr:href={expr}` | `attr:href="bN"` attribute preserved |
| `{#if}` / `{#for}` / `{#match}` active branch | `<sui-if key="bN">…</sui-if>` (or `<sui-for>`, `<sui-match>`) |
| `{#while-loading}` / `{#while-error}` | `<sui-while-loading key="bN">` / `<sui-while-error key="bN">` (boundary wrappers) |
| Inactive branch | `<sui-if key="bN" hidden></sui-if>` |
| Initial scope values | `<script type="application/json" id="__set_init__">…</script>` |

Zero `data-*` attributes in framework-emitted HTML.

`attach()` from `@ship/ui-dom` walks the SSR DOM, reads the JSON island, wires bindings against the manifest, runs lifecycle.mount, returns a disposer:

```ts
import { attach } from '@ship/ui-dom'
const dispose = attach(root, { module, ship, route, navigate, viewName })
```

Production never imports `@ship/ui-dom/legacy` — that pulls in the parser (~5 KB gzip).

## Shell chrome

The shell owns topbar, breadcrumb, status bar. Views contribute metadata + slotted actions via `<ship-view-chrome>` (same element used by j2 views — see `j2-authoring.md` for the slot/attribute reference). Don't reimplement chrome inside the view body.

## Hard rules

1. **Mutations in `on.*` handlers only.** `s.x = v` in a derived or template expression is a type error.
2. **Subscriptions in `<data>` elements.** Never call `ship.*` outside `<data>`.
3. **Templates declare; controllers do.** Declarative → template; imperative → controller.
4. **No `<script>` blocks in templates.** Logic in paired `.ts`.
5. **No `customElements.define` in view code.**
6. **Read signals by call:** `count()` reads, `count.set(n)` writes.
7. **Block syntax for control flow.** Element forms (`<while-loading>`, etc.) are gone.

## Components

SUI views compose Lit `<ship-*>` (current production) and SUI `<sui-*>` (in-flight) — both are real custom elements. Mixing is allowed during the SUI port: use `<ship-*>` until a `<sui-*>` parity exists. See the `ship-components` skill for the component charter.

## What's phased out

- `<component>` wrapper element. Gone. File IS the component.
- `<while-loading>` / `<while-error>` element forms. Replaced by block syntax `{#while-loading}…{/while-loading}` / `{#while-error err}…{/while-error}`.
- Client-side parser / `hydrate()` in production. AOT + `attach()` only.
- `.html.j2` for new views. Fix-only.
